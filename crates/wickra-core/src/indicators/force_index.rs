//! Force Index (Elder).

use crate::error::Result;
use crate::indicators::ema::Ema;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Alexander Elder's Force Index — price change scaled by volume, EMA-smoothed.
///
/// ```text
/// raw_t   = (close_t − close_{t−1}) · volume_t
/// Force_t = EMA(raw, period)_t
/// ```
///
/// The raw force is positive on an up-close and negative on a down-close, and
/// its magnitude grows with the volume that backed the move — a big move on
/// heavy volume registers a large force. Smoothing the raw series with an EMA
/// gives a tradeable line; Elder's classic period is `13`. The first candle
/// only establishes the previous close, so the first raw value appears on
/// candle 2 and the first smoothed value on candle `period + 1`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, ForceIndex};
///
/// let mut indicator = ForceIndex::new(13).unwrap();
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
pub struct ForceIndex {
    period: usize,
    prev_close: Option<f64>,
    ema: Ema,
}

impl ForceIndex {
    /// Construct a new Force Index with the given EMA smoothing period.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`](crate::Error::PeriodZero) if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            period,
            prev_close: None,
            ema: Ema::new(period)?,
        })
    }

    /// Configured smoothing period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for ForceIndex {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev) = self.prev_close else {
            // The first candle only establishes the previous close.
            self.prev_close = Some(candle.close);
            return None;
        };
        let raw = (candle.close - prev) * candle.volume;
        self.prev_close = Some(candle.close);
        self.ema.update(raw)
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.ema.reset();
    }

    fn warmup_period(&self) -> usize {
        // One seed candle establishes the first previous close, then the EMA
        // needs `period` raw values.
        self.period + 1
    }

    fn is_ready(&self) -> bool {
        self.ema.is_ready()
    }

    fn name(&self) -> &'static str {
        "ForceIndex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(close: f64, volume: f64, ts: i64) -> Candle {
        Candle::new(close, close, close, close, volume, ts).unwrap()
    }

    #[test]
    fn reference_values() {
        // ForceIndex(1): EMA(1) has alpha = 1, so it passes raw force through.
        //   candle 1 (close 10) only seeds the previous close -> None.
        //   candle 2: raw = (12 - 10) * 100 = +200.
        //   candle 3: raw = (11 - 12) * 200 = -200.
        let mut fi = ForceIndex::new(1).unwrap();
        let out = fi.batch(&[c(10.0, 100.0, 0), c(12.0, 100.0, 1), c(11.0, 200.0, 2)]);
        assert!(out[0].is_none());
        assert_relative_eq!(out[1].unwrap(), 200.0, epsilon = 1e-9);
        assert_relative_eq!(out[2].unwrap(), -200.0, epsilon = 1e-9);
    }

    #[test]
    fn pure_uptrend_is_positive() {
        // Strictly rising closes on constant volume -> every raw force is
        // positive, so the smoothed force is positive too.
        let candles: Vec<Candle> = (1..40)
            .map(|i| c(f64::from(i), 100.0, i64::from(i)))
            .collect();
        let mut fi = ForceIndex::new(13).unwrap();
        for v in fi.batch(&candles).into_iter().flatten() {
            assert!(v > 0.0, "force {v} should be positive in an uptrend");
        }
    }

    #[test]
    fn pure_downtrend_is_negative() {
        let candles: Vec<Candle> = (1..40)
            .rev()
            .map(|i| c(f64::from(i), 100.0, i64::from(i)))
            .collect();
        let mut fi = ForceIndex::new(13).unwrap();
        for v in fi.batch(&candles).into_iter().flatten() {
            assert!(v < 0.0, "force {v} should be negative in a downtrend");
        }
    }

    #[test]
    fn first_value_on_period_plus_one_candle() {
        let candles: Vec<Candle> = (0..12).map(|i| c(10.0 + i as f64, 50.0, i)).collect();
        let mut fi = ForceIndex::new(5).unwrap();
        let out = fi.batch(&candles);
        for (i, v) in out.iter().enumerate().take(5) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(out[5].is_some(), "first force lands at index period");
        assert_eq!(fi.warmup_period(), 6);
    }

    #[test]
    fn rejects_zero_period() {
        assert!(ForceIndex::new(0).is_err());
    }

    /// Cover the const accessor `period` (58-60) and the Indicator-impl
    /// `name` body (93-95). `warmup_period` is exercised elsewhere.
    #[test]
    fn accessors_and_metadata() {
        let fi = ForceIndex::new(13).unwrap();
        assert_eq!(fi.period(), 13);
        assert_eq!(fi.name(), "ForceIndex");
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..30).map(|i| c(10.0 + i as f64, 50.0, i)).collect();
        let mut fi = ForceIndex::new(13).unwrap();
        fi.batch(&candles);
        assert!(fi.is_ready());
        fi.reset();
        assert!(!fi.is_ready());
        assert_eq!(fi.update(candles[0]), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let close = 100.0 + (i as f64 * 0.3).sin() * 8.0;
                c(close, 10.0 + (i % 5) as f64, i)
            })
            .collect();
        let mut a = ForceIndex::new(13).unwrap();
        let mut b = ForceIndex::new(13).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
