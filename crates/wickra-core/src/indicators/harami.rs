//! Bullish / Bearish Harami candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Harami — a 2-bar reversal pattern. The current candle's body sits entirely
/// inside the previous candle's body and points in the opposite direction.
///
/// ```text
/// prev_body  = |prev.close − prev.open|
/// curr_body  = |curr.close − curr.open|
/// bullish    = prev red & curr green
///             & curr.open >= prev.close & curr.close <= prev.open
///             & curr_body < prev_body
/// bearish    = prev green & curr red
///             & curr.open <= prev.close & curr.close >= prev.open
///             & curr_body < prev_body
/// ```
///
/// Output is `+1.0` for a bullish harami (small green inside a prior red),
/// `−1.0` for a bearish harami (small red inside a prior green), `0.0`
/// otherwise. The first bar always returns `0.0`. Pattern-shape check only —
/// no trend filter is applied; combine with a trend indicator for actionable
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
/// use wickra_core::{Candle, Harami, Indicator};
///
/// let mut indicator = Harami::new();
/// indicator.update(Candle::new(12.0, 12.5, 9.5, 10.0, 1.0, 0).unwrap());
/// let out = indicator
///     .update(Candle::new(10.5, 11.5, 10.4, 11.0, 1.0, 1).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Harami {
    prev: Option<Candle>,
    has_emitted: bool,
}

impl Harami {
    /// Construct a new Harami detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for Harami {
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
        if prev_body <= 0.0 || curr_body <= 0.0 || curr_body >= prev_body {
            return Some(0.0);
        }
        let prev_red = p.close < p.open;
        let prev_green = p.close > p.open;
        let curr_green = candle.close > candle.open;
        let curr_red = candle.close < candle.open;
        // Bullish: small green strictly inside prior red body (open >= prev.close, close <= prev.open).
        if prev_red && curr_green && candle.open >= p.close && candle.close <= p.open {
            Some(1.0)
        } else if prev_green && curr_red && candle.open <= p.close && candle.close >= p.open {
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
        "Harami"
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
        let h = Harami::new();
        assert_eq!(h.name(), "Harami");
        assert_eq!(h.warmup_period(), 2);
        assert!(!h.is_ready());
    }

    #[test]
    fn bullish_harami_is_plus_one() {
        let mut h = Harami::new();
        // Prior red 12 -> 10 (body 2). Current green 10.5 -> 11 inside.
        assert_eq!(h.update(c(12.0, 12.5, 9.5, 10.0, 0)), Some(0.0));
        assert_eq!(h.update(c(10.5, 11.5, 10.4, 11.0, 1)), Some(1.0));
    }

    #[test]
    fn bearish_harami_is_minus_one() {
        let mut h = Harami::new();
        // Prior green 10 -> 12 (body 2). Current red 11.5 -> 11 inside.
        assert_eq!(h.update(c(10.0, 12.5, 9.5, 12.0, 0)), Some(0.0));
        assert_eq!(h.update(c(11.5, 11.6, 10.9, 11.0, 1)), Some(-1.0));
    }

    #[test]
    fn larger_body_is_not_harami() {
        let mut h = Harami::new();
        h.update(c(11.0, 11.2, 9.8, 10.0, 0));
        // Current body bigger than prior.
        assert_eq!(h.update(c(9.5, 12.0, 9.5, 11.5, 1)), Some(0.0));
    }

    #[test]
    fn same_direction_is_not_harami() {
        let mut h = Harami::new();
        h.update(c(10.0, 12.5, 9.5, 12.0, 0));
        // Smaller candle but also green -> 0.
        assert_eq!(h.update(c(11.0, 11.6, 10.9, 11.5, 1)), Some(0.0));
    }

    #[test]
    fn first_bar_returns_zero() {
        let mut h = Harami::new();
        assert_eq!(h.update(c(10.0, 11.0, 9.0, 11.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                if i % 2 == 0 {
                    c(base + 2.0, base + 2.5, base - 0.5, base, i)
                } else {
                    c(base + 1.0, base + 1.5, base + 0.7, base + 1.3, i)
                }
            })
            .collect();
        let mut a = Harami::new();
        let mut b = Harami::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut h = Harami::new();
        h.update(c(12.0, 12.5, 9.5, 10.0, 0));
        h.update(c(10.5, 11.5, 10.4, 11.0, 1));
        assert!(h.is_ready());
        h.reset();
        assert!(!h.is_ready());
        // After reset the next bar again has no prev.
        assert_eq!(h.update(c(12.0, 12.5, 9.5, 10.0, 0)), Some(0.0));
    }
}
