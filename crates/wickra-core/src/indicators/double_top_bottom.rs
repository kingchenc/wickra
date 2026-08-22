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
///     let signal = indicator.update(c);
///     // Nothing is reported until three pivots exist to compare.
///     if i == 4 {
///         assert_eq!(signal, Some(-1.0));
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

    #[inline]
    fn update(&mut self, candle: Candle) -> Option<f64> {
        let advanced = self.swing.update(candle);
        let pivots = self.swing.pivots();
        // Too few pivots to form the shape at all: the indicator cannot
        // judge yet, which is what `None` means.
        if pivots.len() < 3 {
            return None;
        }
        self.has_emitted = true;
        // Armed, but this bar did not close a new pivot, so there is
        // nothing new to match against.
        if !advanced {
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

    #[inline]
    fn warmup_period(&self) -> usize {
        // Three confirmed pivots. The tracker seeds on the first bar without
        // confirming anything and can confirm at most one pivot per bar after
        // that, so the third arrives on the fourth bar at the earliest.
        4
    }

    #[inline]
    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    #[inline]
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
            .filter_map(|c| indicator.update(c))
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = DoubleTopBottom::new();
        assert_eq!(indicator.name(), "DoubleTopBottom");
        assert_eq!(indicator.warmup_period(), 4);
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
        assert_eq!(indicator.update(c), None);
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
