//! Moving Average Envelope.

use crate::error::{Error, Result};
use crate::indicators::sma::Sma;
use crate::traits::Indicator;

/// Moving Average Envelope output: SMA middle line wrapped by a fixed-percent
/// envelope on either side.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaEnvelopeOutput {
    /// Upper envelope: `middle · (1 + percent)`.
    pub upper: f64,
    /// Middle band: SMA over the window.
    pub middle: f64,
    /// Lower envelope: `middle · (1 − percent)`.
    pub lower: f64,
}

/// Moving Average Envelope: an SMA centerline with constant-percent bands on
/// each side.
///
/// ```text
/// middle = SMA(period)
/// upper  = middle · (1 + percent)
/// lower  = middle · (1 − percent)
/// ```
///
/// The envelope is a fixed multiplicative offset around the moving average,
/// so the band width scales with price rather than with realised volatility
/// (contrast Bollinger Bands, whose width is `2·k·σ`, or Keltner Channels,
/// whose width is `2·k·ATR`). It is the oldest band-style overlay still in
/// regular use; chart vendors typically default to `period = 20`,
/// `percent = 0.025` (2.5 %).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, MaEnvelope};
///
/// let mut indicator = MaEnvelope::new(20, 0.025).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct MaEnvelope {
    sma: Sma,
    percent: f64,
}

impl MaEnvelope {
    /// Construct a new Moving Average Envelope.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0` and
    /// [`Error::NonPositiveMultiplier`] if `percent` is not strictly positive
    /// and finite.
    pub fn new(period: usize, percent: f64) -> Result<Self> {
        if !percent.is_finite() || percent <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            sma: Sma::new(period)?,
            percent,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.sma.period()
    }

    /// Configured envelope percent (e.g. `0.025` for ±2.5 %).
    pub const fn percent(&self) -> f64 {
        self.percent
    }
}

impl Indicator for MaEnvelope {
    type Input = f64;
    type Output = MaEnvelopeOutput;

    fn update(&mut self, input: f64) -> Option<MaEnvelopeOutput> {
        let middle = self.sma.update(input)?;
        Some(MaEnvelopeOutput {
            upper: middle * (1.0 + self.percent),
            middle,
            lower: middle * (1.0 - self.percent),
        })
    }

    fn reset(&mut self) {
        self.sma.reset();
    }

    fn warmup_period(&self) -> usize {
        self.sma.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.sma.is_ready()
    }

    fn name(&self) -> &'static str {
        "MaEnvelope"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(MaEnvelope::new(0, 0.025), Err(Error::PeriodZero)));
    }

    #[test]
    fn rejects_non_positive_percent() {
        assert!(matches!(
            MaEnvelope::new(20, 0.0),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            MaEnvelope::new(20, -0.1),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            MaEnvelope::new(20, f64::NAN),
            Err(Error::NonPositiveMultiplier)
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let env = MaEnvelope::new(20, 0.025).unwrap();
        assert_eq!(env.period(), 20);
        assert_relative_eq!(env.percent(), 0.025, epsilon = 1e-12);
        assert_eq!(env.warmup_period(), 20);
        assert_eq!(env.name(), "MaEnvelope");
        assert!(!env.is_ready());
    }

    #[test]
    fn constant_series_yields_flat_envelope() {
        let mut env = MaEnvelope::new(5, 0.01).unwrap();
        let last = env
            .batch(&[100.0_f64; 20])
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last.middle, 100.0, epsilon = 1e-12);
        assert_relative_eq!(last.upper, 101.0, epsilon = 1e-12);
        assert_relative_eq!(last.lower, 99.0, epsilon = 1e-12);
    }

    #[test]
    fn warmup_returns_none() {
        let mut env = MaEnvelope::new(5, 0.05).unwrap();
        for v in [1.0, 2.0, 3.0, 4.0] {
            assert!(env.update(v).is_none());
        }
        assert!(env.update(5.0).is_some());
    }

    #[test]
    fn upper_above_middle_above_lower() {
        let prices: Vec<f64> = (1..=80)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 5.0)
            .collect();
        let mut env = MaEnvelope::new(20, 0.025).unwrap();
        for o in env.batch(&prices).into_iter().flatten() {
            assert!(o.upper >= o.middle);
            assert!(o.middle >= o.lower);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=50).map(|i| f64::from(i) * 0.7 + 100.0).collect();
        let mut a = MaEnvelope::new(10, 0.03).unwrap();
        let mut b = MaEnvelope::new(10, 0.03).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut env = MaEnvelope::new(5, 0.02).unwrap();
        env.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(env.is_ready());
        env.reset();
        assert!(!env.is_ready());
        assert_eq!(env.update(1.0), None);
    }

    /// Reference value: SMA over [10, 20, 30] is 20; with percent = 0.10 the
    /// upper band is 22 and the lower band is 18.
    #[test]
    fn reference_values() {
        let mut env = MaEnvelope::new(3, 0.10).unwrap();
        let out = env.batch(&[10.0, 20.0, 30.0]);
        assert!(out[0].is_none() && out[1].is_none());
        let v = out[2].unwrap();
        assert_relative_eq!(v.middle, 20.0, epsilon = 1e-12);
        assert_relative_eq!(v.upper, 22.0, epsilon = 1e-12);
        assert_relative_eq!(v.lower, 18.0, epsilon = 1e-12);
    }
}
