//! Inverse Fisher Transform (Ehlers).

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Inverse Fisher Transform of a scaled scalar input.
///
/// Compresses the input through `(e^{2x} - 1) / (e^{2x} + 1) = tanh(x)`, the
/// algebraic inverse of the Fisher transform. The output is bounded in
/// `[-1, +1]` (saturating to exactly `±1` for `|scale * input| >= ~19.06`
/// under IEEE 754 doubles), which makes overbought/oversold thresholds at, say, `±0.5`
/// universal across markets and timeframes — the classic use described by
/// Ehlers in *Cybernetic Analysis for Stocks and Futures* (2004).
///
/// The constructor takes a `scale` multiplier so callers can feed raw
/// oscillator readings (e.g. RSI in `[0, 100]`, mapped to `[-5, +5]` with
/// `scale = 0.1` after a `-50` shift) without writing their own scaler.
/// Internally the indicator just computes `tanh(scale * input)`.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, InverseFisherTransform};
///
/// let mut ift = InverseFisherTransform::new(1.0).unwrap();
/// // Large positive input saturates to +1, large negative to -1.
/// assert!(ift.update(10.0).unwrap() > 0.999);
/// assert!(ift.update(-10.0).unwrap() < -0.999);
/// ```
#[derive(Debug, Clone)]
pub struct InverseFisherTransform {
    scale: f64,
    last_value: Option<f64>,
}

impl InverseFisherTransform {
    /// Construct with a multiplicative scale applied before the tanh squash.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPeriod`] if `scale` is not finite or non-positive.
    pub fn new(scale: f64) -> Result<Self> {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(Error::InvalidPeriod {
                message: "scale must be a positive finite number",
            });
        }
        Ok(Self {
            scale,
            last_value: None,
        })
    }

    /// Configured scale.
    pub const fn scale(&self) -> f64 {
        self.scale
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for InverseFisherTransform {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.last_value;
        }
        let scaled = self.scale * input;
        // tanh is numerically safe for any finite input.
        let v = scaled.tanh();
        self.last_value = Some(v);
        Some(v)
    }

    fn reset(&mut self) {
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "InverseFisherTransform"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_non_positive_scale() {
        assert!(matches!(
            InverseFisherTransform::new(0.0),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            InverseFisherTransform::new(-1.0),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            InverseFisherTransform::new(f64::NAN),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let mut ift = InverseFisherTransform::new(0.5).unwrap();
        assert_relative_eq!(ift.scale(), 0.5, epsilon = 1e-15);
        assert_eq!(ift.warmup_period(), 1);
        assert_eq!(ift.name(), "InverseFisherTransform");
        assert!(!ift.is_ready());
        assert!(ift.update(1.0).is_some());
        assert!(ift.is_ready());
        assert!(ift.value().is_some());
    }

    #[test]
    fn zero_input_yields_zero() {
        let mut ift = InverseFisherTransform::new(1.0).unwrap();
        assert_relative_eq!(ift.update(0.0).unwrap(), 0.0, epsilon = 1e-15);
    }

    #[test]
    fn output_bounded_in_closed_unit_interval() {
        // tanh saturates to exactly ±1.0 in IEEE 754 once |x| >= ~19.06, so the
        // output is in the closed interval [-1, +1] rather than strictly open.
        let mut ift = InverseFisherTransform::new(1.0).unwrap();
        for i in -100..=100 {
            let v = ift.update(f64::from(i)).unwrap();
            assert!((-1.0..=1.0).contains(&v), "v={v}");
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut ift = InverseFisherTransform::new(1.0).unwrap();
        ift.update(2.0);
        assert!(ift.is_ready());
        ift.reset();
        assert!(!ift.is_ready());
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut ift = InverseFisherTransform::new(1.0).unwrap();
        ift.update(1.0);
        let before = ift.value();
        assert_eq!(ift.update(f64::NAN), before);
        assert_eq!(ift.update(f64::INFINITY), before);
    }
}
