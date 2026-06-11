//! Rolling excess kurtosis (Pearson's fourth standardised central moment − 3).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Rolling **excess** kurtosis of the last `period` values.
///
/// ```text
/// mean = (1/n) · Σ x
/// m2   = (1/n) · Σ (x − mean)²
/// m4   = (1/n) · Σ (x − mean)⁴
/// Kurtosis = m4 / m2² − 3
/// ```
///
/// The unshifted kurtosis `m4 / m2²` equals `3` for the normal distribution;
/// subtracting `3` gives **excess** kurtosis so that `0` is the Gaussian
/// baseline. Positive readings flag fat tails (heavy outliers compared to
/// normal); negative readings flag light tails (more concentrated than
/// normal). This is the population definition with divisor `n`. A window
/// with zero dispersion yields `0`.
///
/// Each `update` is O(1): four running sums (`Σ x`, `Σ x²`, `Σ x³`, `Σ x⁴`)
/// are maintained as the window slides; the central moments are derived
/// from them via the binomial-expansion identities, so no inner loop runs
/// per bar.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Kurtosis};
///
/// let mut indicator = Kurtosis::new(20).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = indicator.update(f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Kurtosis {
    period: usize,
    window: VecDeque<f64>,
    sum: f64,
    sum_sq: f64,
    sum_cu: f64,
    sum_qu: f64,
}

impl Kurtosis {
    /// Construct a new rolling excess kurtosis with the given period.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `period < 4`.
    pub fn new(period: usize) -> Result<Self> {
        if period < 4 {
            return Err(Error::InvalidPeriod {
                message: "kurtosis needs period >= 4",
            });
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
            sum: 0.0,
            sum_sq: 0.0,
            sum_cu: 0.0,
            sum_qu: 0.0,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for Kurtosis {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            let old = self.window.pop_front().expect("non-empty");
            let sq = old * old;
            self.sum -= old;
            self.sum_sq -= sq;
            self.sum_cu -= old * sq;
            self.sum_qu -= sq * sq;
        }
        self.window.push_back(value);
        let sq = value * value;
        self.sum += value;
        self.sum_sq += sq;
        self.sum_cu += value * sq;
        self.sum_qu += sq * sq;
        if self.window.len() < self.period {
            return None;
        }
        let n = self.period as f64;
        let mean = self.sum / n;
        let m2 = (self.sum_sq / n - mean * mean).max(0.0);
        if m2 == 0.0 {
            // Flat window: kurtosis is undefined, return 0 (Gaussian baseline).
            return Some(0.0);
        }
        // m4 = E[x⁴] − 4·mean·E[x³] + 6·mean²·E[x²] − 3·mean⁴.
        let mean_sq = mean * mean;
        let m4 = self.sum_qu / n - 4.0 * mean * (self.sum_cu / n)
            + 6.0 * mean_sq * (self.sum_sq / n)
            - 3.0 * mean_sq * mean_sq;
        Some(m4 / (m2 * m2) - 3.0)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum = 0.0;
        self.sum_sq = 0.0;
        self.sum_cu = 0.0;
        self.sum_qu = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "Kurtosis"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_period_below_four() {
        assert!(Kurtosis::new(0).is_err());
        assert!(Kurtosis::new(3).is_err());
        assert!(Kurtosis::new(4).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let k = Kurtosis::new(14).unwrap();
        assert_eq!(k.period(), 14);
        assert_eq!(k.warmup_period(), 14);
        assert_eq!(k.name(), "Kurtosis");
    }

    #[test]
    fn two_point_distribution_is_negative_two() {
        // A {a, b, a, b} window has m4/m2² = 1, so excess kurtosis = −2.
        // This is the theoretical minimum for any real distribution.
        let mut k = Kurtosis::new(4).unwrap();
        let out = k.batch(&[-1.0, 1.0, -1.0, 1.0]);
        assert_relative_eq!(out[3].unwrap(), -2.0, epsilon = 1e-9);
    }

    #[test]
    fn constant_series_yields_zero() {
        let mut k = Kurtosis::new(5).unwrap();
        for v in k.batch(&[42.0; 20]).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn outlier_window_is_leptokurtic() {
        // A single large outlier amid otherwise-flat samples has positive
        // excess kurtosis (a heavy tail).
        let mut k = Kurtosis::new(5).unwrap();
        let out = k.batch(&[0.0, 0.0, 0.0, 0.0, 100.0]);
        assert!(out[4].unwrap() > 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut k = Kurtosis::new(5).unwrap();
        k.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(k.is_ready());
        k.reset();
        assert!(!k.is_ready());
        assert_eq!(k.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..60)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        let batch = Kurtosis::new(14).unwrap().batch(&prices);
        let mut b = Kurtosis::new(14).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
