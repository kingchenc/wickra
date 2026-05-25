//! Rolling Omega Ratio — gain-to-loss ratio above a threshold.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Rolling Omega Ratio.
///
/// Over the trailing window of `period` returns and a target `threshold`:
///
/// ```text
/// gains  = Σ max(0, r − threshold)
/// losses = Σ max(0, threshold − r)
/// Omega  = gains / losses
/// ```
///
/// Omega expresses how many units of "above-threshold" return the strategy
/// produces per unit of "below-threshold" shortfall. By construction `Omega
/// ≥ 0`; a window where every return clears the threshold has zero losses and
/// the indicator returns `f64::INFINITY` (in keeping with the standard
/// definition). The Sharpe Ratio collapses risk into a single second-moment
/// number; Omega keeps the full shape of the loss tail.
///
/// Each `update` is O(period) because the partial sums are recomputed across
/// the window — adequate for typical backtest windows (`period ≤ 252`).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, OmegaRatio};
///
/// let mut o = OmegaRatio::new(20, 0.0).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = o.update((f64::from(i) * 0.2).sin() * 0.01);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct OmegaRatio {
    period: usize,
    threshold: f64,
    window: VecDeque<f64>,
}

impl OmegaRatio {
    /// Construct a new rolling Omega Ratio.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize, threshold: f64) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            threshold,
            window: VecDeque::with_capacity(period),
        })
    }

    /// Configured window length.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Configured threshold (per-period).
    pub const fn threshold(&self) -> f64 {
        self.threshold
    }
}

impl Indicator for OmegaRatio {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(input);
        if self.window.len() < self.period {
            return None;
        }
        let mut gains = 0.0_f64;
        let mut losses = 0.0_f64;
        for &r in &self.window {
            let d = r - self.threshold;
            if d >= 0.0 {
                gains += d;
            } else {
                losses += -d;
            }
        }
        if losses == 0.0 {
            return Some(if gains == 0.0 { 0.0 } else { f64::INFINITY });
        }
        Some(gains / losses)
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
        "OmegaRatio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(OmegaRatio::new(0, 0.0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let o = OmegaRatio::new(10, 0.001).unwrap();
        assert_eq!(o.period(), 10);
        assert_relative_eq!(o.threshold(), 0.001, epsilon = 1e-12);
        assert_eq!(o.name(), "OmegaRatio");
        assert_eq!(o.warmup_period(), 10);
    }

    #[test]
    fn all_above_threshold_yields_infinity() {
        let mut o = OmegaRatio::new(4, 0.0).unwrap();
        let out = o.batch(&[0.01, 0.02, 0.03, 0.04]);
        assert!(out[3].unwrap().is_infinite());
    }

    #[test]
    fn flat_at_threshold_yields_zero() {
        // Every return equals threshold -> gains = losses = 0 -> 0 by
        // convention.
        let mut o = OmegaRatio::new(4, 0.01).unwrap();
        let out = o.batch(&[0.01; 4]);
        assert_eq!(out[3], Some(0.0));
    }

    #[test]
    fn reference_value() {
        // returns = [-0.02, 0.01, -0.01, 0.03], threshold = 0.
        // gains  = 0.01 + 0.03 = 0.04
        // losses = 0.02 + 0.01 = 0.03
        // Omega = 0.04 / 0.03 ≈ 1.3333...
        let mut o = OmegaRatio::new(4, 0.0).unwrap();
        let out = o.batch(&[-0.02, 0.01, -0.01, 0.03]);
        assert_relative_eq!(out[3].unwrap(), 0.04 / 0.03, epsilon = 1e-9);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut o = OmegaRatio::new(3, 0.0).unwrap();
        assert_eq!(o.update(f64::NAN), None);
        assert_eq!(o.update(f64::INFINITY), None);
    }

    #[test]
    fn reset_clears_state() {
        let mut o = OmegaRatio::new(3, 0.0).unwrap();
        o.batch(&[0.01, -0.02, 0.005]);
        assert!(o.is_ready());
        o.reset();
        assert!(!o.is_ready());
        assert_eq!(o.update(0.01), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let returns: Vec<f64> = (0..50).map(|i| (f64::from(i) * 0.4).sin() * 0.01).collect();
        let batch = OmegaRatio::new(10, 0.0).unwrap().batch(&returns);
        let mut s = OmegaRatio::new(10, 0.0).unwrap();
        let streamed: Vec<_> = returns.iter().map(|r| s.update(*r)).collect();
        assert_eq!(batch, streamed);
    }
}
