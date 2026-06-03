//! Wedge (rising / falling) reversal chart pattern.

use crate::indicators::pattern_swing::{recent_legs, SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Wedge — a pattern where both trendlines slope the same way but converge,
/// signalling exhaustion of the prevailing move.
///
/// Built on confirmed swing pivots ([`SWING_THRESHOLD`] = 5%); evaluated from the
/// last two swing highs and lows:
///
/// ```text
/// rising wedge  : highs rising  AND lows rising,  lows rising faster  → -1 (bearish)
/// falling wedge : highs falling AND lows falling, highs falling faster → +1 (bullish)
/// ```
///
/// Convergence is the key: in a rising wedge the lower trendline climbs faster
/// than the upper (the range narrows from below); in a falling wedge the upper
/// trendline drops faster than the lower. Output is `+1.0` / `-1.0` / `0.0`;
/// never `None`.
#[derive(Debug, Clone)]
pub struct Wedge {
    swing: SwingTracker,
    has_emitted: bool,
}

impl Wedge {
    /// Construct a new Wedge detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 4),
            has_emitted: false,
        }
    }
}

impl Default for Wedge {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for Wedge {
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
        let high_slope = high_new - high_old;
        let low_slope = low_new - low_old;

        // Rising wedge: both lines slope up, lower line steeper (converging) → bearish.
        if high_slope > 0.0 && low_slope > 0.0 && low_slope > high_slope {
            return Some(-1.0);
        }
        // Falling wedge: both lines slope down, upper line steeper → bullish.
        if high_slope < 0.0 && low_slope < 0.0 && high_slope < low_slope {
            return Some(1.0);
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
        "Wedge"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = Wedge::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = Wedge::new();
        assert_eq!(indicator.name(), "Wedge");
        assert_eq!(indicator.warmup_period(), 5);
        assert!(!indicator.is_ready());
        assert!(!Wedge::default().is_ready());
    }

    #[test]
    fn rising_wedge_is_minus_one() {
        // Highs 100 → 103 (+3), lows 90 → 94 (+4, steeper) → rising wedge.
        let out = run(&[110.0, 90.0, 100.0, 94.0, 103.0]);
        assert_eq!(*out.last().unwrap(), -1.0);
    }

    #[test]
    fn falling_wedge_is_plus_one() {
        // Highs 120 → 106 (-14, steeper), lows 100 → 99 (-1) → falling wedge.
        let out = run(&[120.0, 100.0, 106.0, 99.0]);
        assert_eq!(*out.last().unwrap(), 1.0);
    }

    #[test]
    fn diverging_swings_are_not_a_wedge() {
        // Rising highs but falling lows (broadening) → no wedge.
        let out = run(&[110.0, 100.0, 130.0, 80.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = Wedge::new();
        for c in candles_for_pivots(&[110.0, 90.0, 100.0]) {
            let _ = indicator.update(c);
        }
        indicator.reset();
        assert!(!indicator.is_ready());
        let c = Candle::new(99.5, 100.0, 99.5, 99.5, 1.0, 0).unwrap();
        assert_eq!(indicator.update(c), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = candles_for_pivots(&[110.0, 90.0, 100.0, 94.0, 103.0]);
        let mut a = Wedge::new();
        let mut b = Wedge::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
