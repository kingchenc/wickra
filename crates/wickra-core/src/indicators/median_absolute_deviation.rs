//! Rolling Median Absolute Deviation (MAD), a robust dispersion estimator.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Median Absolute Deviation of the last `period` values.
///
/// ```text
/// med   = median(window)
/// MAD   = median( |x_i − med|  for x_i in window )
/// ```
///
/// MAD is the median analogue of the standard deviation: it is a robust
/// dispersion measure that ignores extreme outliers (a single huge spike
/// barely moves the result) and is widely used as a sturdier alternative
/// to `StdDev` for risk reporting on heavy-tailed return distributions.
/// Multiplying MAD by `1.4826` produces a consistent estimator of the
/// underlying Gaussian standard deviation (the "robust σ"); Wickra returns
/// the raw MAD so the caller chooses whether to scale.
///
/// Each `update` is O(period log period): the window is kept as a deque
/// and copied into a small scratch buffer that is sorted twice (once to
/// pick the median, once to pick the median of absolute deviations). The
/// rolling structure makes the constant factor low; for the typical
/// period range (10–100) this is dwarfed by the streaming overhead.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, MedianAbsoluteDeviation};
///
/// let mut indicator = MedianAbsoluteDeviation::new(20).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct MedianAbsoluteDeviation {
    period: usize,
    window: VecDeque<f64>,
    /// Reusable scratch buffer to avoid allocating per `update`.
    scratch: Vec<f64>,
}

impl MedianAbsoluteDeviation {
    /// Construct a new rolling MAD with the given period.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
            scratch: Vec::with_capacity(period),
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

/// Sort a slice of `f64` in-place using total ordering (NaN-safe).
fn sort_finite(buf: &mut [f64]) {
    buf.sort_by(f64::total_cmp);
}

/// Median of a sorted, non-empty slice.
fn median_sorted(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    let mid = n / 2;
    if n % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) * 0.5
    } else {
        sorted[mid]
    }
}

impl Indicator for MedianAbsoluteDeviation {
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
        // Copy into scratch and sort to find the window median.
        self.scratch.clear();
        self.scratch.extend(self.window.iter().copied());
        sort_finite(&mut self.scratch);
        let med = median_sorted(&self.scratch);
        // Replace with absolute deviations and sort again.
        for x in &mut self.scratch {
            *x = (*x - med).abs();
        }
        sort_finite(&mut self.scratch);
        Some(median_sorted(&self.scratch))
    }

    fn reset(&mut self) {
        self.window.clear();
        self.scratch.clear();
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "MedianAbsoluteDeviation"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(
            MedianAbsoluteDeviation::new(0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let m = MedianAbsoluteDeviation::new(14).unwrap();
        assert_eq!(m.period(), 14);
        assert_eq!(m.warmup_period(), 14);
        assert_eq!(m.name(), "MedianAbsoluteDeviation");
    }

    #[test]
    fn reference_value() {
        // [1, 1, 2, 2, 4, 6, 9]: median = 2, deviations [1,1,0,0,2,4,7],
        // sorted [0,0,1,1,2,4,7] → median = 1.
        let mut m = MedianAbsoluteDeviation::new(7).unwrap();
        let out = m.batch(&[1.0, 1.0, 2.0, 2.0, 4.0, 6.0, 9.0]);
        assert_relative_eq!(out[6].unwrap(), 1.0, epsilon = 1e-12);
    }

    #[test]
    fn constant_series_yields_zero() {
        let mut m = MedianAbsoluteDeviation::new(5).unwrap();
        for v in m.batch(&[42.0; 20]).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn ignores_single_extreme_outlier() {
        // A window of 9 equal values plus 1 huge outlier still has MAD = 0,
        // because more than half the window agrees on the median and the
        // deviations majority are zero.
        let mut m = MedianAbsoluteDeviation::new(10).unwrap();
        let mut prices = vec![5.0; 9];
        prices.push(1_000.0);
        let last = m.batch(&prices).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn reset_clears_state() {
        let mut m = MedianAbsoluteDeviation::new(5).unwrap();
        m.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(m.is_ready());
        m.reset();
        assert!(!m.is_ready());
        assert_eq!(m.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..60)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        let batch = MedianAbsoluteDeviation::new(14).unwrap().batch(&prices);
        let mut b = MedianAbsoluteDeviation::new(14).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
