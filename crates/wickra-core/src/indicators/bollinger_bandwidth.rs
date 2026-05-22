//! Bollinger Bandwidth.

use crate::error::Result;
use crate::traits::Indicator;

use super::BollingerBands;

/// Bollinger Bandwidth — the width of the Bollinger Bands relative to the
/// middle band.
///
/// ```text
/// Bandwidth = (upper − lower) / middle
/// ```
///
/// Because the bands are `middle ± multiplier · stddev`, the bandwidth is
/// `2 · multiplier · stddev / middle` — a normalised volatility reading. Its
/// value is the basis of two classic patterns: the **squeeze** (bandwidth at a
/// multi-month low, signalling a coiled, low-volatility market about to
/// expand) and the **bulge** (bandwidth at an extreme high).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, BollingerBandwidth};
///
/// let mut indicator = BollingerBandwidth::new(20, 2.0).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.3).sin() * 6.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct BollingerBandwidth {
    bands: BollingerBands,
    last: Option<f64>,
}

impl BollingerBandwidth {
    /// Construct a new Bollinger Bandwidth indicator.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PeriodZero`] for `period == 0` and
    /// [`crate::Error::NonPositiveMultiplier`] for `multiplier <= 0`.
    pub fn new(period: usize, multiplier: f64) -> Result<Self> {
        Ok(Self {
            bands: BollingerBands::new(period, multiplier)?,
            last: None,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.bands.period()
    }

    /// Configured multiplier.
    pub const fn multiplier(&self) -> f64 {
        self.bands.multiplier()
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for BollingerBandwidth {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        let o = self.bands.update(input)?;
        let bandwidth = if o.middle == 0.0 {
            // Undefined against a zero middle band.
            0.0
        } else {
            (o.upper - o.lower) / o.middle
        };
        self.last = Some(bandwidth);
        Some(bandwidth)
    }

    fn reset(&mut self) {
        self.bands.reset();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.bands.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "BollingerBandwidth"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_invalid_parameters() {
        assert!(BollingerBandwidth::new(0, 2.0).is_err());
        assert!(BollingerBandwidth::new(20, 0.0).is_err());
        assert!(BollingerBandwidth::new(20, -1.0).is_err());
    }

    #[test]
    fn constant_series_yields_zero() {
        // Flat prices: the bands collapse onto the middle, so width is 0.
        let mut bbw = BollingerBandwidth::new(5, 2.0).unwrap();
        let out = bbw.batch(&[100.0; 20]);
        for v in out.iter().skip(4).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn matches_bands_definition() {
        // Bandwidth must equal (upper - lower) / middle from BollingerBands.
        let prices: Vec<f64> = (1..=60)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 8.0)
            .collect();
        let bbw_out = BollingerBandwidth::new(20, 2.0).unwrap().batch(&prices);
        let bands_out = BollingerBands::new(20, 2.0).unwrap().batch(&prices);
        for (w, b) in bbw_out.iter().zip(bands_out.iter()) {
            match (w, b) {
                (Some(wv), Some(bv)) => {
                    assert_relative_eq!(*wv, (bv.upper - bv.lower) / bv.middle, epsilon = 1e-12);
                }
                (None, None) => {}
                _ => panic!("warmup mismatch"),
            }
        }
    }

    #[test]
    fn output_is_non_negative() {
        let mut bbw = BollingerBandwidth::new(20, 2.0).unwrap();
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 12.0)
            .collect();
        for v in bbw.batch(&prices).into_iter().flatten() {
            assert!(v >= 0.0, "bandwidth must be non-negative, got {v}");
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut bbw = BollingerBandwidth::new(5, 2.0).unwrap();
        bbw.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        assert!(bbw.is_ready());
        bbw.reset();
        assert!(!bbw.is_ready());
        assert_eq!(bbw.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=80)
            .map(|i| 100.0 + (f64::from(i) * 0.3).cos() * 7.0)
            .collect();
        let batch = BollingerBandwidth::new(20, 2.0).unwrap().batch(&prices);
        let mut b = BollingerBandwidth::new(20, 2.0).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
