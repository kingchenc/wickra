#![allow(clippy::doc_markdown)]

//! Tom DeMark TD Clop — a 2-bar open/close engulfing reversal.
//!
//! TD Clop ("CLose/OPen") fires when the current bar's open opens beyond **both**
//! the prior bar's open and close, and its close finishes back beyond both — an
//! open-gap that fully reverses, signalling a turn.
//!
//! - **Buy signal** (`+1.0`): `open < open[-1]` AND `open < close[-1]`
//!   (opens below the whole prior body) AND `close > open[-1]` AND
//!   `close > close[-1]` (closes above it).
//! - **Sell signal** (`-1.0`): `open > open[-1]` AND `open > close[-1]` AND
//!   `close < open[-1]` AND `close < close[-1]`.
//! - Otherwise the output is `0.0`.
//!
//! The one-bar lookback means the first value lands on the second candle.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// TD Clop — 2-bar open/close engulfing reversal detector.
/// # Example
///
/// ```
/// use wickra_core::{TdClop, Candle, Indicator};
///
/// let mut indicator = TdClop::new();
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
pub struct TdClop {
    prev: Option<Candle>,
    last_value: Option<f64>,
}

impl TdClop {
    /// Construct a new `TdClop`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Latest emitted signal if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for TdClop {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev) = self.prev else {
            self.prev = Some(candle);
            self.last_value = Some(0.0);
            return Some(0.0);
        };
        let below_body = candle.open < prev.open && candle.open < prev.close;
        let above_body = candle.close > prev.open && candle.close > prev.close;
        let over_body = candle.open > prev.open && candle.open > prev.close;
        let under_body = candle.close < prev.open && candle.close < prev.close;
        let v = if below_body && above_body {
            1.0
        } else if over_body && under_body {
            -1.0
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
        "TDClop"
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
        let td = TdClop::new();
        assert_eq!(td.warmup_period(), 2);
        assert_eq!(td.name(), "TDClop");
        assert!(!td.is_ready());
        assert_eq!(td.value(), None);
    }

    #[test]
    fn first_bar_seeds_without_signal() {
        let mut td = TdClop::new();
        assert_eq!(td.update(c(10.0, 11.0)), Some(0.0));
        assert!(td.update(c(9.0, 12.0)).is_some());
    }

    #[test]
    fn bullish_clop_buy() {
        // prev body [10, 11]. Current open 9 < both, close 12 > both -> buy.
        let mut td = TdClop::new();
        td.update(c(10.0, 11.0));
        assert_eq!(td.update(c(9.0, 12.0)), Some(1.0));
    }

    #[test]
    fn bearish_clop_sell() {
        // prev body [10, 11]. Current open 12 > both, close 9 < both -> sell.
        let mut td = TdClop::new();
        td.update(c(10.0, 11.0));
        assert_eq!(td.update(c(12.0, 9.0)), Some(-1.0));
    }

    #[test]
    fn no_pattern_is_zero() {
        let mut td = TdClop::new();
        td.update(c(10.0, 11.0));
        assert_eq!(td.update(c(10.5, 11.5)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut td = TdClop::new();
        td.update(c(10.0, 11.0));
        td.update(c(9.0, 12.0));
        assert!(td.is_ready());
        td.reset();
        assert!(!td.is_ready());
        assert_eq!(td.update(c(10.0, 11.0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let b = 100.0 + (f64::from(i) * 0.4).sin() * 5.0;
                c(b, b + 0.5)
            })
            .collect();
        let batch = TdClop::new().batch(&candles);
        let mut b = TdClop::new();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
