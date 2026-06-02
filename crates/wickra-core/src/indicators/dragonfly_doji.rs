//! Dragonfly Doji candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Dragonfly Doji — a single-bar bullish reversal. Open, close, and high sit at
/// the top of the bar while a long lower shadow shows price was driven down hard
/// and then bid all the way back to the open — buyers rejecting the lows.
///
/// ```text
/// range = high − low
/// doji         = |close − open| <= 0.1 * range
/// no upper wick = high − max(open, close) <= 0.1 * range
/// long lower    = min(open, close) − low   >= 0.5 * range
/// ```
///
/// Output is `+1.0` when the dragonfly prints and `0.0` otherwise. Dragonfly Doji
/// is a single-direction (bullish-only) shape, so it never emits `−1.0`. Body and
/// shadow thresholds follow the geometric house style (fixed fractions of the bar
/// range) rather than TA-Lib's rolling averages. Pattern-shape check only — no
/// trend filter is applied; combine with a trend indicator for actionable
/// signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` bullish, `0.0` no pattern — so it drops straight into
/// a machine-learning feature matrix as a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, DragonflyDoji, Indicator};
///
/// let mut indicator = DragonflyDoji::new();
/// // Body at the top, long lower shadow.
/// let candle = Candle::new(10.0, 10.05, 6.0, 10.0, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct DragonflyDoji {
    has_emitted: bool,
}

impl DragonflyDoji {
    /// Construct a new Dragonfly Doji detector.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for DragonflyDoji {
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
        if upper <= 0.1 * range && lower >= 0.5 * range {
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
        "DragonflyDoji"
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
        let t = DragonflyDoji::new();
        assert_eq!(t.name(), "DragonflyDoji");
        assert_eq!(t.warmup_period(), 1);
        assert!(!t.is_ready());
    }

    #[test]
    fn dragonfly_is_plus_one() {
        let mut t = DragonflyDoji::new();
        assert_eq!(t.update(c(10.0, 10.05, 6.0, 10.0, 0)), Some(1.0));
    }

    #[test]
    fn upper_shadow_yields_zero() {
        let mut t = DragonflyDoji::new();
        // Long upper shadow -> not a dragonfly (this is a gravestone shape).
        assert_eq!(t.update(c(10.0, 14.0, 9.95, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn short_lower_shadow_yields_zero() {
        let mut t = DragonflyDoji::new();
        // Body at the top but the lower shadow is too short.
        assert_eq!(t.update(c(10.0, 10.05, 9.6, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn non_doji_yields_zero() {
        let mut t = DragonflyDoji::new();
        assert_eq!(t.update(c(10.0, 12.0, 6.0, 11.5, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = DragonflyDoji::new();
        assert_eq!(t.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 0.05, base - 4.0, base, i)
            })
            .collect();
        let mut a = DragonflyDoji::new();
        let mut b = DragonflyDoji::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = DragonflyDoji::new();
        t.update(c(10.0, 10.05, 6.0, 10.0, 0));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
    }
}
