//! STARC Bands (Stoller Average Range Channel).

use crate::error::{Error, Result};
use crate::indicators::atr::Atr;
use crate::indicators::sma::Sma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// STARC Bands output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StarcBandsOutput {
    /// Upper band: `middle + multiplier · ATR`.
    pub upper: f64,
    /// Middle band: SMA of close.
    pub middle: f64,
    /// Lower band: `middle − multiplier · ATR`.
    pub lower: f64,
}

/// STARC Bands (Stoller Average Range Channel): a close-SMA centerline with
/// bands sized by ATR.
///
/// ```text
/// middle = SMA(close, sma_period)
/// upper  = middle + multiplier · ATR(atr_period)
/// lower  = middle − multiplier · ATR(atr_period)
/// ```
///
/// STARC and [`Keltner`](crate::Keltner) share the same skeleton — moving
/// average plus an ATR offset — but Keltner's centerline is an `EMA` of the
/// typical price while STARC uses an `SMA` of the close. The SMA gives a
/// flatter, less reactive midline that traders use to pick the larger swing
/// targets; Stoller's reference parameters are `SMA(6)` over the close with
/// `ATR(15)` and a multiplier of `2.0`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, StarcBands};
///
/// let mut indicator = StarcBands::new(6, 15, 2.0).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct StarcBands {
    sma: Sma,
    atr: Atr,
    multiplier: f64,
    sma_period: usize,
    atr_period: usize,
}

impl StarcBands {
    /// # Errors
    /// Returns [`Error::PeriodZero`] / [`Error::NonPositiveMultiplier`] on
    /// invalid inputs.
    pub fn new(sma_period: usize, atr_period: usize, multiplier: f64) -> Result<Self> {
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            sma: Sma::new(sma_period)?,
            atr: Atr::new(atr_period)?,
            multiplier,
            sma_period,
            atr_period,
        })
    }

    /// Stoller's classic configuration: SMA(6), ATR(15), multiplier 2.0.
    pub fn classic() -> Self {
        Self::new(6, 15, 2.0).expect("classic STARC parameters are valid")
    }

    /// Configured `(sma_period, atr_period, multiplier)`.
    pub const fn parameters(&self) -> (usize, usize, f64) {
        (self.sma_period, self.atr_period, self.multiplier)
    }
}

impl Indicator for StarcBands {
    type Input = Candle;
    type Output = StarcBandsOutput;

    fn update(&mut self, candle: Candle) -> Option<StarcBandsOutput> {
        // Feed both unconditionally so SMA and ATR warm up in parallel.
        let mid = self.sma.update(candle.close);
        let atr = self.atr.update(candle);
        let (mid, atr) = (mid?, atr?);
        Some(StarcBandsOutput {
            upper: mid + self.multiplier * atr,
            middle: mid,
            lower: mid - self.multiplier * atr,
        })
    }

    fn reset(&mut self) {
        self.sma.reset();
        self.atr.reset();
    }

    fn warmup_period(&self) -> usize {
        self.sma_period.max(self.atr_period)
    }

    fn is_ready(&self) -> bool {
        self.sma.is_ready() && self.atr.is_ready()
    }

    fn name(&self) -> &'static str {
        "StarcBands"
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
    fn rejects_invalid_input() {
        assert!(StarcBands::new(0, 14, 2.0).is_err());
        assert!(StarcBands::new(6, 0, 2.0).is_err());
        assert!(StarcBands::new(6, 14, 0.0).is_err());
        assert!(StarcBands::new(6, 14, -1.0).is_err());
        assert!(StarcBands::new(6, 14, f64::NAN).is_err());
    }

    #[test]
    fn accessors_and_metadata() {
        let s = StarcBands::new(6, 15, 2.0).unwrap();
        let (sp, ap, m) = s.parameters();
        assert_eq!(sp, 6);
        assert_eq!(ap, 15);
        assert_relative_eq!(m, 2.0, epsilon = 1e-12);
        assert_eq!(s.warmup_period(), 15);
        assert_eq!(s.name(), "StarcBands");
    }

    #[test]
    fn flat_market_collapses_bands() {
        let candles: Vec<Candle> = (0..50).map(|_| c(10.0, 10.0, 10.0)).collect();
        let mut s = StarcBands::new(6, 15, 2.0).unwrap();
        let last = s.batch(&candles).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last.upper, last.middle, epsilon = 1e-9);
        assert_relative_eq!(last.lower, last.middle, epsilon = 1e-9);
    }

    #[test]
    fn upper_above_middle_above_lower() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.2).sin() * 5.0;
                c(m + 1.0, m - 1.0, m)
            })
            .collect();
        let mut s = StarcBands::classic();
        for o in s.batch(&candles).into_iter().flatten() {
            assert!(o.upper >= o.middle);
            assert!(o.middle >= o.lower);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| c(f64::from(i) + 1.0, f64::from(i) - 1.0, f64::from(i)))
            .collect();
        let mut a = StarcBands::classic();
        let mut b = StarcBands::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..30)
            .map(|i| c(f64::from(i) + 1.0, f64::from(i) - 1.0, f64::from(i)))
            .collect();
        let mut s = StarcBands::classic();
        s.batch(&candles);
        assert!(s.is_ready());
        s.reset();
        assert!(!s.is_ready());
        assert_eq!(s.update(candles[0]), None);
    }

    /// STARC must equal feeding independent SMA(close) and ATR siblings and
    /// combining them.
    #[test]
    fn matches_independent_sma_and_atr() {
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.2).sin() * 5.0;
                c(m + 1.5, m - 1.5, m)
            })
            .collect();
        let mut s = StarcBands::new(6, 15, 2.0).unwrap();
        let mut sma = Sma::new(6).unwrap();
        let mut atr = Atr::new(15).unwrap();
        for candle in &candles {
            let got = s.update(*candle);
            let mid = sma.update(candle.close);
            let a = atr.update(*candle);
            if let (Some(m), Some(av)) = (mid, a) {
                let o = got.expect("STARC emits once both ready");
                assert_relative_eq!(o.middle, m, epsilon = 1e-9);
                assert_relative_eq!(o.upper, m + 2.0 * av, epsilon = 1e-9);
                assert_relative_eq!(o.lower, m - 2.0 * av, epsilon = 1e-9);
            } else {
                assert!(got.is_none());
            }
        }
    }
}
