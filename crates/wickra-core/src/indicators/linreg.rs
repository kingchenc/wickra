//! Linear Regression (rolling least-squares endpoint).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::indicators::rolling_moments::ShiftedTrend;
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
    /// Rolling fit sums, held relative to a reference point inside the window.
    trend: ShiftedTrend,
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
        if period > crate::error::MAX_PERIOD {
            return Err(Error::InvalidPeriod {
                message: crate::error::PERIOD_ABOVE_MAX,
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
            trend: ShiftedTrend::new(),
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

    #[inline]
    fn update(&mut self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            let front = self.window.pop_front().expect("non-empty");
            self.trend.slide(front);
        }
        let index = self.window.len();
        self.window.push_back(value);
        self.trend.push(value, index);
        if self.trend.needs_reseed(self.period) {
            self.trend.reseed(self.window.iter().copied());
        }

        if self.window.len() < self.period {
            return None;
        }
        let n = self.period as f64;
        let slope = (n * self.trend.sum_xy() - self.sum_x * self.trend.sum_y()) / self.denom;
        // The intercept names an absolute price level, so the reference point
        // the sums are held relative to has to come back here. The slope does
        // not: it is invariant under that shift.
        let intercept = (self.trend.sum_y() - slope * self.sum_x) / n + self.trend.offset();
        Some(intercept + slope * (n - 1.0))
    }

    fn reset(&mut self) {
        self.window.clear();
        self.trend.reset();
    }

    #[inline]
    fn warmup_period(&self) -> usize {
        self.period
    }

    #[inline]
    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    #[inline]
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

    /// Least-squares fit of a window against its own index, computed entirely
    /// on deviations. Returns `(slope, mean, sse)`.
    ///
    /// Forming the residuals as `y - (intercept + slope*i)` instead, with both
    /// sides the size of the price, is exactly what this file stopped doing:
    /// at a price level of 1e8 that subtraction alone costs eight digits. On
    /// the centred scale the fitted line is just `slope * (i - mean_x)`.
    fn centred_fit(window: &[f64]) -> (f64, f64, f64) {
        let n = window.len() as f64;
        let mean = window.iter().sum::<f64>() / n;
        let mean_x = (n - 1.0) / 2.0;
        let (mut sxy, mut sxx) = (0.0, 0.0);
        for (i, &y) in window.iter().enumerate() {
            let dx = i as f64 - mean_x;
            sxy += dx * (y - mean);
            sxx += dx * dx;
        }
        let slope = sxy / sxx;
        let mut sse = 0.0;
        for (i, &y) in window.iter().enumerate() {
            let r = (y - mean) - slope * (i as f64 - mean_x);
            sse += r * r;
        }
        (slope, mean, sse)
    }

    /// A one-unit wobble on top of a large price level: the level-to-deviation
    /// ratio is what drives the cancellation, and scaling the wobble with the
    /// level instead keeps that ratio constant and hides the defect entirely.
    fn high_level_series(bars: usize) -> Vec<f64> {
        (0..bars)
            .map(|i| {
                let t = i as f64;
                1e8 + ((t * 0.11).sin() + 0.4 * (t * 0.37).cos())
            })
            .collect()
    }

    /// The endpoint is an absolute price level, so it is the one place the
    /// reference point the sums are held relative to has to be added back. A
    /// sign error there would show up immediately as an output near zero rather
    /// than near the price; matching a centred fit at a level of 1e8 pins both
    /// the restoration and the slope it is projected along.
    #[test]
    fn endpoint_at_a_high_price_level_matches_a_centred_fit() {
        const P: usize = 20;
        let data = high_level_series(400);
        let mut ind = LinearRegression::new(P).unwrap();
        let mut compared = 0_usize;
        for (i, &v) in data.iter().enumerate() {
            let Some(endpoint) = ind.update(v) else {
                continue;
            };
            let window = &data[i + 1 - P..=i];
            let (slope, mean, _) = centred_fit(window);
            let mean_x = (P as f64 - 1.0) / 2.0;
            // On the centred scale the fit passes through (mean_x, 0), so the
            // endpoint is the window mean plus the slope run from there.
            let want = mean + slope * (P as f64 - 1.0 - mean_x);
            compared += 1;
            assert_relative_eq!(endpoint, want, max_relative = 1e-14);
        }
        assert_eq!(compared, data.len() - ind.warmup_period() + 1);
    }
}
