//! Ehlers SuperSmoother filter.
#![allow(clippy::doc_markdown)]

use std::f64::consts::PI;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Ehlers' 2-pole Butterworth-style "SuperSmoother" lowpass filter.
///
/// From John Ehlers' *Cycle Analytics for Traders* (2013, ch. 3). For a given
/// critical period `period`, the filter coefficients are:
///
/// ```text
/// a1 = exp(-sqrt(2) * pi / period)
/// b1 = 2 * a1 * cos(sqrt(2) * pi / period)
/// c2 = b1
/// c3 = -a1 * a1
/// c1 = 1 - c2 - c3
/// y[t] = c1 * (x[t] + x[t-1]) / 2 + c2 * y[t-1] + c3 * y[t-2]
/// ```
///
/// The implementation needs two prior inputs and two prior outputs to begin
/// running; until then it returns the input itself (a common Ehlers initial
/// condition), which lets downstream filters warm up without long delays.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, SuperSmoother};
///
/// let mut ss = SuperSmoother::new(10).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = ss.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct SuperSmoother {
    period: usize,
    c1: f64,
    c2: f64,
    c3: f64,
    prev_input: Option<f64>,
    prev_output_1: Option<f64>,
    prev_output_2: Option<f64>,
    count: usize,
}

impl SuperSmoother {
    /// Construct a new SuperSmoother with the given critical period.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        let arg = std::f64::consts::SQRT_2 * PI / period as f64;
        let a1 = (-arg).exp();
        let b1 = 2.0 * a1 * arg.cos();
        let c2 = b1;
        let c3 = -a1 * a1;
        let c1 = 1.0 - c2 - c3;
        Ok(Self {
            period,
            c1,
            c2,
            c3,
            prev_input: None,
            prev_output_1: None,
            prev_output_2: None,
            count: 0,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Filter coefficients `(c1, c2, c3)`.
    pub const fn coefficients(&self) -> (f64, f64, f64) {
        (self.c1, self.c2, self.c3)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.prev_output_1
    }
}

impl Indicator for SuperSmoother {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.prev_output_1;
        }
        self.count += 1;
        let output = match (self.prev_input, self.prev_output_1, self.prev_output_2) {
            (Some(p_in), Some(y1), Some(y2)) => {
                let avg = 0.5 * (input + p_in);
                self.c1 * avg + self.c2 * y1 + self.c3 * y2
            }
            _ => input,
        };
        self.prev_output_2 = self.prev_output_1;
        self.prev_output_1 = Some(output);
        self.prev_input = Some(input);
        Some(output)
    }

    fn reset(&mut self) {
        self.prev_input = None;
        self.prev_output_1 = None;
        self.prev_output_2 = None;
        self.count = 0;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.prev_output_1.is_some()
    }

    fn name(&self) -> &'static str {
        "SuperSmoother"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(SuperSmoother::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let mut ss = SuperSmoother::new(10).unwrap();
        assert_eq!(ss.period(), 10);
        assert_eq!(ss.name(), "SuperSmoother");
        assert_eq!(ss.warmup_period(), 1);
        let (c1, c2, c3) = ss.coefficients();
        // Coefficients sum to 1 by construction (steady-state gain == 1).
        assert_relative_eq!(c1 + c2 + c3, 1.0, epsilon = 1e-12);
        assert!(ss.value().is_none());
        ss.update(42.0);
        assert!(ss.value().is_some());
        assert!(ss.is_ready());
    }

    #[test]
    fn first_output_equals_input_then_filters() {
        let mut ss = SuperSmoother::new(10).unwrap();
        // Initial condition: first two outputs equal their inputs.
        assert_eq!(ss.update(100.0), Some(100.0));
        assert_eq!(ss.update(101.0), Some(101.0));
        let third = ss.update(102.0).unwrap();
        // From step 3 onward, the recursive filter activates and the result
        // is no longer the raw input.
        assert!((third - 102.0).abs() < 5.0);
    }

    #[test]
    fn constant_series_converges_to_constant() {
        // Steady-state gain is 1 (c1 + c2 + c3 = 1), so a flat input yields a
        // flat output after warmup.
        let mut ss = SuperSmoother::new(20).unwrap();
        let out = ss.batch(&[50.0_f64; 200]);
        for x in out.iter().skip(50).flatten() {
            assert_relative_eq!(*x, 50.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 5.0)
            .collect();
        let mut a = SuperSmoother::new(15).unwrap();
        let mut b = SuperSmoother::new(15).unwrap();
        let batch = a.batch(&prices);
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut ss = SuperSmoother::new(10).unwrap();
        ss.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        let before = ss.value();
        assert!(before.is_some());
        assert_eq!(ss.update(f64::NAN), before);
        assert_eq!(ss.update(f64::INFINITY), before);
    }

    #[test]
    fn reset_clears_state() {
        let mut ss = SuperSmoother::new(10).unwrap();
        ss.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        assert!(ss.is_ready());
        ss.reset();
        assert!(!ss.is_ready());
        assert_eq!(ss.update(50.0), Some(50.0));
    }
}
