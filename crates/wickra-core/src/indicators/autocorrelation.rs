//! Rolling lag-`k` autocorrelation.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Rolling lag-`lag` autocorrelation of the last `period` inputs.
///
/// Over the trailing window the Pearson correlation between the series and
/// itself shifted by `lag` is computed:
///
/// ```text
/// y_i  for i = 0..period − 1
/// ACF(lag) = Σ ( (y_i − ȳ) · (y_{i + lag} − ȳ) ) / Σ ( y_i − ȳ )²
/// ```
///
/// `+1` means a perfectly repeating pattern at the given lag; `−1` means a
/// perfect alternation. Values near `0` mean the series at `t` and `t −
/// lag` carry no linear relationship — a clean white-noise proxy. The
/// classic application is detecting periodicity (a peak in `|ACF(lag)|`
/// flags a cycle of that length) or testing whether returns are
/// uncorrelated (a key efficient-markets diagnostic).
///
/// `period` must be strictly greater than `lag` so that at least two
/// `(y, y_lagged)` pairs exist. A flat window has zero variance; the
/// indicator returns `0` rather than dividing by zero.
///
/// # Example
///
/// ```
/// use wickra_core::{Autocorrelation, Indicator};
///
/// let mut indicator = Autocorrelation::new(20, 1).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = indicator.update(f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Autocorrelation {
    period: usize,
    lag: usize,
    window: VecDeque<f64>,
}

impl Autocorrelation {
    /// Construct a new rolling lag-`lag` autocorrelation over `period` inputs.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `lag == 0` or `lag >= period`.
    pub fn new(period: usize, lag: usize) -> Result<Self> {
        if lag == 0 {
            return Err(Error::InvalidPeriod {
                message: "autocorrelation lag must be >= 1",
            });
        }
        if period <= lag {
            return Err(Error::InvalidPeriod {
                message: "autocorrelation needs period > lag",
            });
        }
        Ok(Self {
            period,
            lag,
            window: VecDeque::with_capacity(period),
        })
    }

    /// Configured window period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Configured lag.
    pub const fn lag(&self) -> usize {
        self.lag
    }
}

impl Indicator for Autocorrelation {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(value);
        if self.window.len() < self.period {
            return None;
        }
        // ACF over the current window with a single inner pass. The window is
        // small relative to a typical input stream so the O(period) per-bar
        // cost is bounded by the user-chosen `period`; the constant factor
        // is dominated by two adds and one multiply per element.
        let n = self.period as f64;
        let mean = self.window.iter().sum::<f64>() / n;
        let mut denom = 0.0;
        let mut numer = 0.0;
        // The window is a deque; index via slices for cache-friendly access.
        let (front, back) = self.window.as_slices();
        let get = |i: usize| -> f64 {
            if i < front.len() {
                front[i]
            } else {
                back[i - front.len()]
            }
        };
        for i in 0..self.period {
            let d = get(i) - mean;
            denom += d * d;
        }
        let lag = self.lag;
        for i in 0..(self.period - lag) {
            numer += (get(i) - mean) * (get(i + lag) - mean);
        }
        if denom == 0.0 {
            return Some(0.0);
        }
        Some(numer / denom)
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
        "Autocorrelation"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_lag() {
        assert!(Autocorrelation::new(10, 0).is_err());
    }

    #[test]
    fn rejects_lag_geq_period() {
        assert!(Autocorrelation::new(5, 5).is_err());
        assert!(Autocorrelation::new(5, 10).is_err());
    }

    #[test]
    fn accessors_and_metadata() {
        let a = Autocorrelation::new(14, 2).unwrap();
        assert_eq!(a.period(), 14);
        assert_eq!(a.lag(), 2);
        assert_eq!(a.warmup_period(), 14);
        assert_eq!(a.name(), "Autocorrelation");
    }

    #[test]
    fn constant_series_yields_zero() {
        let mut a = Autocorrelation::new(10, 1).unwrap();
        for v in a.batch(&[42.0; 30]).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn alternating_series_lag_one_is_strongly_negative() {
        // [−1, 1, −1, 1, …] alternates each step.
        let prices: Vec<f64> = (0..20)
            .map(|i| if i % 2 == 0 { -1.0 } else { 1.0 })
            .collect();
        let mut a = Autocorrelation::new(10, 1).unwrap();
        let last = a.batch(&prices).into_iter().flatten().last().unwrap();
        assert!(
            last < -0.5,
            "alternating series should be strongly negative, got {last}"
        );
    }

    #[test]
    fn repeating_series_is_strongly_positive_at_period() {
        // A series that repeats every 4 steps must have ACF(4) ≈ +1.
        let pattern = [1.0, 2.0, 3.0, 4.0];
        let prices: Vec<f64> = (0..32).map(|i| pattern[i % 4]).collect();
        let mut a = Autocorrelation::new(16, 4).unwrap();
        let last = a.batch(&prices).into_iter().flatten().last().unwrap();
        assert!(
            last > 0.5,
            "period-4 repeat should ACF(4) > 0.5, got {last}"
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut a = Autocorrelation::new(5, 1).unwrap();
        a.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(a.is_ready());
        a.reset();
        assert!(!a.is_ready());
        assert_eq!(a.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..60).map(|i| (f64::from(i) * 0.3).sin()).collect();
        let batch = Autocorrelation::new(14, 2).unwrap().batch(&prices);
        let mut b = Autocorrelation::new(14, 2).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
