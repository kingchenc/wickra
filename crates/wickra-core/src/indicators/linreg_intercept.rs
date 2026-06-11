//! Linear Regression Intercept (`LINEARREG_INTERCEPT`).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Linear Regression Intercept (`LINEARREG_INTERCEPT`): the intercept `a` of the
/// rolling least-squares fit `y = a + b·x` over the last `period` inputs, indexed
/// `x = 0, 1, …, period − 1`.
///
/// ```text
/// b (slope)     = (n·Σxy − Σx·Σy) / (n·Σxx − (Σx)²)
/// a (intercept) = (Σy − b·Σx) / n
/// ```
///
/// Where [`LinearRegression`](crate::LinearRegression) reports the fitted line at
/// the most recent bar (`a + b·(period − 1)`), this reports its value at the
/// *start* of the window (`x = 0`). Each update is O(1), maintaining the same
/// closed-form sliding-window sums as `LinearRegression`.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, LinRegIntercept};
///
/// let mut indicator = LinRegIntercept::new(14).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct LinRegIntercept {
    period: usize,
    window: VecDeque<f64>,
    sum_x: f64,
    denom: f64,
    sum_y: f64,
    sum_xy: f64,
}

impl LinRegIntercept {
    /// Construct a new rolling linear-regression intercept over `period` inputs.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `period < 2` — a regression line is
    /// undefined for fewer than two points.
    pub fn new(period: usize) -> Result<Self> {
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "linear regression intercept needs period >= 2",
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

impl Indicator for LinRegIntercept {
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
        Some(intercept)
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
        "LINEARREG_INTERCEPT"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_short_period() {
        assert!(matches!(
            LinRegIntercept::new(1),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_report_config() {
        let lr = LinRegIntercept::new(5).unwrap();
        assert_eq!(lr.period(), 5);
        assert_eq!(lr.name(), "LINEARREG_INTERCEPT");
        assert_eq!(lr.warmup_period(), 5);
        assert!(!lr.is_ready());
    }

    #[test]
    fn reference_value() {
        // period 3 over [1, 2, 9]: fit y = 0 + 4x, intercept = 0.
        let mut lr = LinRegIntercept::new(3).unwrap();
        let out: Vec<Option<f64>> = lr.batch(&[1.0, 2.0, 9.0]);
        assert!(out[0].is_none());
        assert!(out[1].is_none());
        assert_relative_eq!(out[2].unwrap(), 0.0, epsilon = 1e-9);
        assert!(lr.is_ready());
    }

    #[test]
    fn slides_and_tracks_a_shifted_line() {
        // After sliding to window [2, 9, 4]... intercept stays finite and the
        // fit is exact for a clean line [10, 12, 14]: y = 10 + 2x, intercept 10.
        let mut lr = LinRegIntercept::new(3).unwrap();
        let out: Vec<Option<f64>> = lr.batch(&[1.0, 10.0, 12.0, 14.0]);
        assert_relative_eq!(out[3].unwrap(), 10.0, epsilon = 1e-9);
    }

    #[test]
    fn reset_clears_state() {
        let mut lr = LinRegIntercept::new(3).unwrap();
        let _ = lr.batch(&[1.0, 2.0, 9.0]);
        assert!(lr.is_ready());
        lr.reset();
        assert!(!lr.is_ready());
        assert_eq!(lr.update(1.0), None);
    }
}
