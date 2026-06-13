#![allow(clippy::doc_markdown)]

//! Tom DeMark TD Open — open-vs-prior-range gap-reversal signal.
//!
//! TD Open flags bars whose open prints *outside* the prior bar's range
//! but whose subsequent action recovers back inside it — a classic
//! gap-and-fade reversal pattern.
//!
//! - **Buy signal** (`+1.0`) on bar `i` when:
//!   1. `open[i] <  low[i - 1]`                  (gap-down open)
//!   2. `high[i] >  low[i - 1]`                  (high recovers above the prior low)
//! - **Sell signal** (`-1.0`) on bar `i` when:
//!   1. `open[i] >  high[i - 1]`                 (gap-up open)
//!   2. `low[i]  <  high[i - 1]`                 (low fades back under the prior high)
//! - Otherwise the output is `0.0`.
//!
//! The one-bar lookback means the indicator emits its first value on the
//! second input candle.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// TD Open — gap-and-fade reversal detector.
/// # Example
///
/// ```
/// use wickra_core::{TdOpen, Candle, Indicator};
///
/// let mut indicator = TdOpen::new();
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
pub struct TdOpen {
    prev: Option<Candle>,
    last_value: Option<f64>,
}

impl TdOpen {
    /// Construct a new `TdOpen`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Latest emitted signal if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for TdOpen {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev) = self.prev else {
            self.prev = Some(candle);
            return None;
        };
        let v = if candle.open < prev.low && candle.high > prev.low {
            1.0
        } else if candle.open > prev.high && candle.low < prev.high {
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
        "TDOpen"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(open: f64, high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new_unchecked(open, high, low, close, 0.0, ts)
    }

    #[test]
    fn buy_signal_on_gap_down_with_recovery() {
        // Prev bar: low=10. Curr open=9 < 10, curr high=11 > 10 -> buy +1.
        let mut td = TdOpen::new();
        assert_eq!(td.update(c(10.0, 11.0, 10.0, 10.5, 0)), None);
        assert_eq!(td.update(c(9.0, 11.0, 8.5, 9.5, 1)), Some(1.0));
    }

    #[test]
    fn sell_signal_on_gap_up_with_fade() {
        // Prev bar: high=12. Curr open=13 > 12, curr low=11 < 12 -> sell -1.
        let mut td = TdOpen::new();
        assert_eq!(td.update(c(10.0, 12.0, 9.0, 11.0, 0)), None);
        assert_eq!(td.update(c(13.0, 13.5, 11.0, 11.5, 1)), Some(-1.0));
    }

    #[test]
    fn no_signal_on_normal_open_within_range() {
        // Open within previous range -> neither gap condition fires.
        let mut td = TdOpen::new();
        assert_eq!(td.update(c(10.0, 12.0, 9.0, 11.0, 0)), None);
        assert_eq!(td.update(c(10.5, 11.5, 9.5, 11.0, 1)), Some(0.0));
    }

    #[test]
    fn gap_down_without_recovery_is_zero() {
        // Open below prev.low, but high stays below prev.low too -> no signal.
        let mut td = TdOpen::new();
        assert_eq!(td.update(c(10.0, 12.0, 10.0, 11.0, 0)), None);
        // Curr open=9, curr high=9.5 -> high < prev.low (10) -> no buy.
        assert_eq!(td.update(c(9.0, 9.5, 8.5, 9.0, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.3).sin() * 5.0;
                c(m, m + 1.0, m - 1.0, m + 0.3, i64::from(i))
            })
            .collect();
        let mut a = TdOpen::new();
        let mut b = TdOpen::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn output_only_in_canonical_set() {
        let candles: Vec<Candle> = (0..120)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.5).sin() * 5.0;
                c(m, m + 1.0, m - 1.0, m + 0.3, i64::from(i))
            })
            .collect();
        let mut td = TdOpen::new();
        for v in td.batch(&candles).into_iter().flatten() {
            assert!(v == -1.0 || v == 0.0 || v == 1.0, "unexpected value {v}");
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut td = TdOpen::new();
        td.update(c(10.0, 11.0, 9.0, 10.0, 0));
        td.update(c(10.5, 11.5, 9.5, 10.5, 1));
        assert!(td.is_ready());
        td.reset();
        assert!(!td.is_ready());
        assert_eq!(td.update(c(10.0, 11.0, 9.0, 10.0, 2)), None);
        assert_eq!(td.value(), None);
    }

    #[test]
    fn accessors_and_metadata() {
        let td = TdOpen::new();
        assert_eq!(td.warmup_period(), 2);
        assert_eq!(td.name(), "TDOpen");
        assert_eq!(td.value(), None);
    }
}
