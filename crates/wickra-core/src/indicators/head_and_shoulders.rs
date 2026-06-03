//! Head-and-Shoulders (and Inverse) reversal chart pattern.

use crate::indicators::pattern_swing::{
    approx_equal, SwingTracker, LEVEL_TOLERANCE, SWING_THRESHOLD,
};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Head-and-Shoulders / Inverse Head-and-Shoulders — a five-pivot reversal
/// pattern with a central extreme (the head) flanked by two lower/higher
/// shoulders at a similar level, joined by a roughly horizontal neckline.
///
/// Built on confirmed swing pivots ([`SWING_THRESHOLD`] = 5%); recognised on the
/// bar that confirms the right shoulder:
///
/// ```text
/// head-and-shoulders top (bearish, -1):
///   LeftShoulder(high) , Trough , Head(high) , Trough , RightShoulder(high)
///   Head > both shoulders ; LeftShoulder ≈ RightShoulder ; Trough₁ ≈ Trough₂
///
/// inverse head-and-shoulders (bullish, +1):
///   LeftShoulder(low) , Peak , Head(low) , Peak , RightShoulder(low)
///   Head < both shoulders ; LeftShoulder ≈ RightShoulder ; Peak₁ ≈ Peak₂
/// ```
///
/// The shoulders must match within [`LEVEL_TOLERANCE`] (3%) and the two neckline
/// points within the same tolerance. Output is `-1.0` for a top, `+1.0` for an
/// inverse, `0.0` otherwise; never `None`.
#[derive(Debug, Clone)]
pub struct HeadAndShoulders {
    swing: SwingTracker,
    has_emitted: bool,
}

impl HeadAndShoulders {
    /// Construct a new Head-and-Shoulders detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 5),
            has_emitted: false,
        }
    }
}

impl Default for HeadAndShoulders {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for HeadAndShoulders {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        if !self.swing.update(candle) {
            return Some(0.0);
        }
        let pivots = self.swing.pivots();
        if pivots.len() < 5 {
            return Some(0.0);
        }
        let n = pivots.len();
        let left_shoulder = pivots[n - 5];
        let neck_1 = pivots[n - 4];
        let head = pivots[n - 3];
        let neck_2 = pivots[n - 2];
        let right_shoulder = pivots[n - 1];

        let shoulders_match =
            approx_equal(left_shoulder.price, right_shoulder.price, LEVEL_TOLERANCE);
        let neckline_flat = approx_equal(neck_1.price, neck_2.price, LEVEL_TOLERANCE);
        let head_is_peak = head.price > left_shoulder.price && head.price > right_shoulder.price;
        let head_is_trough = head.price < left_shoulder.price && head.price < right_shoulder.price;
        let frame_matches = shoulders_match && neckline_flat;

        if right_shoulder.direction > 0.0 {
            // Head-and-shoulders top: head is the highest of the three highs.
            if head_is_peak && frame_matches {
                return Some(-1.0);
            }
        } else if head_is_trough && frame_matches {
            // Inverse: head is the lowest of the three lows.
            return Some(1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.swing.reset();
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        // Five confirmed pivots; the earliest confirmation of the fifth is bar 6.
        6
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "HeadAndShoulders"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = HeadAndShoulders::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = HeadAndShoulders::new();
        assert_eq!(indicator.name(), "HeadAndShoulders");
        assert_eq!(indicator.warmup_period(), 6);
        assert!(!indicator.is_ready());
        assert!(!HeadAndShoulders::default().is_ready());
    }

    #[test]
    fn head_and_shoulders_top_is_minus_one() {
        // LS 100, trough 90, head 120, trough 92, RS 101.
        let out = run(&[100.0, 90.0, 120.0, 92.0, 101.0]);
        assert_eq!(*out.last().unwrap(), -1.0);
        assert!(out[..out.len() - 1].iter().all(|&x| x == 0.0));
    }

    #[test]
    fn inverse_head_and_shoulders_is_plus_one() {
        // Lead high then LS 100, peak 110, head 80, peak 108, RS 101.
        let out = run(&[130.0, 100.0, 110.0, 80.0, 108.0, 101.0]);
        assert_eq!(*out.last().unwrap(), 1.0);
    }

    #[test]
    fn mismatched_shoulders_do_not_trigger() {
        // Right shoulder (115) far from left (100) → no pattern.
        let out = run(&[100.0, 90.0, 130.0, 92.0, 115.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn inverse_mismatched_shoulders_do_not_trigger() {
        // Inverse shape (ends on a low) but the right shoulder (90) diverges from
        // the left (100) → enters the inverse branch yet reports no pattern.
        let out = run(&[130.0, 100.0, 110.0, 80.0, 108.0, 90.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn equal_highs_without_taller_head_do_not_trigger() {
        // Three equal highs (no dominant head) → not H&S (that is a triple top).
        let out = run(&[120.0, 90.0, 120.0, 92.0, 120.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = HeadAndShoulders::new();
        for c in candles_for_pivots(&[100.0, 90.0, 120.0]) {
            let _ = indicator.update(c);
        }
        indicator.reset();
        assert!(!indicator.is_ready());
        let c = Candle::new(99.5, 100.0, 99.5, 99.5, 1.0, 0).unwrap();
        assert_eq!(indicator.update(c), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = candles_for_pivots(&[100.0, 90.0, 120.0, 92.0, 101.0]);
        let mut a = HeadAndShoulders::new();
        let mut b = HeadAndShoulders::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
