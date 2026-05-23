//! Rolling population standard deviation.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Rolling population standard deviation over the last `period` values.
///
/// ```text
/// mean     = (1/n) · Σ price
/// variance = (1/n) · Σ price² − mean²
/// StdDev   = √variance
/// ```
///
/// This is the **population** standard deviation (divisor `n`, not `n − 1`) —
/// the same dispersion measure that drives [`BollingerBands`](crate::BollingerBands).
/// It is maintained as an O(1) rolling state machine: a running sum and a
/// running sum-of-squares, updated by one add and one subtract per bar. Tiny
/// negative variances from floating-point cancellation are clamped to zero
/// before the square root.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, StdDev};
///
/// let mut indicator = StdDev::new(20).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct StdDev {
    period: usize,
    window: VecDeque<f64>,
    sum: f64,
    sum_sq: f64,
    last: Option<f64>,
}

impl StdDev {
    /// Construct a new rolling standard deviation with the given period.
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
            sum: 0.0,
            sum_sq: 0.0,
            last: None,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for StdDev {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            // Non-finite input is ignored; the window is left untouched.
            return self.last;
        }
        if self.window.len() == self.period {
            let old = self.window.pop_front().expect("window is non-empty");
            self.sum -= old;
            self.sum_sq -= old * old;
        }
        self.window.push_back(input);
        self.sum += input;
        self.sum_sq += input * input;
        if self.window.len() < self.period {
            return None;
        }
        let n = self.period as f64;
        let mean = self.sum / n;
        // Clamp floating-point cancellation noise: variance is never negative.
        let variance = (self.sum_sq / n - mean * mean).max(0.0);
        let sd = variance.sqrt();
        self.last = Some(sd);
        Some(sd)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum = 0.0;
        self.sum_sq = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "StdDev"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(StdDev::new(0), Err(Error::PeriodZero)));
    }

    /// Cover the const accessors `period` / `value` and the Indicator-impl
    /// `warmup_period` / `name` methods (lines 64-71, 110-112, 118-120).
    /// Existing tests only inspect numeric outputs of `update` / `batch`.
    #[test]
    fn accessors_and_metadata() {
        let mut sd = StdDev::new(14).unwrap();
        assert_eq!(sd.period(), 14);
        assert_eq!(sd.warmup_period(), 14);
        assert_eq!(sd.name(), "StdDev");
        assert_eq!(sd.value(), None);
        for i in 1..=14 {
            sd.update(f64::from(i));
        }
        assert!(sd.value().is_some());
    }

    #[test]
    fn reference_value() {
        // StdDev(3) of [2, 4, 6]: mean = 4, variance = (4+0+4)/3 = 8/3.
        let mut sd = StdDev::new(3).unwrap();
        let out = sd.batch(&[2.0, 4.0, 6.0]);
        assert_eq!(out[0], None);
        assert_eq!(out[1], None);
        assert_relative_eq!(out[2].unwrap(), (8.0_f64 / 3.0).sqrt(), epsilon = 1e-12);
    }

    #[test]
    fn constant_series_yields_zero() {
        let mut sd = StdDev::new(5).unwrap();
        let out = sd.batch(&[42.0; 20]);
        for v in out.iter().skip(4).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn matches_naive_definition() {
        let prices: Vec<f64> = (1..=60)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 8.0)
            .collect();
        let period = 10;
        let got = StdDev::new(period).unwrap().batch(&prices);
        for (i, g) in got.iter().enumerate() {
            if let Some(value) = g {
                let window = &prices[i + 1 - period..=i];
                let mean = window.iter().sum::<f64>() / period as f64;
                let var = window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period as f64;
                assert_relative_eq!(*value, var.sqrt(), epsilon = 1e-9);
            }
        }
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut sd = StdDev::new(3).unwrap();
        let out = sd.batch(&[2.0, 4.0, 6.0]);
        let last = out[2];
        assert!(last.is_some());
        assert_eq!(sd.update(f64::NAN), last);
        assert_eq!(sd.update(f64::INFINITY), last);
    }

    #[test]
    fn reset_clears_state() {
        let mut sd = StdDev::new(3).unwrap();
        sd.batch(&[1.0, 2.0, 3.0, 4.0]);
        assert!(sd.is_ready());
        sd.reset();
        assert!(!sd.is_ready());
        assert_eq!(sd.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=60)
            .map(|i| 100.0 + (f64::from(i) * 0.3).cos() * 7.0)
            .collect();
        let batch = StdDev::new(14).unwrap().batch(&prices);
        let mut b = StdDev::new(14).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
