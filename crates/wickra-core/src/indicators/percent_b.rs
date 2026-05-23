//! Bollinger %b.

use crate::error::Result;
use crate::traits::Indicator;

use super::BollingerBands;

/// Bollinger %b — where price sits within the Bollinger Bands.
///
/// ```text
/// %b = (price − lower) / (upper − lower)
/// ```
///
/// `%b = 1` means price is exactly on the upper band, `%b = 0` on the lower
/// band, `%b = 0.5` on the middle band. The value is **not** clamped: price
/// breaking above the upper band gives `%b > 1`, breaking below the lower band
/// gives `%b < 0`. That makes %b a clean, scale-free way to compare a price's
/// band position across instruments and to spot band overshoots.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, PercentB};
///
/// let mut indicator = PercentB::new(20, 2.0).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.3).sin() * 6.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct PercentB {
    bands: BollingerBands,
    last: Option<f64>,
}

impl PercentB {
    /// Construct a new %b indicator.
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

impl Indicator for PercentB {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        let o = self.bands.update(input)?;
        let width = o.upper - o.lower;
        let percent_b = if width == 0.0 {
            // Bands collapsed onto the middle: price is exactly mid-band.
            0.5
        } else {
            (input - o.lower) / width
        };
        self.last = Some(percent_b);
        Some(percent_b)
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
        "PercentB"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_invalid_parameters() {
        assert!(PercentB::new(0, 2.0).is_err());
        assert!(PercentB::new(20, 0.0).is_err());
        assert!(PercentB::new(20, -1.0).is_err());
    }

    /// Cover the public const accessors `period`, `multiplier`, `value`
    /// and the Indicator-impl `warmup_period` + `name` methods. Existing
    /// tests only exercise the numeric output of `update` / `batch` /
    /// `reset` / `is_ready`, so the five getter bodies (lines 53-65,
    /// 90-92, 98-100) were dead.
    #[test]
    fn accessors_and_metadata() {
        let mut pb = PercentB::new(20, 2.0).unwrap();
        assert_eq!(pb.period(), 20);
        assert_relative_eq!(pb.multiplier(), 2.0, epsilon = 1e-12);
        assert_eq!(pb.value(), None);
        assert_eq!(pb.warmup_period(), 20);
        assert_eq!(pb.name(), "PercentB");
        for i in 1..=20 {
            pb.update(f64::from(i));
        }
        assert!(pb.value().is_some());
    }

    #[test]
    fn constant_series_yields_midpoint() {
        // Flat prices: bands collapse, price is exactly mid-band -> 0.5.
        let mut pb = PercentB::new(5, 2.0).unwrap();
        let out = pb.batch(&[100.0; 20]);
        for v in out.iter().skip(4).flatten() {
            assert_relative_eq!(*v, 0.5, epsilon = 1e-12);
        }
    }

    #[test]
    fn matches_bands_definition() {
        // %b must equal (price - lower) / (upper - lower) from BollingerBands.
        let prices: Vec<f64> = (1..=60)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 8.0)
            .collect();
        let pb_out = PercentB::new(20, 2.0).unwrap().batch(&prices);
        let bands_out = BollingerBands::new(20, 2.0).unwrap().batch(&prices);
        for (i, (p, b)) in pb_out.iter().zip(bands_out.iter()).enumerate() {
            // Same warmup — emission shape must agree at every index.
            assert_eq!(p.is_some(), b.is_some(), "warmup mismatch at index {i}");
            if let (Some(pv), Some(bv)) = (p, b) {
                let want = (prices[i] - bv.lower) / (bv.upper - bv.lower);
                assert_relative_eq!(*pv, want, epsilon = 1e-12);
            }
        }
    }

    /// Deterministic price-at-middle assertion. With period=3, multiplier=2,
    /// inputs `[1.0, 5.0, 3.0]` give SMA = (1+5+3)/3 = 3.0 at index 2, which
    /// equals the third price exactly. The stddev is √(8/3) ≈ 1.633, so the
    /// bands have non-zero width (the width==0 fallback at line 77 is NOT
    /// taken) and the divide path at line 79 runs. Because price sits on
    /// the centre line of symmetric bands, %b lands on exactly 0.5.
    ///
    /// The previous oscillation-based variant of this test never landed
    /// `prices[i]` within 1e-9 of the rolling SMA, so its inner
    /// `assert_relative_eq!` line was never executed.
    #[test]
    fn price_at_middle_is_half() {
        let mut pb = PercentB::new(3, 2.0).unwrap();
        let out = pb.batch(&[1.0, 5.0, 3.0]);
        assert_eq!(out[0], None);
        assert_eq!(out[1], None);
        let v = out[2].expect("warmed up at index 2");
        assert_relative_eq!(v, 0.5, epsilon = 1e-12);
    }

    #[test]
    fn reset_clears_state() {
        let mut pb = PercentB::new(5, 2.0).unwrap();
        pb.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        assert!(pb.is_ready());
        pb.reset();
        assert!(!pb.is_ready());
        assert_eq!(pb.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=80)
            .map(|i| 100.0 + (f64::from(i) * 0.3).cos() * 7.0)
            .collect();
        let batch = PercentB::new(20, 2.0).unwrap().batch(&prices);
        let mut b = PercentB::new(20, 2.0).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
