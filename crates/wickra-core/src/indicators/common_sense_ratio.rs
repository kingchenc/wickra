//! Common Sense Ratio (Schwager / Carver) — profit factor multiplied by the tail ratio.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Common Sense Ratio over a trailing window of `period` returns.
///
/// ```text
/// ProfitFactor = Σ gains / Σ |losses|              over the window
/// TailRatio    = P95(returns) / |P5(returns)|      over the window
/// CSR          = ProfitFactor · TailRatio
/// ```
///
/// The Common Sense Ratio fuses two views of a return series into one number. The
/// [profit factor](crate::ProfitFactor) captures the *body* of the distribution —
/// how much you make per unit you lose on the average bar. The
/// [`TailRatio`](crate::TailRatio) captures the *extremes* — whether the largest
/// gains outweigh the largest losses. Multiplying them produces a ratio that is
/// only comfortably above `1.0` when a strategy wins on both fronts: a respectable
/// profit factor can still hide catastrophic left-tail risk, and a fat right tail
/// means little if the body bleeds. Above `1.0` the strategy is sound on a
/// common-sense basis; below `1.0` something — body or tail — is working against it.
///
/// Percentiles use linear interpolation over the sorted window. A window with no
/// losses (zero profit-factor denominator) or no left tail (zero P5) reports `0.0`
/// rather than dividing by zero.
///
/// The first value lands after `period` returns; each `update` re-sorts the window
/// (O(period log period)), which is O(1) in the length of the overall series.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, CommonSenseRatio};
///
/// let mut indicator = CommonSenseRatio::new(20).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = indicator.update((f64::from(i) * 0.3).sin() * 0.02);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct CommonSenseRatio {
    period: usize,
    window: VecDeque<f64>,
}

impl CommonSenseRatio {
    /// Construct a Common Sense Ratio over `period` returns.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPeriod`] if `period < 2` (percentiles need at least
    /// two observations).
    pub fn new(period: usize) -> Result<Self> {
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "common sense ratio needs period >= 2",
            });
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
        })
    }

    /// Configured window of returns.
    pub const fn period(&self) -> usize {
        self.period
    }

    fn compute(&self) -> f64 {
        let mut gains = 0.0;
        let mut losses = 0.0;
        for ret in &self.window {
            gains += ret.max(0.0);
            losses += (-ret).max(0.0);
        }
        if losses <= 0.0 {
            return 0.0;
        }
        let mut sorted: Vec<f64> = self.window.iter().copied().collect();
        sorted.sort_unstable_by(f64::total_cmp);
        let lower_tail = percentile(&sorted, 5.0).abs();
        if lower_tail <= 0.0 {
            return 0.0;
        }
        let profit_factor = gains / losses;
        let tail_ratio = percentile(&sorted, 95.0) / lower_tail;
        profit_factor * tail_ratio
    }
}

/// Linear-interpolation percentile of an ascending, non-empty slice.
fn percentile(sorted: &[f64], pct: f64) -> f64 {
    let last_index = sorted.len() - 1;
    #[allow(clippy::cast_precision_loss)]
    let rank = pct / 100.0 * last_index as f64;
    let floor = rank.floor();
    // `rank` lies in `[0, last_index]`, so its floor is a valid in-bounds index.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lower = floor as usize;
    if lower >= last_index {
        return sorted[last_index];
    }
    let frac = rank - floor;
    sorted[lower] + frac * (sorted[lower + 1] - sorted[lower])
}

impl Indicator for CommonSenseRatio {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, ret: f64) -> Option<f64> {
        if !ret.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(ret);
        if self.window.len() < self.period {
            return None;
        }
        Some(self.compute())
    }

    fn reset(&mut self) {
        self.window.clear();
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "CommonSenseRatio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_period_less_than_two() {
        assert!(matches!(
            CommonSenseRatio::new(1),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let csr = CommonSenseRatio::new(20).unwrap();
        assert_eq!(csr.period(), 20);
        assert_eq!(csr.warmup_period(), 20);
        assert_eq!(csr.name(), "CommonSenseRatio");
        assert!(!csr.is_ready());
    }

    #[test]
    fn reference_value() {
        // window [-0.04, -0.02, 0.0, 0.02, 0.04].
        // gains = 0.06, losses = 0.06 -> profit factor 1.0.
        // P95 = 0.036, |P5| = 0.036 -> tail ratio 1.0. CSR = 1.0.
        let mut csr = CommonSenseRatio::new(5).unwrap();
        let out = csr.batch(&[-0.04, -0.02, 0.0, 0.02, 0.04]);
        assert_relative_eq!(out[4].unwrap(), 1.0, epsilon = 1e-9);
    }

    #[test]
    fn no_losses_is_zero() {
        let mut csr = CommonSenseRatio::new(3).unwrap();
        let last = csr
            .batch(&[0.01, 0.02, 0.03])
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn flat_window_is_zero() {
        // All zeros: no losses denominator -> zero (the gains/losses guard fires).
        let mut csr = CommonSenseRatio::new(4).unwrap();
        let last = csr.batch(&[0.0; 4]).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut csr = CommonSenseRatio::new(3).unwrap();
        assert_eq!(csr.update(0.01), None);
        assert_eq!(csr.update(f64::NAN), None);
        assert_eq!(csr.update(-0.02), None);
        assert!(csr.update(0.03).is_some());
    }

    #[test]
    fn reset_clears_state() {
        let mut csr = CommonSenseRatio::new(3).unwrap();
        csr.batch(&[-0.01, 0.0, 0.02]);
        assert!(csr.is_ready());
        csr.reset();
        assert!(!csr.is_ready());
        assert_eq!(csr.update(0.01), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let rets: Vec<f64> = (0..60)
            .map(|i| (f64::from(i) * 0.25).sin() * 0.02)
            .collect();
        let batch = CommonSenseRatio::new(15).unwrap().batch(&rets);
        let mut streamer = CommonSenseRatio::new(15).unwrap();
        let streamed: Vec<_> = rets.iter().map(|r| streamer.update(*r)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn percentile_at_top_returns_last() {
        // The rank floor reaching the final index returns the largest element.
        assert_relative_eq!(percentile(&[1.0, 2.0, 3.0], 100.0), 3.0, epsilon = 1e-12);
    }

    #[test]
    fn zero_lower_tail_is_zero() {
        // One loss but a 5th percentile of exactly zero: the tail term collapses
        // and the indicator reports 0.0 rather than dividing by zero. With period
        // 21 the 5% rank lands on sorted index 1, which is 0.0 here.
        let mut returns = vec![0.0; 21];
        returns[0] = -0.1;
        let mut csr = CommonSenseRatio::new(21).unwrap();
        let last = csr.batch(&returns).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-12);
    }
}
