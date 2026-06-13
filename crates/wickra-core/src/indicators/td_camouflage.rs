#![allow(clippy::doc_markdown)]

//! Tom DeMark TD Camouflage — a hidden-strength/weakness 1-bar reversal pattern.
//!
//! TD Camouflage spots a bar that *looks* weak (or strong) on its close-to-close
//! comparison but reveals the opposite intrabar, "camouflaging" a reversal.
//!
//! - **Buy signal** (`+1.0`): `close < close[-1]` (a lower close, looks bearish),
//!   yet `close > open` (it actually closed up on the bar) and `low < low[-1]`
//!   (it dipped to a new low and was bought back) — hidden accumulation.
//! - **Sell signal** (`-1.0`): `close > close[-1]`, `close < open`, and
//!   `high > high[-1]` — hidden distribution.
//! - Otherwise the output is `0.0`.
//!
//! The one-bar lookback means the first value lands on the second candle.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// TD Camouflage — 1-bar hidden-strength/weakness reversal detector.
/// # Example
///
/// ```
/// use wickra_core::{TdCamouflage, Candle, Indicator};
///
/// let mut indicator = TdCamouflage::new();
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
pub struct TdCamouflage {
    prev: Option<Candle>,
    last_value: Option<f64>,
}

impl TdCamouflage {
    /// Construct a new `TdCamouflage`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Latest emitted signal if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for TdCamouflage {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev) = self.prev else {
            self.prev = Some(candle);
            self.last_value = Some(0.0);
            return Some(0.0);
        };
        let v = if candle.close < prev.close && candle.close > candle.open && candle.low < prev.low
        {
            1.0
        } else if candle.close > prev.close && candle.close < candle.open && candle.high > prev.high
        {
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
        "TDCamouflage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(open: f64, high: f64, low: f64, close: f64) -> Candle {
        Candle::new_unchecked(open, high, low, close, 0.0, 0)
    }

    #[test]
    fn accessors_and_metadata() {
        let td = TdCamouflage::new();
        assert_eq!(td.warmup_period(), 2);
        assert_eq!(td.name(), "TDCamouflage");
        assert!(!td.is_ready());
        assert_eq!(td.value(), None);
    }

    #[test]
    fn first_bar_seeds_without_signal() {
        let mut td = TdCamouflage::new();
        assert_eq!(td.update(c(10.0, 11.0, 9.0, 10.0)), Some(0.0));
        assert!(td.update(c(10.0, 11.0, 8.0, 9.5)).is_some());
    }

    #[test]
    fn bullish_camouflage_buy() {
        // prev close 10. Current: close 9.5 < 10 (lower close), close 9.5 > open 9.0,
        // low 7.0 < prev low 8.0 -> buy.
        let mut td = TdCamouflage::new();
        td.update(c(10.0, 11.0, 8.0, 10.0));
        assert_eq!(td.update(c(9.0, 10.0, 7.0, 9.5)), Some(1.0));
    }

    #[test]
    fn bearish_camouflage_sell() {
        // prev close 10. Current: close 10.5 > 10, close 10.5 < open 11.0,
        // high 12.0 > prev high 11.0 -> sell.
        let mut td = TdCamouflage::new();
        td.update(c(10.0, 11.0, 8.0, 10.0));
        assert_eq!(td.update(c(11.0, 12.0, 10.0, 10.5)), Some(-1.0));
    }

    #[test]
    fn no_pattern_is_zero() {
        let mut td = TdCamouflage::new();
        td.update(c(10.0, 11.0, 9.0, 10.0));
        assert_eq!(td.update(c(10.0, 11.5, 9.5, 11.0)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut td = TdCamouflage::new();
        td.update(c(10.0, 11.0, 9.0, 10.0));
        td.update(c(9.0, 10.0, 7.0, 9.5));
        assert!(td.is_ready());
        td.reset();
        assert!(!td.is_ready());
        assert_eq!(td.update(c(10.0, 11.0, 9.0, 10.0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let b = 100.0 + (f64::from(i) * 0.4).sin() * 5.0;
                c(b, b + 1.0, b - 1.0, b + 0.2)
            })
            .collect();
        let batch = TdCamouflage::new().batch(&candles);
        let mut b = TdCamouflage::new();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
