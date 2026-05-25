//! VWAP Standard-Deviation Bands.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// `VWAP` `StdDev` Bands output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VwapStdDevBandsOutput {
    /// Upper band: `vwap + multiplier · sigma`.
    pub upper: f64,
    /// Middle band: cumulative VWAP of typical price.
    pub middle: f64,
    /// Lower band: `vwap − multiplier · sigma`.
    pub lower: f64,
    /// Volume-weighted standard deviation of typical price about VWAP.
    pub stddev: f64,
}

/// VWAP with volume-weighted standard-deviation envelopes.
///
/// ```text
/// tp_i        = typical_price(candle_i)         // (high + low + close) / 3
/// sum_v       = Σ volume_i
/// sum_pv      = Σ tp_i · volume_i
/// sum_p2v     = Σ tp_i² · volume_i
/// vwap        = sum_pv / sum_v
/// variance    = sum_p2v / sum_v − vwap²         // volume-weighted population variance
/// sigma       = sqrt(max(variance, 0))
/// upper/lower = vwap ± multiplier · sigma
/// ```
///
/// The cumulative running sums make every update O(1) with no per-bar replay,
/// matching the streaming contract of [`Vwap`](crate::Vwap). VWAP and its
/// stddev bands are an intraday-session tool: call [`Indicator::reset`] at
/// the start of each session boundary so the accumulators do not span the gap.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, VwapStdDevBands};
///
/// let mut indicator = VwapStdDevBands::new(2.0).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct VwapStdDevBands {
    multiplier: f64,
    sum_pv: f64,
    sum_p2v: f64,
    sum_v: f64,
    has_emitted: bool,
}

impl VwapStdDevBands {
    /// # Errors
    /// Returns [`Error::NonPositiveMultiplier`] if `multiplier` is not strictly
    /// positive and finite.
    pub fn new(multiplier: f64) -> Result<Self> {
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            multiplier,
            sum_pv: 0.0,
            sum_p2v: 0.0,
            sum_v: 0.0,
            has_emitted: false,
        })
    }

    /// Configured multiplier.
    pub const fn multiplier(&self) -> f64 {
        self.multiplier
    }
}

impl Indicator for VwapStdDevBands {
    type Input = Candle;
    type Output = VwapStdDevBandsOutput;

    fn update(&mut self, candle: Candle) -> Option<VwapStdDevBandsOutput> {
        let tp = candle.typical_price();
        self.sum_pv += tp * candle.volume;
        self.sum_p2v += tp * tp * candle.volume;
        self.sum_v += candle.volume;
        if self.sum_v == 0.0 {
            return None;
        }
        self.has_emitted = true;
        let vwap = self.sum_pv / self.sum_v;
        // Volume-weighted population variance; clamp tiny negative cancellation
        // noise back to zero on near-constant inputs.
        let var = (self.sum_p2v / self.sum_v - vwap * vwap).max(0.0);
        let sigma = var.sqrt();
        Some(VwapStdDevBandsOutput {
            upper: vwap + self.multiplier * sigma,
            middle: vwap,
            lower: vwap - self.multiplier * sigma,
            stddev: sigma,
        })
    }

    fn reset(&mut self) {
        self.sum_pv = 0.0;
        self.sum_p2v = 0.0;
        self.sum_v = 0.0;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "VwapStdDevBands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(h: f64, l: f64, cl: f64, v: f64) -> Candle {
        Candle::new(cl, h, l, cl, v, 0).unwrap()
    }

    #[test]
    fn rejects_non_positive_multiplier() {
        assert!(matches!(
            VwapStdDevBands::new(0.0),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            VwapStdDevBands::new(-1.0),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            VwapStdDevBands::new(f64::NAN),
            Err(Error::NonPositiveMultiplier)
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let v = VwapStdDevBands::new(2.0).unwrap();
        assert_relative_eq!(v.multiplier(), 2.0, epsilon = 1e-12);
        assert_eq!(v.warmup_period(), 1);
        assert_eq!(v.name(), "VwapStdDevBands");
    }

    #[test]
    fn zero_volume_returns_none() {
        let mut v = VwapStdDevBands::new(2.0).unwrap();
        assert!(v.update(c(10.0, 10.0, 10.0, 0.0)).is_none());
    }

    #[test]
    fn constant_price_collapses_bands() {
        let candles: Vec<Candle> = (0..10).map(|_| c(10.0, 10.0, 10.0, 5.0)).collect();
        let mut v = VwapStdDevBands::new(2.0).unwrap();
        let last = v.batch(&candles).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last.middle, 10.0, epsilon = 1e-9);
        assert_relative_eq!(last.stddev, 0.0, epsilon = 1e-9);
        assert_relative_eq!(last.upper, 10.0, epsilon = 1e-9);
        assert_relative_eq!(last.lower, 10.0, epsilon = 1e-9);
    }

    #[test]
    fn upper_above_middle_above_lower() {
        let candles: Vec<Candle> = (0..50)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.2).sin() * 5.0;
                c(m + 1.0, m - 1.0, m, 1.0 + f64::from(i % 5))
            })
            .collect();
        let mut v = VwapStdDevBands::new(2.0).unwrap();
        for o in v.batch(&candles).into_iter().flatten() {
            assert!(o.upper >= o.middle);
            assert!(o.middle >= o.lower);
            assert!(o.stddev >= 0.0);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                c(
                    f64::from(i) + 2.0,
                    f64::from(i),
                    f64::from(i) + 1.0,
                    1.0 + f64::from(i % 4),
                )
            })
            .collect();
        let mut a = VwapStdDevBands::new(2.0).unwrap();
        let mut b = VwapStdDevBands::new(2.0).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..10)
            .map(|i| c(f64::from(i) + 1.0, f64::from(i) - 1.0, f64::from(i), 1.0))
            .collect();
        let mut v = VwapStdDevBands::new(2.0).unwrap();
        v.batch(&candles);
        assert!(v.is_ready());
        v.reset();
        assert!(!v.is_ready());
        // After reset a zero-volume bar still returns `None` (volume is
        // required to define the volume-weighted average).
        assert_eq!(v.update(c(10.0, 10.0, 10.0, 0.0)), None);
    }

    /// Reference: two equal-volume bars at typical prices `tp = 8` and `tp = 12`.
    /// VWAP = (8 + 12) / 2 = 10. Volume-weighted population variance =
    /// (64 + 144) / 2 − 100 = 4. Sigma = 2. With multiplier 1.5: upper = 13,
    /// lower = 7.
    #[test]
    fn reference_values() {
        // typical_price = (high + low + close) / 3. Choose bars where this is
        // exactly 8 and 12. Bar A: high=8, low=8, close=8 → tp=8.
        // Bar B: high=12, low=12, close=12 → tp=12.
        let candles = [c(8.0, 8.0, 8.0, 1.0), c(12.0, 12.0, 12.0, 1.0)];
        let mut v = VwapStdDevBands::new(1.5).unwrap();
        let _ = v.update(candles[0]);
        let out = v.update(candles[1]).unwrap();
        assert_relative_eq!(out.middle, 10.0, epsilon = 1e-9);
        assert_relative_eq!(out.stddev, 2.0, epsilon = 1e-9);
        assert_relative_eq!(out.upper, 13.0, epsilon = 1e-9);
        assert_relative_eq!(out.lower, 7.0, epsilon = 1e-9);
    }
}
