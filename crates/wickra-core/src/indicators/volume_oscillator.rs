//! Volume Oscillator.

use crate::error::{Error, Result};
use crate::indicators::sma::Sma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Volume Oscillator — the percent difference between a fast and a slow SMA
/// of the bar volume.
///
/// ```text
/// VO_t = 100 · (SMA(volume, fast)_t − SMA(volume, slow)_t) / SMA(volume, slow)_t
/// ```
///
/// A positive reading means short-term volume is running above the longer-term
/// average (rising participation), a negative reading the opposite. The line is
/// unbounded above and below `-100`, but stays near zero in stable conditions.
/// Classic configuration is `fast = 14, slow = 28`. The first emission lands
/// after `slow` candles. A slow average of `0` (only possible if every volume
/// in the slow window was zero) collapses the output to `0` rather than NaN.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, VolumeOscillator};
///
/// let mut indicator = VolumeOscillator::new(14, 28).unwrap();
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
pub struct VolumeOscillator {
    fast_period: usize,
    slow_period: usize,
    fast: Sma,
    slow: Sma,
}

impl VolumeOscillator {
    /// Construct a Volume Oscillator with the given SMA periods.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if either period is zero, or
    /// [`Error::InvalidPeriod`] if `fast >= slow`.
    pub fn new(fast: usize, slow: usize) -> Result<Self> {
        if fast == 0 || slow == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "VolumeOscillator needs fast < slow",
            });
        }
        Ok(Self {
            fast_period: fast,
            slow_period: slow,
            fast: Sma::new(fast)?,
            slow: Sma::new(slow)?,
        })
    }

    /// Configured `(fast, slow)` periods.
    pub const fn periods(&self) -> (usize, usize) {
        (self.fast_period, self.slow_period)
    }
}

impl Indicator for VolumeOscillator {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let f = self.fast.update(candle.volume);
        let s = self.slow.update(candle.volume);
        let (fast_v, slow_v) = (f?, s?);
        if slow_v == 0.0 {
            // Whole slow window is zero-volume — the ratio is undefined; report 0.
            return Some(0.0);
        }
        Some(100.0 * (fast_v - slow_v) / slow_v)
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
        "VolumeOscillator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(volume: f64, ts: i64) -> Candle {
        Candle::new(10.0, 10.0, 10.0, 10.0, volume, ts).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(
            VolumeOscillator::new(0, 5),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            VolumeOscillator::new(5, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn rejects_fast_geq_slow() {
        assert!(matches!(
            VolumeOscillator::new(10, 10),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            VolumeOscillator::new(28, 14),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let vo = VolumeOscillator::new(14, 28).unwrap();
        assert_eq!(vo.periods(), (14, 28));
        assert_eq!(vo.name(), "VolumeOscillator");
        assert_eq!(vo.warmup_period(), 28);
    }

    #[test]
    fn constant_volume_yields_zero() {
        // Both SMAs equal the constant volume, so (fast - slow) / slow = 0.
        let mut vo = VolumeOscillator::new(3, 6).unwrap();
        let candles: Vec<Candle> = (0..30i64).map(|i| c(500.0, i)).collect();
        for v in vo.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn zero_volume_window_yields_zero() {
        // All bars carry zero volume — slow SMA is 0, defensive branch returns 0.
        let mut vo = VolumeOscillator::new(2, 4).unwrap();
        let candles: Vec<Candle> = (0..10i64).map(|i| c(0.0, i)).collect();
        let out = vo.batch(&candles);
        assert_relative_eq!(out[3].unwrap(), 0.0, epsilon = 1e-12);
    }

    #[test]
    fn reference_value() {
        // fast=2, slow=4 over volumes [10, 20, 30, 40, 50]:
        //   bar 4 (index 3): fast=(40+30)/2=35, slow=(10+20+30+40)/4=25,
        //                    VO = 100·(35-25)/25 = 40.
        let mut vo = VolumeOscillator::new(2, 4).unwrap();
        let candles = [c(10.0, 0), c(20.0, 1), c(30.0, 2), c(40.0, 3), c(50.0, 4)];
        let out = vo.batch(&candles);
        assert!(out[0].is_none() && out[1].is_none() && out[2].is_none());
        assert_relative_eq!(out[3].unwrap(), 40.0, epsilon = 1e-9);
        // bar 5 (index 4): fast=(50+40)/2=45, slow=(20+30+40+50)/4=35,
        //                  VO = 100·(45-35)/35 = 1000/35.
        assert_relative_eq!(out[4].unwrap(), 1000.0 / 35.0, epsilon = 1e-9);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80i64)
            .map(|i| c(100.0 + ((i % 11) as f64) * 5.0, i))
            .collect();
        let mut a = VolumeOscillator::new(14, 28).unwrap();
        let mut b = VolumeOscillator::new(14, 28).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..60i64).map(|i| c(100.0 + (i as f64), i)).collect();
        let mut vo = VolumeOscillator::new(14, 28).unwrap();
        vo.batch(&candles);
        assert!(vo.is_ready());
        vo.reset();
        assert!(!vo.is_ready());
        assert_eq!(vo.update(candles[0]), None);
    }
}
