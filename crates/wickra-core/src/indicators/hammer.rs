//! Hammer candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Hammer — a single-bar bullish reversal candidate.
///
/// A Hammer has a small real body sitting near the top of the bar, a long
/// lower shadow at least twice the body, and a short or absent upper shadow.
/// It is traditionally read as a rejection of lower prices.
///
/// ```text
/// body         = |close − open|
/// upper_shadow = high − max(open, close)
/// lower_shadow = min(open, close) − low
/// hammer       = lower_shadow >= 2 * body
///               && upper_shadow <= body
///               && body > 0
/// ```
///
/// Output is `+1.0` when the shape matches, `0.0` otherwise. Pattern-shape
/// check only — no trend filter is applied; combine with a trend indicator
/// for actionable signals.
///
/// # Signed ±1 encoding
///
/// A Hammer is bullish by definition, so under the uniform candlestick sign
/// convention (`+1.0` bullish, `−1.0` bearish, `0.0` none) it emits `+1.0`
/// when the shape matches and `0.0` otherwise — it never emits `−1.0`. The
/// same geometry read at the top of an uptrend is the bearish `HangingMan`,
/// which carries the opposite sign.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Hammer, Indicator};
///
/// let mut indicator = Hammer::new();
/// // Open 10, close 10.5, low 5, high 10.6: long lower shadow, tiny upper.
/// let candle = Candle::new(10.0, 10.6, 5.0, 10.5, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Hammer {
    has_emitted: bool,
}

impl Hammer {
    /// Construct a new Hammer detector.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for Hammer {
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
        Some(if lower >= 2.0 * body && upper <= body {
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
        "Hammer"
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
        let h = Hammer::new();
        assert_eq!(h.name(), "Hammer");
        assert_eq!(h.warmup_period(), 1);
        assert!(!h.is_ready());
    }

    #[test]
    fn clean_hammer_is_one() {
        let mut h = Hammer::new();
        // body 0.5 (10 -> 10.5), lower shadow 5.0, upper shadow 0.1.
        assert_eq!(h.update(c(10.0, 10.6, 5.0, 10.5, 0)), Some(1.0));
    }

    #[test]
    fn marubozu_is_not_hammer() {
        let mut h = Hammer::new();
        assert_eq!(h.update(c(10.0, 12.0, 10.0, 12.0, 0)), Some(0.0));
    }

    #[test]
    fn shooting_star_shape_is_not_hammer() {
        // Long upper, short lower -> not a hammer.
        let mut h = Hammer::new();
        assert_eq!(h.update(c(10.5, 15.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn doji_is_not_hammer() {
        let mut h = Hammer::new();
        assert_eq!(h.update(c(10.0, 11.0, 9.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut h = Hammer::new();
        assert_eq!(h.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 2.0, base - 4.0, base + 0.5, i)
            })
            .collect();
        let mut a = Hammer::new();
        let mut b = Hammer::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut h = Hammer::new();
        h.update(c(10.0, 10.6, 5.0, 10.5, 0));
        assert!(h.is_ready());
        h.reset();
        assert!(!h.is_ready());
    }
}
