//! Identical Three Crows candlestick pattern.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Identical Three Crows — a 3-bar bearish reversal: three consecutive red
/// candles with steadily lower closes where each candle opens at (or very near)
/// the prior candle's close, so the bodies stack in an identical staircase.
///
/// ```text
/// tol_n         = tolerance * max(|open|, |prev.close|)
/// all three red                              (close < open)
/// declining closes                           (bar2.close < bar1.close, bar3.close < bar2.close)
/// bar2 opens at bar1's close                 (|bar2.open − bar1.close| <= tol_2)
/// bar3 opens at bar2's close                 (|bar3.open − bar2.close| <= tol_3)
/// ```
///
/// Output is `−1.0` when the pattern completes and `0.0` otherwise. Identical
/// Three Crows is a single-direction (bearish-only) pattern, so it never emits
/// `+1.0`. The first two bars always return `0.0` because the three-bar window
/// is not yet filled. `tolerance` defaults to `0.001` (10 bps relative) and must
/// lie in `[0, 1)`. Pattern-shape check only — no trend filter is applied;
/// combine with a trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `−1.0` bearish, `0.0` no pattern — so it drops straight into
/// a machine-learning feature matrix as a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, IdenticalThreeCrows, Indicator};
///
/// let mut indicator = IdenticalThreeCrows::new();
/// indicator.update(Candle::new(13.0, 13.1, 11.9, 12.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(12.0, 12.1, 10.9, 11.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(11.0, 11.1, 9.9, 10.0, 1.0, 2).unwrap());
/// assert_eq!(out, Some(-1.0));
/// ```
#[derive(Debug, Clone)]
pub struct IdenticalThreeCrows {
    tolerance: f64,
    prev: Option<Candle>,
    prev_prev: Option<Candle>,
    has_emitted: bool,
}

impl Default for IdenticalThreeCrows {
    fn default() -> Self {
        Self::new()
    }
}

impl IdenticalThreeCrows {
    /// Construct a detector with the default relative tolerance (1e-3).
    pub const fn new() -> Self {
        Self {
            tolerance: 0.001,
            prev: None,
            prev_prev: None,
            has_emitted: false,
        }
    }

    /// Construct a detector with a custom relative tolerance.
    ///
    /// `tolerance` must lie in `[0, 1)`.
    pub fn with_tolerance(tolerance: f64) -> Result<Self> {
        if !(0.0..1.0).contains(&tolerance) {
            return Err(Error::InvalidPeriod {
                message: "identical three crows tolerance must lie in [0, 1)",
            });
        }
        Ok(Self {
            tolerance,
            prev: None,
            prev_prev: None,
            has_emitted: false,
        })
    }

    /// Configured relative tolerance.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
}

impl Indicator for IdenticalThreeCrows {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let pp = self.prev_prev;
        let p = self.prev;
        self.prev_prev = self.prev;
        self.prev = Some(candle);
        let (Some(bar1), Some(bar2)) = (pp, p) else {
            return Some(0.0);
        };
        let tol2 = self.tolerance * bar2.open.abs().max(bar1.close.abs());
        let tol3 = self.tolerance * candle.open.abs().max(bar2.close.abs());
        if bar1.close < bar1.open
            && bar2.close < bar2.open
            && candle.close < candle.open
            && bar2.close < bar1.close
            && candle.close < bar2.close
            && (bar2.open - bar1.close).abs() <= tol2
            && (candle.open - bar2.close).abs() <= tol3
        {
            return Some(-1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.prev = None;
        self.prev_prev = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        3
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "IdenticalThreeCrows"
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
        assert!(IdenticalThreeCrows::with_tolerance(-0.01).is_err());
        assert!(IdenticalThreeCrows::with_tolerance(1.0).is_err());
    }

    #[test]
    fn accepts_valid_tolerance() {
        let t = IdenticalThreeCrows::with_tolerance(0.0).unwrap();
        assert!((t.tolerance() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn accessors_and_metadata() {
        let t = IdenticalThreeCrows::default();
        assert_eq!(t.name(), "IdenticalThreeCrows");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
        assert!((t.tolerance() - 0.001).abs() < 1e-12);
    }

    #[test]
    fn identical_three_crows_is_minus_one() {
        let mut t = IdenticalThreeCrows::new();
        // Three red candles, each opening at the prior close, declining.
        assert_eq!(t.update(c(13.0, 13.1, 11.9, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(12.0, 12.1, 10.9, 11.0, 1)), Some(0.0));
        assert_eq!(t.update(c(11.0, 11.1, 9.9, 10.0, 2)), Some(-1.0));
    }

    #[test]
    fn non_identical_opens_yield_zero() {
        let mut t = IdenticalThreeCrows::new();
        t.update(c(13.0, 13.1, 11.9, 12.0, 0));
        t.update(c(12.0, 12.1, 10.9, 11.0, 1));
        // bar3 opens at 10.0, far from bar2's close (11.0) -> not identical.
        assert_eq!(t.update(c(10.0, 10.1, 8.9, 9.0, 2)), Some(0.0));
    }

    #[test]
    fn rising_close_yields_zero() {
        let mut t = IdenticalThreeCrows::new();
        t.update(c(13.0, 13.1, 11.9, 12.0, 0));
        t.update(c(12.0, 12.1, 10.9, 11.0, 1));
        // bar3 is green -> not three crows.
        assert_eq!(t.update(c(11.0, 12.2, 10.9, 12.0, 2)), Some(0.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = IdenticalThreeCrows::new();
        assert_eq!(t.update(c(13.0, 13.1, 11.9, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(12.0, 12.1, 10.9, 11.0, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 - i as f64;
                c(base, base + 0.1, base - 1.1, base - 1.0, i)
            })
            .collect();
        let mut a = IdenticalThreeCrows::new();
        let mut b = IdenticalThreeCrows::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = IdenticalThreeCrows::new();
        t.update(c(13.0, 13.1, 11.9, 12.0, 0));
        t.update(c(12.0, 12.1, 10.9, 11.0, 1));
        t.update(c(11.0, 11.1, 9.9, 10.0, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(13.0, 13.1, 11.9, 12.0, 0)), Some(0.0));
    }
}
