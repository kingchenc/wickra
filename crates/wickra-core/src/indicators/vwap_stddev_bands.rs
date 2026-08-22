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
/// sum_dv      = Σ (tp_i − c) · volume_i         // c is a reference price
/// sum_d2v     = Σ (tp_i − c)² · volume_i
/// vwap        = c + sum_dv / sum_v
/// variance    = sum_d2v / sum_v − (sum_dv / sum_v)²   // volume-weighted, population
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
    /// Reference price the weighted moments are held relative to, seeded from
    /// the first bar of the session. The variance is invariant under this
    /// shift, and it keeps both moments on the scale of the deviation from
    /// that price rather than of the price itself.
    reference: f64,
    /// Whether `reference` has been seeded.
    seeded: bool,
    sum_dv: f64,
    sum_d2v: f64,
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
            reference: 0.0,
            seeded: false,
            sum_dv: 0.0,
            sum_d2v: 0.0,
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

    #[inline]
    fn update(&mut self, candle: Candle) -> Option<VwapStdDevBandsOutput> {
        let tp = candle.typical_price();
        if !self.seeded {
            self.reference = tp;
            self.seeded = true;
        }
        let d = tp - self.reference;
        self.sum_dv += d * candle.volume;
        self.sum_d2v += d * d * candle.volume;
        self.sum_v += candle.volume;
        if self.sum_v == 0.0 {
            return None;
        }
        self.has_emitted = true;
        // The weighted mean deviation carries the whole of the VWAP except the
        // reference price, which comes back only for the absolute band levels.
        let mean_d = self.sum_dv / self.sum_v;
        let vwap = self.reference + mean_d;
        // Volume-weighted population variance; clamp tiny negative cancellation
        // noise back to zero on near-constant inputs.
        let var = (self.sum_d2v / self.sum_v - mean_d * mean_d).max(0.0);
        let sigma = var.sqrt();
        Some(VwapStdDevBandsOutput {
            upper: vwap + self.multiplier * sigma,
            middle: vwap,
            lower: vwap - self.multiplier * sigma,
            stddev: sigma,
        })
    }

    fn reset(&mut self) {
        self.reference = 0.0;
        self.seeded = false;
        self.sum_dv = 0.0;
        self.sum_d2v = 0.0;
        self.sum_v = 0.0;
        self.has_emitted = false;
    }

    #[inline]
    fn warmup_period(&self) -> usize {
        1
    }

    #[inline]
    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    #[inline]
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

    /// The volume-weighted variance was `Σtp²v/Σv − vwap²` on raw typical
    /// prices, and this indicator accumulates over a whole session rather than
    /// a window, so there was nothing to bound it either. At a price level of
    /// 1e8 the deviation came out 32 times too large; against a two-pass
    /// weighted reference it now measures 7.6e-14.
    #[test]
    fn deviation_at_a_high_price_level_matches_a_two_pass_reference() {
        let closes: Vec<f64> = (0..400)
            .map(|i| {
                let t = f64::from(i);
                1e8 + ((t * 0.11).sin() + 0.4 * (t * 0.37).cos())
            })
            .collect();

        let mut ind = VwapStdDevBands::new(2.0).unwrap();
        let (mut prices, mut volumes): (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());
        let mut compared = 0_usize;
        for (i, &c) in closes.iter().enumerate() {
            let volume = 10.0 + (i % 7) as f64;
            // Converted rather than cast: the workspace lint set rejects
            // `usize as i64` as a possible wrap.
            let timestamp = i64::try_from(i).unwrap();
            let candle = Candle::new_unchecked(c, c + 0.5, c - 0.5, c, volume, timestamp);
            let out = ind.update(candle);
            prices.push(candle.typical_price());
            volumes.push(volume);
            let Some(out) = out else { continue };

            // Two passes over the session: the weighted mean first, then the
            // weighted spread about it.
            let total: f64 = volumes.iter().sum();
            let vwap: f64 = prices.iter().zip(&volumes).map(|(p, v)| p * v).sum::<f64>() / total;
            let var: f64 = prices
                .iter()
                .zip(&volumes)
                .map(|(p, v)| v * (p - vwap) * (p - vwap))
                .sum::<f64>()
                / total;
            compared += 1;
            assert_relative_eq!(out.middle, vwap, max_relative = 1e-14);
            assert_relative_eq!(out.stddev, var.sqrt(), max_relative = 1e-9);
        }
        assert_eq!(compared, closes.len());
    }
}
