//! Stick Sandwich candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Stick Sandwich — a 3-bar bullish reversal. A black candle is followed by a
/// white candle that trades entirely above the first close, then a second black
/// candle drives price back down to close at the same level as the first. The
/// matching closes "sandwich" the white candle and mark a support floor.
///
/// ```text
/// bar1 black, bar2 white, bar3 black
/// bar2 trades above bar1's close: low2 > close1
/// matching closes: |close3 − close1| <= 0.1 * (high1 − low1)
/// ```
///
/// Output is `+1.0` when the pattern completes and `0.0` otherwise. Stick Sandwich
/// is a single-direction (bullish-only) reversal, so it never emits `−1.0`. The
/// first two bars always return `0.0` because the three-bar window is not yet
/// filled. The matching-close tolerance follows the geometric house style (a fixed
/// fraction of the first bar's range) rather than TA-Lib's rolling averages.
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
/// use wickra_core::{Candle, Indicator, StickSandwich};
///
/// let mut indicator = StickSandwich::new();
/// indicator.update(Candle::new(12.0, 12.1, 9.9, 10.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(10.5, 11.6, 10.4, 11.5, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(11.5, 11.6, 9.9, 10.0, 1.0, 2).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct StickSandwich {
    c1: Option<Candle>,
    c2: Option<Candle>,
    has_emitted: bool,
}

impl StickSandwich {
    /// Construct a new Stick Sandwich detector.
    pub const fn new() -> Self {
        Self {
            c1: None,
            c2: None,
            has_emitted: false,
        }
    }
}

impl Indicator for StickSandwich {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let bar1 = self.c1;
        let bar2 = self.c2;
        self.c1 = self.c2;
        self.c2 = Some(candle);
        let (Some(bar1), Some(bar2)) = (bar1, bar2) else {
            return Some(0.0);
        };
        // bar1 black, bar2 white, bar3 black.
        if bar1.close >= bar1.open || bar2.close <= bar2.open || candle.close >= candle.open {
            return Some(0.0);
        }
        // The white candle trades entirely above the first close.
        if bar2.low <= bar1.close {
            return Some(0.0);
        }
        // The two black candles close at the same level (the sandwich).
        let range1 = bar1.high - bar1.low;
        if (candle.close - bar1.close).abs() <= 0.1 * range1 {
            return Some(1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.c1 = None;
        self.c2 = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        3
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "StickSandwich"
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
        let t = StickSandwich::new();
        assert_eq!(t.name(), "StickSandwich");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
    }

    #[test]
    fn stick_sandwich_is_plus_one() {
        let mut t = StickSandwich::new();
        assert_eq!(t.update(c(12.0, 12.1, 9.9, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(10.5, 11.6, 10.4, 11.5, 1)), Some(0.0));
        assert_eq!(t.update(c(11.5, 11.6, 9.9, 10.0, 2)), Some(1.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = StickSandwich::new();
        assert_eq!(t.update(c(12.0, 12.1, 9.9, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(10.5, 11.6, 10.4, 11.5, 1)), Some(0.0));
    }

    #[test]
    fn first_candle_not_black_yields_zero() {
        let mut t = StickSandwich::new();
        // bar1 white.
        t.update(c(9.9, 12.1, 9.8, 10.0, 0));
        t.update(c(10.5, 11.6, 10.4, 11.5, 1));
        assert_eq!(t.update(c(11.5, 11.6, 9.9, 10.0, 2)), Some(0.0));
    }

    #[test]
    fn middle_candle_not_white_yields_zero() {
        let mut t = StickSandwich::new();
        t.update(c(12.0, 12.1, 9.9, 10.0, 0));
        // bar2 black.
        t.update(c(11.5, 11.6, 10.4, 10.5, 1));
        assert_eq!(t.update(c(11.5, 11.6, 9.9, 10.0, 2)), Some(0.0));
    }

    #[test]
    fn third_candle_not_black_yields_zero() {
        let mut t = StickSandwich::new();
        t.update(c(12.0, 12.1, 9.9, 10.0, 0));
        t.update(c(10.5, 11.6, 10.4, 11.5, 1));
        // bar3 white.
        assert_eq!(t.update(c(9.9, 11.6, 9.8, 10.0, 2)), Some(0.0));
    }

    #[test]
    fn middle_low_not_above_first_close_yields_zero() {
        let mut t = StickSandwich::new();
        t.update(c(12.0, 12.1, 9.9, 10.0, 0));
        // bar2 white but dips below bar1's close.
        t.update(c(10.5, 11.6, 9.0, 11.5, 1));
        assert_eq!(t.update(c(11.5, 11.6, 9.9, 10.0, 2)), Some(0.0));
    }

    #[test]
    fn mismatched_closes_yield_zero() {
        let mut t = StickSandwich::new();
        t.update(c(12.0, 12.1, 9.9, 10.0, 0));
        t.update(c(10.5, 11.6, 10.4, 11.5, 1));
        // bar3 black but closes well away from bar1's close.
        assert_eq!(t.update(c(11.5, 11.6, 7.9, 8.0, 2)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 2.0, base + 2.1, base - 0.1, base, i)
            })
            .collect();
        let mut a = StickSandwich::new();
        let mut b = StickSandwich::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = StickSandwich::new();
        t.update(c(12.0, 12.1, 9.9, 10.0, 0));
        t.update(c(10.5, 11.6, 10.4, 11.5, 1));
        t.update(c(11.5, 11.6, 9.9, 10.0, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(12.0, 12.1, 9.9, 10.0, 0)), Some(0.0));
    }
}
