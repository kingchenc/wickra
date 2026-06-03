//! Cypher harmonic pattern.

use crate::indicators::pattern_swing::{ratios_in, xabcd, SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Cypher — a 5-point (X-A-B-C-D) harmonic pattern whose C leg is measured
/// against XA (not AB) and whose D retraces the XC leg by `0.786`:
///
/// ```text
/// AB / XA ∈ [0.382, 0.618]
/// BC / XA ∈ [1.13, 1.414]  (C extends beyond A, measured on XA)
/// CD / XC ∈ [0.74, 0.83]   (≈ 0.786 retracement of XC — the D completion)
/// ```
///
/// Output is `+1.0` (bullish, D a swing low), `-1.0` (bearish, D a swing high),
/// or `0.0`; never `None`. See `crates/wickra-core/src/indicators/cypher.rs`.
#[derive(Debug, Clone)]
pub struct Cypher {
    swing: SwingTracker,
    has_emitted: bool,
}

impl Cypher {
    /// Construct a new Cypher detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 5),
            has_emitted: false,
        }
    }
}

impl Default for Cypher {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for Cypher {
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
        let xc = (p.c - p.x).abs();
        let cd = (p.d - p.c).abs();
        let matched = ratios_in(&[
            (ab / xa, 0.382, 0.618),
            (bc / xa, 1.13, 1.414),
            (cd / xc, 0.74, 0.83),
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
        "Cypher"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = Cypher::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = Cypher::new();
        assert_eq!(indicator.name(), "Cypher");
        assert_eq!(indicator.warmup_period(), 6);
        assert!(!indicator.is_ready());
        assert!(!Cypher::default().is_ready());
    }

    #[test]
    fn bullish_cypher_is_plus_one() {
        let out = run(&[150.0, 100.0, 140.0, 120.0, 168.0, 114.55]);
        assert_eq!(*out.last().unwrap(), 1.0);
        assert!(out[..out.len() - 1].iter().all(|&x| x == 0.0));
    }

    #[test]
    fn bearish_cypher_is_minus_one() {
        let out = run(&[150.0, 110.0, 130.0, 82.0, 135.45]);
        assert_eq!(*out.last().unwrap(), -1.0);
    }

    #[test]
    fn out_of_ratio_does_not_trigger() {
        let out = run(&[150.0, 100.0, 140.0, 110.0, 135.0, 105.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = Cypher::new();
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
        let candles = candles_for_pivots(&[150.0, 100.0, 140.0, 120.0, 168.0, 114.55]);
        let mut a = Cypher::new();
        let mut b = Cypher::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
