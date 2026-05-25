//! Awesome Oscillator Histogram.

use crate::error::{Error, Result};
use crate::indicators::awesome_oscillator::AwesomeOscillator;
use crate::indicators::sma::Sma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// "Awesome Oscillator Histogram" — the difference between the Awesome
/// Oscillator and its `sma_period`-bar `SMA`. Positive bars mean `AO` is
/// trending up (bullish acceleration); negative bars mean `AO` is trending
/// down (bearish acceleration).
///
/// ```text
/// AO         = SMA(median, fast) − SMA(median, slow)
/// AOHist     = AO − SMA(AO, sma_period)
/// ```
///
/// With Williams' default `sma_period = 5`, this collapses to the existing
/// `AcceleratorOscillator` for `fast = 5, slow = 34, sma_period = 5`; for any
/// other parameterisation this is a more flexible variant.
///
/// # Example
///
/// ```
/// use wickra_core::{AwesomeOscillatorHistogram, Candle, Indicator};
///
/// let mut hist = AwesomeOscillatorHistogram::classic();
/// let mut last = None;
/// for i in 0..80 {
///     let p = 100.0 + f64::from(i);
///     let candle = Candle::new(p, p + 0.5, p - 0.5, p, 1.0, i64::from(i)).unwrap();
///     last = hist.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct AwesomeOscillatorHistogram {
    fast_period: usize,
    slow_period: usize,
    sma_period: usize,
    ao: AwesomeOscillator,
    sma: Sma,
}

impl AwesomeOscillatorHistogram {
    /// # Errors
    /// - [`Error::PeriodZero`] if any period is zero.
    /// - [`Error::InvalidPeriod`] if `fast >= slow`.
    pub fn new(fast: usize, slow: usize, sma_period: usize) -> Result<Self> {
        if fast == 0 || slow == 0 || sma_period == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "AwesomeOscillatorHistogram fast must be strictly less than slow",
            });
        }
        Ok(Self {
            fast_period: fast,
            slow_period: slow,
            sma_period,
            ao: AwesomeOscillator::new(fast, slow)?,
            sma: Sma::new(sma_period)?,
        })
    }

    /// Bill Williams' Accelerator-equivalent defaults `(5, 34, 5)`.
    pub fn classic() -> Self {
        Self::new(5, 34, 5).expect("classic Awesome Oscillator Histogram parameters are valid")
    }

    /// Configured `(fast_period, slow_period, sma_period)`.
    pub const fn periods(&self) -> (usize, usize, usize) {
        (self.fast_period, self.slow_period, self.sma_period)
    }
}

impl Indicator for AwesomeOscillatorHistogram {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let ao = self.ao.update(candle)?;
        let sma = self.sma.update(ao)?;
        Some(ao - sma)
    }

    fn reset(&mut self) {
        self.ao.reset();
        self.sma.reset();
    }

    fn warmup_period(&self) -> usize {
        // AO emits at `slow` candles; the SMA then needs `sma_period - 1`
        // more AO values to fill its window.
        self.slow_period + self.sma_period - 1
    }

    fn is_ready(&self) -> bool {
        self.sma.is_ready()
    }

    fn name(&self) -> &'static str {
        "AwesomeOscillatorHistogram"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(price: f64, ts: i64) -> Candle {
        Candle::new(price, price + 0.5, price - 0.5, price, 1.0, ts).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(
            AwesomeOscillatorHistogram::new(0, 34, 5),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            AwesomeOscillatorHistogram::new(5, 0, 5),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            AwesomeOscillatorHistogram::new(5, 34, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn rejects_fast_geq_slow() {
        assert!(matches!(
            AwesomeOscillatorHistogram::new(34, 5, 5),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let hist = AwesomeOscillatorHistogram::classic();
        assert_eq!(hist.periods(), (5, 34, 5));
        assert_eq!(hist.warmup_period(), 38);
        assert_eq!(hist.name(), "AwesomeOscillatorHistogram");
    }

    #[test]
    fn constant_series_converges_to_zero() {
        // AO of a flat series is 0; SMA of 0 is 0; difference is 0.
        let mut hist = AwesomeOscillatorHistogram::new(3, 5, 3).unwrap();
        let candles: Vec<Candle> = (0..30).map(|i| candle(42.0, i)).collect();
        let out = hist.batch(&candles);
        for v in out.iter().skip(hist.warmup_period() - 1).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn warmup_emits_first_value_at_warmup_period() {
        let mut hist = AwesomeOscillatorHistogram::new(2, 4, 3).unwrap();
        assert_eq!(hist.warmup_period(), 6);
        let candles: Vec<Candle> = (0..8)
            .map(|i| candle(10.0 + f64::from(i), i64::from(i)))
            .collect();
        let out = hist.batch(&candles);
        for v in out.iter().take(5) {
            assert!(v.is_none());
        }
        assert!(out[5].is_some());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..100_i64)
            .map(|i| candle(100.0 + (i as f64 * 0.3).sin() * 5.0, i))
            .collect();
        let batch = AwesomeOscillatorHistogram::classic().batch(&candles);
        let mut b = AwesomeOscillatorHistogram::classic();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn reset_clears_state() {
        let mut hist = AwesomeOscillatorHistogram::classic();
        let candles: Vec<Candle> = (0..80)
            .map(|i| candle(10.0 + f64::from(i), i64::from(i)))
            .collect();
        hist.batch(&candles);
        assert!(hist.is_ready());
        hist.reset();
        assert!(!hist.is_ready());
    }
}
