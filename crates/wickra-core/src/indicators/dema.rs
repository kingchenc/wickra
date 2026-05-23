//! Double Exponential Moving Average (DEMA).

use crate::error::Result;
use crate::indicators::ema::Ema;
use crate::traits::Indicator;

/// Double Exponential Moving Average: `2 * EMA - EMA(EMA)`.
///
/// Designed by Patrick Mulloy to reduce the lag of a single EMA while keeping
/// the smoothing benefit.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Dema};
///
/// let mut indicator = Dema::new(3).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Dema {
    ema1: Ema,
    ema2: Ema,
    period: usize,
}

impl Dema {
    /// # Errors
    /// Returns [`crate::Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            ema1: Ema::new(period)?,
            ema2: Ema::new(period)?,
            period,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for Dema {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        let e1 = self.ema1.update(input)?;
        let e2 = self.ema2.update(e1)?;
        Some(2.0 * e1 - e2)
    }

    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
    }

    fn warmup_period(&self) -> usize {
        // EMA1 seeds at period, then EMA2 needs another (period - 1) values to seed.
        2 * self.period - 1
    }

    fn is_ready(&self) -> bool {
        self.ema2.is_ready()
    }

    fn name(&self) -> &'static str {
        "DEMA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn constant_series_yields_constant_dema() {
        let mut dema = Dema::new(5).unwrap();
        let out = dema.batch(&[100.0_f64; 60]);
        let last = out.iter().rev().flatten().next().unwrap();
        assert_relative_eq!(*last, 100.0, epsilon = 1e-9);
    }

    #[test]
    fn linear_uptrend_dema_above_ema_eventually() {
        // On a linear uptrend DEMA should be ahead of (greater than) a plain EMA,
        // because the second-order correction removes lag.
        let prices: Vec<f64> = (1..=200).map(f64::from).collect();
        let mut dema = Dema::new(20).unwrap();
        let mut ema = Ema::new(20).unwrap();
        let dema_out = dema.batch(&prices);
        let ema_out = ema.batch(&prices);
        // Compare at the last index where both are ready.
        let d = dema_out.last().unwrap().unwrap();
        let e = ema_out.last().unwrap().unwrap();
        assert!(d > e, "DEMA={d} should exceed EMA={e} on uptrend");
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=80).map(|i| f64::from(i) * 0.5).collect();
        let mut a = Dema::new(7).unwrap();
        let mut b = Dema::new(7).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut dema = Dema::new(5).unwrap();
        dema.batch(&(1..=50).map(f64::from).collect::<Vec<_>>());
        assert!(dema.is_ready());
        dema.reset();
        assert!(!dema.is_ready());
    }

    #[test]
    fn rejects_zero_period() {
        assert!(Dema::new(0).is_err());
    }

    /// Cover the const accessor `period` (43-45) and the Indicator-impl
    /// `warmup_period` (63-66) + `name` (72-74). Existing tests never
    /// inspect these metadata methods.
    #[test]
    fn accessors_and_metadata() {
        let dema = Dema::new(5).unwrap();
        assert_eq!(dema.period(), 5);
        // EMA1 seeds at period (5), EMA2 needs another (period - 1) = 4 ->
        // total warmup = 2*period - 1 = 9.
        assert_eq!(dema.warmup_period(), 9);
        assert_eq!(dema.name(), "DEMA");
    }
}
