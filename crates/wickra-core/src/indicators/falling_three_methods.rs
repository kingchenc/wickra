//! Falling Three Methods candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Falling Three Methods — a 5-bar bearish continuation. A long black candle is
/// followed by three small bars that drift up but stay inside its range (a brief
/// rest), then a second long black candle closes below the first, resuming the
/// decline.
///
/// ```text
/// long body = |close − open| >= 0.5 * (high − low)
/// bar1 black & long
/// bar2, bar3, bar4 small bodies, each contained within bar1's high/low range
/// bar5 black, closing below bar1's close
/// ```
///
/// Output is `−1.0` when the pattern completes and `0.0` otherwise. Falling Three
/// Methods is a single-direction (bearish-only) continuation, so it never emits
/// `+1.0`. The first four bars always return `0.0` because the five-bar window is
/// not yet filled. Body thresholds follow the geometric house style rather than
/// TA-Lib's rolling averages. Pattern-shape check only — no trend filter is
/// applied; combine with a trend indicator for actionable signals.
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
/// use wickra_core::{Candle, FallingThreeMethods, Indicator};
///
/// let mut indicator = FallingThreeMethods::new();
/// indicator.update(Candle::new(15.0, 15.1, 9.9, 10.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(11.0, 12.1, 10.9, 12.0, 1.0, 1).unwrap());
/// indicator.update(Candle::new(11.5, 12.6, 11.4, 12.5, 1.0, 2).unwrap());
/// indicator.update(Candle::new(12.0, 13.1, 11.9, 13.0, 1.0, 3).unwrap());
/// let out = indicator
///     .update(Candle::new(12.5, 12.6, 8.9, 9.0, 1.0, 4).unwrap());
/// assert_eq!(out, Some(-1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct FallingThreeMethods {
    c1: Option<Candle>,
    c2: Option<Candle>,
    c3: Option<Candle>,
    c4: Option<Candle>,
    has_emitted: bool,
}

impl FallingThreeMethods {
    /// Construct a new Falling Three Methods detector.
    pub const fn new() -> Self {
        Self {
            c1: None,
            c2: None,
            c3: None,
            c4: None,
            has_emitted: false,
        }
    }
}

impl Indicator for FallingThreeMethods {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let bar1 = self.c1;
        let bar2 = self.c2;
        let bar3 = self.c3;
        let bar4 = self.c4;
        self.c1 = self.c2;
        self.c2 = self.c3;
        self.c3 = self.c4;
        self.c4 = Some(candle);
        let (Some(bar1), Some(bar2), Some(bar3), Some(bar4)) = (bar1, bar2, bar3, bar4) else {
            return Some(0.0);
        };
        let range1 = bar1.high - bar1.low;
        if range1 <= 0.0 {
            return Some(0.0);
        }
        let body1 = bar1.open - bar1.close;
        if body1 < 0.5 * range1 {
            return Some(0.0); // bar1 must be a long black body
        }
        // The three middle bars stay within bar1's range with smaller bodies.
        for mid in [bar2, bar3, bar4] {
            if (mid.close - mid.open).abs() >= body1 || mid.high > bar1.high || mid.low < bar1.low {
                return Some(0.0);
            }
        }
        // bar5 is a black candle closing below bar1's close.
        if candle.close < candle.open && candle.close < bar1.close {
            return Some(-1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.c1 = None;
        self.c2 = None;
        self.c3 = None;
        self.c4 = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        5
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "FallingThreeMethods"
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
        let t = FallingThreeMethods::new();
        assert_eq!(t.name(), "FallingThreeMethods");
        assert_eq!(t.warmup_period(), 5);
        assert!(!t.is_ready());
    }

    #[test]
    fn falling_three_methods_is_minus_one() {
        let mut t = FallingThreeMethods::new();
        assert_eq!(t.update(c(15.0, 15.1, 9.9, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(11.0, 12.1, 10.9, 12.0, 1)), Some(0.0));
        assert_eq!(t.update(c(11.5, 12.6, 11.4, 12.5, 2)), Some(0.0));
        assert_eq!(t.update(c(12.0, 13.1, 11.9, 13.0, 3)), Some(0.0));
        assert_eq!(t.update(c(12.5, 12.6, 8.9, 9.0, 4)), Some(-1.0));
    }

    #[test]
    fn middle_bar_breaks_range_yields_zero() {
        let mut t = FallingThreeMethods::new();
        t.update(c(15.0, 15.1, 9.9, 10.0, 0));
        t.update(c(11.0, 12.1, 10.9, 12.0, 1));
        // bar3 pokes below bar1's low.
        t.update(c(11.5, 12.6, 9.0, 12.5, 2));
        t.update(c(12.0, 13.1, 11.9, 13.0, 3));
        assert_eq!(t.update(c(12.5, 12.6, 8.9, 9.0, 4)), Some(0.0));
    }

    #[test]
    fn bar5_not_new_low_yields_zero() {
        let mut t = FallingThreeMethods::new();
        t.update(c(15.0, 15.1, 9.9, 10.0, 0));
        t.update(c(11.0, 12.1, 10.9, 12.0, 1));
        t.update(c(11.5, 12.6, 11.4, 12.5, 2));
        t.update(c(12.0, 13.1, 11.9, 13.0, 3));
        // bar5 black but closes above bar1's close.
        assert_eq!(t.update(c(12.5, 12.6, 10.4, 10.5, 4)), Some(0.0));
    }

    #[test]
    fn first_four_bars_return_zero() {
        let mut t = FallingThreeMethods::new();
        assert_eq!(t.update(c(15.0, 15.1, 9.9, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(11.0, 12.1, 10.9, 12.0, 1)), Some(0.0));
        assert_eq!(t.update(c(11.5, 12.6, 11.4, 12.5, 2)), Some(0.0));
        assert_eq!(t.update(c(12.0, 13.1, 11.9, 13.0, 3)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 200.0 - i as f64;
                c(base + 5.0, base + 5.1, base - 0.1, base, i)
            })
            .collect();
        let mut a = FallingThreeMethods::new();
        let mut b = FallingThreeMethods::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = FallingThreeMethods::new();
        t.update(c(15.0, 15.1, 9.9, 10.0, 0));
        t.update(c(11.0, 12.1, 10.9, 12.0, 1));
        t.update(c(11.5, 12.6, 11.4, 12.5, 2));
        t.update(c(12.0, 13.1, 11.9, 13.0, 3));
        t.update(c(12.5, 12.6, 8.9, 9.0, 4));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(15.0, 15.1, 9.9, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_first_bar_yields_zero() {
        let mut t = FallingThreeMethods::new();
        // Flat first bar (range1 == 0) -> rejected.
        t.update(c(10.0, 10.0, 10.0, 10.0, 0));
        t.update(c(11.0, 12.1, 10.9, 12.0, 1));
        t.update(c(11.5, 12.6, 11.4, 12.5, 2));
        t.update(c(12.0, 13.1, 11.9, 13.0, 3));
        assert_eq!(t.update(c(12.5, 12.6, 8.9, 9.0, 4)), Some(0.0));
    }

    #[test]
    fn short_first_body_yields_zero() {
        let mut t = FallingThreeMethods::new();
        // bar1 has a wide range but a tiny body -> not a long black body.
        t.update(c(10.0, 16.0, 9.0, 10.2, 0));
        t.update(c(11.0, 12.1, 10.9, 12.0, 1));
        t.update(c(11.5, 12.6, 11.4, 12.5, 2));
        t.update(c(12.0, 13.1, 11.9, 13.0, 3));
        assert_eq!(t.update(c(12.5, 12.6, 8.9, 9.0, 4)), Some(0.0));
    }
}
