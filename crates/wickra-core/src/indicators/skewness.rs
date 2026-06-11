//! Rolling Pearson skewness (third standardised central moment).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Rolling Pearson skewness of the last `period` values.
///
/// ```text
/// mean = (1/n) · Σ x
/// m2   = (1/n) · Σ (x − mean)²        // population variance
/// m3   = (1/n) · Σ (x − mean)³        // third central moment
/// Skew = m3 / m2^(3/2)
/// ```
///
/// Positive skewness means the right tail (large positive deviations from
/// the mean) is heavier than the left; negative skewness flags the
/// opposite. A symmetric distribution has skewness `0`. This is the
/// population (Pearson) definition with divisor `n`; many statistics
/// packages report the bias-corrected sample skewness instead. The window
/// is required to have at least three points so the moments are
/// well-defined. A window with zero dispersion yields `0`.
///
/// Each `update` is O(1): three running sums (`Σ x`, `Σ x²`, `Σ x³`) are
/// maintained as the window slides; the central moments are then derived
/// from them via the binomial-expansion identities, so no inner loop runs
/// per bar.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Skewness};
///
/// let mut indicator = Skewness::new(20).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = indicator.update(f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Skewness {
    period: usize,
    window: VecDeque<f64>,
    sum: f64,
    sum_sq: f64,
    sum_cu: f64,
}

impl Skewness {
    /// Construct a new rolling skewness with the given period.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `period < 3`.
    pub fn new(period: usize) -> Result<Self> {
        if period < 3 {
            return Err(Error::InvalidPeriod {
                message: "skewness needs period >= 3",
            });
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
            sum: 0.0,
            sum_sq: 0.0,
            sum_cu: 0.0,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for Skewness {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            let old = self.window.pop_front().expect("non-empty");
            self.sum -= old;
            self.sum_sq -= old * old;
            self.sum_cu -= old * old * old;
        }
        self.window.push_back(value);
        self.sum += value;
        self.sum_sq += value * value;
        self.sum_cu += value * value * value;
        if self.window.len() < self.period {
            return None;
        }
        let n = self.period as f64;
        let mean = self.sum / n;
        // m2 = E[x²] − E[x]²
        let m2 = (self.sum_sq / n - mean * mean).max(0.0);
        // m3 = E[x³] − 3·mean·E[x²] + 2·mean³ (binomial expansion).
        let m3 = self.sum_cu / n - 3.0 * mean * (self.sum_sq / n) + 2.0 * mean * mean * mean;
        if m2 == 0.0 {
            // A window with no dispersion has no defined shape; return 0.
            return Some(0.0);
        }
        Some(m3 / m2.powf(1.5))
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum = 0.0;
        self.sum_sq = 0.0;
        self.sum_cu = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "Skewness"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_period_below_three() {
        assert!(Skewness::new(0).is_err());
        assert!(Skewness::new(1).is_err());
        assert!(Skewness::new(2).is_err());
        assert!(Skewness::new(3).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let s = Skewness::new(14).unwrap();
        assert_eq!(s.period(), 14);
        assert_eq!(s.warmup_period(), 14);
        assert_eq!(s.name(), "Skewness");
    }

    #[test]
    fn symmetric_window_is_zero() {
        // Symmetric around its mean — skewness must be (numerically) zero.
        let mut s = Skewness::new(5).unwrap();
        let out = s.batch(&[-2.0, -1.0, 0.0, 1.0, 2.0]);
        assert_relative_eq!(out[4].unwrap(), 0.0, epsilon = 1e-9);
    }

    #[test]
    fn constant_series_yields_zero() {
        let mut s = Skewness::new(5).unwrap();
        for v in s.batch(&[42.0; 20]).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn right_tail_is_positive() {
        // One large positive outlier creates a right-skewed window.
        let mut s = Skewness::new(5).unwrap();
        let out = s.batch(&[0.0, 0.0, 0.0, 0.0, 10.0]);
        assert!(out[4].unwrap() > 0.0);
    }

    #[test]
    fn left_tail_is_negative() {
        // Mirror image — one large negative outlier gives left skew.
        let mut s = Skewness::new(5).unwrap();
        let out = s.batch(&[10.0, 10.0, 10.0, 10.0, 0.0]);
        assert!(out[4].unwrap() < 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut s = Skewness::new(5).unwrap();
        s.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(s.is_ready());
        s.reset();
        assert!(!s.is_ready());
        assert_eq!(s.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..60)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 7.0)
            .collect();
        let batch = Skewness::new(14).unwrap().batch(&prices);
        let mut b = Skewness::new(14).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
