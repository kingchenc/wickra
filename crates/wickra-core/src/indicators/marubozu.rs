//! Marubozu candlestick pattern.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Marubozu — a single-bar strong-continuation candle with body equal to range
/// and (almost) no shadows.
///
/// ```text
/// range        = high − low
/// upper_shadow = high − max(open, close)
/// lower_shadow = min(open, close) − low
/// shadows OK   = upper_shadow <= tol * range && lower_shadow <= tol * range
/// ```
///
/// When the shadow tolerance is satisfied the output is `+1.0` for a bullish
/// Marubozu (close > open) and `−1.0` for a bearish one (close < open). Any
/// candle whose shadows exceed the tolerance — or whose body is zero — yields
/// `0.0`.
///
/// `shadow_tolerance` defaults to `0.05` (5 % of the bar range allowed on each
/// side) and must lie in `[0, 1)`.
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
/// use wickra_core::{Candle, Indicator, Marubozu};
///
/// let mut indicator = Marubozu::new();
/// // Bullish marubozu: open == low, close == high.
/// let candle = Candle::new(10.0, 12.0, 10.0, 12.0, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
/// ```
#[derive(Debug, Clone)]
pub struct Marubozu {
    shadow_tolerance: f64,
    has_emitted: bool,
}

impl Default for Marubozu {
    fn default() -> Self {
        Self::new()
    }
}

impl Marubozu {
    /// Construct a Marubozu detector with the default 5 % shadow tolerance.
    pub const fn new() -> Self {
        Self {
            shadow_tolerance: 0.05,
            has_emitted: false,
        }
    }

    /// Construct a Marubozu detector with a custom shadow tolerance.
    ///
    /// `shadow_tolerance` must lie in `[0, 1)`.
    pub fn with_tolerance(shadow_tolerance: f64) -> Result<Self> {
        if !(0.0..1.0).contains(&shadow_tolerance) {
            return Err(Error::InvalidPeriod {
                message: "marubozu shadow tolerance must lie in [0, 1)",
            });
        }
        Ok(Self {
            shadow_tolerance,
            has_emitted: false,
        })
    }

    /// Configured shadow tolerance.
    pub fn shadow_tolerance(&self) -> f64 {
        self.shadow_tolerance
    }
}

impl Indicator for Marubozu {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let range = candle.high - candle.low;
        if range <= 0.0 {
            return Some(0.0);
        }
        let body = candle.close - candle.open;
        if body == 0.0 {
            return Some(0.0);
        }
        let upper = candle.high - candle.open.max(candle.close);
        let lower = candle.open.min(candle.close) - candle.low;
        let tol = self.shadow_tolerance * range;
        if upper <= tol && lower <= tol {
            Some(if body > 0.0 { 1.0 } else { -1.0 })
        } else {
            Some(0.0)
        }
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
        "Marubozu"
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
        assert!(Marubozu::with_tolerance(-0.01).is_err());
        assert!(Marubozu::with_tolerance(1.0).is_err());
        assert!(Marubozu::with_tolerance(2.0).is_err());
    }

    #[test]
    fn accepts_valid_tolerance() {
        let m = Marubozu::with_tolerance(0.0).unwrap();
        assert!((m.shadow_tolerance() - 0.0).abs() < 1e-12);
        let m = Marubozu::with_tolerance(0.5).unwrap();
        assert!((m.shadow_tolerance() - 0.5).abs() < 1e-12);
    }

    #[test]
    fn accessors_and_metadata() {
        let m = Marubozu::default();
        assert_eq!(m.name(), "Marubozu");
        assert_eq!(m.warmup_period(), 1);
        assert!(!m.is_ready());
        assert!((m.shadow_tolerance() - 0.05).abs() < 1e-12);
    }

    #[test]
    fn bullish_marubozu_is_plus_one() {
        let mut m = Marubozu::new();
        assert_eq!(m.update(c(10.0, 12.0, 10.0, 12.0, 0)), Some(1.0));
    }

    #[test]
    fn bearish_marubozu_is_minus_one() {
        let mut m = Marubozu::new();
        assert_eq!(m.update(c(12.0, 12.0, 10.0, 10.0, 0)), Some(-1.0));
    }

    #[test]
    fn candle_with_long_shadows_is_zero() {
        let mut m = Marubozu::new();
        // Big upper shadow violates tolerance.
        assert_eq!(m.update(c(10.0, 15.0, 10.0, 12.0, 0)), Some(0.0));
    }

    #[test]
    fn doji_is_zero() {
        let mut m = Marubozu::new();
        // body == 0 -> not a marubozu.
        assert_eq!(m.update(c(10.0, 11.0, 9.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut m = Marubozu::new();
        assert_eq!(m.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 2.0, base, base + 2.0, i)
            })
            .collect();
        let mut a = Marubozu::new();
        let mut b = Marubozu::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut m = Marubozu::new();
        m.update(c(10.0, 12.0, 10.0, 12.0, 0));
        assert!(m.is_ready());
        m.reset();
        assert!(!m.is_ready());
    }
}
