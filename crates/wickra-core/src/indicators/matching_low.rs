//! Matching Low candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Matching Low — a 2-bar bullish reversal. Two black candles in a decline close
/// at the *same* level: the second sell-off cannot push price any lower, so the
/// matching closes mark a support floor.
///
/// ```text
/// bar1, bar2 both black
/// equal closes  = |close2 − close1| <= 0.05 · mean(range1, range2)
/// ```
///
/// Output is `+1.0` when the pattern completes and `0.0` otherwise. Matching Low
/// is a single-direction (bullish-only) reversal, so it never emits `−1.0`. The
/// first bar always returns `0.0` because the two-bar window is not yet filled.
/// The close-equality tolerance follows the geometric house style rather than
/// TA-Lib's rolling averages. Pattern-shape check only — no trend filter is
/// applied; combine with a trend indicator for actionable signals.
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
/// use wickra_core::{Candle, Indicator, MatchingLow};
///
/// let mut indicator = MatchingLow::new();
/// indicator.update(Candle::new(15.0, 15.1, 9.9, 10.0, 1.0, 0).unwrap());
/// let out = indicator
///     .update(Candle::new(13.0, 13.1, 9.9, 10.0, 1.0, 1).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct MatchingLow {
    prev: Option<Candle>,
    has_emitted: bool,
}

impl MatchingLow {
    /// Construct a new Matching Low detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for MatchingLow {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let prev = self.prev;
        self.prev = Some(candle);
        let Some(bar1) = prev else {
            return Some(0.0);
        };
        let mean_range = 0.5 * ((bar1.high - bar1.low) + (candle.high - candle.low));
        let tol = 0.05 * mean_range;
        if bar1.close < bar1.open
            && candle.close < candle.open
            && (candle.close - bar1.close).abs() <= tol
        {
            return Some(1.0);
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
        "MatchingLow"
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
        let t = MatchingLow::new();
        assert_eq!(t.name(), "MatchingLow");
        assert_eq!(t.warmup_period(), 2);
        assert!(!t.is_ready());
    }

    #[test]
    fn matching_low_is_plus_one() {
        let mut t = MatchingLow::new();
        assert_eq!(t.update(c(15.0, 15.1, 9.9, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(13.0, 13.1, 9.9, 10.0, 1)), Some(1.0));
    }

    #[test]
    fn different_close_yields_zero() {
        let mut t = MatchingLow::new();
        t.update(c(15.0, 15.1, 9.9, 10.0, 0));
        // Second close well away from the first.
        assert_eq!(t.update(c(13.0, 13.1, 11.4, 11.5, 1)), Some(0.0));
    }

    #[test]
    fn second_bar_white_yields_zero() {
        let mut t = MatchingLow::new();
        t.update(c(15.0, 15.1, 9.9, 10.0, 0));
        assert_eq!(t.update(c(9.0, 10.1, 8.9, 10.0, 1)), Some(0.0));
    }

    #[test]
    fn first_bar_returns_zero() {
        let mut t = MatchingLow::new();
        assert_eq!(t.update(c(15.0, 15.1, 9.9, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 - i as f64;
                c(base + 2.0, base + 2.1, base - 0.1, base, i)
            })
            .collect();
        let mut a = MatchingLow::new();
        let mut b = MatchingLow::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = MatchingLow::new();
        t.update(c(15.0, 15.1, 9.9, 10.0, 0));
        t.update(c(13.0, 13.1, 9.9, 10.0, 1));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(15.0, 15.1, 9.9, 10.0, 0)), Some(0.0));
    }
}
