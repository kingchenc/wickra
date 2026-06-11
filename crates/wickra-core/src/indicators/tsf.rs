//! Time Series Forecast (TSF).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Time Series Forecast (`TSF`): the rolling least-squares line projected one bar
/// past the window.
///
/// Over the last `period` inputs, indexed `x = 0, 1, …, period − 1`, it fits
/// `y = a + b·x` by ordinary least squares and reports the line's value at
/// `x = period` (one step beyond the most recent point):
///
/// ```text
/// b (slope)     = (n·Σxy − Σx·Σy) / (n·Σxx − (Σx)²)
/// a (intercept) = (Σy − b·Σx) / n
/// TSF           = a + b·period
/// ```
///
/// Where [`LinearRegression`](crate::LinearRegression) evaluates the fit at the
/// current bar (`a + b·(period − 1)`), `TSF` advances it one further bar, giving a
/// trend-following one-step-ahead forecast. Each update is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Tsf};
///
/// let mut indicator = Tsf::new(14).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Tsf {
    period: usize,
    window: VecDeque<f64>,
    sum_x: f64,
    denom: f64,
    sum_y: f64,
    sum_xy: f64,
}

impl Tsf {
    /// Construct a new rolling time-series forecast over `period` inputs.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `period < 2` — a regression line is
    /// undefined for fewer than two points.
    pub fn new(period: usize) -> Result<Self> {
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "time series forecast needs period >= 2",
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
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for Tsf {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            let y0 = self.window.pop_front().expect("non-empty");
            self.sum_xy = self.sum_xy - self.sum_y + y0;
            self.sum_y -= y0;
        }
        let k = self.window.len() as f64;
        self.window.push_back(value);
        self.sum_y += value;
        self.sum_xy += k * value;

        if self.window.len() < self.period {
            return None;
        }
        let n = self.period as f64;
        let slope = (n * self.sum_xy - self.sum_x * self.sum_y) / self.denom;
        let intercept = (self.sum_y - slope * self.sum_x) / n;
        Some(intercept + slope * n)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum_y = 0.0;
        self.sum_xy = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "TSF"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_short_period() {
        assert!(matches!(Tsf::new(1), Err(Error::InvalidPeriod { .. })));
    }

    #[test]
    fn accessors_report_config() {
        let tsf = Tsf::new(5).unwrap();
        assert_eq!(tsf.period(), 5);
        assert_eq!(tsf.name(), "TSF");
        assert_eq!(tsf.warmup_period(), 5);
        assert!(!tsf.is_ready());
    }

    #[test]
    fn reference_value() {
        // period 3 over [1, 2, 9]: fit y = 0 + 4x, forecast at x = 3 is 12.
        let mut tsf = Tsf::new(3).unwrap();
        let out: Vec<Option<f64>> = tsf.batch(&[1.0, 2.0, 9.0]);
        assert!(out[0].is_none());
        assert!(out[1].is_none());
        assert_relative_eq!(out[2].unwrap(), 12.0, epsilon = 1e-9);
        assert!(tsf.is_ready());
    }

    #[test]
    fn forecasts_a_clean_line_one_step_ahead() {
        // Window [10, 12, 14]: y = 10 + 2x, forecast at x = 3 is 16.
        let mut tsf = Tsf::new(3).unwrap();
        let out: Vec<Option<f64>> = tsf.batch(&[1.0, 10.0, 12.0, 14.0]);
        assert_relative_eq!(out[3].unwrap(), 16.0, epsilon = 1e-9);
    }

    #[test]
    fn reset_clears_state() {
        let mut tsf = Tsf::new(3).unwrap();
        let _ = tsf.batch(&[1.0, 2.0, 9.0]);
        assert!(tsf.is_ready());
        tsf.reset();
        assert!(!tsf.is_ready());
        assert_eq!(tsf.update(1.0), None);
    }
}
