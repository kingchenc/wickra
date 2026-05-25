//! Ehlers Adaptive Cycle period estimator (for adaptive oscillators).

use crate::indicators::hilbert_dominant_cycle::HilbertDominantCycle;
use crate::traits::Indicator;

/// Ehlers' Adaptive Cycle Indicator.
///
/// Returns half the current dominant cycle period — the "best" lookback for
/// downstream oscillators like an adaptive RSI or adaptive Stochastic, per
/// Ehlers' *Cycle Analytics for Traders* (2013, ch. 11). Halving accounts for
/// the fact that an oscillator over a half-cycle captures the full peak-to-
/// trough swing without aliasing.
///
/// The output is rounded to an integer-valued `f64` and clamped to `[3, 25]`,
/// matching the typical operating range of period-adaptive oscillators.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, AdaptiveCycle};
///
/// let mut ac = AdaptiveCycle::new();
/// let mut last = None;
/// for i in 0..200 {
///     last = ac.update(100.0 + (f64::from(i) * 0.4).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct AdaptiveCycle {
    cycle: HilbertDominantCycle,
    last_value: Option<f64>,
}

impl AdaptiveCycle {
    /// Construct a new adaptive cycle estimator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Current adaptive period if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for AdaptiveCycle {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        let period = self.cycle.update(input)?;
        let half = (period * 0.5).round().clamp(3.0, 25.0);
        self.last_value = Some(half);
        Some(half)
    }

    fn reset(&mut self) {
        self.cycle.reset();
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        self.cycle.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "AdaptiveCycle"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    #[test]
    fn accessors_and_metadata() {
        let mut ac = AdaptiveCycle::new();
        assert_eq!(ac.warmup_period(), 50);
        assert_eq!(ac.name(), "AdaptiveCycle");
        assert!(!ac.is_ready());
        assert!(ac.value().is_none());
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 5.0)
            .collect();
        ac.batch(&prices);
        assert!(ac.is_ready());
        assert!(ac.value().is_some());
    }

    #[test]
    fn output_within_clamp_band() {
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.5).sin() * 5.0)
            .collect();
        let mut ac = AdaptiveCycle::new();
        for v in ac.batch(&prices).into_iter().flatten() {
            assert!((3.0..=25.0).contains(&v), "period {v} out of band");
            assert_eq!(v, v.round(), "expected integer-valued output");
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        let mut a = AdaptiveCycle::new();
        let mut b = AdaptiveCycle::new();
        let batch = a.batch(&prices);
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut ac = AdaptiveCycle::new();
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 5.0)
            .collect();
        ac.batch(&prices);
        let before = ac.value();
        assert!(before.is_some());
        assert_eq!(ac.update(f64::NAN), before);
    }

    #[test]
    fn reset_clears_state() {
        let mut ac = AdaptiveCycle::new();
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 5.0)
            .collect();
        ac.batch(&prices);
        assert!(ac.is_ready());
        ac.reset();
        assert!(!ac.is_ready());
    }
}
