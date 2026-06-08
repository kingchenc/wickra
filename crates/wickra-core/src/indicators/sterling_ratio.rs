//! Sterling Ratio — mean return over the average drawdown of the equity curve.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Sterling Ratio over a trailing window of `period` returns.
///
/// ```text
/// equity_t  = Π_{i<=t} (1 + return_i)          (compounded curve)
/// peak_t    = max_{s<=t} equity_s
/// dd_t      = (peak_t − equity_t) / peak_t      (fractional drawdown, >= 0)
/// Sterling  = mean(returns) / mean(dd_t)
/// ```
///
/// The Sterling Ratio rewards return per unit of *typical* pain: it divides the
/// average per-period return by the **average drawdown** experienced along the
/// compounded equity curve. Of the three drawdown-based ratios Wickra ships it is
/// the gentlest on outliers — averaging the drawdowns means one deep crater does
/// not dominate the way it does in the [`BurkeRatio`](crate::BurkeRatio) (which
/// sums squared drawdowns) or the [`MartinRatio`](crate::MartinRatio) (which uses
/// the root-mean-square percentage drawdown). A window that never draws down has
/// zero average drawdown and the indicator reports `0.0`.
///
/// The first value lands after `period` returns; each `update` rebuilds the equity
/// curve over the window (O(period)), which is O(1) in the length of the overall
/// series.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, SterlingRatio};
///
/// let mut indicator = SterlingRatio::new(12).unwrap();
/// let mut last = None;
/// for i in 0..24 {
///     last = indicator.update((f64::from(i) * 0.5).sin() * 0.05);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct SterlingRatio {
    period: usize,
    window: VecDeque<f64>,
}

impl SterlingRatio {
    /// Construct a Sterling Ratio over `period` returns.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPeriod`] if `period < 2`.
    pub fn new(period: usize) -> Result<Self> {
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "sterling ratio needs period >= 2",
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
        let mut sum_drawdown = 0.0;
        let mut equity = 1.0;
        let mut peak: f64 = 1.0;
        for ret in &self.window {
            sum_return += *ret;
            equity *= 1.0 + *ret;
            peak = peak.max(equity);
            sum_drawdown += (peak - equity) / peak;
        }
        let avg_drawdown = sum_drawdown / length;
        if avg_drawdown > 0.0 {
            (sum_return / length) / avg_drawdown
        } else {
            0.0
        }
    }
}

impl Indicator for SterlingRatio {
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
        "SterlingRatio"
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
            SterlingRatio::new(1),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let sr = SterlingRatio::new(12).unwrap();
        assert_eq!(sr.period(), 12);
        assert_eq!(sr.warmup_period(), 12);
        assert_eq!(sr.name(), "SterlingRatio");
        assert!(!sr.is_ready());
    }

    #[test]
    fn reference_value() {
        // returns [0.1, -0.1, 0.1]:
        //   equity 1.1, 0.99, 1.089; peak stays 1.1.
        //   dd = [0, 0.1, 0.01]; avg_dd = 0.11/3; mean_return = 0.1/3.
        //   Sterling = (0.1/3) / (0.11/3) = 0.1/0.11.
        let mut sr = SterlingRatio::new(3).unwrap();
        let out = sr.batch(&[0.1, -0.1, 0.1]);
        assert_relative_eq!(out[2].unwrap(), 0.1_f64 / 0.11, epsilon = 1e-9);
    }

    #[test]
    fn no_drawdown_is_zero() {
        // Monotonically rising equity never draws down.
        let mut sr = SterlingRatio::new(3).unwrap();
        let last = sr
            .batch(&[0.01, 0.02, 0.03])
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn losing_window_is_negative() {
        let mut sr = SterlingRatio::new(3).unwrap();
        let last = sr
            .batch(&[-0.05, -0.02, -0.03])
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!(last < 0.0);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut sr = SterlingRatio::new(3).unwrap();
        assert_eq!(sr.update(0.1), None);
        assert_eq!(sr.update(f64::NAN), None);
        assert_eq!(sr.update(-0.1), None);
        assert!(sr.update(0.1).is_some());
    }

    #[test]
    fn reset_clears_state() {
        let mut sr = SterlingRatio::new(3).unwrap();
        sr.batch(&[0.1, -0.1, 0.1]);
        assert!(sr.is_ready());
        sr.reset();
        assert!(!sr.is_ready());
        assert_eq!(sr.update(0.1), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let rets: Vec<f64> = (0..60)
            .map(|i| (f64::from(i) * 0.25).sin() * 0.05)
            .collect();
        let batch = SterlingRatio::new(12).unwrap().batch(&rets);
        let mut streamer = SterlingRatio::new(12).unwrap();
        let streamed: Vec<_> = rets.iter().map(|r| streamer.update(*r)).collect();
        assert_eq!(batch, streamed);
    }
}
