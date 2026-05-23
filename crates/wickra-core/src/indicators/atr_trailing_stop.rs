//! ATR Trailing Stop.

use crate::error::{Error, Result};
use crate::indicators::atr::Atr;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// ATR Trailing Stop — a stop level that trails price by a fixed ATR multiple
/// and ratchets in the direction of the trend.
///
/// ```text
/// loss = multiplier · ATR
///
/// stop_t = max(stop_{t−1}, close − loss)   while price holds above the stop
///        = min(stop_{t−1}, close + loss)   while price holds below the stop
///        = close − loss                   on a fresh break above the stop
///        = close + loss                   on a fresh break below the stop
/// ```
///
/// While price stays on one side of the stop the level only ratchets toward
/// price — up in an uptrend, down in a downtrend — never away from it. When a
/// close crosses the stop the level snaps to the opposite side, `loss` away
/// from the new close, flipping the trade. This is the trailing stop used by
/// the well-known "UT Bot"; the first ATR-ready bar seeds the stop below
/// price (a long).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, AtrTrailingStop};
///
/// let mut indicator = AtrTrailingStop::new(14, 3.0).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct AtrTrailingStop {
    atr: Atr,
    multiplier: f64,
    atr_period: usize,
    prev_close: Option<f64>,
    prev_stop: Option<f64>,
}

impl AtrTrailingStop {
    /// Construct an ATR Trailing Stop with an explicit ATR period and multiple.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `atr_period == 0` and
    /// [`Error::NonPositiveMultiplier`] if `multiplier` is not strictly
    /// positive and finite.
    pub fn new(atr_period: usize, multiplier: f64) -> Result<Self> {
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            atr: Atr::new(atr_period)?,
            multiplier,
            atr_period,
            prev_close: None,
            prev_stop: None,
        })
    }

    /// A common configuration: `ATR(14)` with a `3.0` multiplier.
    pub fn classic() -> Self {
        Self::new(14, 3.0).expect("classic ATR Trailing Stop params are valid")
    }

    /// Configured `(atr_period, multiplier)`.
    pub const fn params(&self) -> (usize, f64) {
        (self.atr_period, self.multiplier)
    }
}

impl Indicator for AtrTrailingStop {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let atr = self.atr.update(candle)?;
        let loss = self.multiplier * atr;
        let close = candle.close;

        let stop = match (self.prev_stop, self.prev_close) {
            (Some(prev_stop), Some(prev_close)) => {
                if close > prev_stop && prev_close > prev_stop {
                    // Holding above the stop — ratchet it up only.
                    (close - loss).max(prev_stop)
                } else if close < prev_stop && prev_close < prev_stop {
                    // Holding below the stop — ratchet it down only.
                    (close + loss).min(prev_stop)
                } else if close > prev_stop {
                    // Fresh break above — place the stop below the new close.
                    close - loss
                } else {
                    // Fresh break below — place the stop above the new close.
                    close + loss
                }
            }
            // First ATR-ready bar: seed the stop below price (a long).
            _ => close - loss,
        };

        self.prev_close = Some(close);
        self.prev_stop = Some(stop);
        Some(stop)
    }

    fn reset(&mut self) {
        self.atr.reset();
        self.prev_close = None;
        self.prev_stop = None;
    }

    fn warmup_period(&self) -> usize {
        self.atr_period
    }

    fn is_ready(&self) -> bool {
        self.prev_stop.is_some()
    }

    fn name(&self) -> &'static str {
        "AtrTrailingStop"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(f64::midpoint(high, low), high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn reference_values_flat_market() {
        // Flat candles H=11, L=9, C=10 -> TR=2 -> ATR=2; loss = 3·2 = 6.
        // Seed stop = close - loss = 10 - 6 = 4, and it holds there.
        let candles: Vec<Candle> = (0..20).map(|i| c(11.0, 9.0, 10.0, i)).collect();
        let mut ts = AtrTrailingStop::new(5, 3.0).unwrap();
        for v in ts.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 4.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn uptrend_stop_ratchets_up_and_stays_below_price() {
        let candles: Vec<Candle> = (0..50)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut ts = AtrTrailingStop::new(14, 3.0).unwrap();
        let emitted: Vec<(f64, f64)> = ts
            .batch(&candles)
            .into_iter()
            .zip(candles.iter())
            .filter_map(|(o, c)| o.map(|v| (v, c.close)))
            .collect();
        for w in emitted.windows(2) {
            assert!(
                w[1].0 >= w[0].0 - 1e-9,
                "stop must not loosen in an uptrend"
            );
        }
        for &(stop, close) in &emitted {
            assert!(stop < close, "uptrend stop should sit below the close");
        }
    }

    #[test]
    fn stop_flips_to_the_other_side_when_price_reverses() {
        let mut candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        // A steep decline drags price through the trailing stop.
        candles.extend((0..40).map(|i| {
            let base = 140.0 - 3.0 * i as f64;
            c(base + 1.0, base - 1.0, base, 40 + i)
        }));
        let mut ts = AtrTrailingStop::new(14, 3.0).unwrap();
        let paired: Vec<(f64, f64)> = ts
            .batch(&candles)
            .into_iter()
            .zip(candles.iter())
            .filter_map(|(o, c)| o.map(|v| (v, c.close)))
            .collect();
        assert!(
            paired.iter().any(|&(stop, close)| stop < close),
            "expected a long stretch with the stop below price"
        );
        assert!(
            paired.iter().any(|&(stop, close)| stop > close),
            "expected the stop to flip above price after the reversal"
        );
    }

    #[test]
    fn first_emission_matches_warmup_period() {
        let candles: Vec<Candle> = (0..20)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut ts = AtrTrailingStop::new(8, 3.0).unwrap();
        let out = ts.batch(&candles);
        assert_eq!(ts.warmup_period(), 8);
        for (i, v) in out.iter().enumerate().take(7) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(out[7].is_some(), "first value lands at warmup_period - 1");
    }

    #[test]
    fn rejects_invalid_params() {
        assert!(AtrTrailingStop::new(0, 3.0).is_err());
        assert!(AtrTrailingStop::new(14, 0.0).is_err());
        assert!(AtrTrailingStop::new(14, -1.0).is_err());
        assert!(AtrTrailingStop::new(14, f64::NAN).is_err());
    }

    /// Cover the const accessor `params` (77-79) and the Indicator-impl
    /// `name` body (130-132). `warmup_period` is exercised elsewhere.
    #[test]
    fn accessors_and_metadata() {
        let s = AtrTrailingStop::classic();
        let (atr_p, mult) = s.params();
        assert_eq!(atr_p, 14);
        assert!((mult - 3.0).abs() < 1e-12);
        assert_eq!(s.name(), "AtrTrailingStop");
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut ts = AtrTrailingStop::classic();
        ts.batch(&candles);
        assert!(ts.is_ready());
        ts.reset();
        assert!(!ts.is_ready());
        assert_eq!(ts.update(candles[0]), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.3).sin() * 8.0;
                c(mid + 1.5, mid - 1.5, mid + 0.5, i)
            })
            .collect();
        let mut a = AtrTrailingStop::classic();
        let mut b = AtrTrailingStop::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
