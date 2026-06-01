//! Counterattack candlestick pattern.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Counterattack — a 2-bar reversal where the second bar storms back to close
/// right where the first bar closed. A long candle runs with the trend, then an
/// opposite-coloured long candle opens far in the trend direction and rallies (or
/// sells off) all the way back to the prior close — the two closes meeting forms
/// the "counterattack line".
///
/// ```text
/// long bodies   = |close − open| >= 0.5 * (high − low)   (both bars)
/// equal closes  = |close2 − close1| <= tol * mean(range1, range2)
/// bullish (+1.0): bar1 black (down), bar2 white (up), equal closes
/// bearish (−1.0): bar1 white (up),   bar2 black (down), equal closes
/// ```
///
/// Output is `+1.0` bullish, `−1.0` bearish, and `0.0` when the bodies are short,
/// the colours match, or the closes are not level. The first bar always returns
/// `0.0` because the two-bar window is not yet filled. `equal_tolerance` defaults
/// to `0.05` (TA-Lib's `CDLCOUNTERATTACK` "equal" factor — 5 % of the mean bar
/// range) and must lie in `[0, 1)`. The body-length test uses a fixed half-range
/// fraction rather than TA-Lib's rolling body average, matching the geometric
/// house style of this pattern family. Pattern-shape check only — no trend filter
/// is applied; combine with a trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` bullish, `−1.0` bearish, `0.0` no pattern — so it
/// drops straight into a machine-learning feature matrix where the bullish and
/// bearish variants occupy a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Counterattack, Indicator};
///
/// let mut indicator = Counterattack::new();
/// // Bullish: a long black bar, then a long white bar closing at the same level.
/// indicator.update(Candle::new(20.0, 20.1, 14.9, 15.0, 1.0, 0).unwrap());
/// let out = indicator
///     .update(Candle::new(10.0, 15.1, 9.9, 15.0, 1.0, 1).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone)]
pub struct Counterattack {
    equal_tolerance: f64,
    prev: Option<Candle>,
    has_emitted: bool,
}

impl Default for Counterattack {
    fn default() -> Self {
        Self::new()
    }
}

impl Counterattack {
    /// Construct a Counterattack detector with the default 5 % equal-close tolerance.
    pub const fn new() -> Self {
        Self {
            equal_tolerance: 0.05,
            prev: None,
            has_emitted: false,
        }
    }

    /// Construct a Counterattack detector with a custom equal-close tolerance.
    ///
    /// `equal_tolerance` is the fraction of the mean bar range within which the
    /// two closes must agree and must lie in `[0, 1)`.
    pub fn with_tolerance(equal_tolerance: f64) -> Result<Self> {
        if !(0.0..1.0).contains(&equal_tolerance) {
            return Err(Error::InvalidPeriod {
                message: "counterattack equal tolerance must lie in [0, 1)",
            });
        }
        Ok(Self {
            equal_tolerance,
            prev: None,
            has_emitted: false,
        })
    }

    /// Configured equal-close tolerance.
    pub fn equal_tolerance(&self) -> f64 {
        self.equal_tolerance
    }
}

impl Indicator for Counterattack {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let prev = self.prev;
        self.prev = Some(candle);
        let Some(bar1) = prev else {
            return Some(0.0);
        };
        let range1 = bar1.high - bar1.low;
        let range2 = candle.high - candle.low;
        let body1 = bar1.close - bar1.open;
        let body2 = candle.close - candle.open;
        let long1 = body1.abs() >= 0.5 * range1;
        let long2 = body2.abs() >= 0.5 * range2;
        let tol = self.equal_tolerance * 0.5 * (range1 + range2);
        let equal_close = (candle.close - bar1.close).abs() <= tol;
        if !(long1 && long2 && equal_close) {
            return Some(0.0);
        }
        // Bullish: a long black bar met by a long white bar closing level.
        if body1 < 0.0 && body2 > 0.0 {
            return Some(1.0);
        }
        // Bearish: a long white bar met by a long black bar closing level.
        if body1 > 0.0 && body2 < 0.0 {
            return Some(-1.0);
        }
        Some(0.0)
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
        "Counterattack"
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
        assert!(Counterattack::with_tolerance(-0.01).is_err());
        assert!(Counterattack::with_tolerance(1.0).is_err());
    }

    #[test]
    fn accepts_valid_tolerance() {
        let t = Counterattack::with_tolerance(0.0).unwrap();
        assert!((t.equal_tolerance() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn accessors_and_metadata() {
        let t = Counterattack::default();
        assert_eq!(t.name(), "Counterattack");
        assert_eq!(t.warmup_period(), 2);
        assert!(!t.is_ready());
        assert!((t.equal_tolerance() - 0.05).abs() < 1e-12);
    }

    #[test]
    fn bullish_counterattack_is_plus_one() {
        let mut t = Counterattack::new();
        assert_eq!(t.update(c(20.0, 20.1, 14.9, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(10.0, 15.1, 9.9, 15.0, 1)), Some(1.0));
    }

    #[test]
    fn bearish_counterattack_is_minus_one() {
        let mut t = Counterattack::new();
        assert_eq!(t.update(c(15.0, 20.1, 14.9, 20.0, 0)), Some(0.0));
        assert_eq!(t.update(c(25.0, 25.1, 19.9, 20.0, 1)), Some(-1.0));
    }

    #[test]
    fn unequal_close_yields_zero() {
        let mut t = Counterattack::new();
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        // Second close at 17.0 is far from the first close (15.0) -> not level.
        assert_eq!(t.update(c(10.0, 17.1, 9.9, 17.0, 1)), Some(0.0));
    }

    #[test]
    fn same_color_yields_zero() {
        let mut t = Counterattack::new();
        // Both bars black -> not opposite colours.
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        assert_eq!(t.update(c(20.0, 20.1, 14.9, 15.0, 1)), Some(0.0));
    }

    #[test]
    fn short_body_yields_zero() {
        let mut t = Counterattack::new();
        // Second bar has a tiny body relative to its range.
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        assert_eq!(t.update(c(14.8, 20.0, 9.9, 15.2, 1)), Some(0.0));
    }

    #[test]
    fn first_bar_returns_zero() {
        let mut t = Counterattack::new();
        assert_eq!(t.update(c(20.0, 20.1, 14.9, 15.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 2.0, base - 2.0, base + 1.5, i)
            })
            .collect();
        let mut a = Counterattack::new();
        let mut b = Counterattack::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = Counterattack::new();
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        t.update(c(10.0, 15.1, 9.9, 15.0, 1));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(20.0, 20.1, 14.9, 15.0, 0)), Some(0.0));
    }
}
