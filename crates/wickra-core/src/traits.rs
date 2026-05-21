//! Core traits: the [`Indicator`] state machine and the [`BatchExt`] blanket extension.

/// A streaming technical indicator.
///
/// Every indicator in Wickra implements this trait. The contract is:
///
/// - [`update`](Indicator::update) is called once per input point and must be O(1) in
///   the input length. Pre-existing buffered state may be touched, but no full
///   recomputation over the entire series is permitted.
/// - The returned `Option<Output>` is `None` while the indicator is still in its
///   *warmup* phase (insufficient inputs to produce a defined value), and `Some`
///   once it is ready.
/// - [`reset`](Indicator::reset) clears all state, returning the indicator to the
///   exact configuration it had immediately after construction.
///
/// Implementors that consume scalar prices use `Input = f64` so they automatically
/// gain access to chaining via [`Chain`].
pub trait Indicator {
    /// Type of one input data point (typically `f64` for a price, or `Candle` / `Tick`).
    type Input;
    /// Type of one output value.
    type Output;

    /// Feed one new data point into the indicator and return the freshly computed
    /// output, or `None` if the indicator is still warming up.
    fn update(&mut self, input: Self::Input) -> Option<Self::Output>;

    /// Reset all internal state, leaving the indicator equivalent to a freshly
    /// constructed instance with the same parameters.
    fn reset(&mut self);

    /// Number of inputs required before the first non-`None` output can be produced.
    fn warmup_period(&self) -> usize;

    /// Whether the indicator has emitted at least one value since the last reset.
    fn is_ready(&self) -> bool;

    /// Stable, human-readable indicator name. Used by chaining and diagnostics.
    fn name(&self) -> &'static str;
}

/// Blanket extension that adds batch evaluation to every [`Indicator`].
///
/// The naive `batch` simply replays `update` over a slice, which is always correct
/// because `update` is the only state transition. Concrete indicators may override
/// `batch` if they have a faster vectorized path; the default keeps the contract
/// `batch == repeated update`.
pub trait BatchExt: Indicator {
    /// Run the indicator over a slice of inputs in order, returning one output (or
    /// `None` during warmup) per input.
    fn batch(&mut self, inputs: &[Self::Input]) -> Vec<Option<Self::Output>>
    where
        Self::Input: Clone,
    {
        let mut out = Vec::with_capacity(inputs.len());
        for x in inputs {
            out.push(self.update(x.clone()));
        }
        out
    }

    /// Run an independent copy of the indicator over each input series in parallel.
    ///
    /// Each asset is processed by its own fresh instance built via `make`, so state
    /// never leaks across assets. Requires the `parallel` feature (enabled by
    /// default), which pulls in `rayon`.
    #[cfg(feature = "parallel")]
    fn batch_parallel<F>(
        inputs_per_asset: &[Vec<Self::Input>],
        make: F,
    ) -> Vec<Vec<Option<Self::Output>>>
    where
        Self: Sized + Send,
        Self::Input: Sync + Clone,
        Self::Output: Send,
        F: Fn() -> Self + Sync + Send,
    {
        use rayon::prelude::*;
        inputs_per_asset
            .par_iter()
            .map(|series| {
                let mut ind = make();
                ind.batch(series)
            })
            .collect()
    }
}

impl<T: Indicator> BatchExt for T {}

/// Chain two indicators so the output of the first becomes the input of the second.
///
/// Both indicators must agree on `f64` as the bridging type, which is the common
/// case for price-in/value-out indicators. The chain itself is an indicator, so
/// chains can be nested arbitrarily.
///
/// # Example
///
/// ```
/// use wickra_core::{Chain, Ema, Indicator, Rsi};
///
/// // RSI(7) on top of EMA(14). EMA seeds at input 14, then RSI needs 7+1 more
/// // valid inputs to emit, so the chain becomes ready at input 21.
/// let mut chain = Chain::new(Ema::new(14).unwrap(), Rsi::new(7).unwrap());
/// for i in 1..=21 {
///     chain.update(f64::from(i));
/// }
/// assert!(chain.is_ready());
/// ```
#[derive(Debug, Clone)]
pub struct Chain<A, B>
where
    A: Indicator<Input = f64, Output = f64>,
    B: Indicator<Input = f64>,
{
    first: A,
    second: B,
}

impl<A, B> Chain<A, B>
where
    A: Indicator<Input = f64, Output = f64>,
    B: Indicator<Input = f64>,
{
    /// Construct a chain whose inputs flow through `first` and then `second`.
    pub const fn new(first: A, second: B) -> Self {
        Self { first, second }
    }

    /// Add a third stage on top.
    pub fn then<C>(self, third: C) -> Chain<Self, C>
    where
        C: Indicator<Input = f64>,
        Self: Indicator<Input = f64, Output = f64>,
    {
        Chain::new(self, third)
    }

    /// Borrow the upstream indicator.
    pub const fn first(&self) -> &A {
        &self.first
    }

    /// Borrow the downstream indicator.
    pub const fn second(&self) -> &B {
        &self.second
    }
}

impl<A, B> Indicator for Chain<A, B>
where
    A: Indicator<Input = f64, Output = f64>,
    B: Indicator<Input = f64>,
{
    type Input = f64;
    type Output = B::Output;

    fn update(&mut self, input: f64) -> Option<Self::Output> {
        self.first.update(input).and_then(|v| self.second.update(v))
    }

    fn reset(&mut self) {
        self.first.reset();
        self.second.reset();
    }

    fn warmup_period(&self) -> usize {
        // Conservative upper bound: both stages must warm up.
        self.first.warmup_period() + self.second.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.first.is_ready() && self.second.is_ready()
    }

    fn name(&self) -> &'static str {
        "Chain"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trivial test indicator: identity (passes input through).
    #[derive(Debug, Default)]
    struct Identity {
        seen: bool,
    }

    impl Indicator for Identity {
        type Input = f64;
        type Output = f64;
        fn update(&mut self, input: f64) -> Option<f64> {
            self.seen = true;
            Some(input)
        }
        fn reset(&mut self) {
            self.seen = false;
        }
        fn warmup_period(&self) -> usize {
            0
        }
        fn is_ready(&self) -> bool {
            self.seen
        }
        fn name(&self) -> &'static str {
            "Identity"
        }
    }

    /// Another trivial test indicator: scales input by 2.
    #[derive(Debug, Default)]
    struct Doubler {
        seen: bool,
    }

    impl Indicator for Doubler {
        type Input = f64;
        type Output = f64;
        fn update(&mut self, input: f64) -> Option<f64> {
            self.seen = true;
            Some(input * 2.0)
        }
        fn reset(&mut self) {
            self.seen = false;
        }
        fn warmup_period(&self) -> usize {
            0
        }
        fn is_ready(&self) -> bool {
            self.seen
        }
        fn name(&self) -> &'static str {
            "Doubler"
        }
    }

    #[test]
    fn batch_replays_update() {
        let mut id = Identity::default();
        let out = id.batch(&[1.0, 2.0, 3.0]);
        assert_eq!(out, vec![Some(1.0), Some(2.0), Some(3.0)]);
    }

    #[test]
    fn chain_pipes_first_into_second() {
        let mut c = Chain::new(Doubler::default(), Doubler::default());
        // 5 -> 10 -> 20
        assert_eq!(c.update(5.0), Some(20.0));
    }

    #[test]
    fn chain_is_ready_only_after_both_stages_emit() {
        let mut c = Chain::new(Doubler::default(), Doubler::default());
        assert!(!c.is_ready());
        c.update(1.0);
        assert!(c.is_ready());
    }

    #[test]
    fn chain_reset_propagates() {
        let mut c = Chain::new(Doubler::default(), Doubler::default());
        c.update(1.0);
        assert!(c.is_ready());
        c.reset();
        assert!(!c.is_ready());
    }

    #[test]
    fn chain_three_levels_via_then() {
        let c = Chain::new(Doubler::default(), Doubler::default()).then(Doubler::default());
        let mut c = c;
        // 1 -> 2 -> 4 -> 8
        assert_eq!(c.update(1.0), Some(8.0));
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn batch_parallel_runs_independent_instances() {
        let series: Vec<Vec<f64>> = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let out = Doubler::batch_parallel(&series, Doubler::default);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], vec![Some(2.0), Some(4.0), Some(6.0)]);
        assert_eq!(out[1], vec![Some(8.0), Some(10.0), Some(12.0)]);
    }
}
