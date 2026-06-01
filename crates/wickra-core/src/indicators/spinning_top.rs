//! Spinning Top candlestick pattern.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Spinning Top — a single-bar indecision candle with a small body and two
/// long shadows.
///
/// ```text
/// body         = |close − open|
/// upper_shadow = high − max(open, close)
/// lower_shadow = min(open, close) − low
/// range        = high − low
/// spinning     = body <= body_threshold * range
///               && upper_shadow >= 2 * body
///               && lower_shadow >= 2 * body
///               && body > 0
/// ```
///
/// While direction is ambiguous by intent, the output is direction-signed so
/// downstream filters can distinguish a green spinning top (`+1.0`) from a red
/// one (`−1.0`). A clean Doji (body == 0) is *not* a Spinning Top.
///
/// `body_threshold` defaults to `0.3` and must lie in `(0, 1]`.
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
/// use wickra_core::{Candle, Indicator, SpinningTop};
///
/// let mut indicator = SpinningTop::new();
/// // Body 0.5, both shadows 3.0 -> spinning.
/// let candle = Candle::new(10.0, 13.5, 7.0, 10.5, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
/// ```
#[derive(Debug, Clone)]
pub struct SpinningTop {
    body_threshold: f64,
    has_emitted: bool,
}

impl Default for SpinningTop {
    fn default() -> Self {
        Self::new()
    }
}

impl SpinningTop {
    /// Construct a Spinning Top detector with the default body threshold.
    pub const fn new() -> Self {
        Self {
            body_threshold: 0.3,
            has_emitted: false,
        }
    }

    /// Construct a Spinning Top detector with a custom body / range threshold.
    pub fn with_threshold(body_threshold: f64) -> Result<Self> {
        if !(body_threshold > 0.0 && body_threshold <= 1.0) {
            return Err(Error::InvalidPeriod {
                message: "spinning top body threshold must lie in (0, 1]",
            });
        }
        Ok(Self {
            body_threshold,
            has_emitted: false,
        })
    }

    /// Configured body / range threshold.
    pub fn body_threshold(&self) -> f64 {
        self.body_threshold
    }
}

impl Indicator for SpinningTop {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let range = candle.high - candle.low;
        if range <= 0.0 {
            return Some(0.0);
        }
        let body_signed = candle.close - candle.open;
        let body = body_signed.abs();
        if body <= 0.0 {
            return Some(0.0);
        }
        if body > self.body_threshold * range {
            return Some(0.0);
        }
        let upper = candle.high - candle.open.max(candle.close);
        let lower = candle.open.min(candle.close) - candle.low;
        if upper >= 2.0 * body && lower >= 2.0 * body {
            Some(if body_signed > 0.0 { 1.0 } else { -1.0 })
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
        "SpinningTop"
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
    fn rejects_invalid_threshold() {
        assert!(SpinningTop::with_threshold(0.0).is_err());
        assert!(SpinningTop::with_threshold(1.5).is_err());
    }

    #[test]
    fn accepts_valid_threshold() {
        let s = SpinningTop::with_threshold(0.25).unwrap();
        assert!((s.body_threshold() - 0.25).abs() < 1e-12);
    }

    #[test]
    fn accessors_and_metadata() {
        let s = SpinningTop::default();
        assert_eq!(s.name(), "SpinningTop");
        assert_eq!(s.warmup_period(), 1);
        assert!(!s.is_ready());
        assert!((s.body_threshold() - 0.3).abs() < 1e-12);
    }

    #[test]
    fn green_spinning_top_is_plus_one() {
        let mut s = SpinningTop::new();
        // body 0.5 (10 -> 10.5), upper 3.0, lower 3.0, range 6.5 -> 0.5/6.5 < 0.3.
        assert_eq!(s.update(c(10.0, 13.5, 7.0, 10.5, 0)), Some(1.0));
    }

    #[test]
    fn red_spinning_top_is_minus_one() {
        let mut s = SpinningTop::new();
        assert_eq!(s.update(c(10.5, 13.5, 7.0, 10.0, 0)), Some(-1.0));
    }

    #[test]
    fn marubozu_is_not_spinning() {
        let mut s = SpinningTop::new();
        assert_eq!(s.update(c(10.0, 12.0, 10.0, 12.0, 0)), Some(0.0));
    }

    #[test]
    fn doji_is_not_spinning() {
        // body == 0 fails the body > 0 guard.
        let mut s = SpinningTop::new();
        assert_eq!(s.update(c(10.0, 11.0, 9.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn hammer_shape_is_not_spinning_top() {
        // Lower shadow is long but upper is tiny -> only one long shadow.
        let mut s = SpinningTop::new();
        assert_eq!(s.update(c(10.0, 10.6, 5.0, 10.5, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut s = SpinningTop::new();
        assert_eq!(s.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 3.0, base - 3.0, base + 0.5, i)
            })
            .collect();
        let mut a = SpinningTop::new();
        let mut b = SpinningTop::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut s = SpinningTop::new();
        s.update(c(10.0, 13.5, 7.0, 10.5, 0));
        assert!(s.is_ready());
        s.reset();
        assert!(!s.is_ready());
    }
}
