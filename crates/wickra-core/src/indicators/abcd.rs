//! AB=CD harmonic pattern.

use crate::indicators::pattern_swing::{approx_equal, ratios_in, SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// AB=CD — the simplest four-point harmonic pattern: an A→B leg, a B→C
/// retracement, and a C→D leg that mirrors A→B in length:
///
/// ```text
/// BC / AB ∈ [0.382, 0.886]   (C retraces AB)
/// CD / BC ∈ [1.13, 2.618]    (D extends BC)
/// AB ≈ CD (within 10%)        (the two legs are equal — the defining symmetry)
/// ```
///
/// Read from the last four confirmed pivots `A-B-C-D`. Output is `+1.0`
/// (bullish, D a swing low), `-1.0` (bearish, D a swing high), or `0.0`; never
/// `None`. See `crates/wickra-core/src/indicators/abcd.rs`.
#[derive(Debug, Clone)]
pub struct Abcd {
    swing: SwingTracker,
    has_emitted: bool,
}

impl Abcd {
    /// Construct a new AB=CD detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 4),
            has_emitted: false,
        }
    }
}

impl Default for Abcd {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for Abcd {
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
        let len = pivots.len();
        let pa = pivots[len - 4];
        let pb = pivots[len - 3];
        let pc = pivots[len - 2];
        let pd = pivots[len - 1];
        let ab = (pb.price - pa.price).abs();
        let bc = (pc.price - pb.price).abs();
        let cd = (pd.price - pc.price).abs();
        let ratios_ok = ratios_in(&[(bc / ab, 0.382, 0.886), (cd / bc, 1.13, 2.618)]);
        let legs_equal = approx_equal(ab, cd, 0.10);
        if ratios_ok && legs_equal {
            return Some(if pd.direction < 0.0 { 1.0 } else { -1.0 });
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.swing.reset();
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        5
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "Abcd"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = Abcd::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = Abcd::new();
        assert_eq!(indicator.name(), "Abcd");
        assert_eq!(indicator.warmup_period(), 5);
        assert!(!indicator.is_ready());
        assert!(!Abcd::default().is_ready());
    }

    #[test]
    fn bullish_abcd_is_plus_one() {
        // AB = 40 down, BC = 24.7 up (0.618), CD = 40 down → AB = CD.
        let out = run(&[140.0, 100.0, 124.7, 84.7]);
        assert_eq!(*out.last().unwrap(), 1.0);
        assert!(out[..out.len() - 1].iter().all(|&x| x == 0.0));
    }

    #[test]
    fn bearish_abcd_is_minus_one() {
        let out = run(&[150.0, 100.0, 140.0, 115.3, 155.3]);
        assert_eq!(*out.last().unwrap(), -1.0);
    }

    #[test]
    fn unequal_legs_do_not_trigger() {
        // CD (82) far longer than AB (40) → not an AB=CD.
        let out = run(&[150.0, 100.0, 140.0, 118.0, 200.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = Abcd::new();
        for c in candles_for_pivots(&[140.0, 100.0, 124.7]) {
            let _ = indicator.update(c);
        }
        indicator.reset();
        assert!(!indicator.is_ready());
        let c = Candle::new(99.5, 100.0, 99.5, 99.5, 1.0, 0).unwrap();
        assert_eq!(indicator.update(c), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = candles_for_pivots(&[140.0, 100.0, 124.7, 84.7]);
        let mut a = Abcd::new();
        let mut b = Abcd::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
