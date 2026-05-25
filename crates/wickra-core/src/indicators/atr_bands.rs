//! ATR Bands.

use crate::error::{Error, Result};
use crate::indicators::atr::Atr;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// ATR Bands output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtrBandsOutput {
    /// Upper band: `close + multiplier · ATR`.
    pub upper: f64,
    /// Middle band: the current close.
    pub middle: f64,
    /// Lower band: `close − multiplier · ATR`.
    pub lower: f64,
}

/// ATR Bands: a close-anchored envelope of width `multiplier · ATR`.
///
/// ```text
/// upper = close + multiplier · ATR(period)
/// lower = close − multiplier · ATR(period)
/// ```
///
/// Unlike [`Keltner`](crate::Keltner) or [`StarcBands`](crate::StarcBands), the
/// centerline is the *raw close* rather than a smoothed average — the band
/// rides the price tick-for-tick. This is the standard volatility-targeting
/// envelope traders use to set initial stop-loss and profit targets: an entry
/// at the close sets a `multiplier · ATR` stop and the symmetric target
/// without ever needing to wait for a moving average to warm up.
///
/// # Example
///
/// ```
/// use wickra_core::{AtrBands, Candle, Indicator};
///
/// let mut indicator = AtrBands::new(14, 3.0).unwrap();
/// let mut last = None;
/// for i in 0..30 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct AtrBands {
    atr: Atr,
    multiplier: f64,
}

impl AtrBands {
    /// # Errors
    /// Returns [`Error::PeriodZero`] / [`Error::NonPositiveMultiplier`] on
    /// invalid inputs.
    pub fn new(period: usize, multiplier: f64) -> Result<Self> {
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            atr: Atr::new(period)?,
            multiplier,
        })
    }

    /// Configured ATR period.
    pub const fn period(&self) -> usize {
        self.atr.period()
    }

    /// Configured ATR multiplier.
    pub const fn multiplier(&self) -> f64 {
        self.multiplier
    }
}

impl Indicator for AtrBands {
    type Input = Candle;
    type Output = AtrBandsOutput;

    fn update(&mut self, candle: Candle) -> Option<AtrBandsOutput> {
        let atr = self.atr.update(candle)?;
        Some(AtrBandsOutput {
            upper: candle.close + self.multiplier * atr,
            middle: candle.close,
            lower: candle.close - self.multiplier * atr,
        })
    }

    fn reset(&mut self) {
        self.atr.reset();
    }

    fn warmup_period(&self) -> usize {
        self.atr.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.atr.is_ready()
    }

    fn name(&self) -> &'static str {
        "AtrBands"
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
    fn rejects_zero_period() {
        assert!(matches!(AtrBands::new(0, 3.0), Err(Error::PeriodZero)));
    }

    #[test]
    fn rejects_non_positive_multiplier() {
        assert!(matches!(
            AtrBands::new(14, 0.0),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            AtrBands::new(14, -1.0),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            AtrBands::new(14, f64::INFINITY),
            Err(Error::NonPositiveMultiplier)
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let ab = AtrBands::new(14, 3.0).unwrap();
        assert_eq!(ab.period(), 14);
        assert_relative_eq!(ab.multiplier(), 3.0, epsilon = 1e-12);
        assert_eq!(ab.warmup_period(), 14);
        assert_eq!(ab.name(), "AtrBands");
    }

    #[test]
    fn flat_market_collapses_bands() {
        let candles: Vec<Candle> = (0..30).map(|_| c(10.0, 10.0, 10.0)).collect();
        let mut ab = AtrBands::new(5, 3.0).unwrap();
        let last = ab.batch(&candles).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last.upper, 10.0, epsilon = 1e-9);
        assert_relative_eq!(last.middle, 10.0, epsilon = 1e-9);
        assert_relative_eq!(last.lower, 10.0, epsilon = 1e-9);
    }

    #[test]
    fn upper_above_middle_above_lower() {
        let candles: Vec<Candle> = (0..50)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.2).sin() * 5.0;
                c(m + 1.0, m - 1.0, m)
            })
            .collect();
        let mut ab = AtrBands::new(14, 3.0).unwrap();
        for o in ab.batch(&candles).into_iter().flatten() {
            assert!(o.upper >= o.middle);
            assert!(o.middle >= o.lower);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| c(f64::from(i) + 2.0, f64::from(i), f64::from(i) + 1.0))
            .collect();
        let mut a = AtrBands::new(10, 2.5).unwrap();
        let mut b = AtrBands::new(10, 2.5).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..20)
            .map(|i| c(f64::from(i) + 1.0, f64::from(i) - 1.0, f64::from(i)))
            .collect();
        let mut ab = AtrBands::new(5, 3.0).unwrap();
        ab.batch(&candles);
        assert!(ab.is_ready());
        ab.reset();
        assert!(!ab.is_ready());
        assert_eq!(ab.update(candles[0]), None);
    }

    /// Reference: with constant high-low spread of 2, ATR(period) converges to
    /// 2 immediately; for multiplier 3 the bands are at `close ± 6`.
    #[test]
    fn reference_values_constant_spread() {
        // Five identical candles with TR = 2 each: ATR seeds to 2 on bar 5.
        let candles: Vec<Candle> = (0..5).map(|_| c(11.0, 9.0, 10.0)).collect();
        let mut ab = AtrBands::new(5, 3.0).unwrap();
        let out = ab.batch(&candles);
        assert!(out[0].is_none() && out[3].is_none());
        let v = out[4].unwrap();
        assert_relative_eq!(v.middle, 10.0, epsilon = 1e-9);
        assert_relative_eq!(v.upper, 16.0, epsilon = 1e-9);
        assert_relative_eq!(v.lower, 4.0, epsilon = 1e-9);
    }
}
