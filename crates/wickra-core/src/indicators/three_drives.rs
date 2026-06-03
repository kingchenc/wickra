//! Three Drives harmonic pattern.

use crate::indicators::pattern_swing::{
    approx_equal, ratios_in, xabcd, SwingTracker, SWING_THRESHOLD,
};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Three Drives — a symmetric harmonic pattern of two visible drives separated
/// by two retracements, read from the last five pivots `X-A-B-C-D` (the two
/// drive legs are `A→B` and `C→D`):
///
/// ```text
/// AB / XA ∈ [1.13, 1.75]   (drive 1 extends the prior retracement)
/// CD / BC ∈ [1.13, 1.75]   (drive 2 extends symmetrically)
/// AB ≈ CD (within 20%)      (the two drives are similar in size)
/// XA ≈ BC (within 30%)      (the two retracements are similar)
/// ```
///
/// Output is `+1.0` (bullish, terminal D a swing low — drives down), `-1.0`
/// (bearish, drives up), or `0.0`; never `None`. See
/// `crates/wickra-core/src/indicators/three_drives.rs`.
#[derive(Debug, Clone)]
pub struct ThreeDrives {
    swing: SwingTracker,
    has_emitted: bool,
}

impl ThreeDrives {
    /// Construct a new Three Drives detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 5),
            has_emitted: false,
        }
    }
}

impl Default for ThreeDrives {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for ThreeDrives {
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
        let p = xabcd(pivots);
        let xa = (p.a - p.x).abs();
        let ab = (p.b - p.a).abs();
        let bc = (p.c - p.b).abs();
        let cd = (p.d - p.c).abs();
        let extensions = ratios_in(&[(ab / xa, 1.13, 1.75), (cd / bc, 1.13, 1.75)]);
        let symmetric = approx_equal(ab, cd, 0.20) && approx_equal(xa, bc, 0.30);
        if extensions && symmetric {
            return Some(if p.bullish { 1.0 } else { -1.0 });
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.swing.reset();
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        6
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "ThreeDrives"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = ThreeDrives::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = ThreeDrives::new();
        assert_eq!(indicator.name(), "ThreeDrives");
        assert_eq!(indicator.warmup_period(), 6);
        assert!(!indicator.is_ready());
        assert!(!ThreeDrives::default().is_ready());
    }

    #[test]
    fn bearish_three_drives_is_minus_one() {
        // Three rising drives (120, 128, 136) → bearish exhaustion.
        let out = run(&[120.0, 100.0, 128.0, 108.0, 136.0]);
        assert_eq!(*out.last().unwrap(), -1.0);
        assert!(out[..out.len() - 1].iter().all(|&x| x == 0.0));
    }

    #[test]
    fn bullish_three_drives_is_plus_one() {
        // Three falling drives → bullish exhaustion.
        let out = run(&[150.0, 120.0, 140.0, 112.0, 132.0, 104.0]);
        assert_eq!(*out.last().unwrap(), 1.0);
    }

    #[test]
    fn asymmetric_drives_do_not_trigger() {
        let out = run(&[150.0, 100.0, 140.0, 110.0, 135.0, 105.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = ThreeDrives::new();
        for c in candles_for_pivots(&[120.0, 100.0, 128.0]) {
            let _ = indicator.update(c);
        }
        indicator.reset();
        assert!(!indicator.is_ready());
        let c = Candle::new(99.5, 100.0, 99.5, 99.5, 1.0, 0).unwrap();
        assert_eq!(indicator.update(c), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = candles_for_pivots(&[120.0, 100.0, 128.0, 108.0, 136.0]);
        let mut a = ThreeDrives::new();
        let mut b = ThreeDrives::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
