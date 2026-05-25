//! Ehlers Decycler Oscillator (difference of two decyclers).

use crate::error::{Error, Result};
use crate::indicators::decycler::Decycler;
use crate::traits::Indicator;

/// Difference between a fast and a slow [`Decycler`], producing a smoothed
/// oscillator that crosses zero at trend changes.
///
/// Defined as `fast_decycler - slow_decycler` with `fast_period < slow_period`.
/// The construct removes the trend component that both decyclers share, leaving
/// the medium-frequency cycle band — analogous in spirit to MACD but with
/// Ehlers' zero-lag high-pass filters instead of EMAs.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, DecyclerOscillator};
///
/// let mut dco = DecyclerOscillator::new(10, 30).unwrap();
/// let mut last = None;
/// for i in 0..60 {
///     last = dco.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct DecyclerOscillator {
    fast: Decycler,
    slow: Decycler,
    last_value: Option<f64>,
}

impl DecyclerOscillator {
    /// Construct with the fast and slow periods.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if either period is zero, and
    /// [`Error::InvalidPeriod`] if `fast >= slow`.
    pub fn new(fast: usize, slow: usize) -> Result<Self> {
        if fast == 0 || slow == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "fast period must be strictly less than slow period",
            });
        }
        Ok(Self {
            fast: Decycler::new(fast)?,
            slow: Decycler::new(slow)?,
            last_value: None,
        })
    }

    /// Configured `(fast, slow)` periods.
    pub fn periods(&self) -> (usize, usize) {
        (self.fast.period(), self.slow.period())
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for DecyclerOscillator {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.last_value;
        }
        // Both child `Decycler` instances emit `Some` from the first bar
        // (Ehlers' convention is "output = input" until the recursion warms),
        // so the pair is always populated and the `?` short-circuit never
        // fires in practice.
        let f = self.fast.update(input)?;
        let s = self.slow.update(input)?;
        let v = f - s;
        self.last_value = Some(v);
        Some(v)
    }

    fn reset(&mut self) {
        self.fast.reset();
        self.slow.reset();
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        self.fast.warmup_period().max(self.slow.warmup_period())
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "DecyclerOscillator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_invalid_periods() {
        assert!(matches!(
            DecyclerOscillator::new(0, 20),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            DecyclerOscillator::new(10, 0),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            DecyclerOscillator::new(20, 10),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            DecyclerOscillator::new(10, 10),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let mut dco = DecyclerOscillator::new(10, 30).unwrap();
        assert_eq!(dco.periods(), (10, 30));
        assert_eq!(dco.name(), "DecyclerOscillator");
        assert!(dco.warmup_period() >= 1);
        assert!(!dco.is_ready());
        dco.update(100.0);
        assert!(dco.is_ready());
        assert!(dco.value().is_some());
    }

    #[test]
    fn constant_series_yields_zero() {
        let mut dco = DecyclerOscillator::new(10, 30).unwrap();
        let out = dco.batch(&[42.0_f64; 80]);
        for x in out.iter().flatten() {
            assert_relative_eq!(*x, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..100)
            .map(|i| 100.0 + (f64::from(i) * 0.2).cos() * 6.0)
            .collect();
        let mut a = DecyclerOscillator::new(10, 30).unwrap();
        let mut b = DecyclerOscillator::new(10, 30).unwrap();
        let batch = a.batch(&prices);
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut dco = DecyclerOscillator::new(10, 30).unwrap();
        dco.batch(&(1..=50).map(f64::from).collect::<Vec<_>>());
        let before = dco.value();
        assert!(before.is_some());
        assert_eq!(dco.update(f64::NAN), before);
    }

    #[test]
    fn reset_clears_state() {
        let mut dco = DecyclerOscillator::new(10, 30).unwrap();
        dco.batch(&(1..=50).map(f64::from).collect::<Vec<_>>());
        assert!(dco.is_ready());
        dco.reset();
        assert!(!dco.is_ready());
    }
}
