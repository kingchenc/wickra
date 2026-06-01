//! Bullish / Bearish Engulfing candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Engulfing — a 2-bar reversal pattern. The current candle's body fully
/// engulfs the prior candle's body and points in the opposite direction.
///
/// ```text
/// prev_body  = |prev.close − prev.open|
/// curr_body  = |curr.close − curr.open|
/// bullish    = prev red & curr green
///             & curr.open <= prev.close & curr.close >= prev.open
///             & curr_body > prev_body
/// bearish    = prev green & curr red
///             & curr.open >= prev.close & curr.close <= prev.open
///             & curr_body > prev_body
/// ```
///
/// Output is `+1.0` for a bullish engulfing, `−1.0` for a bearish one, and
/// `0.0` otherwise. The first bar always returns `0.0` because no previous
/// body exists to engulf. Pattern-shape check only — no trend filter is
/// applied; combine with a trend indicator for actionable signals.
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
/// use wickra_core::{Candle, Engulfing, Indicator};
///
/// let mut indicator = Engulfing::new();
/// // Prior red candle followed by a larger green engulfing candle.
/// indicator.update(Candle::new(11.0, 11.2, 9.8, 10.0, 1.0, 0).unwrap());
/// let out = indicator
///     .update(Candle::new(9.5, 12.0, 9.5, 11.5, 1.0, 1).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Engulfing {
    prev: Option<Candle>,
    has_emitted: bool,
}

impl Engulfing {
    /// Construct a new Engulfing detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for Engulfing {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let prev = self.prev;
        self.prev = Some(candle);
        let Some(p) = prev else {
            return Some(0.0);
        };
        let prev_body = (p.close - p.open).abs();
        let curr_body = (candle.close - candle.open).abs();
        if prev_body <= 0.0 || curr_body <= prev_body {
            return Some(0.0);
        }
        let prev_red = p.close < p.open;
        let prev_green = p.close > p.open;
        let curr_green = candle.close > candle.open;
        let curr_red = candle.close < candle.open;
        if prev_red && curr_green && candle.open <= p.close && candle.close >= p.open {
            Some(1.0)
        } else if prev_green && curr_red && candle.open >= p.close && candle.close <= p.open {
            Some(-1.0)
        } else {
            Some(0.0)
        }
    }

    fn reset(&mut self) {
        self.prev = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "Engulfing"
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
        let e = Engulfing::new();
        assert_eq!(e.name(), "Engulfing");
        assert_eq!(e.warmup_period(), 2);
        assert!(!e.is_ready());
    }

    #[test]
    fn bullish_engulfing_is_plus_one() {
        let mut e = Engulfing::new();
        // Prior red 11 -> 10, current green 9.5 -> 11.5 (body 2 > 1).
        assert_eq!(e.update(c(11.0, 11.2, 9.8, 10.0, 0)), Some(0.0));
        assert_eq!(e.update(c(9.5, 12.0, 9.5, 11.5, 1)), Some(1.0));
    }

    #[test]
    fn bearish_engulfing_is_minus_one() {
        let mut e = Engulfing::new();
        // Prior green 10 -> 11, current red 12 -> 9.
        assert_eq!(e.update(c(10.0, 11.2, 9.8, 11.0, 0)), Some(0.0));
        assert_eq!(e.update(c(12.0, 12.0, 9.0, 9.0, 1)), Some(-1.0));
    }

    #[test]
    fn same_direction_is_not_engulfing() {
        let mut e = Engulfing::new();
        e.update(c(10.0, 11.0, 9.8, 11.0, 0));
        // Another green candle that engulfs but matches direction -> 0.
        assert_eq!(e.update(c(9.5, 12.0, 9.5, 11.5, 1)), Some(0.0));
    }

    #[test]
    fn smaller_body_is_not_engulfing() {
        let mut e = Engulfing::new();
        e.update(c(11.0, 11.2, 8.0, 8.5, 0));
        // Body 0.5 < 2.5 -> not engulfing.
        assert_eq!(e.update(c(8.6, 9.0, 8.4, 8.7, 1)), Some(0.0));
    }

    #[test]
    fn first_bar_returns_zero() {
        let mut e = Engulfing::new();
        assert_eq!(e.update(c(10.0, 11.0, 9.0, 11.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                if i % 3 == 0 {
                    c(base + 1.0, base + 1.5, base - 0.5, base, i)
                } else {
                    c(base - 1.0, base + 2.0, base - 1.5, base + 2.0, i)
                }
            })
            .collect();
        let mut a = Engulfing::new();
        let mut b = Engulfing::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut e = Engulfing::new();
        e.update(c(10.0, 11.0, 9.0, 11.0, 0));
        e.update(c(11.0, 12.0, 10.0, 12.0, 1));
        assert!(e.is_ready());
        e.reset();
        assert!(!e.is_ready());
        // After reset the next bar again has no prev.
        assert_eq!(e.update(c(11.0, 11.2, 9.8, 10.0, 0)), Some(0.0));
    }
}
