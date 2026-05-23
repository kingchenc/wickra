//! On-Balance Volume.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// On-Balance Volume: a cumulative signed-volume series.
///
/// Each candle adds `+volume`, `-volume`, or `0` depending on whether its close
/// is above, below, or equal to the previous close. The first value (after the
/// first candle) is conventionally `0`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Obv};
///
/// let mut indicator = Obv::new();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct Obv {
    prev_close: Option<f64>,
    total: f64,
    has_emitted: bool,
}

impl Obv {
    /// Construct a new OBV instance starting at zero.
    pub const fn new() -> Self {
        Self {
            prev_close: None,
            total: 0.0,
            has_emitted: false,
        }
    }

    /// Current cumulative value if at least one candle has been ingested.
    pub const fn value(&self) -> Option<f64> {
        if self.has_emitted {
            Some(self.total)
        } else {
            None
        }
    }
}

impl Indicator for Obv {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        // The first candle establishes the baseline at 0; subsequent candles
        // add/subtract their volume based on close direction. Equal closes do nothing.
        if let Some(prev) = self.prev_close {
            if candle.close > prev {
                self.total += candle.volume;
            } else if candle.close < prev {
                self.total -= candle.volume;
            }
        }
        self.prev_close = Some(candle.close);
        self.has_emitted = true;
        Some(self.total)
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.total = 0.0;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "OBV"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(close: f64, volume: f64) -> Candle {
        Candle::new(close, close, close, close, volume, 0).unwrap()
    }

    /// Cover the `value()` Some branch (line 47) and the Indicator-impl
    /// `warmup_period` (79-81) + `name` (87-89). `reset_clears_state`
    /// hits only the None branch of `value()`; the metadata methods were
    /// never queried.
    #[test]
    fn accessors_and_metadata() {
        let mut obv = Obv::new();
        assert_eq!(obv.warmup_period(), 1);
        assert_eq!(obv.name(), "OBV");
        assert_eq!(obv.value(), None);
        obv.update(c(10.0, 100.0));
        // Baseline 0 — value() Some branch.
        assert_eq!(obv.value(), Some(0.0));
    }

    #[test]
    fn first_candle_baseline_zero() {
        let mut obv = Obv::new();
        assert_relative_eq!(obv.update(c(10.0, 100.0)).unwrap(), 0.0, epsilon = 1e-12);
    }

    #[test]
    fn up_close_adds_volume() {
        let mut obv = Obv::new();
        obv.update(c(10.0, 100.0)); // baseline 0
        let v = obv.update(c(11.0, 50.0)).unwrap();
        assert_relative_eq!(v, 50.0, epsilon = 1e-12);
    }

    #[test]
    fn down_close_subtracts_volume() {
        let mut obv = Obv::new();
        obv.update(c(10.0, 100.0));
        let v = obv.update(c(9.0, 50.0)).unwrap();
        assert_relative_eq!(v, -50.0, epsilon = 1e-12);
    }

    #[test]
    fn equal_close_does_nothing() {
        let mut obv = Obv::new();
        obv.update(c(10.0, 100.0));
        let v = obv.update(c(10.0, 50.0)).unwrap();
        assert_relative_eq!(v, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn cumulative_sequence() {
        let candles = vec![
            c(10.0, 100.0), // baseline
            c(11.0, 20.0),  // +20
            c(10.5, 30.0),  // -30
            c(10.5, 40.0),  // unchanged
            c(12.0, 10.0),  // +10
        ];
        let mut obv = Obv::new();
        let out = obv.batch(&candles);
        assert_relative_eq!(out[0].unwrap(), 0.0, epsilon = 1e-12);
        assert_relative_eq!(out[1].unwrap(), 20.0, epsilon = 1e-12);
        assert_relative_eq!(out[2].unwrap(), -10.0, epsilon = 1e-12);
        assert_relative_eq!(out[3].unwrap(), -10.0, epsilon = 1e-12);
        assert_relative_eq!(out[4].unwrap(), 0.0, epsilon = 1e-12);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..20)
            .map(|i| {
                let cl = 10.0 + (f64::from(i) * 0.5).sin();
                c(cl, 1.0)
            })
            .collect();
        let mut a = Obv::new();
        let mut b = Obv::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut obv = Obv::new();
        obv.batch(&[c(10.0, 50.0), c(11.0, 30.0)]);
        assert!(obv.is_ready());
        obv.reset();
        assert!(!obv.is_ready());
        assert_eq!(obv.value(), None);
    }
}
