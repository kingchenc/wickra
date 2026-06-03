//! Rectangle / Range chart pattern.

use crate::indicators::pattern_swing::{
    approx_equal, recent_legs, SwingTracker, LEVEL_TOLERANCE, SWING_THRESHOLD,
};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Rectangle / Range — price oscillating between a roughly horizontal support
/// and resistance, a mean-reversion (range-trading) structure.
///
/// Built on confirmed swing pivots ([`SWING_THRESHOLD`] = 5%); recognised when the
/// last two highs and the last two lows are each flat within [`LEVEL_TOLERANCE`]
/// (3%):
///
/// ```text
/// flat highs (resistance) AND flat lows (support):
///   last pivot a low  → +1  (a bounce off support — buy the range)
///   last pivot a high → -1  (a rejection at resistance — sell the range)
/// ```
///
/// Unlike the breakout patterns the rectangle is range-bound, so the sign
/// encodes the actionable mean-reversion direction of the just-confirmed touch.
/// Output is `+1.0` / `-1.0` / `0.0`; never `None`.
#[derive(Debug, Clone)]
pub struct RectangleRange {
    swing: SwingTracker,
    has_emitted: bool,
}

impl RectangleRange {
    /// Construct a new Rectangle / Range detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 4),
            has_emitted: false,
        }
    }
}

impl Default for RectangleRange {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for RectangleRange {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        if !self.swing.update(candle) {
            return Some(0.0);
        }
        let pivots = self.swing.pivots();
        if pivots.len() < 4 {
            return Some(0.0);
        }
        let (high_old, high_new, low_old, low_new) = recent_legs(pivots);
        let flat_highs = approx_equal(high_old, high_new, LEVEL_TOLERANCE);
        let flat_lows = approx_equal(low_old, low_new, LEVEL_TOLERANCE);
        if flat_highs && flat_lows {
            let last_is_high = pivots[pivots.len() - 1].direction > 0.0;
            return Some(if last_is_high { -1.0 } else { 1.0 });
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.swing.reset();
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        // Four confirmed pivots; the earliest confirmation of the fourth is bar 5.
        5
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "RectangleRange"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = RectangleRange::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = RectangleRange::new();
        assert_eq!(indicator.name(), "RectangleRange");
        assert_eq!(indicator.warmup_period(), 5);
        assert!(!indicator.is_ready());
        assert!(!RectangleRange::default().is_ready());
    }

    #[test]
    fn range_bounce_off_support_is_plus_one() {
        // Flat highs (120, 121), flat lows (100, 99); last pivot a low → +1.
        let out = run(&[120.0, 100.0, 121.0, 99.0]);
        assert_eq!(*out.last().unwrap(), 1.0);
    }

    #[test]
    fn range_rejection_at_resistance_is_minus_one() {
        // Same range but ending on a high pivot → -1.
        let out = run(&[130.0, 100.0, 120.0, 99.0, 121.0]);
        assert_eq!(*out.last().unwrap(), -1.0);
    }

    #[test]
    fn trending_highs_are_not_a_rectangle() {
        // Rising highs break the flat-resistance requirement → no rectangle.
        let out = run(&[120.0, 100.0, 140.0, 99.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = RectangleRange::new();
        for c in candles_for_pivots(&[120.0, 100.0, 121.0]) {
            let _ = indicator.update(c);
        }
        indicator.reset();
        assert!(!indicator.is_ready());
        let c = Candle::new(99.5, 100.0, 99.5, 99.5, 1.0, 0).unwrap();
        assert_eq!(indicator.update(c), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = candles_for_pivots(&[120.0, 100.0, 121.0, 99.0]);
        let mut a = RectangleRange::new();
        let mut b = RectangleRange::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
