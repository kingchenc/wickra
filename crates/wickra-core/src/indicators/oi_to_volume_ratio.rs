//! OI-to-Volume Ratio — open interest relative to traded volume.

use crate::derivatives::DerivativesTick;
use crate::traits::Indicator;

/// OI-to-Volume Ratio — open interest divided by the tick's total taker volume, a
/// measure of how much position is *held* versus *turned over*.
///
/// ```text
/// OIVR = open_interest / (taker_buy_volume + taker_sell_volume)
/// ```
///
/// A high ratio means open interest dwarfs the volume trading it — positions are
/// being held, not churned (low participation, potential complacency or a coiling
/// market). A low ratio means heavy volume relative to outstanding interest —
/// active churn, often around breakouts or capitulation. Watching the ratio change
/// distinguishes new-money trends (OI and volume both rising) from short-covering
/// or position rolls.
///
/// The ratio is non-negative; a tick with zero taker volume reports `0` rather than
/// dividing by zero. It is stateless — each tick yields one value (no warmup). Each
/// `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{DerivativesTick, Indicator, OiToVolumeRatio};
///
/// let mut indicator = OiToVolumeRatio::new();
/// let tick = DerivativesTick::new(0.0, 100.0, 100.0, 100.0, 5_000.0, 0.0, 0.0, 400.0, 600.0, 0.0, 0.0, 0).unwrap();
/// let oivr = indicator.update(tick).unwrap();
/// assert!((oivr - 5.0).abs() < 1e-12); // 5000 / (400 + 600)
/// ```
#[derive(Debug, Clone, Default)]
pub struct OiToVolumeRatio {
    ready: bool,
}

impl OiToVolumeRatio {
    /// Construct a new OI-to-Volume Ratio. The indicator is parameter-free.
    #[must_use]
    pub const fn new() -> Self {
        Self { ready: false }
    }
}

impl Indicator for OiToVolumeRatio {
    type Input = DerivativesTick;
    type Output = f64;

    fn update(&mut self, tick: DerivativesTick) -> Option<f64> {
        let volume = tick.taker_buy_volume + tick.taker_sell_volume;
        let ratio = if volume > 0.0 {
            tick.open_interest / volume
        } else {
            0.0
        };
        self.ready = true;
        Some(ratio)
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
        "OiToVolumeRatio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn tick(oi: f64, buy: f64, sell: f64) -> DerivativesTick {
        DerivativesTick::new_unchecked(
            0.0, 100.0, 100.0, 100.0, oi, 0.0, 0.0, buy, sell, 0.0, 0.0, 0,
        )
    }

    #[test]
    fn accessors_and_metadata() {
        let o = OiToVolumeRatio::new();
        assert_eq!(o.warmup_period(), 1);
        assert_eq!(o.name(), "OiToVolumeRatio");
        assert!(!o.is_ready());
    }

    #[test]
    fn ratio_reference_value() {
        let mut o = OiToVolumeRatio::new();
        assert_relative_eq!(
            o.update(tick(5_000.0, 400.0, 600.0)).unwrap(),
            5.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn more_volume_lowers_ratio() {
        let mut o = OiToVolumeRatio::new();
        let held = o.update(tick(5_000.0, 100.0, 100.0)).unwrap();
        let churned = o.update(tick(5_000.0, 1_000.0, 1_000.0)).unwrap();
        assert!(churned < held);
    }

    #[test]
    fn zero_volume_is_zero() {
        let mut o = OiToVolumeRatio::new();
        assert_relative_eq!(
            o.update(tick(5_000.0, 0.0, 0.0)).unwrap(),
            0.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn ready_after_first_update() {
        let mut o = OiToVolumeRatio::new();
        assert!(!o.is_ready());
        o.update(tick(5_000.0, 100.0, 100.0));
        assert!(o.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut o = OiToVolumeRatio::new();
        o.update(tick(5_000.0, 100.0, 100.0));
        assert!(o.is_ready());
        o.reset();
        assert!(!o.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let ticks: Vec<DerivativesTick> = (0..40)
            .map(|i| tick(5_000.0, 100.0 + f64::from(i), 100.0))
            .collect();
        let batch = OiToVolumeRatio::new().batch(&ticks);
        let mut b = OiToVolumeRatio::new();
        let streamed: Vec<_> = ticks.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
