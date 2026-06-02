//! Separating Lines candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Separating Lines — a 2-bar continuation. After a counter-trend candle, the next
/// candle of the *opposite* colour opens right back at the prior open and runs as
/// an opening marubozu in the trend direction, so the trend "separates" from the
/// pullback and resumes.
///
/// ```text
/// long body = |close − open| >= 0.5 * (high − low)
/// bar1, bar2 opposite colours
/// bar2 opens at bar1's open                 (|open2 − open1| <= 0.05 · range1)
/// bar2 is a long opening marubozu in its direction
///   white bar2: open2 == low2  (no lower shadow)  -> +1.0
///   black bar2: open2 == high2 (no upper shadow)  -> −1.0
/// ```
///
/// Output is `+1.0` (bullish continuation) or `−1.0` (bearish continuation) when
/// the pattern completes and `0.0` otherwise. The first bar always returns `0.0`
/// because the two-bar window is not yet filled. Open-equality and marubozu
/// thresholds follow the geometric house style rather than TA-Lib's rolling
/// averages. Pattern-shape check only — no trend filter is applied; combine with
/// a trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` bullish, `−1.0` bearish, `0.0` no pattern — so it
/// drops straight into a machine-learning feature matrix where the two directions
/// occupy a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, SeparatingLines};
///
/// let mut indicator = SeparatingLines::new();
/// indicator.update(Candle::new(12.0, 12.1, 9.9, 10.0, 1.0, 0).unwrap());
/// let out = indicator
///     .update(Candle::new(12.0, 14.1, 12.0, 14.0, 1.0, 1).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct SeparatingLines {
    prev: Option<Candle>,
    has_emitted: bool,
}

impl SeparatingLines {
    /// Construct a new Separating Lines detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for SeparatingLines {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let prev = self.prev;
        self.prev = Some(candle);
        let Some(bar1) = prev else {
            return Some(0.0);
        };
        let range1 = bar1.high - bar1.low;
        let range2 = candle.high - candle.low;
        if range1 <= 0.0 || range2 <= 0.0 {
            return Some(0.0);
        }
        // Opens must coincide.
        if (candle.open - bar1.open).abs() > 0.05 * range1 {
            return Some(0.0);
        }
        let body2 = candle.close - candle.open;
        if body2.abs() < 0.5 * range2 {
            return Some(0.0); // bar2 must be a long body
        }
        let tol = 0.05 * range2;
        // Bullish: bar1 black, bar2 a long white opening marubozu (no lower wick).
        if bar1.close < bar1.open && body2 > 0.0 && candle.open - candle.low <= tol {
            return Some(1.0);
        }
        // Bearish: bar1 white, bar2 a long black opening marubozu (no upper wick).
        if bar1.close > bar1.open && body2 < 0.0 && candle.high - candle.open <= tol {
            return Some(-1.0);
        }
        Some(0.0)
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
        "SeparatingLines"
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
        let t = SeparatingLines::new();
        assert_eq!(t.name(), "SeparatingLines");
        assert_eq!(t.warmup_period(), 2);
        assert!(!t.is_ready());
    }

    #[test]
    fn bullish_separating_lines_is_plus_one() {
        let mut t = SeparatingLines::new();
        assert_eq!(t.update(c(12.0, 12.1, 9.9, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(12.0, 14.1, 12.0, 14.0, 1)), Some(1.0));
    }

    #[test]
    fn bearish_separating_lines_is_minus_one() {
        let mut t = SeparatingLines::new();
        assert_eq!(t.update(c(10.0, 12.1, 9.9, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(10.0, 10.0, 7.9, 8.0, 1)), Some(-1.0));
    }

    #[test]
    fn same_color_yields_zero() {
        let mut t = SeparatingLines::new();
        // Both white -> not separating (need opposite colours).
        t.update(c(12.0, 14.1, 11.9, 14.0, 0));
        assert_eq!(t.update(c(12.0, 14.1, 12.0, 14.0, 1)), Some(0.0));
    }

    #[test]
    fn different_open_yields_zero() {
        let mut t = SeparatingLines::new();
        t.update(c(12.0, 12.1, 9.9, 10.0, 0));
        // bar2 opens far from bar1's open.
        assert_eq!(t.update(c(13.0, 15.1, 13.0, 15.0, 1)), Some(0.0));
    }

    #[test]
    fn opening_shadow_yields_zero() {
        let mut t = SeparatingLines::new();
        t.update(c(12.0, 12.1, 9.9, 10.0, 0));
        // White bar2 but it has a lower shadow -> not an opening marubozu.
        assert_eq!(t.update(c(12.0, 14.1, 11.0, 14.0, 1)), Some(0.0));
    }

    #[test]
    fn first_bar_returns_zero() {
        let mut t = SeparatingLines::new();
        assert_eq!(t.update(c(12.0, 12.1, 9.9, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 2.0, base, base + 1.9, i)
            })
            .collect();
        let mut a = SeparatingLines::new();
        let mut b = SeparatingLines::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = SeparatingLines::new();
        t.update(c(12.0, 12.1, 9.9, 10.0, 0));
        t.update(c(12.0, 14.1, 12.0, 14.0, 1));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(12.0, 12.1, 9.9, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = SeparatingLines::new();
        // Flat first bar (range1 == 0) -> rejected.
        t.update(c(10.0, 10.0, 10.0, 10.0, 0));
        assert_eq!(t.update(c(10.0, 12.0, 9.0, 11.0, 1)), Some(0.0));
    }

    #[test]
    fn short_second_body_yields_zero() {
        let mut t = SeparatingLines::new();
        t.update(c(10.0, 12.0, 8.0, 9.0, 0));
        // Opens coincide but bar2's body is too short to be a separating line.
        assert_eq!(t.update(c(10.0, 11.0, 9.0, 10.1, 1)), Some(0.0));
    }
}
