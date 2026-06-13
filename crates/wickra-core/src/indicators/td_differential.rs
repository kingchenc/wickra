#![allow(clippy::doc_markdown)]

//! Tom DeMark TD Differential — 2-bar momentum-divergence reversal pattern.
//!
//! TD Differential flags an exhaustion-and-reversal candle whose buying or
//! selling pressure has shifted from the prior bar. The rules use the
//! current bar's close vs the prior bar's close (direction filter), the
//! buying pressure `close - low` and the selling pressure `high - close`.
//!
//! - **Buy signal** (`+1.0`) on bar `i` when:
//!   1. `close[i]   <  close[i - 1]`                  (down day)
//!   2. `close[i]   -  low[i]   >  close[i - 1] - low[i - 1]`   (more buying pressure than the prior bar)
//!   3. `high[i]    -  close[i] <  high[i - 1] - close[i - 1]`  (less selling pressure than the prior bar)
//! - **Sell signal** (`-1.0`) on bar `i` when:
//!   1. `close[i]   >  close[i - 1]`
//!   2. `high[i]    -  close[i] >  high[i - 1] - close[i - 1]`
//!   3. `close[i]   -  low[i]   <  close[i - 1] - low[i - 1]`
//! - Otherwise the output is `0.0`.
//!
//! The two-bar lookback means the indicator emits its first value on the
//! second input candle.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// TD Differential — 2-bar reversal pattern detector.
/// # Example
///
/// ```
/// use wickra_core::{TdDifferential, Candle, Indicator};
///
/// let mut indicator = TdDifferential::new();
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
pub struct TdDifferential {
    prev: Option<Candle>,
    last_value: Option<f64>,
}

impl TdDifferential {
    /// Construct a new `TdDifferential`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Latest emitted signal if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for TdDifferential {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev) = self.prev else {
            self.prev = Some(candle);
            return None;
        };
        let buying_now = candle.close - candle.low;
        let buying_prev = prev.close - prev.low;
        let selling_now = candle.high - candle.close;
        let selling_prev = prev.high - prev.close;

        let v = if candle.close < prev.close
            && buying_now > buying_prev
            && selling_now < selling_prev
        {
            1.0
        } else if candle.close > prev.close
            && selling_now > selling_prev
            && buying_now < buying_prev
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
        "TDDifferential"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new_unchecked(close, high, low, close, 0.0, ts)
    }

    #[test]
    fn buy_signal_on_strong_down_close_with_more_buying_pressure() {
        // Prev bar: high=10, low=8, close=9 -> buying=1, selling=1.
        // Curr bar: high=9, low=7, close=8.5 -> close<prev.close (8.5<9),
        // buying=1.5 > 1, selling=0.5 < 1 -> buy signal +1.
        let mut td = TdDifferential::new();
        assert_eq!(td.update(c(10.0, 8.0, 9.0, 0)), None);
        assert_eq!(td.update(c(9.0, 7.0, 8.5, 1)), Some(1.0));
    }

    #[test]
    fn sell_signal_on_strong_up_close_with_more_selling_pressure() {
        // Prev bar: high=10, low=8, close=9 -> buying=1, selling=1.
        // Curr bar: high=12, low=9, close=10.5 -> close>prev.close (10.5>9),
        // selling=1.5 > 1, buying=1.5 > 1 -> condition 3 fails -> no signal.
        // Build a real sell case:
        // Curr bar: high=12, low=9.5, close=10.5 ->
        //   close>prev.close: 10.5>9 ✓
        //   selling = 12 - 10.5 = 1.5 > prev.selling 1 ✓
        //   buying  = 10.5 - 9.5 = 1.0 < prev.buying 1 → NO (need strict <).
        // Curr bar: high=12, low=9.8, close=10.5 ->
        //   buying = 0.7 < 1 ✓; selling = 1.5 > 1 ✓; close>prev ✓ -> sell.
        let mut td = TdDifferential::new();
        assert_eq!(td.update(c(10.0, 8.0, 9.0, 0)), None);
        assert_relative_eq!(td.update(c(12.0, 9.8, 10.5, 1)).unwrap(), -1.0);
    }

    #[test]
    fn no_signal_on_neutral_bar() {
        // Identical bars -> equality everywhere -> zero.
        let mut td = TdDifferential::new();
        assert_eq!(td.update(c(10.0, 8.0, 9.0, 0)), None);
        assert_eq!(td.update(c(10.0, 8.0, 9.0, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.3).sin() * 5.0;
                c(m + 1.0, m - 1.0, m, i64::from(i))
            })
            .collect();
        let mut a = TdDifferential::new();
        let mut b = TdDifferential::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn output_only_in_canonical_set() {
        // Every emitted value is in {-1, 0, +1}.
        let candles: Vec<Candle> = (0..120)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.5).sin() * 5.0;
                c(m + 1.0, m - 1.0, m, i64::from(i))
            })
            .collect();
        let mut td = TdDifferential::new();
        for v in td.batch(&candles).into_iter().flatten() {
            assert!(v == -1.0 || v == 0.0 || v == 1.0, "unexpected value {v}");
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut td = TdDifferential::new();
        td.update(c(10.0, 8.0, 9.0, 0));
        td.update(c(11.0, 9.0, 10.0, 1));
        assert!(td.is_ready());
        td.reset();
        assert!(!td.is_ready());
        assert_eq!(td.update(c(10.0, 8.0, 9.0, 2)), None);
        assert_eq!(td.value(), None);
    }

    #[test]
    fn accessors_and_metadata() {
        let td = TdDifferential::new();
        assert_eq!(td.warmup_period(), 2);
        assert_eq!(td.name(), "TDDifferential");
        assert_eq!(td.value(), None);
    }
}
