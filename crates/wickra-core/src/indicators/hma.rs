//! Hull Moving Average (HMA).

use crate::error::{Error, Result};
use crate::indicators::wma::Wma;
use crate::traits::Indicator;

/// Hull Moving Average: `WMA(2 * WMA(n/2) - WMA(n), sqrt(n))`.
///
/// Designed by Alan Hull as a lag-free moving average that is also responsive.
/// The square root of the period is rounded to the nearest integer (minimum 1).
#[derive(Debug, Clone)]
pub struct Hma {
    period: usize,
    half_wma: Wma,
    full_wma: Wma,
    smooth_wma: Wma,
}

impl Hma {
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        let half = (period / 2).max(1);
        let smooth = (period as f64).sqrt().round() as usize;
        let smooth = smooth.max(1);
        Ok(Self {
            period,
            half_wma: Wma::new(half)?,
            full_wma: Wma::new(period)?,
            smooth_wma: Wma::new(smooth)?,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for Hma {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        let h = self.half_wma.update(input)?;
        let f = self.full_wma.update(input)?;
        let diff = 2.0 * h - f;
        self.smooth_wma.update(diff)
    }

    fn reset(&mut self) {
        self.half_wma.reset();
        self.full_wma.reset();
        self.smooth_wma.reset();
    }

    fn warmup_period(&self) -> usize {
        let sm = (self.period as f64).sqrt().round() as usize;
        self.period + sm.max(1) - 1
    }

    fn is_ready(&self) -> bool {
        self.smooth_wma.is_ready()
    }

    fn name(&self) -> &'static str {
        "HMA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn constant_series_yields_constant_hma() {
        let mut hma = Hma::new(9).unwrap();
        let out = hma.batch(&[10.0_f64; 80]);
        let last = out.iter().rev().flatten().next().unwrap();
        assert_relative_eq!(*last, 10.0, epsilon = 1e-9);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=100).map(|i| f64::from(i) * 0.7).collect();
        let mut a = Hma::new(9).unwrap();
        let mut b = Hma::new(9).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut hma = Hma::new(9).unwrap();
        hma.batch(&(1..=80).map(f64::from).collect::<Vec<_>>());
        assert!(hma.is_ready());
        hma.reset();
        assert!(!hma.is_ready());
    }

    #[test]
    fn rejects_zero_period() {
        assert!(Hma::new(0).is_err());
    }
}
