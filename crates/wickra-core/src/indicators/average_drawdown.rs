//! Rolling Average Drawdown.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Rolling Average Drawdown.
///
/// Input is treated as an equity-curve sample. The indicator scans the
/// trailing window of `period` values, tracks the running peak inside the
/// window, and reports the **mean** of all bar-by-bar drawdowns (the average
/// "pain" of being under water):
///
/// ```text
/// drawdown_t = (peak_t − equity_t) / peak_t  (running peak inside window)
/// AvgDD      = mean(drawdown_t over window)
/// ```
///
/// Output is non-negative (a fraction; `0.05` ≈ 5 % average drawdown). This
/// is the **Pain Index** under a different name — see [`crate::PainIndex`]
/// for the same metric exposed under its conventional label.
///
/// Each `update` is O(period).
#[derive(Debug, Clone)]
pub struct AverageDrawdown {
    period: usize,
    window: VecDeque<f64>,
}

impl AverageDrawdown {
    /// Construct a new rolling Average Drawdown.
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
        })
    }

    /// Configured window length.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for AverageDrawdown {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(input);
        if self.window.len() < self.period {
            return None;
        }
        let mut peak = f64::NEG_INFINITY;
        let mut sum_dd = 0.0_f64;
        for &v in &self.window {
            if v > peak {
                peak = v;
            }
            if peak > 0.0 {
                sum_dd += (peak - v) / peak;
            }
        }
        Some(sum_dd / self.period as f64)
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
        "AverageDrawdown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(AverageDrawdown::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let a = AverageDrawdown::new(10).unwrap();
        assert_eq!(a.period(), 10);
        assert_eq!(a.name(), "AverageDrawdown");
        assert_eq!(a.warmup_period(), 10);
    }

    #[test]
    fn pure_uptrend_yields_zero() {
        let mut a = AverageDrawdown::new(5).unwrap();
        let out = a.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        for v in out.into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn reference_value() {
        // window [100, 120, 90, 110]:
        // peaks: 100, 120, 120, 120; dd: 0, 0, (30/120)=.25, (10/120)=.0833...
        // avg = (.25 + .0833...) / 4 = .0833...
        let mut a = AverageDrawdown::new(4).unwrap();
        let out = a.batch(&[100.0, 120.0, 90.0, 110.0]);
        let expected = (0.25 + (10.0 / 120.0)) / 4.0;
        assert_relative_eq!(out[3].unwrap(), expected, epsilon = 1e-12);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut a = AverageDrawdown::new(3).unwrap();
        assert_eq!(a.update(f64::NAN), None);
        assert_eq!(a.update(f64::INFINITY), None);
    }

    #[test]
    fn reset_clears_state() {
        let mut a = AverageDrawdown::new(3).unwrap();
        a.batch(&[100.0, 90.0, 110.0]);
        assert!(a.is_ready());
        a.reset();
        assert!(!a.is_ready());
        assert_eq!(a.update(100.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..40)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 8.0)
            .collect();
        let batch = AverageDrawdown::new(10).unwrap().batch(&prices);
        let mut s = AverageDrawdown::new(10).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| s.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn non_positive_peak_yields_zero() {
        let mut a = AverageDrawdown::new(3).unwrap();
        let out = a.batch(&[0.0_f64; 6]);
        for v in out.into_iter().flatten() {
            assert_eq!(v, 0.0);
        }
    }
}
