//! Cypher harmonic pattern.

use crate::indicators::pattern_swing::{ratios_in, xabcd, SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Cypher — a 5-point (X-A-B-C-D) harmonic pattern (Darren Oglesbee) whose C
/// point projects the XA leg beyond A and whose D retraces the XC leg by
/// `0.786`:
///
/// ```text
/// AB / XA ∈ [0.382, 0.618]
/// XC / XA ∈ [1.272, 1.414]  (C projects XA beyond A — measured X-to-C)
/// CD / XC ∈ [0.74, 0.83]    (≈ 0.786 retracement of XC — the D completion)
/// ```
///
/// The C constraint is the X-to-C projection, not B-to-C: unlike the Gartley
/// family, the Cypher measures its third point against the initial XA leg
/// rather than against AB.
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

    #[inline]
    fn update(&mut self, candle: Candle) -> Option<f64> {
        let advanced = self.swing.update(candle);
        let pivots = self.swing.pivots();
        // Too few pivots to form the shape at all: the indicator cannot
        // judge yet, which is what `None` means.
        if pivots.len() < 5 {
            return None;
        }
        self.has_emitted = true;
        // Armed, but this bar did not close a new pivot, so there is
        // nothing new to match against.
        if !advanced {
            return Some(0.0);
        }
        let p = xabcd(pivots);
        let xa = (p.a - p.x).abs();
        let ab = (p.b - p.a).abs();
        let xc = (p.c - p.x).abs();
        let cd = (p.d - p.c).abs();
        let matched = ratios_in(&[
            (ab / xa, 0.382, 0.618),
            (xc / xa, 1.272, 1.414),
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

    #[inline]
    fn warmup_period(&self) -> usize {
        6
    }

    #[inline]
    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    #[inline]
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
            .filter_map(|c| indicator.update(c))
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
        // X=100 A=140 B=120 C=152 D=111.128:
        // AB/XA = 20/40 = 0.5, XC/XA = 52/40 = 1.3, CD/XC = 40.872/52 = 0.786.
        let out = run(&[150.0, 100.0, 140.0, 120.0, 152.0, 111.128]);
        assert_eq!(*out.last().unwrap(), 1.0);
        assert!(out[..out.len() - 1].iter().all(|&x| x == 0.0));
    }

    #[test]
    fn bearish_cypher_is_minus_one() {
        // X=150 A=110 B=130 C=98 D=138.872: same ratios, terminal D a swing high.
        let out = run(&[150.0, 110.0, 130.0, 98.0, 138.872]);
        assert_eq!(*out.last().unwrap(), -1.0);
    }

    #[test]
    fn out_of_ratio_does_not_trigger() {
        let out = run(&[150.0, 100.0, 140.0, 110.0, 135.0, 105.0]);
        assert_eq!(*out.last().unwrap(), 0.0);
    }

    #[test]
    fn c_beyond_the_projection_window_does_not_trigger() {
        // X=100 A=140 B=120 C=168 D=114.55: BC/XA = 1.2 sits inside the old
        // (incorrect) window, but XC/XA = 1.7 is outside the 1.272-1.414
        // projection, so the pattern must not match.
        let out = run(&[150.0, 100.0, 140.0, 120.0, 168.0, 114.55]);
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
        assert_eq!(indicator.update(c), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = candles_for_pivots(&[150.0, 100.0, 140.0, 120.0, 152.0, 111.128]);
        let mut a = Cypher::new();
        let mut b = Cypher::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
