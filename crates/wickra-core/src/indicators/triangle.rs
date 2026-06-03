//! Triangle (ascending / descending / symmetrical) chart pattern.

use crate::indicators::pattern_swing::{
    approx_equal, recent_legs, SwingTracker, LEVEL_TOLERANCE, SWING_THRESHOLD,
};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Triangle — a consolidation pattern bounded by two converging trendlines,
/// detected from the two most recent swing highs and lows.
///
/// Built on confirmed swing pivots ([`SWING_THRESHOLD`] = 5%); evaluated on every
/// bar that confirms a new pivot once four pivots exist:
///
/// ```text
/// ascending   : flat highs   + rising lows    → +1 (bullish bias)
/// descending  : falling highs + flat lows      → -1 (bearish bias)
/// symmetrical : falling highs + rising lows     → +1 if the last pivot is a low
///                                                 (an up-bounce), else -1
/// ```
///
/// "Flat" means the two highs (or lows) are within [`LEVEL_TOLERANCE`] (3%) of
/// each other; "rising"/"falling" means they differ by more than that tolerance.
/// The symmetrical case is directionally neutral, so its sign follows the
/// momentum of the most recently confirmed swing. Output is `+1.0` / `-1.0` /
/// `0.0`; never `None`.
#[derive(Debug, Clone)]
pub struct Triangle {
    swing: SwingTracker,
    has_emitted: bool,
}

impl Triangle {
    /// Construct a new Triangle detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 4),
            has_emitted: false,
        }
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for Triangle {
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
        let rising_lows = low_new > low_old * (1.0 + LEVEL_TOLERANCE);
        let falling_highs = high_new < high_old * (1.0 - LEVEL_TOLERANCE);
        let last_is_high = pivots[pivots.len() - 1].direction > 0.0;

        if flat_highs && rising_lows {
            return Some(1.0); // ascending
        }
        if falling_highs && flat_lows {
            return Some(-1.0); // descending
        }
        if falling_highs && rising_lows {
            // symmetrical: lean with the latest swing's momentum.
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
        "Triangle"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = Triangle::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = Triangle::new();
        assert_eq!(indicator.name(), "Triangle");
        assert_eq!(indicator.warmup_period(), 5);
        assert!(!indicator.is_ready());
        assert!(!Triangle::default().is_ready());
    }

    #[test]
    fn ascending_triangle_is_plus_one() {
        // Flat highs (120, 120), rising lows (100 → 110).
        let out = run(&[130.0, 100.0, 120.0, 110.0, 120.0]);
        assert_eq!(*out.last().unwrap(), 1.0);
    }

    #[test]
    fn descending_triangle_is_minus_one() {
        // Falling highs (120 → 110), flat lows (100, 99).
        let out = run(&[120.0, 100.0, 110.0, 99.0]);
        assert_eq!(*out.last().unwrap(), -1.0);
    }

    #[test]
    fn symmetrical_triangle_ending_low_is_plus_one() {
        // Falling highs (120 → 113), rising lows (100 → 106); last pivot a low.
        let out = run(&[120.0, 100.0, 113.0, 106.0]);
        assert_eq!(*out.last().unwrap(), 1.0);
    }

    #[test]
    fn symmetrical_triangle_ending_high_is_minus_one() {
        // Same convergence but ending on a high pivot.
        let out = run(&[130.0, 100.0, 120.0, 106.0, 113.0]);
        assert_eq!(*out.last().unwrap(), -1.0);
    }

    #[test]
    fn expanding_swings_are_not_a_triangle() {
        // Rising highs and falling lows (broadening) → no converging triangle.
        let out = run(&[110.0, 100.0, 130.0, 80.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = Triangle::new();
        for c in candles_for_pivots(&[130.0, 100.0, 120.0]) {
            let _ = indicator.update(c);
        }
        indicator.reset();
        assert!(!indicator.is_ready());
        let c = Candle::new(99.5, 100.0, 99.5, 99.5, 1.0, 0).unwrap();
        assert_eq!(indicator.update(c), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = candles_for_pivots(&[130.0, 100.0, 120.0, 110.0, 120.0]);
        let mut a = Triangle::new();
        let mut b = Triangle::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
