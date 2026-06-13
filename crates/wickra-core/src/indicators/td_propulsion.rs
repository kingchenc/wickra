#![allow(clippy::doc_markdown)]

//! Tom DeMark TD Propulsion — a 2-bar trend-continuation thrust signal.
//!
//! TD Propulsion qualifies a continuation thrust: the bar opens on the trend side
//! of the prior close and then closes beyond the prior bar's extreme, "propelling"
//! the move forward.
//!
//! - **Propulsion up** (`+1.0`): `open >= close[-1]` (opens at or above the prior
//!   close) AND `close > high[-1]` (closes above the prior high).
//! - **Propulsion down** (`-1.0`): `open <= close[-1]` AND `close < low[-1]`.
//! - Otherwise the output is `0.0`.
//!
//! The one-bar lookback means the first value lands on the second candle.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// TD Propulsion — 2-bar trend-continuation thrust detector.
/// # Example
///
/// ```
/// use wickra_core::{TdPropulsion, Candle, Indicator};
///
/// let mut indicator = TdPropulsion::new();
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
pub struct TdPropulsion {
    prev: Option<Candle>,
    last_value: Option<f64>,
}

impl TdPropulsion {
    /// Construct a new `TdPropulsion`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Latest emitted signal if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for TdPropulsion {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev) = self.prev else {
            self.prev = Some(candle);
            self.last_value = Some(0.0);
            return Some(0.0);
        };
        let v = if candle.open >= prev.close && candle.close > prev.high {
            1.0
        } else if candle.open <= prev.close && candle.close < prev.low {
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
        "TDPropulsion"
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
        let td = TdPropulsion::new();
        assert_eq!(td.warmup_period(), 2);
        assert_eq!(td.name(), "TDPropulsion");
        assert!(!td.is_ready());
        assert_eq!(td.value(), None);
    }

    #[test]
    fn first_bar_seeds_without_signal() {
        let mut td = TdPropulsion::new();
        assert_eq!(td.update(c(10.0, 11.0, 9.0, 10.0)), Some(0.0));
        assert!(td.update(c(10.5, 12.0, 10.0, 11.5)).is_some());
    }

    #[test]
    fn propulsion_up() {
        // prev close 10, high 11. Current open 10.5 >= 10, close 11.5 > 11 -> +1.
        let mut td = TdPropulsion::new();
        td.update(c(9.5, 11.0, 9.0, 10.0));
        assert_eq!(td.update(c(10.5, 12.0, 10.0, 11.5)), Some(1.0));
    }

    #[test]
    fn propulsion_down() {
        // prev close 10, low 9. Current open 9.5 <= 10, close 8.5 < 9 -> -1.
        let mut td = TdPropulsion::new();
        td.update(c(10.5, 11.0, 9.0, 10.0));
        assert_eq!(td.update(c(9.5, 10.0, 8.0, 8.5)), Some(-1.0));
    }

    #[test]
    fn no_thrust_is_zero() {
        let mut td = TdPropulsion::new();
        td.update(c(9.5, 11.0, 9.0, 10.0));
        // close 10.5 not above prior high 11 -> 0.
        assert_eq!(td.update(c(10.5, 10.8, 10.0, 10.5)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut td = TdPropulsion::new();
        td.update(c(9.5, 11.0, 9.0, 10.0));
        td.update(c(10.5, 12.0, 10.0, 11.5));
        assert!(td.is_ready());
        td.reset();
        assert!(!td.is_ready());
        assert_eq!(td.update(c(9.5, 11.0, 9.0, 10.0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let b = 100.0 + (f64::from(i) * 0.4).sin() * 5.0;
                c(b, b + 1.0, b - 1.0, b + 0.3)
            })
            .collect();
        let batch = TdPropulsion::new().batch(&candles);
        let mut b = TdPropulsion::new();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
