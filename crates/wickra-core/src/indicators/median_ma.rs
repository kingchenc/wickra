//! Median Moving Average.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Median Moving Average — the rolling median of the last `period` inputs.
///
/// For an odd `period` the output is the middle order statistic of the window;
/// for an even `period` it is the average of the two central values. Because it
/// is a rank statistic rather than a sum, the median MA is far more robust to
/// single outliers than the [`Sma`](crate::Sma): a lone spike shifts the rank
/// by at most one position instead of dragging the whole average.
///
/// Each `update` slides the window and computes the median by sorting a copy of
/// the `period` buffered values — O(`period` · log `period`) per step, with the
/// period fixed and bounded.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, MedianMa};
///
/// let mut indicator = MedianMa::new(5).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct MedianMa {
    period: usize,
    window: VecDeque<f64>,
}

impl MedianMa {
    /// Construct a new median moving average over `period` inputs.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if the window is full.
    pub fn value(&self) -> Option<f64> {
        if self.window.len() != self.period {
            return None;
        }
        let mut sorted: Vec<f64> = self.window.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).expect("window holds only finite values"));
        let mid = self.period / 2;
        if self.period % 2 == 1 {
            Some(sorted[mid])
        } else {
            Some(f64::midpoint(sorted[mid - 1], sorted[mid]))
        }
    }
}

impl Indicator for MedianMa {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.value();
        }
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(input);
        self.value()
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
        "MedianMA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(MedianMa::new(0), Err(Error::PeriodZero)));
    }

    /// Cover the const accessor `period` and the Indicator-impl `warmup_period`
    /// + `name`.
    #[test]
    fn accessors_and_metadata() {
        let mma = MedianMa::new(7).unwrap();
        assert_eq!(mma.period(), 7);
        assert_eq!(mma.warmup_period(), 7);
        assert_eq!(mma.name(), "MedianMA");
    }

    #[test]
    fn warmup_returns_none_then_odd_median() {
        let mut mma = MedianMa::new(3).unwrap();
        assert_eq!(mma.update(5.0), None);
        assert_eq!(mma.update(1.0), None);
        // median of [5, 1, 3] = 3 (middle order statistic).
        assert_relative_eq!(mma.update(3.0).unwrap(), 3.0, epsilon = 1e-12);
    }

    #[test]
    fn even_period_averages_two_central_values() {
        // median of [1, 2, 3, 4] = (2 + 3) / 2 = 2.5.
        let mut mma = MedianMa::new(4).unwrap();
        let v = mma.batch(&[1.0, 2.0, 3.0, 4.0]);
        assert_relative_eq!(v[3].unwrap(), 2.5, epsilon = 1e-12);
    }

    #[test]
    fn robust_to_single_outlier() {
        // A lone spike does not move the median of an odd window the way it
        // would move an SMA. median of [10, 11, 9999] = 11.
        let mut mma = MedianMa::new(3).unwrap();
        let v = mma.batch(&[10.0, 11.0, 9999.0]);
        assert_relative_eq!(v[2].unwrap(), 11.0, epsilon = 1e-12);
    }

    #[test]
    fn period_one_is_pass_through() {
        let mut mma = MedianMa::new(1).unwrap();
        assert_relative_eq!(mma.update(5.5).unwrap(), 5.5, epsilon = 1e-12);
        assert_relative_eq!(mma.update(7.5).unwrap(), 7.5, epsilon = 1e-12);
    }

    #[test]
    fn slides_window_correctly() {
        // After [1,2,3] the window slides to [2,3,4] -> median 3, then [3,4,5] -> 4.
        let mut mma = MedianMa::new(3).unwrap();
        let v = mma.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_relative_eq!(v[2].unwrap(), 2.0, epsilon = 1e-12);
        assert_relative_eq!(v[3].unwrap(), 3.0, epsilon = 1e-12);
        assert_relative_eq!(v[4].unwrap(), 4.0, epsilon = 1e-12);
    }

    #[test]
    fn reset_clears_state() {
        let mut mma = MedianMa::new(4).unwrap();
        mma.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(mma.is_ready());
        mma.reset();
        assert!(!mma.is_ready());
        assert_eq!(mma.update(10.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=20).map(|i| (f64::from(i) * 0.7).sin() * 5.0).collect();
        let mut a = MedianMa::new(5).unwrap();
        let mut b = MedianMa::new(5).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn ignores_non_finite_input_but_keeps_state() {
        let mut mma = MedianMa::new(3).unwrap();
        mma.update(5.0);
        mma.update(1.0);
        let ready = mma
            .update(3.0)
            .expect("MedianMA(3) ready after three inputs");
        assert_eq!(mma.update(f64::NAN), Some(ready));
        assert_eq!(mma.update(f64::INFINITY), Some(ready));
        // Window still [5, 1, 3] -> next real input slides to [1, 3, 8] -> median 3.
        assert_relative_eq!(mma.update(8.0).unwrap(), 3.0, epsilon = 1e-12);
    }
}
