//! Three Outside Up / Down candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Three Outside Up / Down — a confirmed Engulfing: the first two bars form
/// an Engulfing pattern and the third bar confirms direction.
///
/// **Three Outside Up** (`+1.0`):
/// 1. Bar 1 is a red candle.
/// 2. Bar 2 is a green candle that engulfs Bar 1's body.
/// 3. Bar 3 is a green candle with `close > b2.close`.
///
/// **Three Outside Down** (`−1.0`): the mirror — green, bearish engulfing,
/// followed by a red bar closing below Bar 2's close.
///
/// Pattern-shape check only — no trend filter is applied; combine with a trend
/// indicator for actionable signals.
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
/// use wickra_core::{Candle, Indicator, ThreeOutside};
///
/// let mut indicator = ThreeOutside::new();
/// indicator.update(Candle::new(11.0, 11.2, 9.8, 10.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(9.5, 12.0, 9.5, 11.5, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(11.5, 13.0, 11.4, 12.5, 1.0, 2).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ThreeOutside {
    prev: Option<Candle>,
    prev_prev: Option<Candle>,
    has_emitted: bool,
}

impl ThreeOutside {
    /// Construct a new Three Outside Up / Down detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            prev_prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for ThreeOutside {
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
        if body1 <= 0.0 || body2 <= body1 {
            return Some(0.0);
        }
        let b1_red = b1.close < b1.open;
        let b1_green = b1.close > b1.open;
        let b2_green = b2.close > b2.open;
        let b2_red = b2.close < b2.open;
        let b3_green = candle.close > candle.open;
        let b3_red = candle.close < candle.open;
        // Bullish: prior red, engulfing green (b2 fully engulfs b1 body), then green confirms above b2.close.
        if b1_red
            && b2_green
            && b2.open <= b1.close
            && b2.close >= b1.open
            && b3_green
            && candle.close > b2.close
        {
            return Some(1.0);
        }
        // Bearish: prior green, engulfing red, then red confirms below b2.close.
        if b1_green
            && b2_red
            && b2.open >= b1.close
            && b2.close <= b1.open
            && b3_red
            && candle.close < b2.close
        {
            return Some(-1.0);
        }
        Some(0.0)
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
        "ThreeOutside"
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
        let t = ThreeOutside::new();
        assert_eq!(t.name(), "ThreeOutside");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
    }

    #[test]
    fn three_outside_up_is_plus_one() {
        let mut t = ThreeOutside::new();
        assert_eq!(t.update(c(11.0, 11.2, 9.8, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(9.5, 12.0, 9.5, 11.5, 1)), Some(0.0));
        // Bar 3 green close 12.5 > b2.close 11.5.
        assert_eq!(t.update(c(11.5, 13.0, 11.4, 12.5, 2)), Some(1.0));
    }

    #[test]
    fn three_outside_down_is_minus_one() {
        let mut t = ThreeOutside::new();
        assert_eq!(t.update(c(10.0, 11.2, 9.8, 11.0, 0)), Some(0.0));
        assert_eq!(t.update(c(12.0, 12.0, 9.0, 9.0, 1)), Some(0.0));
        // Bar 3 red close 8.0 < b2.close 9.0.
        assert_eq!(t.update(c(9.0, 9.1, 7.9, 8.0, 2)), Some(-1.0));
    }

    #[test]
    fn unconfirmed_third_bar_yields_zero() {
        let mut t = ThreeOutside::new();
        t.update(c(11.0, 11.2, 9.8, 10.0, 0));
        t.update(c(9.5, 12.0, 9.5, 11.5, 1));
        // Bar 3 green but does not exceed b2.close 11.5.
        assert_eq!(t.update(c(11.0, 11.4, 10.9, 11.3, 2)), Some(0.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = ThreeOutside::new();
        assert_eq!(t.update(c(11.0, 11.2, 9.8, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(9.5, 12.0, 9.5, 11.5, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 1.5, base - 0.5, base + 1.0, i)
            })
            .collect();
        let mut a = ThreeOutside::new();
        let mut b = ThreeOutside::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = ThreeOutside::new();
        t.update(c(11.0, 11.2, 9.8, 10.0, 0));
        t.update(c(9.5, 12.0, 9.5, 11.5, 1));
        t.update(c(11.5, 13.0, 11.4, 12.5, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(11.0, 11.2, 9.8, 10.0, 0)), Some(0.0));
    }
}
