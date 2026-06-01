//! Three Inside Up / Down candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Three Inside Up / Down — a confirmed Harami: the first two bars form a
/// Harami and the third bar confirms direction by closing beyond the first
/// bar's body.
///
/// **Three Inside Up** (`+1.0`):
/// 1. Bar 1 is a long red candle.
/// 2. Bar 2 is a small green candle whose body sits inside Bar 1's body.
/// 3. Bar 3 is a green candle whose close exceeds Bar 1's open.
///
/// **Three Inside Down** (`−1.0`): the mirror — long green, small red inside,
/// red closing below Bar 1's open.
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
/// use wickra_core::{Candle, Indicator, ThreeInside};
///
/// let mut indicator = ThreeInside::new();
/// indicator.update(Candle::new(12.0, 12.5, 9.5, 10.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(10.5, 11.5, 10.4, 11.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(11.0, 13.0, 10.9, 12.5, 1.0, 2).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ThreeInside {
    prev: Option<Candle>,
    prev_prev: Option<Candle>,
    has_emitted: bool,
}

impl ThreeInside {
    /// Construct a new Three Inside Up / Down detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            prev_prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for ThreeInside {
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
        if body1 <= 0.0 || body2 <= 0.0 || body2 >= body1 {
            return Some(0.0);
        }
        let b1_red = b1.close < b1.open;
        let b1_green = b1.close > b1.open;
        let b2_green = b2.close > b2.open;
        let b2_red = b2.close < b2.open;
        let b3_green = candle.close > candle.open;
        let b3_red = candle.close < candle.open;
        // Bullish: prior red, inside green harami, then green confirms above b1.open.
        if b1_red
            && b2_green
            && b2.open >= b1.close
            && b2.close <= b1.open
            && b3_green
            && candle.close > b1.open
        {
            return Some(1.0);
        }
        // Bearish: prior green, inside red harami, then red confirms below b1.open.
        if b1_green
            && b2_red
            && b2.open <= b1.close
            && b2.close >= b1.open
            && b3_red
            && candle.close < b1.open
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
        "ThreeInside"
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
        let t = ThreeInside::new();
        assert_eq!(t.name(), "ThreeInside");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
    }

    #[test]
    fn three_inside_up_is_plus_one() {
        let mut t = ThreeInside::new();
        assert_eq!(t.update(c(12.0, 12.5, 9.5, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(10.5, 11.5, 10.4, 11.0, 1)), Some(0.0));
        // Bar 3 closes 12.5 > Bar 1 open 12.
        assert_eq!(t.update(c(11.0, 13.0, 10.9, 12.5, 2)), Some(1.0));
    }

    #[test]
    fn three_inside_down_is_minus_one() {
        let mut t = ThreeInside::new();
        assert_eq!(t.update(c(10.0, 12.5, 9.5, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(11.5, 11.6, 10.9, 11.0, 1)), Some(0.0));
        // Bar 3 closes 9.5 < Bar 1 open 10.
        assert_eq!(t.update(c(11.0, 11.1, 9.3, 9.5, 2)), Some(-1.0));
    }

    #[test]
    fn unconfirmed_third_bar_yields_zero() {
        let mut t = ThreeInside::new();
        t.update(c(12.0, 12.5, 9.5, 10.0, 0));
        t.update(c(10.5, 11.5, 10.4, 11.0, 1));
        // Bar 3 green but closes below b1.open (12).
        assert_eq!(t.update(c(11.0, 11.8, 10.9, 11.5, 2)), Some(0.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = ThreeInside::new();
        assert_eq!(t.update(c(12.0, 12.5, 9.5, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(10.5, 11.5, 10.4, 11.0, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 1.0, base - 0.5, base + 0.5, i)
            })
            .collect();
        let mut a = ThreeInside::new();
        let mut b = ThreeInside::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = ThreeInside::new();
        t.update(c(12.0, 12.5, 9.5, 10.0, 0));
        t.update(c(10.5, 11.5, 10.4, 11.0, 1));
        t.update(c(11.0, 13.0, 10.9, 12.5, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(12.0, 12.5, 9.5, 10.0, 0)), Some(0.0));
    }
}
