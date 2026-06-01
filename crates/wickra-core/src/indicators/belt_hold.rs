//! Belt-hold candlestick pattern.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Belt-hold — a single-bar reversal: a long candle that opens at one extreme of
/// its range (an "opening marubozu") and runs the other way.
///
/// ```text
/// range        = high − low
/// bullish (+1.0): green, opens at the low  (open − low <= tol * range) & long body
/// bearish (−1.0): red,   opens at the high (high − open <= tol * range) & long body
/// long body    = |close − open| >= 0.5 * range
/// ```
///
/// Output is `0.0` when the opening side carries a shadow, the body is short, or
/// the range is degenerate. `shadow_tolerance` defaults to `0.05` (5 % of the bar
/// range allowed on the opening side) and must lie in `[0, 1)`. Pattern-shape
/// check only — no trend filter is applied; combine with a trend indicator for
/// actionable signals.
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
/// use wickra_core::{BeltHold, Candle, Indicator};
///
/// let mut indicator = BeltHold::new();
/// // Bullish belt-hold: opens at the low, closes near the high.
/// let candle = Candle::new(10.0, 12.0, 10.0, 11.5, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
/// ```
#[derive(Debug, Clone)]
pub struct BeltHold {
    shadow_tolerance: f64,
    has_emitted: bool,
}

impl Default for BeltHold {
    fn default() -> Self {
        Self::new()
    }
}

impl BeltHold {
    /// Construct a Belt-hold detector with the default 5 % opening-shadow tolerance.
    pub const fn new() -> Self {
        Self {
            shadow_tolerance: 0.05,
            has_emitted: false,
        }
    }

    /// Construct a Belt-hold detector with a custom opening-shadow tolerance.
    ///
    /// `shadow_tolerance` must lie in `[0, 1)`.
    pub fn with_tolerance(shadow_tolerance: f64) -> Result<Self> {
        if !(0.0..1.0).contains(&shadow_tolerance) {
            return Err(Error::InvalidPeriod {
                message: "belt-hold shadow tolerance must lie in [0, 1)",
            });
        }
        Ok(Self {
            shadow_tolerance,
            has_emitted: false,
        })
    }

    /// Configured opening-shadow tolerance.
    pub fn shadow_tolerance(&self) -> f64 {
        self.shadow_tolerance
    }
}

impl Indicator for BeltHold {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let range = candle.high - candle.low;
        if range <= 0.0 {
            return Some(0.0);
        }
        let body = candle.close - candle.open;
        if body.abs() < 0.5 * range {
            return Some(0.0);
        }
        let tol = self.shadow_tolerance * range;
        // Bullish: opens at the low (no lower shadow), green body.
        if body > 0.0 && candle.open - candle.low <= tol {
            return Some(1.0);
        }
        // Bearish: opens at the high (no upper shadow), red body.
        if body < 0.0 && candle.high - candle.open <= tol {
            return Some(-1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "BeltHold"
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
        assert!(BeltHold::with_tolerance(-0.01).is_err());
        assert!(BeltHold::with_tolerance(1.0).is_err());
    }

    #[test]
    fn accepts_valid_tolerance() {
        let t = BeltHold::with_tolerance(0.0).unwrap();
        assert!((t.shadow_tolerance() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn accessors_and_metadata() {
        let t = BeltHold::default();
        assert_eq!(t.name(), "BeltHold");
        assert_eq!(t.warmup_period(), 1);
        assert!(!t.is_ready());
        assert!((t.shadow_tolerance() - 0.05).abs() < 1e-12);
    }

    #[test]
    fn bullish_belt_hold_is_plus_one() {
        let mut t = BeltHold::new();
        assert_eq!(t.update(c(10.0, 12.0, 10.0, 11.5, 0)), Some(1.0));
    }

    #[test]
    fn bearish_belt_hold_is_minus_one() {
        let mut t = BeltHold::new();
        assert_eq!(t.update(c(12.0, 12.0, 10.0, 10.5, 0)), Some(-1.0));
    }

    #[test]
    fn opening_shadow_yields_zero() {
        let mut t = BeltHold::new();
        // Opens 0.5 above the low -> lower shadow exceeds tolerance.
        assert_eq!(t.update(c(10.5, 12.0, 10.0, 11.5, 0)), Some(0.0));
    }

    #[test]
    fn short_body_yields_zero() {
        let mut t = BeltHold::new();
        // Body 0.5 < half the range (1.0) -> not a long belt-hold.
        assert_eq!(t.update(c(10.0, 12.0, 10.0, 10.5, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = BeltHold::new();
        assert_eq!(t.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 2.0, base, base + 1.8, i)
            })
            .collect();
        let mut a = BeltHold::new();
        let mut b = BeltHold::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = BeltHold::new();
        t.update(c(10.0, 12.0, 10.0, 11.5, 0));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
    }
}
