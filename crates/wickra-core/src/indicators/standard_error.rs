//! Standard Error of the rolling least-squares regression.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Standard Error of the regression line fit over the last `period` inputs.
///
/// Over the trailing window indexed `x = 0, 1, …, period − 1` the OLS line
/// `y = a + b·x` is fitted, then:
///
/// ```text
/// slope     = (n·Σxy − Σx·Σy) / (n·Σxx − (Σx)²)
/// SS_total  = Σy² − n·ȳ²                            // total sum of squares
/// RSS       = SS_total − slope² · S_xx              // residual sum of squares
/// StdErr    = √( RSS / (n − 2) )                    // n − 2 residual d.o.f.
/// ```
///
/// where `S_xx = (n·Σxx − (Σx)²) / n` is the centred sum of squares of the
/// design.
///
/// This is the textbook **standard error of estimate** of OLS: it measures
/// the typical distance between the observed prices and the fitted line,
/// using the residual degrees of freedom `n − 2`. It is the spread that
/// drives [`crate::BollingerBands`]-style bands around a regression instead of
/// around an SMA — when the price hugs its trend, `StdErr` is small.
///
/// Each `update` is O(1): the `Σx` and `Σxx` terms depend only on `period`
/// and are precomputed once, while `Σy`, `Σxy`, and `Σy²` are maintained
/// incrementally as the window slides. Tiny floating-point cancellation
/// noise that could drive the residual sum of squares slightly negative is
/// clamped to zero before the square root.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, StandardError};
///
/// let mut indicator = StandardError::new(14).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = indicator.update(100.0 + f64::from(i) + (f64::from(i) * 0.5).sin());
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct StandardError {
    period: usize,
    window: VecDeque<f64>,
    sum_x: f64,
    /// `n·Σxx − (Σx)²` — OLS denominator, constant in `period`.
    denom: f64,
    sum_y: f64,
    sum_xy: f64,
    sum_y_sq: f64,
}

impl StandardError {
    /// Construct a new rolling standard error of regression.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `period < 3` — the residual
    /// degrees of freedom `n − 2` would be non-positive.
    pub fn new(period: usize) -> Result<Self> {
        if period < 3 {
            return Err(Error::InvalidPeriod {
                message: "standard error needs period >= 3",
            });
        }
        let n = period as f64;
        let sum_x = n * (n - 1.0) / 2.0;
        let sum_xx = (n - 1.0) * n * (2.0 * n - 1.0) / 6.0;
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
            sum_x,
            denom: n * sum_xx - sum_x * sum_x,
            sum_y: 0.0,
            sum_xy: 0.0,
            sum_y_sq: 0.0,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for StandardError {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            // Slide: pop oldest, shift indices, then push the new value at index n − 1.
            let y0 = self.window.pop_front().expect("non-empty");
            self.sum_xy = self.sum_xy - self.sum_y + y0;
            self.sum_y -= y0;
            self.sum_y_sq -= y0 * y0;
        }
        let k = self.window.len() as f64;
        self.window.push_back(value);
        self.sum_y += value;
        self.sum_xy += k * value;
        self.sum_y_sq += value * value;

        if self.window.len() < self.period {
            return None;
        }
        let n = self.period as f64;
        let slope = (n * self.sum_xy - self.sum_x * self.sum_y) / self.denom;
        let mean_y = self.sum_y / n;
        let ss_total = self.sum_y_sq - n * mean_y * mean_y;
        // S_xx = denom / n
        let s_xx = self.denom / n;
        let rss = (ss_total - slope * slope * s_xx).max(0.0);
        Some((rss / (n - 2.0)).sqrt())
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum_y = 0.0;
        self.sum_xy = 0.0;
        self.sum_y_sq = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "StandardError"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_period_below_three() {
        assert!(StandardError::new(0).is_err());
        assert!(StandardError::new(2).is_err());
        assert!(StandardError::new(3).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let se = StandardError::new(14).unwrap();
        assert_eq!(se.period(), 14);
        assert_eq!(se.warmup_period(), 14);
        assert_eq!(se.name(), "StandardError");
    }

    #[test]
    fn perfect_line_has_zero_error() {
        // Residuals from a perfectly linear fit are zero, so SE = 0.
        let prices: Vec<f64> = (0..30).map(|i| 2.0 * f64::from(i) + 5.0).collect();
        let mut se = StandardError::new(10).unwrap();
        for v in se.batch(&prices).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn constant_series_yields_zero() {
        let mut se = StandardError::new(5).unwrap();
        for v in se.batch(&[42.0; 20]).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn matches_naive_definition() {
        // Compare the O(1) update against a fresh-from-scratch OLS refit each bar.
        fn naive(window: &[f64]) -> f64 {
            let n = window.len() as f64;
            let mean_y = window.iter().sum::<f64>() / n;
            let mut sum_xy = 0.0;
            let mut sum_x = 0.0;
            let mut sum_xx = 0.0;
            for (i, &y) in window.iter().enumerate() {
                let x = i as f64;
                sum_xy += x * y;
                sum_x += x;
                sum_xx += x * x;
            }
            let mean_x = sum_x / n;
            let s_xx = sum_xx - n * mean_x * mean_x;
            let slope = (sum_xy - n * mean_x * mean_y) / s_xx;
            let intercept = mean_y - slope * mean_x;
            let rss: f64 = window
                .iter()
                .enumerate()
                .map(|(i, &y)| {
                    let r = y - (intercept + slope * i as f64);
                    r * r
                })
                .sum();
            (rss / (n - 2.0)).sqrt()
        }

        let prices: Vec<f64> = (0..60)
            .map(|i| 100.0 + f64::from(i) * 0.5 + (f64::from(i) * 0.7).sin() * 3.0)
            .collect();
        let period = 14;
        let got = StandardError::new(period).unwrap().batch(&prices);
        for (i, g) in got.iter().enumerate() {
            if let Some(v) = g {
                let expected = naive(&prices[i + 1 - period..=i]);
                assert_relative_eq!(*v, expected, epsilon = 1e-9);
            }
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut se = StandardError::new(5).unwrap();
        se.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(se.is_ready());
        se.reset();
        assert!(!se.is_ready());
        assert_eq!(se.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..60)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 10.0)
            .collect();
        let batch = StandardError::new(14).unwrap().batch(&prices);
        let mut b = StandardError::new(14).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
