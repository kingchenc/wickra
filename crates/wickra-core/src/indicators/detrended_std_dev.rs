//! Population standard deviation of residuals from a rolling OLS detrend.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Detrended (residual) standard deviation over the last `period` inputs.
///
/// Over the trailing window indexed `x = 0, 1, …, period − 1` the OLS line
/// `y = a + b·x` is fitted and the residual sum of squares is then divided
/// by `n` (population convention):
///
/// ```text
/// slope     = (n·Σxy − Σx·Σy) / (n·Σxx − (Σx)²)
/// SS_total  = Σy² − n·ȳ²
/// RSS       = SS_total − slope² · ( denom / n )
/// DetrendedStdDev = √( RSS / n )
/// ```
///
/// Unlike [`crate::StdDev`], which measures dispersion around the rolling
/// **mean**, `DetrendedStdDev` measures dispersion around the rolling
/// **linear trend** — the portion of the price action that is *not*
/// explained by the local slope. On a strongly trending series this is
/// much smaller than `StdDev`; on a sideways, mean-reverting series the
/// two converge.
///
/// The divisor is `n` (population), matching the convention of
/// [`crate::StdDev`]; use [`crate::StandardError`] when you want the
/// textbook standard error of estimate with `n − 2` residual degrees of
/// freedom.
///
/// Each `update` is O(1) via the same rolling sums as
/// [`crate::LinearRegression`], plus a running `Σy²`. Floating-point
/// cancellation noise in the residual is clamped to zero before the square
/// root.
///
/// # Example
///
/// ```
/// use wickra_core::{DetrendedStdDev, Indicator};
///
/// let mut indicator = DetrendedStdDev::new(14).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = indicator.update(100.0 + f64::from(i) + (f64::from(i) * 0.3).sin());
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct DetrendedStdDev {
    period: usize,
    window: VecDeque<f64>,
    sum_x: f64,
    /// `n·Σxx − (Σx)²` — OLS denominator, constant in `period`.
    denom: f64,
    sum_y: f64,
    sum_xy: f64,
    sum_y_sq: f64,
}

impl DetrendedStdDev {
    /// Construct a new rolling detrended standard deviation.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `period < 2` — a regression line
    /// is undefined for fewer than two points.
    pub fn new(period: usize) -> Result<Self> {
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "detrended stddev needs period >= 2",
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

impl Indicator for DetrendedStdDev {
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
        let s_xx = self.denom / n;
        let rss = (ss_total - slope * slope * s_xx).max(0.0);
        Some((rss / n).sqrt())
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
        "DetrendedStdDev"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_period_below_two() {
        assert!(DetrendedStdDev::new(0).is_err());
        assert!(DetrendedStdDev::new(1).is_err());
        assert!(DetrendedStdDev::new(2).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let d = DetrendedStdDev::new(14).unwrap();
        assert_eq!(d.period(), 14);
        assert_eq!(d.warmup_period(), 14);
        assert_eq!(d.name(), "DetrendedStdDev");
    }

    #[test]
    fn perfect_line_has_zero_residual() {
        // Residuals are zero on a perfectly linear series.
        let prices: Vec<f64> = (0..30).map(|i| 2.0 * f64::from(i) + 5.0).collect();
        let mut d = DetrendedStdDev::new(10).unwrap();
        for v in d.batch(&prices).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn constant_series_yields_zero() {
        let mut d = DetrendedStdDev::new(5).unwrap();
        for v in d.batch(&[42.0; 20]).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn never_exceeds_stddev() {
        // The detrended residual is the projection of (y - ȳ) orthogonal to
        // the trend axis, so its norm cannot exceed the raw stddev. Equality
        // holds iff the OLS slope is exactly zero.
        let prices: Vec<f64> = (0..60)
            .map(|i| 50.0 + f64::from(i) * 0.5 + (f64::from(i) * 0.7).sin() * 4.0)
            .collect();
        let mut d = DetrendedStdDev::new(14).unwrap();
        let mut sd = crate::StdDev::new(14).unwrap();
        for &p in &prices {
            let (dv, sv) = (d.update(p), sd.update(p));
            assert_eq!(dv.is_some(), sv.is_some());
            if let (Some(dv), Some(sv)) = (dv, sv) {
                assert!(dv <= sv + 1e-9, "detrended {dv} should be <= stddev {sv}");
            }
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut d = DetrendedStdDev::new(5).unwrap();
        d.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(d.is_ready());
        d.reset();
        assert!(!d.is_ready());
        assert_eq!(d.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..60)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 10.0)
            .collect();
        let batch = DetrendedStdDev::new(14).unwrap().batch(&prices);
        let mut b = DetrendedStdDev::new(14).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
