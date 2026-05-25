//! Doji candlestick pattern.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Doji — a candle whose body is negligible relative to its range.
///
/// A Doji prints whenever the absolute distance between open and close is
/// small compared to the total `high − low` range. It is the canonical
/// indecision bar and a building block for many three-bar reversal patterns.
///
/// ```text
/// body  = |close − open|
/// range = high − low
/// doji  = body <= body_threshold * range
/// ```
///
/// The output is `+1.0` when a Doji is detected and `0.0` otherwise. Doji is
/// directionless — no `−1.0` is emitted. Pattern-shape check only — no trend
/// filter is applied; combine with a trend indicator for actionable signals.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Doji, Indicator};
///
/// let mut indicator = Doji::default();
/// let candle = Candle::new(10.0, 11.0, 9.0, 10.0, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
/// ```
#[derive(Debug, Clone)]
pub struct Doji {
    body_threshold: f64,
    has_emitted: bool,
}

impl Default for Doji {
    fn default() -> Self {
        Self::new()
    }
}

impl Doji {
    /// Construct a Doji detector with the default body threshold (`0.1`).
    pub const fn new() -> Self {
        Self {
            body_threshold: 0.1,
            has_emitted: false,
        }
    }

    /// Construct a Doji detector with a custom body / range threshold.
    ///
    /// `body_threshold` must lie in `(0, 1]`.
    pub fn with_threshold(body_threshold: f64) -> Result<Self> {
        if !(body_threshold > 0.0 && body_threshold <= 1.0) {
            return Err(Error::InvalidPeriod {
                message: "doji body threshold must lie in (0, 1]",
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

impl Indicator for Doji {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let range = candle.high - candle.low;
        if range <= 0.0 {
            return Some(0.0);
        }
        let body = (candle.close - candle.open).abs();
        Some(if body <= self.body_threshold * range {
            1.0
        } else {
            0.0
        })
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
        "Doji"
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
        assert!(Doji::with_threshold(0.0).is_err());
        assert!(Doji::with_threshold(-0.1).is_err());
        assert!(Doji::with_threshold(1.5).is_err());
    }

    #[test]
    fn accepts_valid_threshold() {
        let d = Doji::with_threshold(0.05).unwrap();
        assert!((d.body_threshold() - 0.05).abs() < 1e-12);
    }

    #[test]
    fn accessors_and_metadata() {
        let d = Doji::new();
        assert_eq!(d.name(), "Doji");
        assert_eq!(d.warmup_period(), 1);
        assert!(!d.is_ready());
        assert!((d.body_threshold() - 0.1).abs() < 1e-12);
    }

    #[test]
    fn obvious_doji_is_one() {
        let mut d = Doji::new();
        // open == close, full range -> body / range = 0.
        assert_eq!(d.update(c(10.0, 11.0, 9.0, 10.0, 0)), Some(1.0));
        assert!(d.is_ready());
    }

    #[test]
    fn marubozu_is_not_doji() {
        // Big body, no shadows -> body / range = 1.0 > 0.1.
        let mut d = Doji::new();
        assert_eq!(d.update(c(10.0, 12.0, 10.0, 12.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut d = Doji::new();
        assert_eq!(d.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 2.0, base - 2.0, base + 1.0, i)
            })
            .collect();
        let mut a = Doji::new();
        let mut b = Doji::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut d = Doji::new();
        d.update(c(10.0, 11.0, 9.0, 10.0, 0));
        assert!(d.is_ready());
        d.reset();
        assert!(!d.is_ready());
    }
}
