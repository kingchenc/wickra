//! Exponential Moving Average.

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Exponential Moving Average with smoothing factor `alpha = 2 / (period + 1)`.
///
/// The first value is seeded with the simple mean of the first `period` inputs
/// (the classical TA-Lib convention). From then on each new input contributes
/// `alpha * input + (1 - alpha) * previous`.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Ema};
///
/// let mut indicator = Ema::new(3).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Ema {
    period: usize,
    alpha: f64,
    /// `1 - alpha`, precomputed so the recurrence avoids a subtraction per tick.
    /// Cached value, so the steady-state output is bit-for-bit unchanged.
    one_minus_alpha: f64,
    /// Latest EMA value, valid only once `seeded` is true. Stored as a bare `f64`
    /// (plus the `seeded` flag) rather than `Option<f64>` so the steady-state
    /// recurrence reads and writes 8 bytes with no enum-tag handling per tick.
    current: f64,
    /// Whether `current` holds a real value yet (warmup complete).
    seeded: bool,
    warmup_buf: Vec<f64>,
}

impl Ema {
    /// Construct an EMA with the given period.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        let alpha = 2.0 / (period as f64 + 1.0);
        Ok(Self {
            period,
            alpha,
            one_minus_alpha: 1.0 - alpha,
            current: 0.0,
            seeded: false,
            warmup_buf: Vec::with_capacity(period),
        })
    }

    /// Construct an EMA with a custom smoothing factor `alpha in (0, 1]`.
    ///
    /// The reported `period` is derived from `alpha` via `2/alpha - 1` and rounded;
    /// `warmup_period()` falls back to `1` because the implementation seeds from the
    /// very first input.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPeriod`] if `alpha` is not in `(0.0, 1.0]` or non-finite.
    pub fn with_alpha(alpha: f64) -> Result<Self> {
        if !alpha.is_finite() || alpha <= 0.0 || alpha > 1.0 {
            return Err(Error::InvalidPeriod {
                message: "alpha must be in (0.0, 1.0]",
            });
        }
        Ok(Self {
            period: 1,
            alpha,
            one_minus_alpha: 1.0 - alpha,
            current: 0.0,
            seeded: false,
            warmup_buf: Vec::with_capacity(1),
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Smoothing factor.
    pub const fn alpha(&self) -> f64 {
        self.alpha
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        if self.seeded {
            Some(self.current)
        } else {
            None
        }
    }

    /// Internal helper that feeds a value without finiteness validation. The caller
    /// guarantees `input.is_finite()`. Used by MACD which has already validated.
    pub(crate) fn step_unchecked(&mut self, input: f64) -> Option<f64> {
        if self.seeded {
            let new = self
                .alpha
                .mul_add(input, self.one_minus_alpha * self.current);
            self.current = new;
            return Some(new);
        }
        self.warmup_buf.push(input);
        if self.warmup_buf.len() == self.period {
            let seed = self.warmup_buf.iter().copied().sum::<f64>() / self.period as f64;
            self.current = seed;
            self.seeded = true;
            return Some(seed);
        }
        None
    }
}

impl Indicator for Ema {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.value();
        }
        self.step_unchecked(input)
    }

    fn reset(&mut self) {
        self.current = 0.0;
        self.seeded = false;
        self.warmup_buf.clear();
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.seeded
    }

    fn name(&self) -> &'static str {
        "EMA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    /// Independent reference: SMA-seeded EMA computed straight from the definition.
    fn ema_naive(prices: &[f64], period: usize) -> Vec<Option<f64>> {
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut out = Vec::with_capacity(prices.len());
        let mut state: Option<f64> = None;
        for (i, &p) in prices.iter().enumerate() {
            if let Some(prev) = state {
                let v = alpha * p + (1.0 - alpha) * prev;
                state = Some(v);
                out.push(Some(v));
            } else if i + 1 == period {
                let seed = prices[..period].iter().sum::<f64>() / period as f64;
                state = Some(seed);
                out.push(Some(seed));
            } else {
                out.push(None);
            }
        }
        out
    }

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(Ema::new(0), Err(Error::PeriodZero)));
    }

    /// Cover the const accessor `period` (74-77) and the Indicator-impl
    /// `warmup_period` (123-125) + `name` (131-133). `alpha` and `value`
    /// are exercised by other tests and downstream consumers; only the
    /// three metadata methods were dead.
    #[test]
    fn accessors_and_metadata() {
        let ema = Ema::new(14).unwrap();
        assert_eq!(ema.period(), 14);
        assert_eq!(ema.warmup_period(), 14);
        assert_eq!(ema.name(), "EMA");
    }

    #[test]
    fn warmup_returns_none_until_seed() {
        let mut ema = Ema::new(3).unwrap();
        assert_eq!(ema.update(1.0), None);
        assert_eq!(ema.update(2.0), None);
        assert_eq!(ema.update(3.0), Some(2.0)); // seed = SMA([1,2,3]) = 2
    }

    #[test]
    fn first_value_equals_sma_seed() {
        let mut ema = Ema::new(5).unwrap();
        let inputs = [10.0, 20.0, 30.0, 40.0, 50.0];
        let mut last = None;
        for v in inputs {
            last = ema.update(v);
        }
        assert_relative_eq!(last.unwrap(), 30.0, epsilon = 1e-12);
    }

    #[test]
    fn alpha_matches_period_formula() {
        let ema = Ema::new(10).unwrap();
        assert_relative_eq!(ema.alpha(), 2.0 / 11.0, epsilon = 1e-15);
    }

    #[test]
    fn step_after_seed_uses_alpha_formula() {
        // period=3 => alpha = 0.5; seed = mean([1,2,3]) = 2; next input 10
        // expected = 0.5*10 + 0.5*2 = 6
        let mut ema = Ema::new(3).unwrap();
        ema.batch(&[1.0, 2.0, 3.0]);
        assert_relative_eq!(ema.update(10.0).unwrap(), 6.0, epsilon = 1e-12);
    }

    #[test]
    fn constant_series_converges_to_constant() {
        let mut ema = Ema::new(10).unwrap();
        let out = ema.batch(&[42.0_f64; 100]);
        for x in out.iter().skip(9) {
            assert_relative_eq!(x.unwrap(), 42.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn with_alpha_validates_range() {
        assert!(Ema::with_alpha(0.5).is_ok());
        assert!(Ema::with_alpha(1.0).is_ok());
        assert!(matches!(
            Ema::with_alpha(0.0),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            Ema::with_alpha(1.5),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            Ema::with_alpha(f64::NAN),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn reset_clears_state() {
        let mut ema = Ema::new(3).unwrap();
        ema.batch(&[1.0, 2.0, 3.0]);
        assert!(ema.is_ready());
        ema.reset();
        assert!(!ema.is_ready());
        assert_eq!(ema.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=30).map(f64::from).collect();
        let mut a = Ema::new(5).unwrap();
        let mut b = Ema::new(5).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut ema = Ema::new(3).unwrap();
        ema.batch(&[1.0, 2.0, 3.0]);
        let before = ema.value();
        assert_eq!(ema.update(f64::NAN), before);
        assert_eq!(ema.update(f64::INFINITY), before);
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(48))]
        #[test]
        fn ema_matches_naive(
            period in 1usize..20,
            prices in proptest::collection::vec(-1000.0_f64..1000.0, 0..150),
        ) {
            let mut ema = Ema::new(period).unwrap();
            let got = ema.batch(&prices);
            let want = ema_naive(&prices, period);
            proptest::prop_assert_eq!(got.len(), want.len());
            for (g, w) in got.iter().zip(want.iter()) {
                match (g, w) {
                    (None, None) => {}
                    (Some(a), Some(b)) => proptest::prop_assert!(
                        (a - b).abs() <= 1e-9 * a.abs().max(1.0),
                        "got={a} want={b}"
                    ),
                    _ => proptest::prop_assert!(false, "warmup mismatch"),
                }
            }
        }
    }
}
