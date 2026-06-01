//! Inverted Hammer candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Inverted Hammer — a single-bar bullish reversal candidate.
///
/// An Inverted Hammer is the mirror of a Hammer: small real body near the
/// bottom of the bar, a long upper shadow at least twice the body and a
/// short or absent lower shadow.
///
/// ```text
/// body         = |close − open|
/// upper_shadow = high − max(open, close)
/// lower_shadow = min(open, close) − low
/// inverted     = upper_shadow >= 2 * body
///               && lower_shadow <= body
///               && body > 0
/// ```
///
/// Output is `+1.0` when the shape matches, `0.0` otherwise. Pattern-shape
/// check only — no trend filter is applied; combine with a trend indicator
/// for actionable signals.
///
/// # Signed ±1 encoding
///
/// An Inverted Hammer is bullish by definition, so under the uniform
/// candlestick sign convention (`+1.0` bullish, `−1.0` bearish, `0.0` none) it
/// emits `+1.0` when the shape matches and `0.0` otherwise — it never emits
/// `−1.0`. The same geometry read at the top of an uptrend is the bearish
/// `ShootingStar`, which carries the opposite sign.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, InvertedHammer};
///
/// let mut indicator = InvertedHammer::new();
/// // Open 10, close 10.5, low 9.9, high 15: long upper shadow, tiny lower.
/// let candle = Candle::new(10.0, 15.0, 9.9, 10.5, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct InvertedHammer {
    has_emitted: bool,
}

impl InvertedHammer {
    /// Construct a new Inverted Hammer detector.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for InvertedHammer {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let range = candle.high - candle.low;
        if range <= 0.0 {
            return Some(0.0);
        }
        let body = (candle.close - candle.open).abs();
        if body <= 0.0 {
            return Some(0.0);
        }
        let upper = candle.high - candle.open.max(candle.close);
        let lower = candle.open.min(candle.close) - candle.low;
        Some(if upper >= 2.0 * body && lower <= body {
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
        "InvertedHammer"
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
    fn accessors_and_metadata() {
        let h = InvertedHammer::new();
        assert_eq!(h.name(), "InvertedHammer");
        assert_eq!(h.warmup_period(), 1);
        assert!(!h.is_ready());
    }

    #[test]
    fn clean_inverted_hammer_is_one() {
        let mut h = InvertedHammer::new();
        // body 0.5, upper shadow 4.5, lower shadow 0.1.
        assert_eq!(h.update(c(10.0, 15.0, 9.9, 10.5, 0)), Some(1.0));
    }

    #[test]
    fn hammer_shape_is_not_inverted() {
        let mut h = InvertedHammer::new();
        assert_eq!(h.update(c(10.0, 10.6, 5.0, 10.5, 0)), Some(0.0));
    }

    #[test]
    fn doji_is_not_inverted_hammer() {
        let mut h = InvertedHammer::new();
        assert_eq!(h.update(c(10.0, 11.0, 9.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut h = InvertedHammer::new();
        assert_eq!(h.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 4.0, base - 0.1, base + 0.5, i)
            })
            .collect();
        let mut a = InvertedHammer::new();
        let mut b = InvertedHammer::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut h = InvertedHammer::new();
        h.update(c(10.0, 15.0, 9.9, 10.5, 0));
        assert!(h.is_ready());
        h.reset();
        assert!(!h.is_ready());
    }
}
