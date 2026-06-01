//! Morning Star / Evening Star candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Morning Star / Evening Star — a 3-bar reversal pattern.
///
/// **Morning Star** (bullish, `+1.0`):
/// 1. Bar 1 is a long red candle.
/// 2. Bar 2 has a small body (the "star") — colour does not matter.
/// 3. Bar 3 is a long green candle that closes above the midpoint of Bar 1.
///
/// **Evening Star** (bearish, `−1.0`): the mirror image — long green, small
/// body, long red closing below Bar 1's midpoint.
///
/// The "long" qualifier is enforced by requiring the outer bars' bodies to be
/// at least twice the size of the star's body. Pattern-shape check only — no
/// trend filter is applied; combine with a trend indicator for actionable
/// signals.
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
/// use wickra_core::{Candle, Indicator, MorningEveningStar};
///
/// let mut indicator = MorningEveningStar::new();
/// indicator.update(Candle::new(12.0, 12.2, 9.5, 10.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(9.9, 10.1, 9.7, 9.95, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(10.1, 12.0, 10.0, 11.8, 1.0, 2).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct MorningEveningStar {
    prev: Option<Candle>,
    prev_prev: Option<Candle>,
    has_emitted: bool,
}

impl MorningEveningStar {
    /// Construct a new Morning / Evening Star detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            prev_prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for MorningEveningStar {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let pp = self.prev_prev;
        let p = self.prev;
        self.prev_prev = self.prev;
        self.prev = Some(candle);
        let (Some(b1), Some(b2)) = (pp, p) else {
            return Some(0.0);
        };
        let body1 = (b1.close - b1.open).abs();
        let body2 = (b2.close - b2.open).abs();
        let body3 = (candle.close - candle.open).abs();
        if body1 <= 0.0 || body3 <= 0.0 {
            return Some(0.0);
        }
        // Star body must be small relative to the outer bars.
        if body1 < 2.0 * body2 || body3 < 2.0 * body2 {
            return Some(0.0);
        }
        let mid1 = f64::midpoint(b1.open, b1.close);
        let bar1_red = b1.close < b1.open;
        let bar1_green = b1.close > b1.open;
        let bar3_green = candle.close > candle.open;
        let bar3_red = candle.close < candle.open;
        if bar1_red && bar3_green && candle.close > mid1 {
            Some(1.0)
        } else if bar1_green && bar3_red && candle.close < mid1 {
            Some(-1.0)
        } else {
            Some(0.0)
        }
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
        "MorningEveningStar"
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
        let m = MorningEveningStar::new();
        assert_eq!(m.name(), "MorningEveningStar");
        assert_eq!(m.warmup_period(), 3);
        assert!(!m.is_ready());
    }

    #[test]
    fn morning_star_is_plus_one() {
        let mut m = MorningEveningStar::new();
        // Long red 12 -> 10 (body 2). Star small body. Long green 10.1 -> 11.8 (body 1.7).
        // Mid of bar 1 = 11. Bar 3 closes at 11.8 > 11.
        assert_eq!(m.update(c(12.0, 12.2, 9.5, 10.0, 0)), Some(0.0));
        assert_eq!(m.update(c(9.9, 10.1, 9.7, 9.95, 1)), Some(0.0));
        assert_eq!(m.update(c(10.1, 12.0, 10.0, 11.8, 2)), Some(1.0));
    }

    #[test]
    fn evening_star_is_minus_one() {
        let mut m = MorningEveningStar::new();
        // Long green 10 -> 12, star, long red 11.9 -> 10.2 (body 1.7).
        // Mid of bar 1 = 11. Bar 3 closes at 10.2 < 11.
        assert_eq!(m.update(c(10.0, 12.2, 9.8, 12.0, 0)), Some(0.0));
        assert_eq!(m.update(c(12.1, 12.3, 11.9, 12.05, 1)), Some(0.0));
        assert_eq!(m.update(c(11.9, 12.0, 10.1, 10.2, 2)), Some(-1.0));
    }

    #[test]
    fn big_star_body_is_not_star() {
        let mut m = MorningEveningStar::new();
        m.update(c(12.0, 12.2, 9.5, 10.0, 0));
        // Star body 1.5 -> body1=2, body3 needs to be >= 3 to satisfy 2*body2.
        m.update(c(9.5, 11.5, 9.5, 11.0, 1));
        assert_eq!(m.update(c(10.1, 12.0, 10.0, 11.8, 2)), Some(0.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut m = MorningEveningStar::new();
        assert_eq!(m.update(c(12.0, 12.2, 9.5, 10.0, 0)), Some(0.0));
        assert_eq!(m.update(c(9.9, 10.1, 9.7, 9.95, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                match i % 3 {
                    0 => c(base + 2.0, base + 2.5, base - 0.5, base, i),
                    1 => c(base + 0.1, base + 0.3, base - 0.1, base + 0.15, i),
                    _ => c(base, base + 2.5, base - 0.5, base + 2.0, i),
                }
            })
            .collect();
        let mut a = MorningEveningStar::new();
        let mut b = MorningEveningStar::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut m = MorningEveningStar::new();
        m.update(c(12.0, 12.2, 9.5, 10.0, 0));
        m.update(c(9.9, 10.1, 9.7, 9.95, 1));
        m.update(c(10.1, 12.0, 10.0, 11.8, 2));
        assert!(m.is_ready());
        m.reset();
        assert!(!m.is_ready());
        assert_eq!(m.update(c(12.0, 12.2, 9.5, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn doji_outer_bar_yields_zero() {
        // Bar1 is a doji (body == 0): body1 == 0 -> guard triggers, returns 0.
        let mut m = MorningEveningStar::new();
        m.update(c(10.0, 11.0, 9.0, 10.0, 0)); // doji bar1
        m.update(c(9.9, 10.1, 9.7, 9.95, 1));
        assert_eq!(m.update(c(10.1, 12.0, 10.0, 11.8, 2)), Some(0.0));
    }

    #[test]
    fn same_direction_bars_yield_zero() {
        // Bar1 red, star small, bar3 also red (wrong direction) -> falls through to else 0.
        let mut m = MorningEveningStar::new();
        m.update(c(12.0, 12.2, 9.5, 10.0, 0)); // long red (body 2)
        m.update(c(9.9, 10.1, 9.7, 9.95, 1)); // small star
                                              // Bar3 red, closes below mid (11); doesn't match morning star (bar3 must be green)
                                              // and also doesn't match evening star (bar1 must be green).
        assert_eq!(m.update(c(11.0, 11.2, 9.0, 9.5, 2)), Some(0.0));
    }
}
