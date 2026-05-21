//! Keltner Channels.

use crate::error::{Error, Result};
use crate::indicators::atr::Atr;
use crate::indicators::ema::Ema;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Keltner Channels output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeltnerOutput {
    /// Upper band = middle + multiplier * ATR.
    pub upper: f64,
    /// Middle band = EMA of typical price.
    pub middle: f64,
    /// Lower band = middle - multiplier * ATR.
    pub lower: f64,
}

/// Keltner Channels: an EMA centerline with bands sized by ATR.
#[derive(Debug, Clone)]
pub struct Keltner {
    ema: Ema,
    atr: Atr,
    multiplier: f64,
    ema_period: usize,
    atr_period: usize,
}

impl Keltner {
    /// # Errors
    /// Returns [`Error::PeriodZero`] / [`Error::NonPositiveMultiplier`] on invalid inputs.
    pub fn new(ema_period: usize, atr_period: usize, multiplier: f64) -> Result<Self> {
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            ema: Ema::new(ema_period)?,
            atr: Atr::new(atr_period)?,
            multiplier,
            ema_period,
            atr_period,
        })
    }

    /// Classic configuration: EMA(20), ATR(10), 2.0x multiplier.
    pub fn classic() -> Self {
        Self::new(20, 10, 2.0).expect("classic Keltner parameters are valid")
    }

    /// Configured `(ema_period, atr_period, multiplier)`.
    pub const fn periods(&self) -> (usize, usize, f64) {
        (self.ema_period, self.atr_period, self.multiplier)
    }
}

impl Indicator for Keltner {
    type Input = Candle;
    type Output = KeltnerOutput;

    fn update(&mut self, candle: Candle) -> Option<KeltnerOutput> {
        let mid = self.ema.update(candle.typical_price())?;
        let atr = self.atr.update(candle)?;
        Some(KeltnerOutput {
            upper: mid + self.multiplier * atr,
            middle: mid,
            lower: mid - self.multiplier * atr,
        })
    }

    fn reset(&mut self) {
        self.ema.reset();
        self.atr.reset();
    }

    fn warmup_period(&self) -> usize {
        self.ema_period.max(self.atr_period)
    }

    fn is_ready(&self) -> bool {
        self.ema.is_ready() && self.atr.is_ready()
    }

    fn name(&self) -> &'static str {
        "KeltnerChannels"
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
    fn flat_market_collapses_bands() {
        let candles: Vec<Candle> = (0..50).map(|_| c(10.0, 10.0, 10.0)).collect();
        let mut k = Keltner::new(20, 10, 2.0).unwrap();
        let last = k.batch(&candles).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last.upper, last.middle, epsilon = 1e-9);
        assert_relative_eq!(last.lower, last.middle, epsilon = 1e-9);
    }

    #[test]
    fn upper_above_middle_above_lower() {
        let candles: Vec<Candle> = (0..100)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.2).sin() * 5.0;
                c(m + 1.0, m - 1.0, m)
            })
            .collect();
        let mut k = Keltner::classic();
        for o in k.batch(&candles).into_iter().flatten() {
            assert!(o.upper >= o.middle);
            assert!(o.middle >= o.lower);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..50)
            .map(|i| c(f64::from(i) + 1.0, f64::from(i) - 1.0, f64::from(i)))
            .collect();
        let mut a = Keltner::classic();
        let mut b = Keltner::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn rejects_invalid_input() {
        assert!(Keltner::new(0, 10, 2.0).is_err());
        assert!(Keltner::new(20, 10, 0.0).is_err());
        assert!(Keltner::new(20, 10, -1.0).is_err());
    }
}
