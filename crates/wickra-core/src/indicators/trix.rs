//! TRIX: triple-smoothed EMA percent rate of change.

use crate::error::Result;
use crate::indicators::ema::Ema;
use crate::traits::Indicator;

/// TRIX: the 1-period percent rate of change of a triple-smoothed EMA.
///
/// `TRIX = 100 * (TR_t - TR_{t-1}) / TR_{t-1}` where
/// `TR_t = EMA(EMA(EMA(price)))`.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Trix};
///
/// let mut indicator = Trix::new(3).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Trix {
    ema1: Ema,
    ema2: Ema,
    ema3: Ema,
    prev_tr: Option<f64>,
    period: usize,
}

impl Trix {
    /// # Errors
    /// Returns [`crate::Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            ema1: Ema::new(period)?,
            ema2: Ema::new(period)?,
            ema3: Ema::new(period)?,
            prev_tr: None,
            period,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for Trix {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        let e1 = self.ema1.update(input)?;
        let e2 = self.ema2.update(e1)?;
        let e3 = self.ema3.update(e2)?;
        match self.prev_tr {
            Some(prev) if prev != 0.0 => {
                let trix = 100.0 * (e3 - prev) / prev;
                self.prev_tr = Some(e3);
                Some(trix)
            }
            Some(_) => {
                self.prev_tr = Some(e3);
                Some(0.0)
            }
            None => {
                self.prev_tr = Some(e3);
                None
            }
        }
    }

    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
        self.ema3.reset();
        self.prev_tr = None;
    }

    fn warmup_period(&self) -> usize {
        // Triple EMA seeds at 3*period-2; plus one extra for the rate of change.
        3 * self.period - 1
    }

    fn is_ready(&self) -> bool {
        self.prev_tr.is_some() && self.ema3.is_ready()
    }

    fn name(&self) -> &'static str {
        "TRIX"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn constant_series_yields_zero_trix() {
        let mut trix = Trix::new(5).unwrap();
        let out = trix.batch(&[100.0_f64; 80]);
        let last = out.iter().rev().flatten().next().unwrap();
        assert_relative_eq!(*last, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn rising_series_eventually_positive_trix() {
        let prices: Vec<f64> = (1..=200).map(f64::from).collect();
        let mut trix = Trix::new(5).unwrap();
        let last = trix.batch(&prices).into_iter().flatten().last().unwrap();
        assert!(last > 0.0);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=80).map(|i| f64::from(i) * 1.3).collect();
        let mut a = Trix::new(7).unwrap();
        let mut b = Trix::new(7).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut trix = Trix::new(5).unwrap();
        trix.batch(&(1..=80).map(f64::from).collect::<Vec<_>>());
        assert!(trix.is_ready());
        trix.reset();
        assert!(!trix.is_ready());
    }

    #[test]
    fn rejects_zero_period() {
        assert!(Trix::new(0).is_err());
    }

    /// Cover the const accessor `period` (47-49) and the Indicator-impl
    /// `warmup_period` (84-87) + `name` (93-95). Existing tests never
    /// inspect these metadata methods.
    #[test]
    fn accessors_and_metadata() {
        let trix = Trix::new(5).unwrap();
        assert_eq!(trix.period(), 5);
        // Triple EMA seeds at 3*5-2 = 13; +1 for the rate-of-change pair = 14.
        assert_eq!(trix.warmup_period(), 14);
        assert_eq!(trix.name(), "TRIX");
    }

    /// Cover the `Some(_)` match arm at lines 66-68 — the degenerate path
    /// where the previous triple-EMA value is exactly 0.0 (which would
    /// otherwise divide by zero on the percent-rate formula). A series of
    /// all-zero inputs collapses every EMA stage to 0.0, so once the
    /// indicator warms up `prev_tr` is `Some(0.0)` and every subsequent
    /// emission must take the fallback branch and return 0.0.
    #[test]
    fn zero_input_series_yields_zero_trix() {
        let mut trix = Trix::new(3).unwrap();
        let out = trix.batch(&[0.0_f64; 20]);
        let last = out.into_iter().flatten().last().expect("emits");
        assert_eq!(last, 0.0);
    }
}
