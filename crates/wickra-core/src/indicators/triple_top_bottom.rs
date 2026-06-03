//! Triple Top / Triple Bottom reversal chart pattern.

use crate::indicators::pattern_swing::{
    approx_equal, SwingTracker, LEVEL_TOLERANCE, SWING_THRESHOLD,
};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Triple Top / Triple Bottom — a three-peak (or three-trough) reversal pattern,
/// a stronger variant of the double top/bottom.
///
/// Built on confirmed swing pivots ([`SWING_THRESHOLD`] = 5%). A pattern is
/// recognised on the bar that confirms the **third** matching extreme:
///
/// ```text
/// triple top    : High₁ , Low , High₂ , Low , High₃   High₁ ≈ High₂ ≈ High₃ → -1
/// triple bottom : Low₁  , High, Low₂  , High, Low₃     Low₁  ≈ Low₂  ≈ Low₃  → +1
/// ```
///
/// The three same-direction extremes (positions `n-5`, `n-3`, `n-1` in the pivot
/// history) must all lie within [`LEVEL_TOLERANCE`] (3%) of one another.
///
/// Output is `+1.0` for a triple bottom, `-1.0` for a triple top, and `0.0`
/// otherwise; never `None`.
#[derive(Debug, Clone)]
pub struct TripleTopBottom {
    swing: SwingTracker,
    has_emitted: bool,
}

impl TripleTopBottom {
    /// Construct a new Triple Top / Triple Bottom detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 5),
            has_emitted: false,
        }
    }
}

impl Default for TripleTopBottom {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for TripleTopBottom {
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
        let first = pivots[n - 5];
        let middle = pivots[n - 3];
        let last = pivots[n - 1];
        let outer_match = approx_equal(first.price, middle.price, LEVEL_TOLERANCE);
        let inner_match = approx_equal(middle.price, last.price, LEVEL_TOLERANCE);
        if outer_match && inner_match {
            return Some(if last.direction > 0.0 { -1.0 } else { 1.0 });
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.swing.reset();
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        // Five confirmed pivots are needed; the earliest bar that can confirm a
        // fifth pivot is the sixth.
        6
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "TripleTopBottom"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = TripleTopBottom::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = TripleTopBottom::new();
        assert_eq!(indicator.name(), "TripleTopBottom");
        assert_eq!(indicator.warmup_period(), 6);
        assert!(!indicator.is_ready());
        assert!(!TripleTopBottom::default().is_ready());
    }

    #[test]
    fn triple_top_is_minus_one() {
        // Three ~equal highs (120, 121, 119) → triple top on the third.
        let out = run(&[120.0, 100.0, 121.0, 99.0, 119.0]);
        assert_eq!(*out.last().unwrap(), -1.0);
        assert!(out[..out.len() - 1].iter().all(|&x| x == 0.0));
    }

    #[test]
    fn triple_bottom_is_plus_one() {
        // Lead high then three ~equal lows (100, 99, 101) → triple bottom.
        let out = run(&[130.0, 100.0, 120.0, 99.0, 122.0, 101.0]);
        assert_eq!(*out.last().unwrap(), 1.0);
    }

    #[test]
    fn unequal_third_peak_does_not_trigger() {
        // Third high (140) diverges from the first two (120, 121) → no pattern.
        let out = run(&[120.0, 100.0, 121.0, 99.0, 140.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
        assert!(out.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = TripleTopBottom::new();
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
        let candles = candles_for_pivots(&[120.0, 100.0, 121.0, 99.0, 119.0]);
        let mut a = TripleTopBottom::new();
        let mut b = TripleTopBottom::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
