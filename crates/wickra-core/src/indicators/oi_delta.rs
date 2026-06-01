//! Open-Interest Delta — the tick-over-tick change in open interest.

use crate::derivatives::DerivativesTick;
use crate::traits::Indicator;

/// Open-Interest Delta — the change in open interest from the previous tick.
///
/// ```text
/// delta = openInterestₜ − openInterestₜ₋₁
/// ```
///
/// Open interest is the count of outstanding contracts; its change separates new
/// positioning from mere turnover. Read together with price, rising OI confirms
/// a trend (fresh money entering) while falling OI flags an unwind (positions
/// closing) — the raw input to the [OI / price divergence] signal. A positive
/// delta is net position-building, a negative delta net liquidation/closing.
///
/// The first tick only seeds the previous value and returns `None`; from the
/// second tick on the indicator emits the delta.
///
/// `Input = DerivativesTick`, `Output = f64`.
///
/// [OI / price divergence]: crate::OIPriceDivergence
///
/// # Example
///
/// ```
/// use wickra_core::{DerivativesTick, Indicator, OpenInterestDelta};
///
/// fn tick(oi: f64) -> DerivativesTick {
///     DerivativesTick::new(0.0, 100.0, 100.0, 100.0, oi, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0)
///         .unwrap()
/// }
///
/// let mut oid = OpenInterestDelta::new();
/// assert_eq!(oid.update(tick(1_000.0)), None); // seeds the previous OI
/// assert_eq!(oid.update(tick(1_250.0)), Some(250.0));
/// assert_eq!(oid.update(tick(1_100.0)), Some(-150.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct OpenInterestDelta {
    prev: Option<f64>,
    has_emitted: bool,
}

impl OpenInterestDelta {
    /// Construct a new open-interest delta indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for OpenInterestDelta {
    type Input = DerivativesTick;
    type Output = f64;

    fn update(&mut self, tick: DerivativesTick) -> Option<f64> {
        let oi = tick.open_interest;
        let delta = self.prev.map(|prev| oi - prev);
        self.prev = Some(oi);
        if delta.is_some() {
            self.has_emitted = true;
        }
        delta
    }

    fn reset(&mut self) {
        self.prev = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "OpenInterestDelta"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn tick(oi: f64) -> DerivativesTick {
        DerivativesTick::new_unchecked(
            0.0, 100.0, 100.0, 100.0, oi, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0,
        )
    }

    #[test]
    fn accessors_and_metadata() {
        let oid = OpenInterestDelta::new();
        assert_eq!(oid.name(), "OpenInterestDelta");
        assert_eq!(oid.warmup_period(), 2);
        assert!(!oid.is_ready());
    }

    #[test]
    fn seeds_then_emits_delta() {
        let mut oid = OpenInterestDelta::new();
        assert_eq!(oid.update(tick(1_000.0)), None);
        assert!(!oid.is_ready());
        assert_eq!(oid.update(tick(1_250.0)), Some(250.0));
        assert!(oid.is_ready());
        assert_eq!(oid.update(tick(1_100.0)), Some(-150.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let ticks: Vec<DerivativesTick> = (0..20)
            .map(|i| tick(1_000.0 + f64::from(i * i % 13) * 10.0))
            .collect();
        let mut a = OpenInterestDelta::new();
        let mut b = OpenInterestDelta::new();
        assert_eq!(
            a.batch(&ticks),
            ticks.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut oid = OpenInterestDelta::new();
        oid.update(tick(1_000.0));
        oid.update(tick(1_250.0));
        assert!(oid.is_ready());
        oid.reset();
        assert!(!oid.is_ready());
        // After reset the next tick only re-seeds, returning None.
        assert_eq!(oid.update(tick(2_000.0)), None);
    }
}
