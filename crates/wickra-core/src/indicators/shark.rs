//! Shark harmonic pattern.

use crate::indicators::pattern_swing::{ratios_in, xabcd, SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Shark — a 5-point (X-A-B-C-D) harmonic pattern characterised by an
/// **expansion** leg (AB longer than XA) and a `0.886`–`1.13` D completion:
///
/// ```text
/// AB / XA ∈ [1.13, 1.618]  (expansion — B overshoots X)
/// BC / AB ∈ [1.618, 2.24]
/// CD / BC ∈ [0.382, 0.886]
/// AD / XA ∈ [0.886, 1.13]  (the defining D completion near A)
/// ```
///
/// This is the 5-point reading of the Shark; output is `+1.0` (bullish, D a
/// swing low), `-1.0` (bearish, D a swing high), or `0.0`; never `None`. See
/// `crates/wickra-core/src/indicators/shark.rs`.
#[derive(Debug, Clone)]
pub struct Shark {
    swing: SwingTracker,
    has_emitted: bool,
}

impl Shark {
    /// Construct a new Shark detector.
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 5),
            has_emitted: false,
        }
    }
}

impl Default for Shark {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for Shark {
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
            (ab / xa, 1.13, 1.618),
            (bc / ab, 1.618, 2.24),
            (cd / bc, 0.382, 0.886),
            (ad / xa, 0.886, 1.13),
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
        "Shark"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;

    fn run(pivots: &[f64]) -> Vec<f64> {
        let mut indicator = Shark::new();
        candles_for_pivots(pivots)
            .into_iter()
            .map(|c| indicator.update(c).unwrap())
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = Shark::new();
        assert_eq!(indicator.name(), "Shark");
        assert_eq!(indicator.warmup_period(), 6);
        assert!(!indicator.is_ready());
        assert!(!Shark::default().is_ready());
    }

    #[test]
    fn bullish_shark_is_plus_one() {
        let out = run(&[150.0, 100.0, 140.0, 88.0, 186.8, 100.0]);
        assert_eq!(*out.last().unwrap(), 1.0);
        assert!(out[..out.len() - 1].iter().all(|&x| x == 0.0));
    }

    #[test]
    fn bearish_shark_is_minus_one() {
        let out = run(&[150.0, 110.0, 162.0, 60.2, 150.0]);
        assert_eq!(*out.last().unwrap(), -1.0);
    }

    #[test]
    fn out_of_ratio_does_not_trigger() {
        let out = run(&[150.0, 100.0, 140.0, 110.0, 135.0, 105.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = Shark::new();
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
        let candles = candles_for_pivots(&[150.0, 100.0, 140.0, 88.0, 186.8, 100.0]);
        let mut a = Shark::new();
        let mut b = Shark::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
