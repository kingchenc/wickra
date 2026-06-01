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
/// # Signed ±1 encoding
///
/// By default the output is `+1.0` when a Doji is detected and `0.0`
/// otherwise — a direction-less detection flag. For a drop-in machine-learning
/// feature where every candlestick pattern shares the same sign convention
/// (`+1.0` bullish, `−1.0` bearish, `0.0` none), switch the detector into
/// signed mode with [`Doji::signed`]. A detected Doji is then classified by
/// where its (negligible) body sits within the bar's range:
///
/// ```text
/// pos = (0.5 * (open + close) − low) / (high − low)
/// pos > 2/3  ->  +1.0   dragonfly  (long lower shadow, bullish)
/// pos < 1/3  ->  −1.0   gravestone (long upper shadow, bearish)
/// else       ->   0.0   long-legged / standard (neutral)
/// ```
///
/// Pattern-shape check only — no trend filter is applied; combine with a trend
/// indicator for actionable signals.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Doji, Indicator};
///
/// // Default: direction-less detection flag.
/// let mut indicator = Doji::default();
/// let candle = Candle::new(10.0, 11.0, 9.0, 10.0, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
///
/// // Signed: a dragonfly Doji (body at the top, long lower shadow) is bullish.
/// let mut signed = Doji::new().signed();
/// let dragonfly = Candle::new(10.0, 10.05, 6.0, 10.0, 1.0, 0).unwrap();
/// assert_eq!(signed.update(dragonfly), Some(1.0));
/// ```
#[derive(Debug, Clone)]
pub struct Doji {
    body_threshold: f64,
    signed: bool,
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
            signed: false,
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
            signed: false,
            has_emitted: false,
        })
    }

    /// Switch to the signed dragonfly / gravestone encoding (consuming builder).
    ///
    /// In signed mode a detected Doji emits `+1.0` (dragonfly, bullish),
    /// `−1.0` (gravestone, bearish) or `0.0` (long-legged / neutral) instead of
    /// the default direction-less `+1.0` detection flag. See the type-level
    /// docs for the exact classification rule.
    #[must_use]
    pub fn signed(mut self) -> Self {
        self.signed = true;
        self
    }

    /// Configured body / range threshold.
    pub fn body_threshold(&self) -> f64 {
        self.body_threshold
    }

    /// Whether this detector emits the signed dragonfly / gravestone encoding.
    pub fn is_signed(&self) -> bool {
        self.signed
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
        if body > self.body_threshold * range {
            return Some(0.0);
        }
        if !self.signed {
            return Some(1.0);
        }
        // Signed mode: classify the Doji by where its (negligible) body sits
        // within the high–low range.
        let body_mid = 0.5 * (candle.open + candle.close);
        let pos = (body_mid - candle.low) / range;
        if pos > 2.0 / 3.0 {
            Some(1.0)
        } else if pos < 1.0 / 3.0 {
            Some(-1.0)
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
        let d = Doji::default();
        assert_eq!(d.name(), "Doji");
        assert_eq!(d.warmup_period(), 1);
        assert!(!d.is_ready());
        assert!(!d.is_signed());
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

    #[test]
    fn signed_accessor_and_builder() {
        let d = Doji::new().signed();
        assert!(d.is_signed());
        // The consuming builder composes with `with_threshold`.
        let t = Doji::with_threshold(0.05).unwrap().signed();
        assert!(t.is_signed());
        assert!((t.body_threshold() - 0.05).abs() < 1e-12);
    }

    #[test]
    fn signed_dragonfly_is_plus_one() {
        // Body at the top of the range, long lower shadow -> bullish.
        let mut d = Doji::new().signed();
        assert_eq!(d.update(c(10.0, 10.05, 6.0, 10.0, 0)), Some(1.0));
    }

    #[test]
    fn signed_gravestone_is_minus_one() {
        // Body at the bottom of the range, long upper shadow -> bearish.
        let mut d = Doji::new().signed();
        assert_eq!(d.update(c(10.0, 14.0, 9.95, 10.0, 0)), Some(-1.0));
    }

    #[test]
    fn signed_long_legged_is_zero() {
        // Body centred, symmetric shadows -> neutral.
        let mut d = Doji::new().signed();
        assert_eq!(d.update(c(10.0, 12.0, 8.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn signed_non_doji_is_zero() {
        // A large body is not a Doji at all -> 0 regardless of position.
        let mut d = Doji::new().signed();
        assert_eq!(d.update(c(10.0, 12.0, 10.0, 12.0, 0)), Some(0.0));
    }

    #[test]
    fn signed_zero_range_is_zero() {
        let mut d = Doji::new().signed();
        assert_eq!(d.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn signed_batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                // Alternate dragonfly / gravestone / centred Doji shapes.
                match i % 3 {
                    0 => c(base, base + 0.05, base - 4.0, base, i),
                    1 => c(base, base + 4.0, base - 0.05, base, i),
                    _ => c(base, base + 2.0, base - 2.0, base, i),
                }
            })
            .collect();
        let mut a = Doji::new().signed();
        let mut b = Doji::new().signed();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn signed_survives_reset() {
        let mut d = Doji::new().signed();
        d.update(c(10.0, 10.05, 6.0, 10.0, 0));
        assert!(d.is_ready());
        d.reset();
        assert!(!d.is_ready());
        // `reset` clears only the streaming state, not the signed configuration.
        assert!(d.is_signed());
        assert_eq!(d.update(c(10.0, 10.05, 6.0, 10.0, 1)), Some(1.0));
    }
}
