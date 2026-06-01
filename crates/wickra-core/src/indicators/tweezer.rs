//! Tweezer Top / Bottom candlestick pattern.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Tweezer — a 2-bar reversal pattern where two consecutive candles share an
/// extreme.
///
/// ```text
/// tol           = tolerance * |prev.high| + tolerance * |prev.low|   (per leg)
/// tweezer_top   = |curr.high − prev.high| <= tol_high
/// tweezer_bot   = |curr.low  − prev.low|  <= tol_low
/// ```
///
/// The output is `−1.0` for a Tweezer Top (matched highs), `+1.0` for a
/// Tweezer Bottom (matched lows), and `0.0` otherwise. If *both* extremes
/// match — a flat pair of candles — the bottom wins by convention (bullish
/// rejection of the low). `tolerance` defaults to `0.001` (10 bps relative)
/// and must lie in `[0, 1)`.
///
/// Pattern-shape check only — no trend filter is applied; combine with a trend
/// indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector already emits the uniform candlestick sign convention shared
/// across the pattern family — `+1.0` bullish, `−1.0` bearish, `0.0` no
/// pattern — so it drops straight into a machine-learning feature matrix where
/// the bullish and bearish variants of the pattern occupy a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Tweezer};
///
/// let mut indicator = Tweezer::new();
/// indicator.update(Candle::new(11.0, 12.0, 9.5, 9.6, 1.0, 0).unwrap());
/// // Matching low.
/// let out = indicator.update(Candle::new(9.7, 10.5, 9.5, 10.2, 1.0, 1).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone)]
pub struct Tweezer {
    tolerance: f64,
    prev: Option<Candle>,
    has_emitted: bool,
}

impl Default for Tweezer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tweezer {
    /// Construct a Tweezer detector with the default relative tolerance (1e-3).
    pub const fn new() -> Self {
        Self {
            tolerance: 0.001,
            prev: None,
            has_emitted: false,
        }
    }

    /// Construct a Tweezer detector with a custom relative tolerance.
    ///
    /// `tolerance` must lie in `[0, 1)`.
    pub fn with_tolerance(tolerance: f64) -> Result<Self> {
        if !(0.0..1.0).contains(&tolerance) {
            return Err(Error::InvalidPeriod {
                message: "tweezer tolerance must lie in [0, 1)",
            });
        }
        Ok(Self {
            tolerance,
            prev: None,
            has_emitted: false,
        })
    }

    /// Configured relative tolerance.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
}

impl Indicator for Tweezer {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let prev = self.prev;
        self.prev = Some(candle);
        let Some(p) = prev else {
            return Some(0.0);
        };
        let tol_high = self.tolerance * p.high.abs().max(candle.high.abs());
        let tol_low = self.tolerance * p.low.abs().max(candle.low.abs());
        let match_low = (candle.low - p.low).abs() <= tol_low;
        let match_high = (candle.high - p.high).abs() <= tol_high;
        if match_low {
            Some(1.0)
        } else if match_high {
            Some(-1.0)
        } else {
            Some(0.0)
        }
    }

    fn reset(&mut self) {
        self.prev = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "Tweezer"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(open: f64, high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(open, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn rejects_invalid_tolerance() {
        assert!(Tweezer::with_tolerance(-0.01).is_err());
        assert!(Tweezer::with_tolerance(1.0).is_err());
    }

    #[test]
    fn accepts_valid_tolerance() {
        let t = Tweezer::with_tolerance(0.0).unwrap();
        assert!((t.tolerance() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn accessors_and_metadata() {
        let t = Tweezer::default();
        assert_eq!(t.name(), "Tweezer");
        assert_eq!(t.warmup_period(), 2);
        assert!(!t.is_ready());
        assert!((t.tolerance() - 0.001).abs() < 1e-12);
    }

    #[test]
    fn tweezer_bottom_is_plus_one() {
        let mut t = Tweezer::new();
        assert_eq!(t.update(c(11.0, 12.0, 9.5, 9.6, 0)), Some(0.0));
        // Matching low 9.5.
        assert_eq!(t.update(c(9.7, 10.5, 9.5, 10.2, 1)), Some(1.0));
    }

    #[test]
    fn tweezer_top_is_minus_one() {
        let mut t = Tweezer::new();
        assert_eq!(t.update(c(9.0, 12.0, 8.5, 11.0, 0)), Some(0.0));
        // Matching high 12.0.
        assert_eq!(t.update(c(11.5, 12.0, 11.0, 11.4, 1)), Some(-1.0));
    }

    #[test]
    fn distinct_extremes_yield_zero() {
        let mut t = Tweezer::new();
        t.update(c(10.0, 11.0, 9.0, 10.5, 0));
        assert_eq!(t.update(c(10.6, 11.5, 9.6, 11.2, 1)), Some(0.0));
    }

    #[test]
    fn first_bar_returns_zero() {
        let mut t = Tweezer::new();
        assert_eq!(t.update(c(10.0, 11.0, 9.0, 10.5, 0)), Some(0.0));
    }

    #[test]
    fn matched_both_extremes_prefers_bottom() {
        // Identical candles match both highs and lows -> bottom (+1.0).
        let mut t = Tweezer::new();
        t.update(c(10.0, 11.0, 9.0, 10.5, 0));
        assert_eq!(t.update(c(10.0, 11.0, 9.0, 10.5, 1)), Some(1.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + (i as f64 * 0.1).sin();
                c(base, base + 2.0, base - 2.0, base + 0.5, i)
            })
            .collect();
        let mut a = Tweezer::new();
        let mut b = Tweezer::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = Tweezer::new();
        t.update(c(10.0, 11.0, 9.0, 10.5, 0));
        t.update(c(10.0, 11.0, 9.0, 10.5, 1));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(10.0, 11.0, 9.0, 10.5, 0)), Some(0.0));
    }
}
