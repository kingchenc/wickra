//! Doji Star candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Doji Star — a 2-bar reversal warning. A long trending body is followed by a
/// doji whose tiny body gaps away in the direction of the trend, the indecision
/// hinting the move is about to turn.
///
/// ```text
/// long body  = |close − open| >= 0.5 * (high − low)        (bar1)
/// doji       = |close − open| <= 0.1 * (high − low)        (bar2)
/// bullish (+1.0): bar1 black, doji body gaps DOWN below it  (max(o2,c2) < close1)
/// bearish (−1.0): bar1 white, doji body gaps UP above it    (min(o2,c2) > close1)
/// ```
///
/// Output is `+1.0` (bullish star, after a black bar) or `−1.0` (bearish star,
/// after a white bar) when the pattern completes, and `0.0` otherwise. The first
/// bar always returns `0.0` because the two-bar window is not yet filled. Doji
/// thresholds follow the geometric house style (fixed half-range body for the
/// long bar, tenth-range body for the doji) rather than TA-Lib's rolling
/// averages. Pattern-shape check only — no trend filter is applied; combine with
/// a trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` bullish, `−1.0` bearish, `0.0` no pattern — so it
/// drops straight into a machine-learning feature matrix where the bullish and
/// bearish variants occupy a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, DojiStar, Indicator};
///
/// let mut indicator = DojiStar::new();
/// // Long black bar, then a doji gapping down -> bullish star.
/// indicator.update(Candle::new(20.0, 20.2, 14.8, 15.0, 1.0, 0).unwrap());
/// let out = indicator
///     .update(Candle::new(13.0, 13.1, 12.9, 13.0, 1.0, 1).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct DojiStar {
    prev: Option<Candle>,
    has_emitted: bool,
}

impl DojiStar {
    /// Construct a new Doji Star detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for DojiStar {
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
        let body1 = bar1.close - bar1.open;
        if body1.abs() < 0.5 * range1 {
            return Some(0.0);
        }
        if (candle.close - candle.open).abs() > 0.1 * range2 {
            return Some(0.0);
        }
        let doji_top = candle.open.max(candle.close);
        let doji_bottom = candle.open.min(candle.close);
        // Bullish: long black bar, doji body gaps down below it.
        if body1 < 0.0 && doji_top < bar1.close {
            return Some(1.0);
        }
        // Bearish: long white bar, doji body gaps up above it.
        if body1 > 0.0 && doji_bottom > bar1.close {
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
        "DojiStar"
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
        let t = DojiStar::new();
        assert_eq!(t.name(), "DojiStar");
        assert_eq!(t.warmup_period(), 2);
        assert!(!t.is_ready());
    }

    #[test]
    fn bullish_doji_star_is_plus_one() {
        let mut t = DojiStar::new();
        assert_eq!(t.update(c(20.0, 20.2, 14.8, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(13.0, 13.1, 12.9, 13.0, 1)), Some(1.0));
    }

    #[test]
    fn bearish_doji_star_is_minus_one() {
        let mut t = DojiStar::new();
        assert_eq!(t.update(c(15.0, 20.2, 14.8, 20.0, 0)), Some(0.0));
        assert_eq!(t.update(c(22.0, 22.1, 21.9, 22.0, 1)), Some(-1.0));
    }

    #[test]
    fn second_bar_not_doji_yields_zero() {
        let mut t = DojiStar::new();
        t.update(c(20.0, 20.2, 14.8, 15.0, 0));
        // Wide body, not a doji.
        assert_eq!(t.update(c(13.0, 13.2, 11.0, 11.5, 1)), Some(0.0));
    }

    #[test]
    fn no_gap_yields_zero() {
        let mut t = DojiStar::new();
        t.update(c(20.0, 20.2, 14.8, 15.0, 0));
        // Doji overlaps bar1's body (no gap down).
        assert_eq!(t.update(c(16.0, 16.1, 15.9, 16.0, 1)), Some(0.0));
    }

    #[test]
    fn short_first_body_yields_zero() {
        let mut t = DojiStar::new();
        // First bar body too short to be the "long" leg.
        t.update(c(20.0, 24.0, 16.0, 19.5, 0));
        assert_eq!(t.update(c(13.0, 13.1, 12.9, 13.0, 1)), Some(0.0));
    }

    #[test]
    fn first_bar_returns_zero() {
        let mut t = DojiStar::new();
        assert_eq!(t.update(c(20.0, 20.2, 14.8, 15.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                if i % 2 == 0 {
                    c(base + 5.0, base + 5.2, base - 0.2, base, i)
                } else {
                    c(base - 3.0, base - 2.9, base - 3.1, base - 3.0, i)
                }
            })
            .collect();
        let mut a = DojiStar::new();
        let mut b = DojiStar::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = DojiStar::new();
        t.update(c(20.0, 20.2, 14.8, 15.0, 0));
        t.update(c(13.0, 13.1, 12.9, 13.0, 1));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(20.0, 20.2, 14.8, 15.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = DojiStar::new();
        t.update(c(20.0, 20.2, 14.8, 15.0, 0));
        // Flat second bar (high == low) -> zero-range guard.
        assert_eq!(t.update(c(13.0, 13.0, 13.0, 13.0, 1)), Some(0.0));
    }
}
