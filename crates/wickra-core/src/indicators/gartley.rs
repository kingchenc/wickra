//! Gartley harmonic pattern.

use crate::indicators::pattern_swing::{ratios_in, xabcd, SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Gartley — the classic 5-point (X-A-B-C-D) harmonic pattern, recognised from
/// confirmed swing pivots when the legs fall inside the Gartley Fibonacci
/// windows:
///
/// ```text
/// AB / XA ∈ [0.55, 0.70]   (≈ 0.618 retracement of XA)
/// BC / AB ∈ [0.382, 0.886]
/// CD / BC ∈ [1.13, 1.618]
/// AD / XA ∈ [0.74, 0.84]   (≈ 0.786 — the defining D completion)
/// ```
///
/// Output is `+1.0` when the terminal point D is a swing low (bullish
/// completion), `-1.0` when D is a swing high (bearish), and `0.0` otherwise;
/// never `None`. See `crates/wickra-core/src/indicators/gartley.rs`.
#[derive(Debug, Clone)]
pub struct Gartley {
    swing: SwingTracker,
    has_emitted: bool,
}

impl Gartley {
    /// Construct a new Gartley detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 5),
            has_emitted: false,
        }
    }
}

impl Default for Gartley {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for Gartley {
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
            (ab / xa, 0.55, 0.70),
            (bc / ab, 0.382, 0.886),
            (cd / bc, 1.13, 1.618),
            (ad / xa, 0.74, 0.84),
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
        "Gartley"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = Gartley::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = Gartley::new();
        assert_eq!(indicator.name(), "Gartley");
        assert_eq!(indicator.warmup_period(), 6);
        assert!(!indicator.is_ready());
        assert!(!Gartley::default().is_ready());
    }

    #[test]
    fn bullish_gartley_is_plus_one() {
        let out = run(&[150.0, 100.0, 140.0, 115.3, 127.65, 108.56]);
        assert_eq!(*out.last().unwrap(), 1.0);
        assert!(out[..out.len() - 1].iter().all(|&x| x == 0.0));
    }

    #[test]
    fn bearish_gartley_is_minus_one() {
        let out = run(&[150.0, 110.0, 134.7, 122.35, 141.44]);
        assert_eq!(*out.last().unwrap(), -1.0);
    }

    #[test]
    fn out_of_ratio_does_not_trigger() {
        // Five pivots but the D completion (AD/XA ≈ 0.25) is far from 0.786.
        let out = run(&[150.0, 100.0, 140.0, 110.0, 135.0, 105.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = Gartley::new();
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
        let candles = candles_for_pivots(&[150.0, 100.0, 140.0, 115.3, 127.65, 108.56]);
        let mut a = Gartley::new();
        let mut b = Gartley::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
