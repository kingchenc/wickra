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
/// instead of averaging it away.
///
/// Each `update` is O(1): the `Σx` and `Σxx` terms depend only on `period` and
/// are precomputed once, while `Σy` and `Σxy` are maintained incrementally as
/// the window slides. The closed-form sliding-window identity for
/// `x = 0, 1, …, period − 1` is
///
/// ```text
/// new_sum_xy = old_sum_xy − old_sum_y + popped_y0    // index shift by −1
/// new_sum_y  = old_sum_y  − popped_y0
/// // then push the new value at index n−1:
/// sum_xy += (n − 1) · new_value
/// sum_y  += new_value
/// ```
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
    /// Closed form of `Σx` over `x = 0, 1, …, period − 1` — constant in `period`.
    sum_x: f64,
    /// Closed form of `n · Σxx − (Σx)²` — constant in `period`, the OLS
    /// denominator.
    denom: f64,
    /// Running sum of the values currently in the window.
    sum_y: f64,
    /// Running `Σ(x · y)` where `x` is the position of each value within the
    /// trailing window (`0` for the oldest, `period − 1` for the newest).
    sum_xy: f64,
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
            sum_y: 0.0,
            sum_xy: 0.0,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for LinearRegression {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            // Sliding phase: pop the oldest, then shift every remaining index
            // down by 1 in the running `sum_xy`. The identity
            //   Σ((i − 1) · y_i for i = 1..n−1) = Σ(i · y_i) − Σ(y_i) + y_0
            // gives the closed-form update below.
            let y0 = self.window.pop_front().expect("non-empty");
            self.sum_xy = self.sum_xy - self.sum_y + y0;
            self.sum_y -= y0;
        }
        // Append at position `k = current length` before the push. During
        // warmup `k` ranges over `0..period − 1`; once the window is full it
        // is always `period − 1`.
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
        Some(intercept + slope * (n - 1.0))
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

    /// Cover the const accessor `period` (92-94) and the Indicator-impl
    /// `name` body (142-144). `warmup_period` is exercised elsewhere.
    #[test]
    fn accessors_and_metadata() {
        let lr = LinearRegression::new(14).unwrap();
        assert_eq!(lr.period(), 14);
        assert_eq!(lr.name(), "LinearRegression");
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

    /// Incremental OLS equivalence: the O(1) implementation must agree to
    /// `1e-9` with a fresh-from-scratch O(n) refit on every bar, on inputs
    /// chosen to stress every code path: a noisy ramp (sliding phase
    /// dominates), a step function (the new value differs sharply from the
    /// popped one), and constants (the floating-point accumulators must not
    /// drift).
    #[test]
    fn incremental_matches_naive_fit_bar_by_bar() {
        fn naive_endpoint(window: &[f64]) -> f64 {
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
            let denom = n * sum_xx - sum_x * sum_x;
            let slope = (n * sum_xy - sum_x * sum_y) / denom;
            let intercept = (sum_y - slope * sum_x) / n;
            intercept + slope * (n - 1.0)
        }

        fn check(prices: &[f64], period: usize) {
            let mut lr = LinearRegression::new(period).unwrap();
            for (t, p) in prices.iter().enumerate() {
                let streaming = lr.update(*p);
                if t + 1 >= period {
                    let lo = t + 1 - period;
                    let expected = naive_endpoint(&prices[lo..=t]);
                    let got = streaming.expect("warmed up");
                    assert!(
                        (got - expected).abs() < 1e-9,
                        "endpoint diverges at t={t}, period={period}: got={got}, expected={expected}",
                    );
                }
            }
        }

        let noisy_ramp: Vec<f64> = (0..120)
            .map(|i| 100.0 + f64::from(i) * 0.5 + (f64::from(i) * 0.7).sin() * 3.0)
            .collect();
        check(&noisy_ramp, 5);
        check(&noisy_ramp, 14);
        check(&noisy_ramp, 30);

        let mut step = vec![1.0; 30];
        step.extend(std::iter::repeat_n(100.0, 30));
        step.extend(std::iter::repeat_n(0.001, 30));
        check(&step, 5);
        check(&step, 14);

        let constant = vec![42.0; 50];
        check(&constant, 8);
        check(&constant, 25);
    }
}
