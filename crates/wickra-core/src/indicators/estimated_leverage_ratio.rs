//! Estimated Leverage Ratio — open interest per unit of aggregate position size.

use crate::derivatives::DerivativesTick;
use crate::traits::Indicator;

/// Estimated Leverage Ratio (ELR) — open interest relative to the aggregate
/// long+short position size, a proxy for how leveraged outstanding positions are.
///
/// ```text
/// ELR = open_interest / (long_size + short_size)
/// ```
///
/// The classic estimated leverage ratio compares open interest (the notional of
/// outstanding contracts) to the capital backing it. With the size fields of a
/// [`DerivativesTick`] standing in for the position base, the ratio rises when a
/// given pool of positions controls more open interest — i.e. when the market is
/// running hotter leverage. Spikes in ELR mark crowded, fragile conditions where a
/// move can cascade into liquidations; a falling ELR marks deleveraging.
///
/// The ratio is non-negative; a tick with zero aggregate size reports `0` rather
/// than dividing by zero. It is stateless — each tick yields one value (no warmup).
/// Each `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{DerivativesTick, Indicator, EstimatedLeverageRatio};
///
/// let mut indicator = EstimatedLeverageRatio::new();
/// let tick = DerivativesTick::new(0.0001, 100.0, 100.0, 100.0, 1_000.0, 400.0, 600.0, 0.0, 0.0, 0.0, 0.0, 0).unwrap();
/// let elr = indicator.update(tick).unwrap();
/// assert!((elr - 1.0).abs() < 1e-12); // 1000 / (400 + 600)
/// ```
#[derive(Debug, Clone, Default)]
pub struct EstimatedLeverageRatio {
    ready: bool,
}

impl EstimatedLeverageRatio {
    /// Construct a new Estimated Leverage Ratio. The indicator is parameter-free.
    #[must_use]
    pub const fn new() -> Self {
        Self { ready: false }
    }
}

impl Indicator for EstimatedLeverageRatio {
    type Input = DerivativesTick;
    type Output = f64;

    fn update(&mut self, tick: DerivativesTick) -> Option<f64> {
        let base = tick.long_size + tick.short_size;
        let elr = if base > 0.0 {
            tick.open_interest / base
        } else {
            0.0
        };
        self.ready = true;
        Some(elr)
    }

    fn reset(&mut self) {
        self.ready = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn name(&self) -> &'static str {
        "EstimatedLeverageRatio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn tick(oi: f64, long: f64, short: f64) -> DerivativesTick {
        DerivativesTick::new_unchecked(
            0.0, 100.0, 100.0, 100.0, oi, long, short, 0.0, 0.0, 0.0, 0.0, 0,
        )
    }

    #[test]
    fn accessors_and_metadata() {
        let e = EstimatedLeverageRatio::new();
        assert_eq!(e.warmup_period(), 1);
        assert_eq!(e.name(), "EstimatedLeverageRatio");
        assert!(!e.is_ready());
    }

    #[test]
    fn ratio_reference_value() {
        let mut e = EstimatedLeverageRatio::new();
        // 1000 / (400 + 600) = 1.0.
        assert_relative_eq!(
            e.update(tick(1_000.0, 400.0, 600.0)).unwrap(),
            1.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn higher_oi_raises_ratio() {
        let mut e = EstimatedLeverageRatio::new();
        let low = e.update(tick(1_000.0, 500.0, 500.0)).unwrap();
        let high = e.update(tick(3_000.0, 500.0, 500.0)).unwrap();
        assert!(high > low);
    }

    #[test]
    fn zero_base_is_zero() {
        let mut e = EstimatedLeverageRatio::new();
        assert_relative_eq!(
            e.update(tick(1_000.0, 0.0, 0.0)).unwrap(),
            0.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn ready_after_first_update() {
        let mut e = EstimatedLeverageRatio::new();
        assert!(!e.is_ready());
        e.update(tick(1_000.0, 500.0, 500.0));
        assert!(e.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut e = EstimatedLeverageRatio::new();
        e.update(tick(1_000.0, 500.0, 500.0));
        assert!(e.is_ready());
        e.reset();
        assert!(!e.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let ticks: Vec<DerivativesTick> = (0..40)
            .map(|i| tick(1_000.0 + f64::from(i) * 10.0, 500.0, 500.0))
            .collect();
        let batch = EstimatedLeverageRatio::new().batch(&ticks);
        let mut b = EstimatedLeverageRatio::new();
        let streamed: Vec<_> = ticks.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
