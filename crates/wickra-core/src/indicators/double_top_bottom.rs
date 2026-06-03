//! Double Top / Double Bottom reversal chart pattern.

use crate::indicators::pattern_swing::{
    approx_equal, SwingTracker, LEVEL_TOLERANCE, SWING_THRESHOLD,
};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Double Top / Double Bottom — a two-peak (or two-trough) reversal pattern.
///
/// The detector tracks confirmed swing pivots (a non-repainting percent-threshold
/// zig-zag, [`SWING_THRESHOLD`] = 5%). A pattern is recognised on the bar that
/// confirms the **second** matching extreme:
///
/// ```text
/// double top    : … High₁ , Low , High₂   with  High₁ ≈ High₂   → -1 (bearish)
/// double bottom : … Low₁  , High , Low₂    with  Low₁  ≈ Low₂    → +1 (bullish)
/// ```
///
/// Two extremes count as the same level when they are within
/// [`LEVEL_TOLERANCE`] (3%) of each other. Because pivots strictly alternate
/// high/low, the trough between the twin tops (or the peak between the twin
/// bottoms) is guaranteed to sit beyond both, so no extra separation check is
/// needed.
///
/// Output is `+1.0` for a double bottom, `-1.0` for a double top, and `0.0` on
/// every other bar (including warmup and bars that confirm a pivot which does
/// not complete the pattern). Like the candlestick family this detector never
/// returns `None`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, DoubleTopBottom, Indicator};
///
/// let mut indicator = DoubleTopBottom::new();
/// for (i, &(high, low)) in [
///     (100.0, 99.5),
///     (120.0, 119.5),
///     (110.0, 100.0), // confirms the first top at 120
///     (120.0, 119.0), // confirms the trough at 100
///     (115.0, 110.0), // confirms the second top at 120 → double top
/// ]
/// .iter()
/// .enumerate()
/// {
///     let c = Candle::new(low, high, low, low, 1.0, i as i64).unwrap();
///     let signal = indicator.update(c).unwrap();
///     if i == 4 {
///         assert_eq!(signal, -1.0);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct DoubleTopBottom {
    swing: SwingTracker,
    has_emitted: bool,
}

impl DoubleTopBottom {
    /// Construct a new Double Top / Double Bottom detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 3),
            has_emitted: false,
        }
    }
}

impl Default for DoubleTopBottom {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for DoubleTopBottom {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        if !self.swing.update(candle) {
            return Some(0.0);
        }
        let pivots = self.swing.pivots();
        if pivots.len() < 3 {
            return Some(0.0);
        }
        let first = pivots[pivots.len() - 3];
        let last = pivots[pivots.len() - 1];
        if approx_equal(first.price, last.price, LEVEL_TOLERANCE) {
            // `last` is the just-confirmed extreme: a high → double top (bearish),
            // a low → double bottom (bullish).
            return Some(if last.direction > 0.0 { -1.0 } else { 1.0 });
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.swing.reset();
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        // The first complete pattern needs three confirmed pivots; the earliest
        // bar that can confirm a third pivot is the fifth.
        5
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "DoubleTopBottom"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = DoubleTopBottom::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = DoubleTopBottom::new();
        assert_eq!(indicator.name(), "DoubleTopBottom");
        assert_eq!(indicator.warmup_period(), 5);
        assert!(!indicator.is_ready());
        assert!(!DoubleTopBottom::default().is_ready());
    }

    #[test]
    fn double_top_is_minus_one() {
        // Twin highs 120 / 120 with a 100 trough → double top on the second.
        let out = run(&[120.0, 100.0, 120.0]);
        assert_eq!(*out.last().unwrap(), -1.0);
        // All earlier bars are warmup / non-completing.
        assert!(out[..out.len() - 1].iter().all(|&x| x == 0.0));
    }

    #[test]
    fn double_bottom_is_plus_one() {
        // Lead high, then twin lows 100 / 99 around a 120 peak → double bottom.
        let out = run(&[130.0, 100.0, 120.0, 99.0]);
        assert_eq!(*out.last().unwrap(), 1.0);
    }

    #[test]
    fn unequal_tops_do_not_trigger() {
        // Second top 140 diverges from the first (120) → no pattern.
        let out = run(&[120.0, 100.0, 140.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
        assert!(out.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = DoubleTopBottom::new();
        for c in candles_for_pivots(&[120.0, 100.0, 120.0]) {
            let _ = indicator.update(c);
        }
        indicator.reset();
        assert!(!indicator.is_ready());
        let c = Candle::new(99.5, 100.0, 99.5, 99.5, 1.0, 0).unwrap();
        assert_eq!(indicator.update(c), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = candles_for_pivots(&[120.0, 100.0, 120.0]);
        let mut a = DoubleTopBottom::new();
        let mut b = DoubleTopBottom::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
