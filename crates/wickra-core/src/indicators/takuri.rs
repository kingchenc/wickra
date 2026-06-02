//! Takuri candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Takuri — a single-bar bullish reversal, a stricter Dragonfly Doji. Open, close,
/// and high sit at the very top of the bar with a negligible upper shadow, while an
/// exceptionally long lower shadow shows price was driven sharply down and then bid
/// all the way back — an emphatic rejection of the lows.
///
/// ```text
/// range = high − low
/// doji            = |close − open| <= 0.1 * range
/// negligible upper = high − max(open, close) <= 0.05 * range
/// very long lower  = min(open, close) − low   >= 0.7  * range
/// ```
///
/// Output is `+1.0` when the Takuri prints and `0.0` otherwise. Takuri is a
/// single-direction (bullish-only) shape, so it never emits `−1.0`. Its tighter
/// upper-shadow and longer lower-shadow thresholds make it a strict subset of
/// [`crate::DragonflyDoji`]. Body and shadow thresholds follow the geometric house
/// style (fixed fractions of the bar range) rather than TA-Lib's rolling averages.
/// Pattern-shape check only — no trend filter is applied; combine with a trend
/// indicator for actionable signals.
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
/// use wickra_core::{Candle, Indicator, Takuri};
///
/// let mut indicator = Takuri::new();
/// // Body at the top, very long lower shadow.
/// let candle = Candle::new(10.0, 10.05, 7.0, 10.0, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Takuri {
    has_emitted: bool,
}

impl Takuri {
    /// Construct a new Takuri detector.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for Takuri {
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
        if upper <= 0.05 * range && lower >= 0.7 * range {
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
        "Takuri"
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
        let t = Takuri::new();
        assert_eq!(t.name(), "Takuri");
        assert_eq!(t.warmup_period(), 1);
        assert!(!t.is_ready());
    }

    #[test]
    fn takuri_is_plus_one() {
        let mut t = Takuri::new();
        assert_eq!(t.update(c(10.0, 10.05, 7.0, 10.0, 0)), Some(1.0));
    }

    #[test]
    fn non_doji_body_yields_zero() {
        let mut t = Takuri::new();
        // Large body -> not a doji.
        assert_eq!(t.update(c(10.0, 12.0, 7.0, 11.5, 0)), Some(0.0));
    }

    #[test]
    fn upper_shadow_yields_zero() {
        let mut t = Takuri::new();
        // Long upper shadow -> not a Takuri.
        assert_eq!(t.update(c(10.0, 14.0, 7.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn dragonfly_but_not_takuri_yields_zero() {
        let mut t = Takuri::new();
        // Upper shadow ~0.07 of range: a Dragonfly Doji, but exceeds Takuri's
        // tighter 0.05 ceiling.
        assert_eq!(t.update(c(10.0, 10.24, 7.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = Takuri::new();
        assert_eq!(t.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 0.02, base - 4.0, base, i)
            })
            .collect();
        let mut a = Takuri::new();
        let mut b = Takuri::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = Takuri::new();
        t.update(c(10.0, 10.05, 7.0, 10.0, 0));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
    }
}
