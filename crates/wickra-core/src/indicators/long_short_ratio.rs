//! Long/Short Ratio — aggregate long size relative to short size.

use crate::derivatives::DerivativesTick;
use crate::traits::Indicator;

/// Long/Short Ratio — the aggregate long size divided by the aggregate short
/// size carried by each tick.
///
/// ```text
/// longShortRatio = longSize / shortSize
/// ```
///
/// Exchanges publish the long/short account (or position) ratio as a crowd
/// positioning gauge: a ratio above `1` means longs outweigh shorts, below `1`
/// the reverse. Extremes are a contrarian signal — an overwhelmingly long crowd
/// is fuel for a long squeeze. When the short side is zero the ratio is
/// undefined and the indicator reports `0.0`.
///
/// `Input = DerivativesTick`, `Output = f64`. Stateless; ready after the first
/// tick.
///
/// # Example
///
/// ```
/// use wickra_core::{DerivativesTick, Indicator, LongShortRatio};
///
/// fn tick(long: f64, short: f64) -> DerivativesTick {
///     DerivativesTick::new(0.0, 100.0, 100.0, 100.0, 0.0, long, short, 0.0, 0.0, 0.0, 0.0, 0)
///         .unwrap()
/// }
///
/// let mut lsr = LongShortRatio::new();
/// // 600 longs vs 400 shorts -> 1.5.
/// assert_eq!(lsr.update(tick(600.0, 400.0)), Some(1.5));
/// ```
#[derive(Debug, Clone, Default)]
pub struct LongShortRatio {
    has_emitted: bool,
}

impl LongShortRatio {
    /// Construct a new long/short ratio indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for LongShortRatio {
    type Input = DerivativesTick;
    type Output = f64;

    fn update(&mut self, tick: DerivativesTick) -> Option<f64> {
        self.has_emitted = true;
        if tick.short_size == 0.0 {
            // No short side to divide by: the ratio is undefined.
            return Some(0.0);
        }
        Some(tick.long_size / tick.short_size)
    }

    fn reset(&mut self) {
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "LongShortRatio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn tick(long: f64, short: f64) -> DerivativesTick {
        DerivativesTick::new_unchecked(
            0.0, 100.0, 100.0, 100.0, 0.0, long, short, 0.0, 0.0, 0.0, 0.0, 0,
        )
    }

    #[test]
    fn accessors_and_metadata() {
        let lsr = LongShortRatio::new();
        assert_eq!(lsr.name(), "LongShortRatio");
        assert_eq!(lsr.warmup_period(), 1);
        assert!(!lsr.is_ready());
    }

    #[test]
    fn divides_long_by_short() {
        let mut lsr = LongShortRatio::new();
        assert_eq!(lsr.update(tick(600.0, 400.0)), Some(1.5));
        assert_eq!(lsr.update(tick(400.0, 800.0)), Some(0.5));
        assert!(lsr.is_ready());
    }

    #[test]
    fn zero_short_is_zero() {
        let mut lsr = LongShortRatio::new();
        assert_eq!(lsr.update(tick(600.0, 0.0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let ticks: Vec<DerivativesTick> = (0..20)
            .map(|i| {
                tick(
                    500.0 + f64::from(i % 5) * 10.0,
                    400.0 + f64::from(i % 3) * 10.0,
                )
            })
            .collect();
        let mut a = LongShortRatio::new();
        let mut b = LongShortRatio::new();
        assert_eq!(
            a.batch(&ticks),
            ticks.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut lsr = LongShortRatio::new();
        lsr.update(tick(600.0, 400.0));
        assert!(lsr.is_ready());
        lsr.reset();
        assert!(!lsr.is_ready());
    }
}
