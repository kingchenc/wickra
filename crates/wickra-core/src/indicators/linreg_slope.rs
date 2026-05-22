//! Linear Regression Slope.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Linear Regression Slope — the slope of a rolling least-squares fit.
///
/// Over the last `period` inputs, indexed `x = 0, 1, …, period − 1`, it fits
/// the line `y = a + b·x` by ordinary least squares and reports the slope:
///
/// ```text
/// b = (n·Σxy − Σx·Σy) / (n·Σxx − (Σx)²)
/// ```
///
/// This is TA-Lib's `LINEARREG_SLOPE`: a momentum-like reading of how steeply
/// price is trending over the window — positive while it rises, negative
/// while it falls, near zero when it is flat — without the band-pass quirks
/// of a difference-based oscillator. The `Σx` terms depend only on `period`,
/// so they are computed once; each `update` is O(period).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, LinRegSlope};
///
/// let mut indicator = LinRegSlope::new(14).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct LinRegSlope {
    period: usize,
    window: VecDeque<f64>,
    sum_x: f64,
    denom: f64,
}

impl LinRegSlope {
    /// Construct a new rolling linear-regression slope over `period` inputs.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `period < 2` — a regression line is
    /// undefined for fewer than two points.
    pub fn new(period: usize) -> Result<Self> {
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "linear regression slope needs period >= 2",
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
}

impl Indicator for LinRegSlope {
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
        let n = self.period as f64;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        for (x, &y) in self.window.iter().enumerate() {
            sum_y += y;
            sum_xy += x as f64 * y;
        }
        Some((n * sum_xy - self.sum_x * sum_y) / self.denom)
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
        "LinRegSlope"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn reference_values() {
        // period 3 over [1, 2, 9]: fit y = 0 + 4x, so the slope is 4.
        let mut ls = LinRegSlope::new(3).unwrap();
        let out = ls.batch(&[1.0, 2.0, 9.0]);
        assert!(out[0].is_none());
        assert!(out[1].is_none());
        assert_relative_eq!(out[2].unwrap(), 4.0, epsilon = 1e-9);
    }

    #[test]
    fn perfect_line_returns_its_step() {
        // A series rising by a fixed step has exactly that slope.
        let prices: Vec<f64> = (0..40).map(|i| 2.5 * f64::from(i) + 7.0).collect();
        let mut ls = LinRegSlope::new(10).unwrap();
        for v in ls.batch(&prices).into_iter().flatten() {
            assert_relative_eq!(v, 2.5, epsilon = 1e-6);
        }
    }

    #[test]
    fn constant_series_has_zero_slope() {
        let mut ls = LinRegSlope::new(8).unwrap();
        for v in ls.batch(&[42.0; 20]).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn falling_series_has_negative_slope() {
        let prices: Vec<f64> = (0..30).map(|i| 100.0 - f64::from(i)).collect();
        let mut ls = LinRegSlope::new(10).unwrap();
        for v in ls.batch(&prices).into_iter().flatten() {
            assert!(v < 0.0, "a falling series must have a negative slope");
        }
    }

    #[test]
    fn first_value_on_period_th_input() {
        let mut ls = LinRegSlope::new(5).unwrap();
        let out = ls.batch(&[1.0, 3.0, 2.0, 5.0, 4.0, 6.0]);
        for (i, v) in out.iter().enumerate().take(4) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(out[4].is_some(), "first value lands at index period - 1");
        assert_eq!(ls.warmup_period(), 5);
    }

    #[test]
    fn rejects_period_below_two() {
        assert!(LinRegSlope::new(0).is_err());
        assert!(LinRegSlope::new(1).is_err());
        assert!(LinRegSlope::new(2).is_ok());
    }

    #[test]
    fn reset_clears_state() {
        let mut ls = LinRegSlope::new(5).unwrap();
        ls.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(ls.is_ready());
        ls.reset();
        assert!(!ls.is_ready());
        assert_eq!(ls.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..60)
            .map(|i| 50.0 + (f64::from(i) * 0.3).sin() * 10.0)
            .collect();
        let mut a = LinRegSlope::new(14).unwrap();
        let mut b = LinRegSlope::new(14).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
