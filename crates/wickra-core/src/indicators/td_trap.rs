#![allow(clippy::doc_markdown)]

//! Tom DeMark TD Trap — an inside-bar ("trap") followed by a range breakout.
//!
//! A TD Trap forms when one bar is an **inside bar** (its high below and low above
//! the prior bar's), coiling the market; the next bar that closes beyond the trap
//! bar's high or low triggers the directional signal.
//!
//! - **Buy signal** (`+1.0`): the prior bar was an inside bar and the current
//!   `close` is above that inside bar's `high`.
//! - **Sell signal** (`-1.0`): the prior bar was an inside bar and the current
//!   `close` is below that inside bar's `low`.
//! - Otherwise the output is `0.0`.
//!
//! The two-bar lookback (one to set the inside bar, one before it) means the first
//! value lands on the third candle.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// TD Trap — inside-bar breakout signal detector.
/// # Example
///
/// ```
/// use wickra_core::{TdTrap, Candle, Indicator};
///
/// let mut indicator = TdTrap::new();
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
pub struct TdTrap {
    prev1: Option<Candle>,
    prev2: Option<Candle>,
    last_value: Option<f64>,
}

impl TdTrap {
    /// Construct a new `TdTrap`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Latest emitted signal if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for TdTrap {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let (Some(trap), Some(before)) = (self.prev1, self.prev2) else {
            // Not enough history yet: emit a neutral 0.0 while seeding.
            self.prev2 = self.prev1;
            self.prev1 = Some(candle);
            self.last_value = Some(0.0);
            return Some(0.0);
        };
        let is_inside = trap.high < before.high && trap.low > before.low;
        let v = if is_inside && candle.close > trap.high {
            1.0
        } else if is_inside && candle.close < trap.low {
            -1.0
        } else {
            0.0
        };
        self.prev2 = self.prev1;
        self.prev1 = Some(candle);
        self.last_value = Some(v);
        Some(v)
    }

    fn reset(&mut self) {
        self.prev1 = None;
        self.prev2 = None;
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        3
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "TDTrap"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(high: f64, low: f64, close: f64) -> Candle {
        Candle::new_unchecked(f64::midpoint(high, low), high, low, close, 0.0, 0)
    }

    #[test]
    fn accessors_and_metadata() {
        let td = TdTrap::new();
        assert_eq!(td.warmup_period(), 3);
        assert_eq!(td.name(), "TDTrap");
        assert!(!td.is_ready());
        assert_eq!(td.value(), None);
    }

    #[test]
    fn first_two_bars_seed_without_signal() {
        let mut td = TdTrap::new();
        assert_eq!(td.update(c(110.0, 90.0, 100.0)), Some(0.0));
        assert_eq!(td.update(c(108.0, 95.0, 102.0)), Some(0.0));
        assert!(td.update(c(112.0, 100.0, 110.0)).is_some());
    }

    #[test]
    fn inside_then_breakout_up_buys() {
        // bar0 wide [90,110]; bar1 inside [95,108]; bar2 close 109 > 108 -> +1.
        let mut td = TdTrap::new();
        td.update(c(110.0, 90.0, 100.0));
        td.update(c(108.0, 95.0, 102.0)); // inside bar (high<110, low>90)
        assert_eq!(td.update(c(112.0, 100.0, 109.0)), Some(1.0));
    }

    #[test]
    fn inside_then_breakdown_sells() {
        let mut td = TdTrap::new();
        td.update(c(110.0, 90.0, 100.0));
        td.update(c(108.0, 95.0, 102.0)); // inside bar
        assert_eq!(td.update(c(100.0, 92.0, 94.0)), Some(-1.0)); // close 94 < 95
    }

    #[test]
    fn no_inside_bar_is_zero() {
        let mut td = TdTrap::new();
        td.update(c(110.0, 90.0, 100.0));
        td.update(c(115.0, 85.0, 100.0)); // outside bar, not inside
        assert_eq!(td.update(c(120.0, 110.0, 118.0)), Some(0.0));
    }

    #[test]
    fn inside_but_no_breakout_is_zero() {
        let mut td = TdTrap::new();
        td.update(c(110.0, 90.0, 100.0));
        td.update(c(108.0, 95.0, 102.0)); // inside bar
        assert_eq!(td.update(c(107.0, 96.0, 103.0)), Some(0.0)); // close 103 within [95,108]
    }

    #[test]
    fn reset_clears_state() {
        let mut td = TdTrap::new();
        td.update(c(110.0, 90.0, 100.0));
        td.update(c(108.0, 95.0, 102.0));
        td.update(c(112.0, 100.0, 109.0));
        assert!(td.is_ready());
        td.reset();
        assert!(!td.is_ready());
        assert_eq!(td.update(c(110.0, 90.0, 100.0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let b = 100.0 + (f64::from(i) * 0.4).sin() * 6.0;
                c(b + 2.0, b - 2.0, b)
            })
            .collect();
        let batch = TdTrap::new().batch(&candles);
        let mut b = TdTrap::new();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
