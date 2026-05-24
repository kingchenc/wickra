//! Double Bollinger Bands (Kathy Lien).

use crate::error::{Error, Result};
use crate::indicators::bollinger::BollingerBands;
use crate::traits::Indicator;

/// Double Bollinger Bands output: two concentric bands at `k_inner` and
/// `k_outer` standard deviations around a shared SMA middle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoubleBollingerOutput {
    /// Outer upper band: `middle + k_outer · stddev`.
    pub upper_outer: f64,
    /// Inner upper band: `middle + k_inner · stddev`.
    pub upper_inner: f64,
    /// Middle band: SMA over the window.
    pub middle: f64,
    /// Inner lower band: `middle − k_inner · stddev`.
    pub lower_inner: f64,
    /// Outer lower band: `middle − k_outer · stddev`.
    pub lower_outer: f64,
}

/// Double Bollinger Bands: two concentric Bollinger envelopes (Kathy Lien).
///
/// ```text
/// middle      = SMA(period)
/// sigma       = population stddev over the window
/// upper_outer = middle + k_outer · sigma          // wide channel (often 2σ)
/// upper_inner = middle + k_inner · sigma          // narrow channel (often 1σ)
/// lower_inner = middle − k_inner · sigma
/// lower_outer = middle − k_outer · sigma
/// ```
///
/// Lien's trading framework partitions price into three zones:
///
/// - **Sell zone:** close below `lower_inner`.
/// - **Neutral zone:** close between `lower_inner` and `upper_inner`.
/// - **Buy zone:** close above `upper_inner`.
///
/// A close beyond the outer band marks an extended move that traders typically
/// fade or trail. The constructor enforces `k_outer > k_inner` so the outputs
/// remain monotonically ordered.
///
/// # Example
///
/// ```
/// use wickra_core::{DoubleBollinger, Indicator};
///
/// let mut indicator = DoubleBollinger::new(20, 1.0, 2.0).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.3).sin() * 6.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct DoubleBollinger {
    inner: BollingerBands,
    k_inner: f64,
    k_outer: f64,
}

impl DoubleBollinger {
    /// Construct a new Double Bollinger Bands indicator.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0`,
    /// [`Error::NonPositiveMultiplier`] if either `k_inner` or `k_outer` is
    /// non-positive or non-finite, and [`Error::InvalidPeriod`] if
    /// `k_outer <= k_inner` (the outer band must strictly enclose the inner
    /// band so the zone-partitioning interpretation holds).
    pub fn new(period: usize, k_inner: f64, k_outer: f64) -> Result<Self> {
        if !k_inner.is_finite() || k_inner <= 0.0 || !k_outer.is_finite() || k_outer <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        if k_outer <= k_inner {
            return Err(Error::InvalidPeriod {
                message: "double bollinger requires k_outer > k_inner",
            });
        }
        // Build the inner state on the outer multiplier so the upper/lower
        // outputs of `BollingerBands::update` already give us the outer band;
        // the inner band is reconstructed from the same `stddev`.
        Ok(Self {
            inner: BollingerBands::new(period, k_outer)?,
            k_inner,
            k_outer,
        })
    }

    /// Kathy Lien's classic configuration: SMA(20) with `±1σ` and `±2σ` bands.
    pub fn classic() -> Self {
        Self::new(20, 1.0, 2.0).expect("classic Double Bollinger parameters are valid")
    }

    /// Configured `(period, k_inner, k_outer)`.
    pub const fn parameters(&self) -> (usize, f64, f64) {
        (self.inner.period(), self.k_inner, self.k_outer)
    }
}

impl Indicator for DoubleBollinger {
    type Input = f64;
    type Output = DoubleBollingerOutput;

    fn update(&mut self, value: f64) -> Option<DoubleBollingerOutput> {
        let o = self.inner.update(value)?;
        Some(DoubleBollingerOutput {
            upper_outer: o.upper,
            upper_inner: o.middle + self.k_inner * o.stddev,
            middle: o.middle,
            lower_inner: o.middle - self.k_inner * o.stddev,
            lower_outer: o.lower,
        })
    }

    fn reset(&mut self) {
        self.inner.reset();
    }

    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

    fn name(&self) -> &'static str {
        "DoubleBollinger"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(
            DoubleBollinger::new(0, 1.0, 2.0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn rejects_non_positive_multiplier() {
        assert!(matches!(
            DoubleBollinger::new(20, 0.0, 2.0),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            DoubleBollinger::new(20, 1.0, -2.0),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            DoubleBollinger::new(20, f64::NAN, 2.0),
            Err(Error::NonPositiveMultiplier)
        ));
    }

    #[test]
    fn rejects_outer_not_greater_than_inner() {
        assert!(matches!(
            DoubleBollinger::new(20, 2.0, 1.0),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            DoubleBollinger::new(20, 2.0, 2.0),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let db = DoubleBollinger::classic();
        let (p, ki, ko) = db.parameters();
        assert_eq!(p, 20);
        assert_relative_eq!(ki, 1.0, epsilon = 1e-12);
        assert_relative_eq!(ko, 2.0, epsilon = 1e-12);
        assert_eq!(db.warmup_period(), 20);
        assert_eq!(db.name(), "DoubleBollinger");
    }

    #[test]
    fn constant_series_collapses_all_bands() {
        let mut db = DoubleBollinger::new(10, 1.0, 2.0).unwrap();
        let last = db
            .batch(&[5.0_f64; 20])
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last.middle, 5.0, epsilon = 1e-12);
        assert_relative_eq!(last.upper_outer, 5.0, epsilon = 1e-12);
        assert_relative_eq!(last.upper_inner, 5.0, epsilon = 1e-12);
        assert_relative_eq!(last.lower_inner, 5.0, epsilon = 1e-12);
        assert_relative_eq!(last.lower_outer, 5.0, epsilon = 1e-12);
    }

    #[test]
    fn bands_strictly_ordered_with_dispersion() {
        let prices: Vec<f64> = (0..80)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 6.0)
            .collect();
        let mut db = DoubleBollinger::classic();
        for o in db.batch(&prices).into_iter().flatten() {
            assert!(o.upper_outer >= o.upper_inner);
            assert!(o.upper_inner >= o.middle);
            assert!(o.middle >= o.lower_inner);
            assert!(o.lower_inner >= o.lower_outer);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..50).map(|i| f64::from(i) * 0.7).collect();
        let mut a = DoubleBollinger::new(10, 1.0, 2.0).unwrap();
        let mut b = DoubleBollinger::new(10, 1.0, 2.0).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut db = DoubleBollinger::new(5, 1.0, 2.0).unwrap();
        db.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(db.is_ready());
        db.reset();
        assert!(!db.is_ready());
        assert_eq!(db.update(1.0), None);
    }

    /// The inner band must agree with running a separate `BollingerBands` at
    /// the inner multiplier.
    #[test]
    fn inner_band_matches_separate_bollinger() {
        let prices: Vec<f64> = (0..80)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 6.0)
            .collect();
        let mut db = DoubleBollinger::new(20, 1.0, 2.0).unwrap();
        let mut bb_inner = BollingerBands::new(20, 1.0).unwrap();
        let mut bb_outer = BollingerBands::new(20, 2.0).unwrap();
        for p in &prices {
            let d = db.update(*p);
            let i = bb_inner.update(*p);
            let o = bb_outer.update(*p);
            if let (Some(d), Some(i), Some(o)) = (d, i, o) {
                assert_relative_eq!(d.middle, i.middle, epsilon = 1e-9);
                assert_relative_eq!(d.upper_inner, i.upper, epsilon = 1e-9);
                assert_relative_eq!(d.lower_inner, i.lower, epsilon = 1e-9);
                assert_relative_eq!(d.upper_outer, o.upper, epsilon = 1e-9);
                assert_relative_eq!(d.lower_outer, o.lower, epsilon = 1e-9);
            }
        }
    }
}
