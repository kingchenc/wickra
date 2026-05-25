//! Ehlers Following Adaptive Moving Average (FAMA).

use crate::error::Result;
use crate::indicators::mama::Mama;
use crate::traits::Indicator;

/// Scalar wrapper that exposes only the FAMA line from a [`Mama`] indicator.
///
/// FAMA (Following Adaptive Moving Average) is MAMA's lagging companion in
/// Ehlers' MESA construction. It uses half MAMA's adaptive alpha, so it
/// reacts later than MAMA — MAMA crossing above FAMA marks a trend
/// confirmation, MAMA below FAMA a reversal. See [`Mama`] for the joint
/// `(mama, fama)` output; this wrapper exposes the slow line as a plain
/// scalar indicator so it can be chained directly.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Fama};
///
/// let mut fama = Fama::new(0.5, 0.05).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = fama.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Fama {
    inner: Mama,
    last_value: Option<f64>,
}

impl Fama {
    /// Construct with the same `(fast_limit, slow_limit)` semantics as [`Mama`].
    ///
    /// # Errors
    ///
    /// Forwards [`Mama::new`]'s validation errors.
    pub fn new(fast_limit: f64, slow_limit: f64) -> Result<Self> {
        Ok(Self {
            inner: Mama::new(fast_limit, slow_limit)?,
            last_value: None,
        })
    }

    /// Default `(0.5, 0.05)` parameters.
    pub fn classic() -> Self {
        Self {
            inner: Mama::classic(),
            last_value: None,
        }
    }

    /// Configured `(fast_limit, slow_limit)`.
    pub const fn limits(&self) -> (f64, f64) {
        self.inner.limits()
    }

    /// Current FAMA value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for Fama {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        let v = self.inner.update(input)?.fama;
        self.last_value = Some(v);
        Some(v)
    }

    fn reset(&mut self) {
        self.inner.reset();
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "FAMA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::traits::BatchExt;

    #[test]
    fn rejects_invalid_limits() {
        assert!(matches!(
            Fama::new(0.0, 0.05),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            Fama::new(0.05, 0.5),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn new_with_valid_limits_constructs_via_mama() {
        // `classic()` bypasses `new` by going through `Mama::classic`; this
        // test exercises the happy-path `Ok(Self { inner: Mama::new(..)? })`
        // arm so the `?` doesn't only collapse to the error path.
        let mut fama = Fama::new(0.5, 0.05).expect("valid Mama limits");
        assert_eq!(fama.limits(), (0.5, 0.05));
        for i in 0..60 {
            fama.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
        }
        assert!(fama.value().is_some());
    }

    #[test]
    fn accessors_and_metadata() {
        let mut fama = Fama::classic();
        assert_eq!(fama.limits(), (0.5, 0.05));
        assert_eq!(fama.warmup_period(), 33);
        assert_eq!(fama.name(), "FAMA");
        assert!(!fama.is_ready());
        for i in 0..60 {
            fama.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
        }
        assert!(fama.is_ready());
        assert!(fama.value().is_some());
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).cos() * 5.0)
            .collect();
        let mut a = Fama::classic();
        let mut b = Fama::classic();
        let batch = a.batch(&prices);
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut fama = Fama::classic();
        let prices: Vec<f64> = (0..100)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        fama.batch(&prices);
        let before = fama.value();
        assert!(before.is_some());
        assert_eq!(fama.update(f64::NAN), before);
    }

    #[test]
    fn reset_clears_state() {
        let mut fama = Fama::classic();
        let prices: Vec<f64> = (0..100)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        fama.batch(&prices);
        assert!(fama.is_ready());
        fama.reset();
        assert!(!fama.is_ready());
    }
}
