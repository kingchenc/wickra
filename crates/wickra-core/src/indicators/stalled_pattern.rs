//! Stalled Pattern (Deliberation) candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Stalled Pattern (also called Deliberation) — a 3-bar bearish reversal warning.
/// Two long white candles push higher, then a small-bodied white candle opens at
/// or near the top of the second body and barely advances — the rally is running
/// out of breath, hinting that buyers are losing control.
///
/// ```text
/// long body  = |close − open| >= 0.5 * (high − low)
/// small body = |close − open| <= 0.3 * (high − low)
/// bar1, bar2 long white; bar3 small white
/// rising closes: close3 > close2 > close1
/// bar3 rides the shoulder: open3 >= close2 − 0.1 * (high2 − low2)
/// ```
///
/// Output is `−1.0` when the pattern completes and `0.0` otherwise. Stalled Pattern
/// is a single-direction (bearish-only) warning, so it never emits `+1.0`. The
/// first two bars always return `0.0` because the three-bar window is not yet
/// filled. Body thresholds follow the geometric house style rather than TA-Lib's
/// rolling averages. Pattern-shape check only — no trend filter is applied; combine
/// with a trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `−1.0` bearish, `0.0` no pattern — so it drops straight into
/// a machine-learning feature matrix as a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, StalledPattern};
///
/// let mut indicator = StalledPattern::new();
/// indicator.update(Candle::new(10.0, 12.05, 9.9, 12.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(11.0, 14.05, 10.9, 14.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(14.0, 14.6, 13.95, 14.15, 1.0, 2).unwrap());
/// assert_eq!(out, Some(-1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct StalledPattern {
    c1: Option<Candle>,
    c2: Option<Candle>,
    has_emitted: bool,
}

impl StalledPattern {
    /// Construct a new Stalled Pattern detector.
    pub const fn new() -> Self {
        Self {
            c1: None,
            c2: None,
            has_emitted: false,
        }
    }
}

impl Indicator for StalledPattern {
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
        let range1 = bar1.high - bar1.low;
        let range2 = bar2.high - bar2.low;
        let range3 = candle.high - candle.low;
        if range1 <= 0.0 || range2 <= 0.0 || range3 <= 0.0 {
            return Some(0.0);
        }
        // All three candles are white.
        if bar1.close <= bar1.open || bar2.close <= bar2.open || candle.close <= candle.open {
            return Some(0.0);
        }
        // Rising closes.
        if candle.close <= bar2.close || bar2.close <= bar1.close {
            return Some(0.0);
        }
        // bar1 and bar2 are long bodies.
        if bar1.close - bar1.open < 0.5 * range1 || bar2.close - bar2.open < 0.5 * range2 {
            return Some(0.0);
        }
        // bar3 is a small body.
        if candle.close - candle.open > 0.3 * range3 {
            return Some(0.0);
        }
        // bar3 opens at or near the top of bar2's body (rides the shoulder).
        if candle.open >= bar2.close - 0.1 * range2 {
            return Some(-1.0);
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
        "StalledPattern"
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
        let t = StalledPattern::new();
        assert_eq!(t.name(), "StalledPattern");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
    }

    #[test]
    fn stalled_pattern_is_minus_one() {
        let mut t = StalledPattern::new();
        assert_eq!(t.update(c(10.0, 12.05, 9.9, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(11.0, 14.05, 10.9, 14.0, 1)), Some(0.0));
        assert_eq!(t.update(c(14.0, 14.6, 13.95, 14.15, 2)), Some(-1.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = StalledPattern::new();
        assert_eq!(t.update(c(10.0, 12.05, 9.9, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(11.0, 14.05, 10.9, 14.0, 1)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = StalledPattern::new();
        t.update(c(10.0, 12.05, 9.9, 12.0, 0));
        t.update(c(11.0, 14.05, 10.9, 14.0, 1));
        // bar3 has zero range.
        assert_eq!(t.update(c(14.0, 14.0, 14.0, 14.0, 2)), Some(0.0));
    }

    #[test]
    fn non_white_yields_zero() {
        let mut t = StalledPattern::new();
        t.update(c(10.0, 12.05, 9.9, 12.0, 0));
        t.update(c(11.0, 14.05, 10.9, 14.0, 1));
        // bar3 is black.
        assert_eq!(t.update(c(14.2, 14.6, 13.95, 14.05, 2)), Some(0.0));
    }

    #[test]
    fn non_rising_closes_yield_zero() {
        let mut t = StalledPattern::new();
        t.update(c(10.0, 12.05, 9.9, 12.0, 0));
        t.update(c(11.0, 14.05, 10.9, 14.0, 1));
        // bar3 closes below bar2's close (white but not advancing).
        assert_eq!(t.update(c(13.5, 14.0, 13.45, 13.6, 2)), Some(0.0));
    }

    #[test]
    fn short_first_bodies_yield_zero() {
        let mut t = StalledPattern::new();
        // bar1 is white but its body is short relative to range.
        t.update(c(11.5, 14.0, 10.0, 12.0, 0));
        t.update(c(11.0, 14.05, 10.9, 14.0, 1));
        assert_eq!(t.update(c(14.0, 14.6, 13.95, 14.15, 2)), Some(0.0));
    }

    #[test]
    fn large_third_body_yields_zero() {
        let mut t = StalledPattern::new();
        t.update(c(10.0, 12.05, 9.9, 12.0, 0));
        t.update(c(11.0, 14.05, 10.9, 14.0, 1));
        // bar3 has a large body (not a small stalling candle).
        assert_eq!(t.update(c(14.0, 16.05, 13.95, 16.0, 2)), Some(0.0));
    }

    #[test]
    fn third_bar_off_shoulder_yields_zero() {
        let mut t = StalledPattern::new();
        t.update(c(10.0, 12.05, 9.9, 12.0, 0));
        t.update(c(11.0, 14.05, 10.9, 14.0, 1));
        // bar3 is a small white candle but opens well below bar2's close,
        // so it is not riding the shoulder.
        assert_eq!(t.update(c(13.6, 14.1, 12.55, 14.05, 2)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 2.05, base - 0.1, base + 2.0, i)
            })
            .collect();
        let mut a = StalledPattern::new();
        let mut b = StalledPattern::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = StalledPattern::new();
        t.update(c(10.0, 12.05, 9.9, 12.0, 0));
        t.update(c(11.0, 14.05, 10.9, 14.0, 1));
        t.update(c(14.0, 14.6, 13.95, 14.15, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(10.0, 12.05, 9.9, 12.0, 0)), Some(0.0));
    }
}
