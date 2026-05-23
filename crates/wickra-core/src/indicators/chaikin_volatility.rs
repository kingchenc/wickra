//! Chaikin Volatility.

use crate::error::Result;
use crate::indicators::ema::Ema;
use crate::indicators::roc::Roc;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Chaikin Volatility — the rate of change of a smoothed high-low spread.
///
/// ```text
/// spread_t   = high_t − low_t
/// smoothed_t = EMA(spread, ema_period)_t
/// ChaikinVol = 100 · (smoothed_t − smoothed_{t−roc_period}) / smoothed_{t−roc_period}
/// ```
///
/// Marc Chaikin's volatility measure tracks not the *level* of the trading
/// range but how fast it is *widening or narrowing*. A rising value means
/// ranges are expanding (often near a top, as fear spikes); a falling value
/// means they are contracting (often a quiet, complacent market). The classic
/// configuration smooths the spread with a `10`-period EMA and takes its
/// `10`-period rate of change.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, ChaikinVolatility};
///
/// let mut indicator = ChaikinVolatility::new(10, 10).unwrap();
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
pub struct ChaikinVolatility {
    ema: Ema,
    roc: Roc,
    ema_period: usize,
    roc_period: usize,
}

impl ChaikinVolatility {
    /// Construct a Chaikin Volatility with explicit EMA and rate-of-change
    /// periods.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`](crate::Error::PeriodZero) if either period
    /// is zero.
    pub fn new(ema_period: usize, roc_period: usize) -> Result<Self> {
        Ok(Self {
            ema: Ema::new(ema_period)?,
            roc: Roc::new(roc_period)?,
            ema_period,
            roc_period,
        })
    }

    /// Marc Chaikin's classic configuration: `EMA(10)` of the spread, `ROC(10)`.
    pub fn classic() -> Self {
        Self::new(10, 10).expect("classic Chaikin Volatility params are valid")
    }

    /// Configured `(ema_period, roc_period)`.
    pub const fn periods(&self) -> (usize, usize) {
        (self.ema_period, self.roc_period)
    }
}

impl Indicator for ChaikinVolatility {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let spread = candle.high - candle.low;
        let smoothed = self.ema.update(spread)?;
        self.roc.update(smoothed)
    }

    fn reset(&mut self) {
        self.ema.reset();
        self.roc.reset();
    }

    fn warmup_period(&self) -> usize {
        // The EMA emits at candle `ema_period`; the ROC then needs
        // `roc_period` more smoothed values to span its lookback.
        self.ema_period + self.roc_period
    }

    fn is_ready(&self) -> bool {
        self.roc.is_ready()
    }

    fn name(&self) -> &'static str {
        "ChaikinVolatility"
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
    fn constant_range_yields_zero() {
        // A constant high-low spread smooths to a constant EMA, whose rate of
        // change is zero.
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut cv = ChaikinVolatility::new(10, 10).unwrap();
        for v in cv.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn widening_range_reads_positive() {
        // Each bar's range is strictly wider than the last -> expanding
        // volatility -> positive Chaikin Volatility.
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                let half = 1.0 + i as f64 * 0.1;
                c(100.0 + half, 100.0 - half, 100.0, i)
            })
            .collect();
        let mut cv = ChaikinVolatility::new(10, 10).unwrap();
        for v in cv.batch(&candles).into_iter().flatten() {
            assert!(v > 0.0, "an expanding range should read positive, got {v}");
        }
    }

    #[test]
    fn matches_independent_ema_and_roc() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let half = 1.0 + (i as f64 * 0.2).sin().abs() * 2.0;
                c(100.0 + half, 100.0 - half, 100.0, i)
            })
            .collect();
        let mut cv = ChaikinVolatility::new(10, 10).unwrap();
        let mut ema = Ema::new(10).unwrap();
        let mut roc = Roc::new(10).unwrap();
        for (i, candle) in candles.iter().enumerate() {
            let got = cv.update(*candle);
            match ema.update(candle.high - candle.low) {
                Some(e) => {
                    let want = roc.update(e);
                    assert_eq!(got, want, "i={i}");
                }
                None => assert!(got.is_none(), "i={i}"),
            }
        }
    }

    #[test]
    fn first_emission_matches_warmup_period() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut cv = ChaikinVolatility::new(5, 5).unwrap();
        let out = cv.batch(&candles);
        assert_eq!(cv.warmup_period(), 10);
        for (i, v) in out.iter().enumerate().take(9) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(out[9].is_some(), "first value lands at warmup_period - 1");
    }

    #[test]
    fn rejects_zero_period() {
        assert!(ChaikinVolatility::new(0, 10).is_err());
        assert!(ChaikinVolatility::new(10, 0).is_err());
    }

    /// Cover the const accessor `periods` (69-71) and the Indicator-impl
    /// `name` body (99-101). `warmup_period` is exercised elsewhere.
    #[test]
    fn accessors_and_metadata() {
        let cv = ChaikinVolatility::new(10, 10).unwrap();
        assert_eq!(cv.periods(), (10, 10));
        assert_eq!(cv.name(), "ChaikinVolatility");
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut cv = ChaikinVolatility::classic();
        cv.batch(&candles);
        assert!(cv.is_ready());
        cv.reset();
        assert!(!cv.is_ready());
        assert_eq!(cv.update(candles[0]), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let half = 1.0 + (i as f64 * 0.25).sin().abs() * 3.0;
                c(100.0 + half, 100.0 - half, 100.0, i)
            })
            .collect();
        let mut a = ChaikinVolatility::classic();
        let mut b = ChaikinVolatility::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
