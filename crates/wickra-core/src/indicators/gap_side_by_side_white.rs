//! Gap Side-by-Side White Lines candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Gap Side-by-Side White Lines — a 3-bar continuation. After a gap away from the
/// first bar, two white candles of similar size open at roughly the same level
/// (side by side) and hold the gap open, signalling the trend resumes in the gap
/// direction.
///
/// ```text
/// bar2, bar3 both white
/// bar2 body gaps away from bar1 body            (up or down)
/// bar3 opens beside bar2                          (|open3 − open2| <= 0.1 · range2)
/// bar3 body is similar in size to bar2           (neither more than twice the other)
/// gap up   -> +1.0   (bullish continuation)
/// gap down -> −1.0   (bearish continuation — "downside" gap side-by-side white)
/// ```
///
/// Output is `+1.0` (gap up) or `−1.0` (gap down) when the pattern completes and
/// `0.0` otherwise. The first two bars always return `0.0` because the three-bar
/// window is not yet filled. Open-equality and body-similarity thresholds follow
/// the geometric house style rather than TA-Lib's rolling averages. Pattern-shape
/// check only — no trend filter is applied; combine with a trend indicator for
/// actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` bullish, `−1.0` bearish, `0.0` no pattern — so it
/// drops straight into a machine-learning feature matrix where the two gap
/// directions occupy a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, GapSideBySideWhite, Indicator};
///
/// let mut indicator = GapSideBySideWhite::new();
/// indicator.update(Candle::new(10.0, 11.1, 9.9, 11.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(13.0, 14.1, 12.9, 14.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(13.0, 14.1, 12.9, 14.0, 1.0, 2).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct GapSideBySideWhite {
    prev: Option<Candle>,
    prev_prev: Option<Candle>,
    has_emitted: bool,
}

impl GapSideBySideWhite {
    /// Construct a new Gap Side-by-Side White Lines detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            prev_prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for GapSideBySideWhite {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let bar1 = self.prev_prev;
        let bar2 = self.prev;
        self.prev_prev = self.prev;
        self.prev = Some(candle);
        let (Some(bar1), Some(bar2)) = (bar1, bar2) else {
            return Some(0.0);
        };
        let range2 = bar2.high - bar2.low;
        if range2 <= 0.0 {
            return Some(0.0);
        }
        // Both of the side-by-side bars must be white.
        if bar2.close <= bar2.open || candle.close <= candle.open {
            return Some(0.0);
        }
        // Side by side: opens level and bodies of comparable size.
        if (candle.open - bar2.open).abs() > 0.1 * range2 {
            return Some(0.0);
        }
        let body2 = bar2.close - bar2.open;
        let body3 = candle.close - candle.open;
        if body2 > 2.0 * body3 || body3 > 2.0 * body2 {
            return Some(0.0);
        }
        let bar1_top = bar1.open.max(bar1.close);
        let bar1_bottom = bar1.open.min(bar1.close);
        let bar2_bottom = bar2.open.min(bar2.close);
        let bar2_top = bar2.open.max(bar2.close);
        if bar2_bottom > bar1_top {
            return Some(1.0); // gap up -> bullish continuation
        }
        if bar2_top < bar1_bottom {
            return Some(-1.0); // gap down -> bearish continuation
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.prev = None;
        self.prev_prev = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        3
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "GapSideBySideWhite"
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
        let t = GapSideBySideWhite::new();
        assert_eq!(t.name(), "GapSideBySideWhite");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
    }

    #[test]
    fn gap_up_is_plus_one() {
        let mut t = GapSideBySideWhite::new();
        assert_eq!(t.update(c(10.0, 11.1, 9.9, 11.0, 0)), Some(0.0));
        assert_eq!(t.update(c(13.0, 14.1, 12.9, 14.0, 1)), Some(0.0));
        assert_eq!(t.update(c(13.0, 14.1, 12.9, 14.0, 2)), Some(1.0));
    }

    #[test]
    fn gap_down_is_minus_one() {
        let mut t = GapSideBySideWhite::new();
        assert_eq!(t.update(c(14.0, 14.1, 12.9, 13.0, 0)), Some(0.0));
        assert_eq!(t.update(c(10.0, 11.1, 9.9, 11.0, 1)), Some(0.0));
        assert_eq!(t.update(c(10.0, 11.1, 9.9, 11.0, 2)), Some(-1.0));
    }

    #[test]
    fn second_bar_black_yields_zero() {
        let mut t = GapSideBySideWhite::new();
        t.update(c(10.0, 11.1, 9.9, 11.0, 0));
        // bar3 is black -> not two white lines.
        t.update(c(13.0, 14.1, 12.9, 14.0, 1));
        assert_eq!(t.update(c(14.0, 14.1, 12.9, 13.0, 2)), Some(0.0));
    }

    #[test]
    fn not_side_by_side_yields_zero() {
        let mut t = GapSideBySideWhite::new();
        t.update(c(10.0, 11.1, 9.9, 11.0, 0));
        t.update(c(13.0, 14.1, 12.9, 14.0, 1));
        // bar3 opens far from bar2's open -> not side by side.
        assert_eq!(t.update(c(16.0, 17.1, 15.9, 17.0, 2)), Some(0.0));
    }

    #[test]
    fn no_gap_yields_zero() {
        let mut t = GapSideBySideWhite::new();
        t.update(c(10.0, 13.1, 9.9, 13.0, 0));
        // bar2 overlaps bar1 (no gap).
        t.update(c(12.0, 13.1, 11.9, 13.0, 1));
        assert_eq!(t.update(c(12.0, 13.1, 11.9, 13.0, 2)), Some(0.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = GapSideBySideWhite::new();
        assert_eq!(t.update(c(10.0, 11.1, 9.9, 11.0, 0)), Some(0.0));
        assert_eq!(t.update(c(13.0, 14.1, 12.9, 14.0, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64 * 3.0;
                c(base, base + 1.1, base - 0.1, base + 1.0, i)
            })
            .collect();
        let mut a = GapSideBySideWhite::new();
        let mut b = GapSideBySideWhite::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = GapSideBySideWhite::new();
        t.update(c(10.0, 11.1, 9.9, 11.0, 0));
        t.update(c(13.0, 14.1, 12.9, 14.0, 1));
        t.update(c(13.0, 14.1, 12.9, 14.0, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(10.0, 11.1, 9.9, 11.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = GapSideBySideWhite::new();
        t.update(c(10.0, 11.1, 9.9, 11.0, 0));
        // Flat second bar (range2 == 0) -> rejected.
        t.update(c(13.0, 13.0, 13.0, 13.0, 1));
        assert_eq!(t.update(c(13.0, 14.1, 12.9, 14.0, 2)), Some(0.0));
    }

    #[test]
    fn body_size_mismatch_yields_zero() {
        let mut t = GapSideBySideWhite::new();
        t.update(c(10.0, 11.1, 9.9, 11.0, 0));
        t.update(c(13.0, 16.0, 12.9, 15.0, 1)); // white, body 2.0
                                                // Level open, white, but its body is more than 2x smaller -> rejected.
        assert_eq!(t.update(c(13.0, 13.7, 12.9, 13.5, 2)), Some(0.0)); // body 0.5
    }
}
