//! Burke Ratio — mean return over the square root of the summed squared drawdowns.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Burke Ratio over a trailing window of `period` returns.
///
/// ```text
/// equity_t = Π_{i<=t} (1 + return_i)          (compounded curve)
/// peak_t   = max_{s<=t} equity_s
/// dd_t     = (peak_t − equity_t) / peak_t      (fractional drawdown, >= 0)
/// Burke    = mean(returns) / sqrt( Σ dd_t² )
/// ```
///
/// The Burke Ratio divides the average per-period return by the **Euclidean norm of
/// the drawdowns** — the square root of the *sum* of squared drawdowns. Squaring
/// penalises deep drawdowns far more than shallow ones, and summing (rather than
/// averaging) means the denominator grows with both the depth and the *number* of
/// drawdowns. This makes Burke the most outlier-sensitive of Wickra's three
/// drawdown ratios: where the [`SterlingRatio`](crate::SterlingRatio) averages raw
/// drawdowns and shrugs off a single crater, Burke makes that crater dominate.
/// The [`MartinRatio`](crate::MartinRatio) sits between them with a root-*mean*
/// square of percentage drawdowns. A window that never draws down has a zero
/// denominator and the indicator reports `0.0`.
///
/// The first value lands after `period` returns; each `update` rebuilds the equity
/// curve over the window (O(period)), which is O(1) in the length of the overall
/// series.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, BurkeRatio};
///
/// let mut indicator = BurkeRatio::new(12).unwrap();
/// let mut last = None;
/// for i in 0..24 {
///     last = indicator.update((f64::from(i) * 0.5).sin() * 0.05);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct BurkeRatio {
    period: usize,
    window: VecDeque<f64>,
}

impl BurkeRatio {
    /// Construct a Burke Ratio over `period` returns.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPeriod`] if `period < 2`.
    pub fn new(period: usize) -> Result<Self> {
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "burke ratio needs period >= 2",
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
        #[allow(clippy::cast_precision_loss)]
        let length = self.window.len() as f64;
        let mut sum_return = 0.0;
        let mut sum_drawdown_sq = 0.0;
        let mut equity = 1.0;
        let mut peak: f64 = 1.0;
        for ret in &self.window {
            sum_return += *ret;
            equity *= 1.0 + *ret;
            peak = peak.max(equity);
            let drawdown = (peak - equity) / peak;
            sum_drawdown_sq += drawdown * drawdown;
        }
        let denom = sum_drawdown_sq.sqrt();
        if denom > 0.0 {
            (sum_return / length) / denom
        } else {
            0.0
        }
    }
}

impl Indicator for BurkeRatio {
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
        "BurkeRatio"
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
            BurkeRatio::new(1),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let br = BurkeRatio::new(12).unwrap();
        assert_eq!(br.period(), 12);
        assert_eq!(br.warmup_period(), 12);
        assert_eq!(br.name(), "BurkeRatio");
        assert!(!br.is_ready());
    }

    #[test]
    fn reference_value() {
        // returns [0.1, -0.1, 0.1]: dd = [0, 0.1, 0.01].
        // Σ dd² = 0.01 + 0.0001 = 0.0101; denom = sqrt(0.0101).
        // Burke = (0.1/3) / sqrt(0.0101).
        let mut br = BurkeRatio::new(3).unwrap();
        let out = br.batch(&[0.1, -0.1, 0.1]);
        let expected = (0.1_f64 / 3.0) / (0.0101_f64).sqrt();
        assert_relative_eq!(out[2].unwrap(), expected, epsilon = 1e-9);
    }

    #[test]
    fn no_drawdown_is_zero() {
        let mut br = BurkeRatio::new(3).unwrap();
        let last = br
            .batch(&[0.01, 0.02, 0.03])
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn losing_window_is_negative() {
        let mut br = BurkeRatio::new(3).unwrap();
        let last = br
            .batch(&[-0.05, -0.02, -0.03])
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!(last < 0.0);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut br = BurkeRatio::new(3).unwrap();
        assert_eq!(br.update(0.1), None);
        assert_eq!(br.update(f64::NAN), None);
        assert_eq!(br.update(-0.1), None);
        assert!(br.update(0.1).is_some());
    }

    #[test]
    fn reset_clears_state() {
        let mut br = BurkeRatio::new(3).unwrap();
        br.batch(&[0.1, -0.1, 0.1]);
        assert!(br.is_ready());
        br.reset();
        assert!(!br.is_ready());
        assert_eq!(br.update(0.1), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let rets: Vec<f64> = (0..60)
            .map(|i| (f64::from(i) * 0.25).sin() * 0.05)
            .collect();
        let batch = BurkeRatio::new(12).unwrap().batch(&rets);
        let mut streamer = BurkeRatio::new(12).unwrap();
        let streamed: Vec<_> = rets.iter().map(|r| streamer.update(*r)).collect();
        assert_eq!(batch, streamed);
    }
}
