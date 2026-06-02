//! Long-Legged Doji candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Long-Legged Doji — a single-bar indecision signal. A doji with long shadows on
/// *both* sides: price ranged widely up and down yet closed essentially where it
/// opened, a tug-of-war that often precedes a turn.
///
/// ```text
/// range = high − low
/// doji        = |close − open| <= 0.1 * range
/// long upper  = high − max(open, close) >= 0.3 * range
/// long lower  = min(open, close) − low  >= 0.3 * range
/// ```
///
/// Output is `+1.0` when the long-legged doji prints and `0.0` otherwise. This is
/// a non-directional indecision flag — it never emits `−1.0` (use
/// `DragonflyDoji` / `GravestoneDoji` for the directional single-shadow variants).
/// Body and shadow thresholds follow the geometric house style (fixed fractions
/// of the bar range) rather than TA-Lib's rolling averages. Pattern-shape check
/// only — no trend filter is applied; combine with a trend indicator for
/// actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` detected, `0.0` no pattern — so it drops straight into
/// a machine-learning feature matrix as a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, LongLeggedDoji, Indicator};
///
/// let mut indicator = LongLeggedDoji::new();
/// // Tiny body, long shadows on both sides.
/// let candle = Candle::new(10.0, 12.0, 8.0, 10.05, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct LongLeggedDoji {
    has_emitted: bool,
}

impl LongLeggedDoji {
    /// Construct a new Long-Legged Doji detector.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for LongLeggedDoji {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let range = candle.high - candle.low;
        if range <= 0.0 {
            return Some(0.0);
        }
        if (candle.close - candle.open).abs() > 0.1 * range {
            return Some(0.0);
        }
        let upper = candle.high - candle.open.max(candle.close);
        let lower = candle.open.min(candle.close) - candle.low;
        if upper >= 0.3 * range && lower >= 0.3 * range {
            return Some(1.0);
        }
        Some(0.0)
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
        "LongLeggedDoji"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(open: f64, high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(open, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let t = LongLeggedDoji::new();
        assert_eq!(t.name(), "LongLeggedDoji");
        assert_eq!(t.warmup_period(), 1);
        assert!(!t.is_ready());
    }

    #[test]
    fn long_legged_is_plus_one() {
        let mut t = LongLeggedDoji::new();
        assert_eq!(t.update(c(10.0, 12.0, 8.0, 10.05, 0)), Some(1.0));
    }

    #[test]
    fn one_sided_shadow_yields_zero() {
        let mut t = LongLeggedDoji::new();
        // Dragonfly shape: long lower shadow but no upper -> not long-legged.
        assert_eq!(t.update(c(10.0, 10.05, 6.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn non_doji_yields_zero() {
        let mut t = LongLeggedDoji::new();
        assert_eq!(t.update(c(10.0, 12.0, 8.0, 11.5, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = LongLeggedDoji::new();
        assert_eq!(t.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 3.0, base - 3.0, base + 0.05, i)
            })
            .collect();
        let mut a = LongLeggedDoji::new();
        let mut b = LongLeggedDoji::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = LongLeggedDoji::new();
        t.update(c(10.0, 12.0, 8.0, 10.05, 0));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
    }
}
