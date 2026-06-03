//! Bat harmonic pattern.

use crate::indicators::pattern_swing::{ratios_in, xabcd, SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Bat — a 5-point (X-A-B-C-D) harmonic pattern with a shallow B and a deep
/// `0.886` D completion:
///
/// ```text
/// AB / XA ∈ [0.382, 0.50]
/// BC / AB ∈ [0.382, 0.886]
/// CD / BC ∈ [1.618, 2.618]
/// AD / XA ∈ [0.84, 0.93]   (≈ 0.886 — the defining D completion)
/// ```
///
/// Output is `+1.0` (bullish, D a swing low), `-1.0` (bearish, D a swing high),
/// or `0.0`; never `None`. See `crates/wickra-core/src/indicators/bat.rs`.
#[derive(Debug, Clone)]
pub struct Bat {
    swing: SwingTracker,
    has_emitted: bool,
}

impl Bat {
    /// Construct a new Bat detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 5),
            has_emitted: false,
        }
    }
}

impl Default for Bat {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for Bat {
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
        let ad = (p.d - p.a).abs();
        let matched = ratios_in(&[
            (ab / xa, 0.382, 0.50),
            (bc / ab, 0.382, 0.886),
            (cd / bc, 1.618, 2.618),
            (ad / xa, 0.84, 0.93),
        ]);
        if matched {
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
        "Bat"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = Bat::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = Bat::new();
        assert_eq!(indicator.name(), "Bat");
        assert_eq!(indicator.warmup_period(), 6);
        assert!(!indicator.is_ready());
        assert!(!Bat::default().is_ready());
    }

    #[test]
    fn bullish_bat_is_plus_one() {
        let out = run(&[150.0, 100.0, 140.0, 122.0, 137.0, 104.56]);
        assert_eq!(*out.last().unwrap(), 1.0);
        assert!(out[..out.len() - 1].iter().all(|&x| x == 0.0));
    }

    #[test]
    fn bearish_bat_is_minus_one() {
        let out = run(&[150.0, 110.0, 128.0, 113.0, 145.44]);
        assert_eq!(*out.last().unwrap(), -1.0);
    }

    #[test]
    fn out_of_ratio_does_not_trigger() {
        let out = run(&[150.0, 100.0, 140.0, 110.0, 135.0, 105.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = Bat::new();
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
        let candles = candles_for_pivots(&[150.0, 100.0, 140.0, 122.0, 137.0, 104.56]);
        let mut a = Bat::new();
        let mut b = Bat::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
