//! Flag / Pennant continuation chart pattern.

use crate::indicators::pattern_swing::{SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Maximum size of the consolidation swing relative to the pole for a
/// flag/pennant to qualify — the pullback must retrace less than half the pole.
const MAX_RETRACE_FRACTION: f64 = 0.5;

/// Flag / Pennant — a brief consolidation against a sharp prior move (the
/// "pole"), resolving in the pole's direction.
///
/// Built on confirmed swing pivots ([`SWING_THRESHOLD`] = 5%); evaluated from the
/// last three pivots `pole_start → pole_end → consolidation`:
///
/// ```text
/// pole      = |pole_end − pole_start|     (the sharp impulse)
/// pullback  = |consolidation − pole_end|  (the shallow counter-move)
/// qualifies when pullback < 0.5 · pole
/// bull flag : pole_end is a swing high → +1 (up-pole, continuation up)
/// bear flag : pole_end is a swing low  → -1 (down-pole, continuation down)
/// ```
///
/// The detector fires on the bar that confirms the consolidation pivot (the flag
/// is complete; the breakout is expected to follow). Output is `+1.0` / `-1.0` /
/// `0.0`; never `None`.
#[derive(Debug, Clone)]
pub struct FlagPennant {
    swing: SwingTracker,
    has_emitted: bool,
}

impl FlagPennant {
    /// Construct a new Flag / Pennant detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 3),
            has_emitted: false,
        }
    }
}

impl Default for FlagPennant {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for FlagPennant {
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
        let n = pivots.len();
        let pole_start = pivots[n - 3];
        let pole_end = pivots[n - 2];
        let consolidation = pivots[n - 1];
        let pole = (pole_end.price - pole_start.price).abs();
        let pullback = (consolidation.price - pole_end.price).abs();

        if pole > 0.0 && pullback < MAX_RETRACE_FRACTION * pole {
            // pole_end a high → up-pole → bull flag; a low → bear flag.
            return Some(if pole_end.direction > 0.0 { 1.0 } else { -1.0 });
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.swing.reset();
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        // Three confirmed pivots; the earliest confirmation of the third is bar 4.
        4
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "FlagPennant"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = FlagPennant::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = FlagPennant::new();
        assert_eq!(indicator.name(), "FlagPennant");
        assert_eq!(indicator.warmup_period(), 4);
        assert!(!indicator.is_ready());
        assert!(!FlagPennant::default().is_ready());
    }

    #[test]
    fn bull_flag_is_plus_one() {
        // Up-pole 100 → 140 (40), shallow pullback to 130 (10 < 20) → bull flag.
        let out = run(&[150.0, 100.0, 140.0, 130.0]);
        assert_eq!(*out.last().unwrap(), 1.0);
    }

    #[test]
    fn bear_flag_is_minus_one() {
        // Down-pole 140 → 100 (40), shallow pullback to 110 (10 < 20) → bear flag.
        let out = run(&[140.0, 100.0, 110.0]);
        assert_eq!(*out.last().unwrap(), -1.0);
    }

    #[test]
    fn deep_pullback_is_not_a_flag() {
        // Pole 100 → 140 (40) but pullback to 104 (36 > 20) → not a flag.
        let out = run(&[150.0, 100.0, 140.0, 104.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = FlagPennant::new();
        for c in candles_for_pivots(&[150.0, 100.0, 140.0]) {
            let _ = indicator.update(c);
        }
        indicator.reset();
        assert!(!indicator.is_ready());
        let c = Candle::new(99.5, 100.0, 99.5, 99.5, 1.0, 0).unwrap();
        assert_eq!(indicator.update(c), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = candles_for_pivots(&[150.0, 100.0, 140.0, 130.0]);
        let mut a = FlagPennant::new();
        let mut b = FlagPennant::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
