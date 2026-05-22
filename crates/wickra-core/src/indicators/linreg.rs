//! Linear Regression (rolling least-squares endpoint).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Linear Regression — the endpoint of a rolling least-squares fit.
///
/// Over the last `period` inputs, indexed `x = 0, 1, …, period − 1`, it fits
/// the line `y = a + b·x` by ordinary least squares and reports the line's
/// value at the most recent point:
///
/// ```text
/// b (slope)     = (n·Σxy − Σx·Σy) / (n·Σxx − (Σx)²)
/// a (intercept) = (Σy − b·Σx) / n
/// LinearReg     = a + b·(period − 1)
/// ```
///
/// This is TA-Lib's `LINEARREG`: a smoothed price that lags less than an SMA
/// because it extrapolates the *local trend* forward to the current bar
/// instead of averaging it away. The `Σx` terms depend only on `period`, so
/// they are computed once; each `update` is O(period).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, LinearRegression};
///
/// let mut indicator = LinearRegression::new(14).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct LinearRegression {
    period: usize,
    window: VecDeque<f64>,
    sum_x: f64,
    denom: f64,
}

impl LinearRegression {
    /// Construct a new rolling linear regression over `period` inputs.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `period < 2` — a regression line is
    /// undefined for fewer than two points.
    pub fn new(period: usize) -> Result<Self> {
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "linear regression needs period >= 2",
            });
        }
        let n = period as f64;
        // Closed forms for x = 0, 1, …, period − 1.
        let sum_x = n * (n - 1.0) / 2.0;
        let sum_xx = (n - 1.0) * n * (2.0 * n - 1.0) / 6.0;
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
            sum_x,
            denom: n * sum_xx - sum_x * sum_x,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Ordinary-least-squares `(slope, endpoint)` over the current full window.
    fn fit(&self) -> (f64, f64) {
        let n = self.period as f64;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        for (x, &y) in self.window.iter().enumerate() {
            sum_y += y;
            sum_xy += x as f64 * y;
        }
        let slope = (n * sum_xy - self.sum_x * sum_y) / self.denom;
        let intercept = (sum_y - slope * self.sum_x) / n;
        (slope, intercept + slope * (n - 1.0))
    }
}

impl Indicator for LinearRegression {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, value: f64) -> Option<f64> {
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(value);
        if self.window.len() < self.period {
            return None;
        }
        Some(self.fit().1)
    }

    fn reset(&mut self) {
        self.window.clear();
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "LinearRegression"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn reference_values() {
        // period 3 over [1, 2, 9]: fit y = 0 + 4x, endpoint = 0 + 4·2 = 8.
        let mut lr = LinearRegression::new(3).unwrap();
        let out = lr.batch(&[1.0, 2.0, 9.0]);
        assert!(out[0].is_none());
        assert!(out[1].is_none());
        assert_relative_eq!(out[2].unwrap(), 8.0, epsilon = 1e-9);
    }

    #[test]
    fn perfect_line_returns_current_value() {
        // The regression of a perfectly linear series is that line itself, so
        // its endpoint equals the current value.
        let prices: Vec<f64> = (0..40).map(|i| 2.0 * f64::from(i) + 5.0).collect();
        let mut lr = LinearRegression::new(10).unwrap();
        for (i, v) in lr.batch(&prices).into_iter().enumerate() {
            if let Some(v) = v {
                assert_relative_eq!(v, 2.0 * i as f64 + 5.0, epsilon = 1e-6);
            }
        }
    }

    #[test]
    fn constant_series_returns_the_constant() {
        let mut lr = LinearRegression::new(8).unwrap();
        for v in lr.batch(&[42.0; 20]).into_iter().flatten() {
            assert_relative_eq!(v, 42.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn first_value_on_period_th_input() {
        let mut lr = LinearRegression::new(5).unwrap();
        let out = lr.batch(&[1.0, 3.0, 2.0, 5.0, 4.0, 6.0]);
        for (i, v) in out.iter().enumerate().take(4) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(out[4].is_some(), "first value lands at index period - 1");
        assert_eq!(lr.warmup_period(), 5);
    }

    #[test]
    fn rejects_period_below_two() {
        assert!(LinearRegression::new(0).is_err());
        assert!(LinearRegression::new(1).is_err());
        assert!(LinearRegression::new(2).is_ok());
    }

    #[test]
    fn reset_clears_state() {
        let mut lr = LinearRegression::new(5).unwrap();
        lr.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(lr.is_ready());
        lr.reset();
        assert!(!lr.is_ready());
        assert_eq!(lr.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..60)
            .map(|i| 50.0 + (f64::from(i) * 0.3).sin() * 10.0)
            .collect();
        let mut a = LinearRegression::new(14).unwrap();
        let mut b = LinearRegression::new(14).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
