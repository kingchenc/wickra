//! MACD Histogram (standalone).

use crate::error::Result;
use crate::indicators::macd::MacdIndicator;
use crate::traits::Indicator;

/// MACD Histogram — the `macd − signal` bar of [`MacdIndicator`] as a
/// standalone scalar indicator.
///
/// ```text
/// macd      = EMA(fast) − EMA(slow)
/// signal    = EMA(macd, signal)
/// histogram = macd − signal
/// ```
///
/// The histogram is the most actively traded part of MACD: it crosses zero
/// exactly when the MACD line crosses its signal, and its slope measures
/// whether that momentum is accelerating or fading. This wrapper exposes just
/// that series for pipelines that want a plain `f64` stream rather than the
/// full [`MacdOutput`](crate::MacdOutput); for the line and signal alongside
/// it, use [`MacdIndicator`](crate::MacdIndicator) directly.
///
/// Standard parameters are `fast = 12`, `slow = 26`, `signal = 9`, so the
/// first value lands after `slow + signal − 1` inputs — exactly when
/// [`MacdIndicator`] emits its first full output.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, MacdHistogram};
///
/// let mut indicator = MacdHistogram::new(12, 26, 9).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct MacdHistogram {
    macd: MacdIndicator,
}

impl MacdHistogram {
    /// Construct a MACD histogram with the given periods.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if any period is zero, and
    /// [`Error::InvalidPeriod`] if `fast >= slow`.
    pub fn new(fast: usize, slow: usize, signal: usize) -> Result<Self> {
        Ok(Self {
            macd: MacdIndicator::new(fast, slow, signal)?,
        })
    }

    /// Default `(12, 26, 9)` configuration, matching every classical chart package.
    pub fn classic() -> Self {
        Self::new(12, 26, 9).expect("classic MACD periods are valid")
    }

    /// Configured periods as `(fast, slow, signal)`.
    pub const fn periods(&self) -> (usize, usize, usize) {
        self.macd.periods()
    }
}

impl Indicator for MacdHistogram {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        self.macd.update(input).map(|out| out.histogram)
    }

    fn reset(&mut self) {
        self.macd.reset();
    }

    fn warmup_period(&self) -> usize {
        self.macd.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.macd.is_ready()
    }

    fn name(&self) -> &'static str {
        "MacdHistogram"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_invalid_periods() {
        assert!(matches!(
            MacdHistogram::new(0, 26, 9),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            MacdHistogram::new(12, 26, 0),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            MacdHistogram::new(26, 12, 9),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let osc = MacdHistogram::classic();
        assert_eq!(osc.periods(), (12, 26, 9));
        assert_eq!(osc.name(), "MacdHistogram");
        assert_eq!(osc.warmup_period(), 26 + 9 - 1);
        assert!(!osc.is_ready());
    }

    #[test]
    fn equals_macd_histogram_field() {
        // The standalone series must be exactly MacdIndicator's histogram bar.
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 8.0)
            .collect();
        let hist = MacdHistogram::classic().batch(&prices);
        let full = MacdIndicator::classic().batch(&prices);
        assert_eq!(hist.len(), full.len());
        for (h, m) in hist.iter().zip(full.iter()) {
            assert_eq!(h.is_some(), m.is_some());
            if let (Some(h), Some(m)) = (h, m) {
                assert_relative_eq!(*h, m.histogram, epsilon = 1e-12);
            }
        }
    }

    #[test]
    fn warmup_emits_first_value_at_warmup_period() {
        let mut osc = MacdHistogram::new(3, 6, 3).unwrap();
        let warmup = osc.warmup_period();
        assert_eq!(warmup, 6 + 3 - 1);
        for i in 1..warmup {
            assert!(osc.update(100.0 + i as f64).is_none());
        }
        assert!(osc.update(100.0 + warmup as f64).is_some());
        assert!(osc.is_ready());
    }

    #[test]
    fn constant_series_converges_to_zero() {
        let mut osc = MacdHistogram::classic();
        let out = osc.batch(&[100.0_f64; 200]);
        let last = out.iter().rev().flatten().next().expect("emits a value");
        assert_relative_eq!(*last, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=100)
            .map(|i| (f64::from(i) * 0.4).cos() * 10.0)
            .collect();
        let mut a = MacdHistogram::classic();
        let mut b = MacdHistogram::classic();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut osc = MacdHistogram::classic();
        osc.batch(&(1..=80).map(f64::from).collect::<Vec<_>>());
        assert!(osc.is_ready());
        osc.reset();
        assert!(!osc.is_ready());
        assert_eq!(osc.update(1.0), None);
    }
}
