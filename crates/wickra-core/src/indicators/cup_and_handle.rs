//! Cup-and-Handle (and Inverse) continuation chart pattern.

use crate::indicators::pattern_swing::{
    approx_equal, SwingTracker, LEVEL_TOLERANCE, SWING_THRESHOLD,
};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Cup-and-Handle / Inverse — a rounded base (the cup) followed by a shallow
/// pullback (the handle) near the rim, then a breakout in the cup's direction.
///
/// Built on confirmed swing pivots ([`SWING_THRESHOLD`] = 5%) and read from the
/// last four pivots:
///
/// ```text
/// cup-and-handle (bullish, +1):  Rim(high) , Cup(low) , Rim(high) , Handle(low)
///   the two rims match (±3%) ; the handle low sits ABOVE the cup low (a shallow
///   pullback) and below the right rim
///
/// inverse (bearish, -1):         Rim(low) , Cap(high) , Rim(low) , Handle(high)
///   the two rims match ; the handle high sits BELOW the cap high and above the
///   right rim
/// ```
///
/// The shallow handle (closer to the rim than the cup extreme) is what
/// distinguishes a cup-and-handle from a plain double bottom/top. Output is
/// `+1.0` / `-1.0` / `0.0`; never `None`.
#[derive(Debug, Clone)]
pub struct CupAndHandle {
    swing: SwingTracker,
    has_emitted: bool,
}

impl CupAndHandle {
    /// Construct a new Cup-and-Handle detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 4),
            has_emitted: false,
        }
    }
}

impl Default for CupAndHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for CupAndHandle {
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
        let n = pivots.len();
        let rim_left = pivots[n - 4];
        let extreme = pivots[n - 3];
        let rim_right = pivots[n - 2];
        let handle = pivots[n - 1];
        let rims_match = approx_equal(rim_left.price, rim_right.price, LEVEL_TOLERANCE);

        if handle.direction < 0.0 {
            // Bullish cup-and-handle: rims are highs, cup is the low between them,
            // handle is a shallow low above the cup but below the right rim.
            if rims_match && handle.price > extreme.price && handle.price < rim_right.price {
                return Some(1.0);
            }
        } else if rims_match && handle.price < extreme.price && handle.price > rim_right.price {
            // Inverse: rims are lows, cap is the high, handle a shallow high.
            return Some(-1.0);
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
        "CupAndHandle"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = CupAndHandle::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = CupAndHandle::new();
        assert_eq!(indicator.name(), "CupAndHandle");
        assert_eq!(indicator.warmup_period(), 5);
        assert!(!indicator.is_ready());
        assert!(!CupAndHandle::default().is_ready());
    }

    #[test]
    fn cup_and_handle_is_plus_one() {
        // Rims 120/121, cup 90 (deep), handle 110 (shallow, above the cup).
        let out = run(&[120.0, 90.0, 121.0, 110.0]);
        assert_eq!(*out.last().unwrap(), 1.0);
    }

    #[test]
    fn inverse_cup_and_handle_is_minus_one() {
        // Lead high then rims 100/101, cap 130, handle 110 (below cap, above rim).
        let out = run(&[140.0, 100.0, 130.0, 101.0, 110.0]);
        assert_eq!(*out.last().unwrap(), -1.0);
    }

    #[test]
    fn deep_handle_is_not_cup_and_handle() {
        // Handle (85) below the cup low (90) → a double bottom, not cup-and-handle.
        let out = run(&[120.0, 90.0, 121.0, 85.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn inverse_with_mismatched_rims_does_not_trigger() {
        // Inverse shape (ends high) but the rims (100 / 90) diverge → enters the
        // inverse branch yet reports no pattern.
        let out = run(&[140.0, 100.0, 130.0, 90.0, 110.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = CupAndHandle::new();
        for c in candles_for_pivots(&[120.0, 90.0, 121.0]) {
            let _ = indicator.update(c);
        }
        indicator.reset();
        assert!(!indicator.is_ready());
        let c = Candle::new(99.5, 100.0, 99.5, 99.5, 1.0, 0).unwrap();
        assert_eq!(indicator.update(c), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = candles_for_pivots(&[120.0, 90.0, 121.0, 110.0]);
        let mut a = CupAndHandle::new();
        let mut b = CupAndHandle::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
