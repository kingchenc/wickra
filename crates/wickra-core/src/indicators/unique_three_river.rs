//! Unique Three River candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Unique Three River (Bottom) — a 3-bar bullish reversal. A long black candle is
/// followed by a smaller black candle whose body sits inside the first but whose
/// long lower shadow probes a new low, then a small white candle that stays below
/// the second body. The fresh low that fails to hold marks an exhausted decline.
///
/// ```text
/// bar1 long black: open1 − close1 >= 0.5 * (high1 − low1)
/// bar2 black, body inside bar1's body, with a new low (low2 < low1)
/// bar3 small white, contained below bar2's body (high3 <= close2)
/// small body: close3 − open3 <= 0.3 * (high3 − low3)
/// ```
///
/// Output is `+1.0` when the pattern completes and `0.0` otherwise. Unique Three
/// River is a single-direction (bullish-only) reversal, so it never emits `−1.0`.
/// The first two bars always return `0.0` because the three-bar window is not yet
/// filled. Body thresholds follow the geometric house style rather than TA-Lib's
/// rolling averages. Pattern-shape check only — no trend filter is applied; combine
/// with a trend indicator for actionable signals.
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
/// use wickra_core::{Candle, Indicator, UniqueThreeRiver};
///
/// let mut indicator = UniqueThreeRiver::new();
/// indicator.update(Candle::new(15.0, 15.1, 10.0, 10.5, 1.0, 0).unwrap());
/// indicator.update(Candle::new(14.0, 14.1, 9.0, 11.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(10.2, 10.9, 9.5, 10.4, 1.0, 2).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct UniqueThreeRiver {
    c1: Option<Candle>,
    c2: Option<Candle>,
    has_emitted: bool,
}

impl UniqueThreeRiver {
    /// Construct a new Unique Three River detector.
    pub const fn new() -> Self {
        Self {
            c1: None,
            c2: None,
            has_emitted: false,
        }
    }
}

impl Indicator for UniqueThreeRiver {
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
        // bar1 is a long black body.
        if bar1.open <= bar1.close {
            return Some(0.0);
        }
        let range1 = bar1.high - bar1.low;
        if bar1.open - bar1.close < 0.5 * range1 {
            return Some(0.0);
        }
        // bar2 is black, its body inside bar1's body, with a new low.
        if bar2.open <= bar2.close {
            return Some(0.0);
        }
        if bar2.open > bar1.open || bar2.close < bar1.close {
            return Some(0.0);
        }
        if bar2.low >= bar1.low {
            return Some(0.0);
        }
        // bar3 is a small white candle contained below bar2's body.
        if candle.close <= candle.open {
            return Some(0.0);
        }
        let range3 = candle.high - candle.low;
        if candle.close - candle.open > 0.3 * range3 {
            return Some(0.0);
        }
        if candle.high > bar2.close {
            return Some(0.0);
        }
        Some(1.0)
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
        "UniqueThreeRiver"
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
        let t = UniqueThreeRiver::new();
        assert_eq!(t.name(), "UniqueThreeRiver");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
    }

    #[test]
    fn unique_three_river_is_plus_one() {
        let mut t = UniqueThreeRiver::new();
        assert_eq!(t.update(c(15.0, 15.1, 10.0, 10.5, 0)), Some(0.0));
        assert_eq!(t.update(c(14.0, 14.1, 9.0, 11.0, 1)), Some(0.0));
        assert_eq!(t.update(c(10.2, 10.9, 9.5, 10.4, 2)), Some(1.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = UniqueThreeRiver::new();
        assert_eq!(t.update(c(15.0, 15.1, 10.0, 10.5, 0)), Some(0.0));
        assert_eq!(t.update(c(14.0, 14.1, 9.0, 11.0, 1)), Some(0.0));
    }

    #[test]
    fn first_bar_not_black_yields_zero() {
        let mut t = UniqueThreeRiver::new();
        // bar1 white.
        t.update(c(10.5, 15.1, 10.0, 15.0, 0));
        t.update(c(14.0, 14.1, 9.0, 11.0, 1));
        assert_eq!(t.update(c(10.2, 10.9, 9.5, 10.4, 2)), Some(0.0));
    }

    #[test]
    fn first_bar_short_body_yields_zero() {
        let mut t = UniqueThreeRiver::new();
        // bar1 black but its body is short relative to range.
        t.update(c(15.0, 15.1, 10.0, 14.5, 0));
        t.update(c(14.0, 14.1, 9.0, 11.0, 1));
        assert_eq!(t.update(c(10.2, 10.9, 9.5, 10.4, 2)), Some(0.0));
    }

    #[test]
    fn second_bar_not_black_yields_zero() {
        let mut t = UniqueThreeRiver::new();
        t.update(c(15.0, 15.1, 10.0, 10.5, 0));
        // bar2 white.
        t.update(c(11.0, 14.1, 9.0, 13.0, 1));
        assert_eq!(t.update(c(10.2, 10.9, 9.5, 10.4, 2)), Some(0.0));
    }

    #[test]
    fn second_bar_not_inside_yields_zero() {
        let mut t = UniqueThreeRiver::new();
        t.update(c(15.0, 15.1, 10.0, 10.5, 0));
        // bar2 black but opens above bar1's open -> body not inside.
        t.update(c(16.0, 16.1, 9.0, 11.0, 1));
        assert_eq!(t.update(c(10.2, 10.9, 9.5, 10.4, 2)), Some(0.0));
    }

    #[test]
    fn second_bar_no_new_low_yields_zero() {
        let mut t = UniqueThreeRiver::new();
        t.update(c(15.0, 15.1, 10.0, 10.5, 0));
        // bar2 black, inside, but does not make a new low.
        t.update(c(14.0, 14.1, 10.5, 11.0, 1));
        assert_eq!(t.update(c(10.2, 10.9, 9.5, 10.4, 2)), Some(0.0));
    }

    #[test]
    fn third_bar_not_white_yields_zero() {
        let mut t = UniqueThreeRiver::new();
        t.update(c(15.0, 15.1, 10.0, 10.5, 0));
        t.update(c(14.0, 14.1, 9.0, 11.0, 1));
        // bar3 black.
        assert_eq!(t.update(c(10.6, 10.9, 9.5, 10.2, 2)), Some(0.0));
    }

    #[test]
    fn third_bar_large_body_yields_zero() {
        let mut t = UniqueThreeRiver::new();
        t.update(c(15.0, 15.1, 10.0, 10.5, 0));
        t.update(c(14.0, 14.1, 9.0, 11.0, 1));
        // bar3 white but with a large body.
        assert_eq!(t.update(c(9.6, 10.9, 9.5, 10.8, 2)), Some(0.0));
    }

    #[test]
    fn third_bar_not_below_second_yields_zero() {
        let mut t = UniqueThreeRiver::new();
        t.update(c(15.0, 15.1, 10.0, 10.5, 0));
        t.update(c(14.0, 14.1, 9.0, 11.0, 1));
        // bar3 small white but pokes above bar2's close.
        assert_eq!(t.update(c(10.5, 11.5, 10.4, 10.7, 2)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 200.0 - i as f64;
                c(base, base + 0.1, base - 5.2, base - 5.0, i)
            })
            .collect();
        let mut a = UniqueThreeRiver::new();
        let mut b = UniqueThreeRiver::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = UniqueThreeRiver::new();
        t.update(c(15.0, 15.1, 10.0, 10.5, 0));
        t.update(c(14.0, 14.1, 9.0, 11.0, 1));
        t.update(c(10.2, 10.9, 9.5, 10.4, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(15.0, 15.1, 10.0, 10.5, 0)), Some(0.0));
    }
}
