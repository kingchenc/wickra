#![allow(clippy::doc_markdown)]

//! Tom DeMark TD Range Projection — next-bar high/low projection from the
//! current bar's open/high/low/close (DeMark's "X-projection" pivot).
//!
//! After each bar closes, DeMark proposes a projected high and low for the
//! *next* bar derived from a pivot weighted by the relationship between
//! the close and the open:
//!
//! ```text
//! if close < open:    pivot_sum = high + 2*low  + close
//! if close > open:    pivot_sum = 2*high + low  + close
//! if close == open:   pivot_sum = high + low    + 2*close
//!
//! projected_high = pivot_sum / 2 - low
//! projected_low  = pivot_sum / 2 - high
//! ```
//!
//! The indicator is stateless beyond the current bar — every bar's input
//! deterministically produces a projection — but it is wrapped in the same
//! `Indicator` state-machine API as the rest of Wickra so it composes with
//! the streaming/batch infrastructure.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Output of [`TdRangeProjection`]: the projected high and low for the
/// next bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TdRangeProjectionOutput {
    /// Projected high for the next bar.
    pub high: f64,
    /// Projected low for the next bar.
    pub low: f64,
}

/// TD Range Projection — next-bar high/low pivot.
#[derive(Debug, Clone, Default)]
pub struct TdRangeProjection {
    last_value: Option<TdRangeProjectionOutput>,
}

impl TdRangeProjection {
    /// Construct a new `TdRangeProjection`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Latest projection if available.
    pub const fn value(&self) -> Option<TdRangeProjectionOutput> {
        self.last_value
    }
}

impl Indicator for TdRangeProjection {
    type Input = Candle;
    type Output = TdRangeProjectionOutput;

    fn update(&mut self, candle: Candle) -> Option<TdRangeProjectionOutput> {
        let pivot_sum = if candle.close < candle.open {
            candle.high + 2.0 * candle.low + candle.close
        } else if candle.close > candle.open {
            2.0 * candle.high + candle.low + candle.close
        } else {
            candle.high + candle.low + 2.0 * candle.close
        };
        let half = pivot_sum / 2.0;
        let out = TdRangeProjectionOutput {
            high: half - candle.low,
            low: half - candle.high,
        };
        self.last_value = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "TDRangeProjection"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(open: f64, high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new_unchecked(open, high, low, close, 0.0, ts)
    }

    #[test]
    fn bullish_bar_close_above_open_uses_double_high_pivot() {
        // open=10, high=12, low=9, close=11 -> close > open
        // pivot_sum = 2*12 + 9 + 11 = 44; half = 22.
        // projHigh = 22 - 9 = 13; projLow = 22 - 12 = 10.
        let mut p = TdRangeProjection::new();
        let v = p.update(c(10.0, 12.0, 9.0, 11.0, 0)).unwrap();
        assert_relative_eq!(v.high, 13.0, epsilon = 1e-12);
        assert_relative_eq!(v.low, 10.0, epsilon = 1e-12);
    }

    #[test]
    fn bearish_bar_close_below_open_uses_double_low_pivot() {
        // open=11, high=12, low=9, close=10 -> close < open
        // pivot_sum = 12 + 2*9 + 10 = 40; half = 20.
        // projHigh = 20 - 9 = 11; projLow = 20 - 12 = 8.
        let mut p = TdRangeProjection::new();
        let v = p.update(c(11.0, 12.0, 9.0, 10.0, 0)).unwrap();
        assert_relative_eq!(v.high, 11.0, epsilon = 1e-12);
        assert_relative_eq!(v.low, 8.0, epsilon = 1e-12);
    }

    #[test]
    fn doji_close_equals_open_uses_double_close_pivot() {
        // open=close=10, high=12, low=9 -> doji branch.
        // pivot_sum = 12 + 9 + 2*10 = 41; half = 20.5.
        // projHigh = 20.5 - 9 = 11.5; projLow = 20.5 - 12 = 8.5.
        let mut p = TdRangeProjection::new();
        let v = p.update(c(10.0, 12.0, 9.0, 10.0, 0)).unwrap();
        assert_relative_eq!(v.high, 11.5, epsilon = 1e-12);
        assert_relative_eq!(v.low, 8.5, epsilon = 1e-12);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.3).sin() * 5.0;
                c(m, m + 1.0, m - 1.0, m + 0.3, i64::from(i))
            })
            .collect();
        let mut a = TdRangeProjection::new();
        let mut b = TdRangeProjection::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut p = TdRangeProjection::new();
        p.update(c(10.0, 12.0, 9.0, 11.0, 0));
        assert!(p.is_ready());
        p.reset();
        assert!(!p.is_ready());
        assert_eq!(p.value(), None);
    }

    #[test]
    fn accessors_and_metadata() {
        let p = TdRangeProjection::new();
        assert_eq!(p.warmup_period(), 1);
        assert_eq!(p.name(), "TDRangeProjection");
        assert_eq!(p.value(), None);
    }
}
