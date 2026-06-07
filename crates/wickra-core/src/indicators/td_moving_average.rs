#![allow(clippy::doc_markdown)]

//! Tom DeMark TD Moving Averages — the ST1 (fast) and ST2 (slow) trend ribbon.

use crate::error::{Error, Result};
use crate::indicators::sma::Sma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Output of [`TdMovingAverage`]: the fast (`st1`) and slow (`st2`) moving-average
/// lines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TdMovingAverageOutput {
    /// ST1 — the fast (short) moving average.
    pub st1: f64,
    /// ST2 — the slow (long) moving average.
    pub st2: f64,
}

/// Tom DeMark **TD Moving Averages** — a two-line trend ribbon (ST1 fast, ST2
/// slow) computed on the median price, whose relationship defines the trend.
///
/// ```text
/// price = (high + low) / 2          (median price)
/// st1   = SMA(price, period_st1)    (fast / "Sequential Trend 1")
/// st2   = SMA(price, period_st2)    (slow / "Sequential Trend 2")
/// ```
///
/// DeMark's moving-average pair frames the trend objectively: when `st1` is above
/// `st2` the trend is up, below it down, and the cross marks the change. Using the
/// **median price** rather than the close de-emphasises closing noise. This is a
/// streaming dual-SMA implementation of the ST1/ST2 ribbon; read the lines and
/// their crossover exactly as a fast/slow moving-average system.
///
/// `period_st1` must be strictly smaller than `period_st2`. The first value lands
/// once the slow average is seeded (`period_st2` inputs). Each `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, TdMovingAverage};
///
/// let mut indicator = TdMovingAverage::new(5, 13).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let base = 100.0 + f64::from(i);
///     let c = Candle::new(base, base + 1.0, base - 1.0, base, 1_000.0, 0).unwrap();
///     last = indicator.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct TdMovingAverage {
    st1: Sma,
    st2: Sma,
    period_st1: usize,
    period_st2: usize,
    last: Option<TdMovingAverageOutput>,
}

impl TdMovingAverage {
    /// Construct TD Moving Averages with the given fast and slow periods.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if either period is `0`, and
    /// [`Error::InvalidPeriod`] if `period_st1 >= period_st2`.
    pub fn new(period_st1: usize, period_st2: usize) -> Result<Self> {
        if period_st1 == 0 || period_st2 == 0 {
            return Err(Error::PeriodZero);
        }
        if period_st1 >= period_st2 {
            return Err(Error::InvalidPeriod {
                message: "TD moving average ST1 period must be strictly less than ST2",
            });
        }
        Ok(Self {
            st1: Sma::new(period_st1)?,
            st2: Sma::new(period_st2)?,
            period_st1,
            period_st2,
            last: None,
        })
    }

    /// Configured `(period_st1, period_st2)`.
    pub const fn periods(&self) -> (usize, usize) {
        (self.period_st1, self.period_st2)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<TdMovingAverageOutput> {
        self.last
    }
}

impl Indicator for TdMovingAverage {
    type Input = Candle;
    type Output = TdMovingAverageOutput;

    fn update(&mut self, candle: Candle) -> Option<TdMovingAverageOutput> {
        let price = candle.median_price();
        let fast = self.st1.update(price);
        let slow = self.st2.update(price);
        if let (Some(st1), Some(st2)) = (fast, slow) {
            let out = TdMovingAverageOutput { st1, st2 };
            self.last = Some(out);
            return Some(out);
        }
        None
    }

    fn reset(&mut self) {
        self.st1.reset();
        self.st2.reset();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.period_st2
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "TDMovingAverage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(median: f64) -> Candle {
        Candle::new_unchecked(median, median + 1.0, median - 1.0, median, 1_000.0, 0)
    }

    #[test]
    fn rejects_invalid_periods() {
        assert!(matches!(
            TdMovingAverage::new(0, 13),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            TdMovingAverage::new(13, 5),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            TdMovingAverage::new(5, 5),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let td = TdMovingAverage::new(5, 13).unwrap();
        assert_eq!(td.periods(), (5, 13));
        assert_eq!(td.warmup_period(), 13);
        assert_eq!(td.name(), "TDMovingAverage");
        assert!(!td.is_ready());
        assert_eq!(td.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut td = TdMovingAverage::new(2, 4).unwrap();
        let candles: Vec<Candle> = (0..8).map(|i| c(100.0 + f64::from(i))).collect();
        let out = td.batch(&candles);
        for v in out.iter().take(3) {
            assert!(v.is_none());
        }
        assert!(out[3].is_some());
    }

    #[test]
    fn fast_leads_slow_in_uptrend() {
        let mut td = TdMovingAverage::new(3, 7).unwrap();
        let candles: Vec<Candle> = (0..40).map(|i| c(100.0 + f64::from(i))).collect();
        let out = td.batch(&candles).into_iter().flatten().last().unwrap();
        assert!(out.st1 > out.st2, "fast MA should lead in an uptrend");
    }

    #[test]
    fn fast_below_slow_in_downtrend() {
        let mut td = TdMovingAverage::new(3, 7).unwrap();
        let candles: Vec<Candle> = (0..40).map(|i| c(200.0 - f64::from(i))).collect();
        let out = td.batch(&candles).into_iter().flatten().last().unwrap();
        assert!(out.st1 < out.st2, "fast MA should trail in a downtrend");
    }

    #[test]
    fn flat_series_equal_lines() {
        let mut td = TdMovingAverage::new(2, 4).unwrap();
        let out = td
            .batch(&[c(50.0); 10])
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(out.st1, 50.0, epsilon = 1e-9);
        assert_relative_eq!(out.st2, 50.0, epsilon = 1e-9);
    }

    #[test]
    fn reset_clears_state() {
        let mut td = TdMovingAverage::new(2, 4).unwrap();
        td.batch(&(0..10).map(|i| c(100.0 + f64::from(i))).collect::<Vec<_>>());
        assert!(td.is_ready());
        td.reset();
        assert!(!td.is_ready());
        assert_eq!(td.value(), None);
        assert_eq!(td.update(c(100.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| c(100.0 + (f64::from(i) * 0.25).sin() * 9.0))
            .collect();
        let batch = TdMovingAverage::new(5, 13).unwrap().batch(&candles);
        let mut b = TdMovingAverage::new(5, 13).unwrap();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
