//! Rolling Pearson correlation between two synchronised series.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Rolling Pearson correlation between two synchronised series.
///
/// Each `update` receives one `(x, y)` pair (e.g. the latest close of the
/// asset and of the benchmark). Over the trailing window of `period`
/// pairs:
///
/// ```text
/// cov_xy   = (1/n) · Σ x·y − x̄·ȳ
/// var_x    = (1/n) · Σ x² − x̄²
/// var_y    = (1/n) · Σ y² − ȳ²
/// Pearson  = cov_xy / √(var_x · var_y)
/// ```
///
/// Output is in `[−1, +1]`. `+1` means a perfect positive linear
/// relationship; `−1` is a perfect inverse one; `0` means no linear
/// relationship. It is the same statistic `SciPy` / `NumPy` report as
/// `pearsonr` and the standardised relative of [`crate::Beta`] — Beta
/// scales Pearson by the ratio of standard deviations.
///
/// Each `update` is O(1): five running sums (`Σx`, `Σy`, `Σx²`, `Σy²`,
/// `Σxy`) are maintained as the window slides. A flat series in either
/// channel gives an undefined ratio; the indicator returns `0` in that
/// case rather than producing `NaN`. The output is clamped to `[−1, +1]`
/// to absorb tiny floating-point overshoots near the boundaries.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, PearsonCorrelation};
///
/// let mut indicator = PearsonCorrelation::new(20).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = indicator.update((f64::from(i), 2.0 * f64::from(i) + 1.0));
/// }
/// // A perfectly linear pair → +1.
/// assert!((last.unwrap() - 1.0).abs() < 1e-9);
/// ```
#[derive(Debug, Clone)]
pub struct PearsonCorrelation {
    period: usize,
    window: VecDeque<(f64, f64)>,
    sum_x: f64,
    sum_y: f64,
    sum_xx: f64,
    sum_yy: f64,
    sum_xy: f64,
}

impl PearsonCorrelation {
    /// Construct a new rolling Pearson correlation.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `period < 2` — correlation is
    /// undefined for fewer than two pairs.
    pub fn new(period: usize) -> Result<Self> {
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "pearson correlation needs period >= 2",
            });
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
            sum_x: 0.0,
            sum_y: 0.0,
            sum_xx: 0.0,
            sum_yy: 0.0,
            sum_xy: 0.0,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for PearsonCorrelation {
    type Input = (f64, f64);
    type Output = f64;

    fn update(&mut self, input: (f64, f64)) -> Option<f64> {
        let (x, y) = input;
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            let (ox, oy) = self.window.pop_front().expect("non-empty");
            self.sum_x -= ox;
            self.sum_y -= oy;
            self.sum_xx -= ox * ox;
            self.sum_yy -= oy * oy;
            self.sum_xy -= ox * oy;
        }
        self.window.push_back((x, y));
        self.sum_x += x;
        self.sum_y += y;
        self.sum_xx += x * x;
        self.sum_yy += y * y;
        self.sum_xy += x * y;
        if self.window.len() < self.period {
            return None;
        }
        let n = self.period as f64;
        let mean_x = self.sum_x / n;
        let mean_y = self.sum_y / n;
        let var_x = (self.sum_xx / n - mean_x * mean_x).max(0.0);
        let var_y = (self.sum_yy / n - mean_y * mean_y).max(0.0);
        let cov = self.sum_xy / n - mean_x * mean_y;
        let denom = (var_x * var_y).sqrt();
        if denom == 0.0 {
            // At least one channel is flat: correlation is undefined.
            return Some(0.0);
        }
        Some((cov / denom).clamp(-1.0, 1.0))
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum_x = 0.0;
        self.sum_y = 0.0;
        self.sum_xx = 0.0;
        self.sum_yy = 0.0;
        self.sum_xy = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "PearsonCorrelation"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_period_below_two() {
        assert!(PearsonCorrelation::new(0).is_err());
        assert!(PearsonCorrelation::new(1).is_err());
        assert!(PearsonCorrelation::new(2).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let p = PearsonCorrelation::new(14).unwrap();
        assert_eq!(p.period(), 14);
        assert_eq!(p.warmup_period(), 14);
        assert_eq!(p.name(), "PearsonCorrelation");
    }

    #[test]
    fn perfect_positive_is_one() {
        let pairs: Vec<(f64, f64)> = (0..10)
            .map(|i| (f64::from(i), 3.0 * f64::from(i) + 1.0))
            .collect();
        let last = PearsonCorrelation::new(5)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, 1.0, epsilon = 1e-9);
    }

    #[test]
    fn perfect_negative_is_minus_one() {
        let pairs: Vec<(f64, f64)> = (0..10)
            .map(|i| (f64::from(i), -2.0 * f64::from(i) + 5.0))
            .collect();
        let last = PearsonCorrelation::new(5)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, -1.0, epsilon = 1e-9);
    }

    #[test]
    fn constant_channel_yields_zero() {
        let pairs: Vec<(f64, f64)> = (0..10).map(|i| (f64::from(i), 7.0)).collect();
        let last = PearsonCorrelation::new(5)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn output_in_minus_one_to_one_range() {
        let pairs: Vec<(f64, f64)> = (0..60)
            .map(|i| {
                let t = f64::from(i);
                (100.0 + t.sin() * 5.0, 50.0 + (t * 0.3).cos() * 3.0)
            })
            .collect();
        let mut p = PearsonCorrelation::new(20).unwrap();
        for v in p.batch(&pairs).into_iter().flatten() {
            assert!((-1.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut p = PearsonCorrelation::new(5).unwrap();
        p.batch(&[(1.0, 2.0), (2.0, 4.0), (3.0, 6.0), (4.0, 8.0), (5.0, 10.0)]);
        assert!(p.is_ready());
        p.reset();
        assert!(!p.is_ready());
        assert_eq!(p.update((1.0, 1.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let pairs: Vec<(f64, f64)> = (0..60)
            .map(|i| {
                let t = f64::from(i);
                (t.sin(), (t * 0.5).cos())
            })
            .collect();
        let batch = PearsonCorrelation::new(14).unwrap().batch(&pairs);
        let mut b = PearsonCorrelation::new(14).unwrap();
        let streamed: Vec<_> = pairs.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn non_finite_input_returns_none() {
        let mut p = PearsonCorrelation::new(3).unwrap();
        assert_eq!(p.update((f64::NAN, 1.0)), None);
        assert_eq!(p.update((1.0, f64::INFINITY)), None);
        // The rejected ticks leave no trace: a fresh window still warms up.
        assert_eq!(p.update((1.0, 2.0)), None);
        assert_eq!(p.update((2.0, 5.0)), None);
        assert!(p.update((3.0, 7.0)).is_some());
    }
}
