//! Shooting Star candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Shooting Star — a single-bar bearish reversal candidate.
///
/// A Shooting Star has the same geometry as an Inverted Hammer (small body
/// near the bottom, long upper shadow ≥ 2× body, short lower shadow) but is
/// read bearishly because it appears at the top of an uptrend.
///
/// ```text
/// body         = |close − open|
/// upper_shadow = high − max(open, close)
/// lower_shadow = min(open, close) − low
/// star         = upper_shadow >= 2 * body
///               && lower_shadow <= body
///               && body > 0
/// ```
///
/// Output is `−1.0` when the shape matches, `0.0` otherwise. Pattern-shape
/// check only — no trend filter is applied; combine with a trend indicator
/// for actionable signals.
///
/// # Signed ±1 encoding
///
/// A Shooting Star is bearish by definition, so under the uniform candlestick
/// sign convention (`+1.0` bullish, `−1.0` bearish, `0.0` none) it emits
/// `−1.0` when the shape matches and `0.0` otherwise — it never emits `+1.0`.
/// The same geometry read at the bottom of a downtrend is the bullish
/// `InvertedHammer`, which carries the opposite sign.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, ShootingStar};
///
/// let mut indicator = ShootingStar::new();
/// let candle = Candle::new(10.0, 15.0, 9.9, 10.5, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(-1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ShootingStar {
    has_emitted: bool,
}

impl ShootingStar {
    /// Construct a new Shooting Star detector.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for ShootingStar {
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
            -1.0
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
        "ShootingStar"
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
        let s = ShootingStar::new();
        assert_eq!(s.name(), "ShootingStar");
        assert_eq!(s.warmup_period(), 1);
        assert!(!s.is_ready());
    }

    #[test]
    fn clean_shooting_star_is_minus_one() {
        let mut s = ShootingStar::new();
        assert_eq!(s.update(c(10.0, 15.0, 9.9, 10.5, 0)), Some(-1.0));
    }

    #[test]
    fn hammer_shape_is_not_shooting_star() {
        let mut s = ShootingStar::new();
        assert_eq!(s.update(c(10.0, 10.6, 5.0, 10.5, 0)), Some(0.0));
    }

    #[test]
    fn doji_is_not_shooting_star() {
        let mut s = ShootingStar::new();
        assert_eq!(s.update(c(10.0, 11.0, 9.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut s = ShootingStar::new();
        assert_eq!(s.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 4.0, base - 0.1, base + 0.5, i)
            })
            .collect();
        let mut a = ShootingStar::new();
        let mut b = ShootingStar::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut s = ShootingStar::new();
        s.update(c(10.0, 15.0, 9.9, 10.5, 0));
        assert!(s.is_ready());
        s.reset();
        assert!(!s.is_ready());
    }
}
