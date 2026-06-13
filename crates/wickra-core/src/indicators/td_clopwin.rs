#![allow(clippy::doc_markdown)]

//! Tom DeMark TD Clopwin — a 2-bar "close/open within" inside-body pattern.
//!
//! TD Clopwin ("CLose/OPen WInthIN") is the inside-body cousin of TD Clop: the
//! current bar's open **and** close both sit within the prior bar's real body,
//! marking a compression bar whose direction hints at the next move.
//!
//! - **Buy signal** (`+1.0`): current `open` and `close` are both inside the prior
//!   bar's body `[min(open,close)[-1], max(open,close)[-1]]` AND `close >= open`
//!   (a bullish inside bar).
//! - **Sell signal** (`-1.0`): both inside the prior body AND `close < open`
//!   (a bearish inside bar).
//! - Otherwise the output is `0.0`.
//!
//! The one-bar lookback means the first value lands on the second candle.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// TD Clopwin — 2-bar inside-body compression pattern detector.
/// # Example
///
/// ```
/// use wickra_core::{TdClopwin, Candle, Indicator};
///
/// let mut indicator = TdClopwin::new();
/// // `None` during warmup, then `Some(_)` once enough bars are seen.
/// let mut out = None;
/// for i in 0..40i64 {
///     let p = 100.0 + (i as f64 * 0.4).sin() * 5.0;
///     let candle = Candle::new(p, p + 1.5, p - 1.5, p + 0.3, 1_000.0, i).unwrap();
///     out = indicator.update(candle);
/// }
/// let _ = out;
/// ```
#[derive(Debug, Clone, Default)]
pub struct TdClopwin {
    prev: Option<Candle>,
    last_value: Option<f64>,
}

impl TdClopwin {
    /// Construct a new `TdClopwin`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Latest emitted signal if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for TdClopwin {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev) = self.prev else {
            self.prev = Some(candle);
            self.last_value = Some(0.0);
            return Some(0.0);
        };
        let body_low = prev.open.min(prev.close);
        let body_high = prev.open.max(prev.close);
        let open_in = candle.open >= body_low && candle.open <= body_high;
        let close_in = candle.close >= body_low && candle.close <= body_high;
        let v = if open_in && close_in {
            if candle.close >= candle.open {
                1.0
            } else {
                -1.0
            }
        } else {
            0.0
        };
        self.prev = Some(candle);
        self.last_value = Some(v);
        Some(v)
    }

    fn reset(&mut self) {
        self.prev = None;
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "TDClopwin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(open: f64, close: f64) -> Candle {
        let high = open.max(close) + 1.0;
        let low = open.min(close) - 1.0;
        Candle::new_unchecked(open, high, low, close, 0.0, 0)
    }

    #[test]
    fn accessors_and_metadata() {
        let td = TdClopwin::new();
        assert_eq!(td.warmup_period(), 2);
        assert_eq!(td.name(), "TDClopwin");
        assert!(!td.is_ready());
        assert_eq!(td.value(), None);
    }

    #[test]
    fn first_bar_seeds_without_signal() {
        let mut td = TdClopwin::new();
        assert_eq!(td.update(c(10.0, 14.0)), Some(0.0));
        assert!(td.update(c(11.0, 13.0)).is_some());
    }

    #[test]
    fn bullish_inside_body_buy() {
        // prev body [10, 14]. Current open 11, close 13 both inside, close>open -> +1.
        let mut td = TdClopwin::new();
        td.update(c(10.0, 14.0));
        assert_eq!(td.update(c(11.0, 13.0)), Some(1.0));
    }

    #[test]
    fn bearish_inside_body_sell() {
        // prev body [10, 14]. Current open 13, close 11 inside, close<open -> -1.
        let mut td = TdClopwin::new();
        td.update(c(10.0, 14.0));
        assert_eq!(td.update(c(13.0, 11.0)), Some(-1.0));
    }

    #[test]
    fn outside_body_is_zero() {
        let mut td = TdClopwin::new();
        td.update(c(10.0, 14.0));
        // close 16 outside the prior body -> 0.
        assert_eq!(td.update(c(11.0, 16.0)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut td = TdClopwin::new();
        td.update(c(10.0, 14.0));
        td.update(c(11.0, 13.0));
        assert!(td.is_ready());
        td.reset();
        assert!(!td.is_ready());
        assert_eq!(td.update(c(10.0, 14.0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let b = 100.0 + (f64::from(i) * 0.4).sin() * 5.0;
                c(b, b + 0.3)
            })
            .collect();
        let batch = TdClopwin::new().batch(&candles);
        let mut b = TdClopwin::new();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
