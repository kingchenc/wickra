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

    /// Cover the public const accessors `period`, `multiplier`, `value` and
    /// the Indicator-impl `warmup_period` + `name` methods. None of the
    /// pre-existing tests inspected the metadata surface — they only fed
    /// numeric updates and asserted on the bandwidth values, leaving the
    /// five getter bodies (lines 54-66, 90-92, 98-100) untouched.
    #[test]
    fn accessors_and_metadata() {
        let mut bbw = BollingerBandwidth::new(20, 2.0).unwrap();
        assert_eq!(bbw.period(), 20);
        assert_relative_eq!(bbw.multiplier(), 2.0, epsilon = 1e-12);
        // value() before warmup must be the literal None branch of self.last.
        assert_eq!(bbw.value(), None);
        assert_eq!(bbw.warmup_period(), 20);
        assert_eq!(bbw.name(), "BollingerBandwidth");
        // Drive past warmup so value() exercises the Some branch as well.
        for i in 1..=20 {
            bbw.update(f64::from(i));
        }
        assert!(bbw.value().is_some());
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

    /// Cover the defensive `o.middle == 0.0` branch in `update` (line 77).
    /// All other tests use price levels ≈100, so the rolling SMA is always
    /// strictly positive and the zero-middle fallback is unreachable. Feed
    /// a symmetric series whose 5-bar mean is exactly 0 to force the branch
    /// and assert the indicator yields exactly 0.0 (rather than inf/nan).
    #[test]
    fn zero_middle_band_yields_zero_bandwidth() {
        let mut bbw = BollingerBandwidth::new(5, 2.0).unwrap();
        // sum(-2, -1, 0, 1, 2) = 0 exactly in IEEE-754, so the SMA middle
        // lands on exactly 0.0 at the fifth input. Stddev > 0, so absent
        // the guard the next line would divide by zero.
        let out = bbw.batch(&[-2.0, -1.0, 0.0, 1.0, 2.0]);
        assert_eq!(out[..4], [None, None, None, None]);
        let v = out[4].expect("warmed up");
        assert_eq!(v, 0.0, "zero-middle fallback must emit exactly 0.0");
    }

    #[test]
    fn matches_bands_definition() {
        // Bandwidth must equal (upper - lower) / middle from BollingerBands.
        let prices: Vec<f64> = (1..=60)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 8.0)
            .collect();
        let bbw_out = BollingerBandwidth::new(20, 2.0).unwrap().batch(&prices);
        let bands_out = BollingerBands::new(20, 2.0).unwrap().batch(&prices);
        for (i, (w, b)) in bbw_out.iter().zip(bands_out.iter()).enumerate() {
            // Same warmup period on both — emission shape must agree at every index.
            assert_eq!(w.is_some(), b.is_some(), "warmup mismatch at index {i}");
            if let (Some(wv), Some(bv)) = (w, b) {
                assert_relative_eq!(*wv, (bv.upper - bv.lower) / bv.middle, epsilon = 1e-12);
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
