//! Awesome Oscillator (Bill Williams).

use crate::error::{Error, Result};
use crate::indicators::sma::Sma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Awesome Oscillator: `SMA(median_price, 5) - SMA(median_price, 34)`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, AwesomeOscillator};
///
/// let mut indicator = AwesomeOscillator::new(3, 10).unwrap();
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
pub struct AwesomeOscillator {
    fast: Sma,
    slow: Sma,
    fast_period: usize,
    slow_period: usize,
}

impl AwesomeOscillator {
    /// # Errors
    /// Returns [`Error::PeriodZero`] for zero periods or [`Error::InvalidPeriod`] when fast >= slow.
    pub fn new(fast: usize, slow: usize) -> Result<Self> {
        if fast == 0 || slow == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "AO fast period must be strictly less than slow",
            });
        }
        Ok(Self {
            fast: Sma::new(fast)?,
            slow: Sma::new(slow)?,
            fast_period: fast,
            slow_period: slow,
        })
    }

    /// Classic Bill Williams configuration: (5, 34).
    pub fn classic() -> Self {
        Self::new(5, 34).expect("classic AO periods are valid")
    }

    /// Configured `(fast, slow)` periods.
    pub const fn periods(&self) -> (usize, usize) {
        (self.fast_period, self.slow_period)
    }
}

impl Indicator for AwesomeOscillator {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let median = candle.median_price();
        let f = self.fast.update(median);
        let s = self.slow.update(median);
        match (f, s) {
            (Some(a), Some(b)) => Some(a - b),
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.fast.reset();
        self.slow.reset();
    }

    fn warmup_period(&self) -> usize {
        self.slow_period
    }

    fn is_ready(&self) -> bool {
        self.slow.is_ready()
    }

    fn name(&self) -> &'static str {
        "AwesomeOscillator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(h: f64, l: f64, cl: f64) -> Candle {
        Candle::new(cl, h, l, cl, 1.0, 0).unwrap()
    }

    #[test]
    fn constant_series_yields_zero() {
        let candles: Vec<Candle> = (0..80).map(|_| c(11.0, 9.0, 10.0)).collect();
        let mut ao = AwesomeOscillator::classic();
        let last = ao.batch(&candles).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn rejects_fast_geq_slow() {
        assert!(AwesomeOscillator::new(34, 5).is_err());
        assert!(AwesomeOscillator::new(5, 5).is_err());
        assert!(AwesomeOscillator::new(0, 5).is_err());
    }

    /// Cover the const accessor `periods` (59-61) and the Indicator-impl
    /// `warmup_period` (83-85) + `name` (91-93). Existing tests never
    /// inspect these metadata methods.
    #[test]
    fn accessors_and_metadata() {
        let ao = AwesomeOscillator::classic();
        assert_eq!(ao.periods(), (5, 34));
        assert_eq!(ao.warmup_period(), 34);
        assert_eq!(ao.name(), "AwesomeOscillator");
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..50)
            .map(|i| c(f64::from(i) + 1.0, f64::from(i) - 1.0, f64::from(i)))
            .collect();
        let mut a = AwesomeOscillator::classic();
        let mut b = AwesomeOscillator::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..50)
            .map(|i| c(f64::from(i) + 1.0, f64::from(i) - 1.0, f64::from(i)))
            .collect();
        let mut ao = AwesomeOscillator::classic();
        ao.batch(&candles);
        assert!(ao.is_ready());
        ao.reset();
        assert!(!ao.is_ready());
        assert_eq!(ao.update(candles[0]), None);
    }
}
