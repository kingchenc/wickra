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
/// of a difference-based oscillator.
///
/// Each `update` is O(1): the same incremental OLS state as
/// [`LinearRegression`](crate::LinearRegression) is maintained — `Σx` and
/// `Σxx` are precomputed once from `period`, while `Σy` and `Σxy` are slid
/// forward in closed form on every push.
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
    /// Closed form of `Σx` over `x = 0, 1, …, period − 1` — constant in `period`.
    sum_x: f64,
    /// Closed form of `n · Σxx − (Σx)²` — constant in `period`.
    denom: f64,
    /// Running sum of the values currently in the window.
    sum_y: f64,
    /// Running `Σ(x · y)` where `x` is the position within the trailing window.
    sum_xy: f64,
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
            sum_y: 0.0,
            sum_xy: 0.0,
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
        if !value.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            // Sliding-window identity: when the window slides one step forward
            // the indices `x` for every kept entry shift down by 1, so
            //   new_sum_xy = old_sum_xy − old_sum_y + y0
            // (`y0` is the popped front value).
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
        Some((n * self.sum_xy - self.sum_x * self.sum_y) / self.denom)
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

    /// Cover the const accessor `period` (80-82) and the Indicator-impl
    /// `name` body (125-127). `warmup_period` is exercised elsewhere.
    #[test]
    fn accessors_and_metadata() {
        let ls = LinRegSlope::new(14).unwrap();
        assert_eq!(ls.period(), 14);
        assert_eq!(ls.name(), "LinRegSlope");
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

    /// Incremental OLS equivalence for the slope: the O(1) implementation must
    /// agree bar-by-bar with a fresh-from-scratch O(n) refit, on a noisy ramp
    /// (sliding-phase dominated) and a step function (large pop/push deltas).
    #[test]
    fn incremental_matches_naive_slope_bar_by_bar() {
        fn naive_slope(window: &[f64]) -> f64 {
            let n = window.len() as f64;
            let mut sum_y = 0.0;
            let mut sum_xy = 0.0;
            let mut sum_x = 0.0;
            let mut sum_xx = 0.0;
            for (i, &y) in window.iter().enumerate() {
                let x = i as f64;
                sum_y += y;
                sum_xy += x * y;
                sum_x += x;
                sum_xx += x * x;
            }
            (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x)
        }

        fn check(prices: &[f64], period: usize) {
            let mut ls = LinRegSlope::new(period).unwrap();
            for (t, p) in prices.iter().enumerate() {
                let streaming = ls.update(*p);
                if t + 1 >= period {
                    let lo = t + 1 - period;
                    let expected = naive_slope(&prices[lo..=t]);
                    let got = streaming.expect("warmed up");
                    assert!(
                        (got - expected).abs() < 1e-9,
                        "slope diverges at t={t}, period={period}: got={got}, expected={expected}",
                    );
                }
            }
        }

        let noisy_ramp: Vec<f64> = (0..120)
            .map(|i| 100.0 + f64::from(i) * 0.5 + (f64::from(i) * 0.7).sin() * 3.0)
            .collect();
        check(&noisy_ramp, 5);
        check(&noisy_ramp, 14);

        let mut step = vec![1.0; 30];
        step.extend(std::iter::repeat_n(100.0, 30));
        check(&step, 7);
    }
}
