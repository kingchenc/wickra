//! Ehlers Fisher Transform.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Ehlers' Fisher Transform of price.
///
/// Normalises the most recent price to `[-1, +1]` via min/max over a `period`
/// window, smooths the normalised value with a 0.33 / 0.67 IIR step, and
/// applies the Fisher transform `0.5 * ln((1+x)/(1-x))`. The result has a
/// near-Gaussian distribution, so extreme readings stand out cleanly. A
/// secondary signal is produced by lagging the Fisher value by one bar (the
/// classic trigger), making the indicator a two-line crossover system in
/// charts.
///
/// Only the primary Fisher value is exposed here as a scalar; the lagged
/// trigger is one update behind by construction.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, FisherTransform};
///
/// let mut ft = FisherTransform::new(10).unwrap();
/// let mut last = None;
/// for i in 0..30 {
///     last = ft.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct FisherTransform {
    period: usize,
    window: VecDeque<f64>,
    smoothed: f64,
    last_fisher: Option<f64>,
}

impl FisherTransform {
    /// Construct with the rolling extrema window length.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
            smoothed: 0.0,
            last_fisher: None,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current Fisher value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_fisher
    }
}

impl Indicator for FisherTransform {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.last_fisher;
        }
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(input);
        if self.window.len() < self.period {
            return None;
        }
        let max = self
            .window
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let min = self.window.iter().copied().fold(f64::INFINITY, f64::min);
        let range = max - min;
        // Normalise to roughly [-1, +1]; centred midpoint when range == 0.
        let raw = if range > 0.0 {
            ((input - min) / range).mul_add(2.0, -1.0)
        } else {
            0.0
        };
        // Ehlers IIR: 0.33 * raw + 0.67 * prev_smoothed, then clamp.
        self.smoothed = 0.33f64.mul_add(raw, 0.67 * self.smoothed);
        // Clamp strictly inside (-1, +1) to keep the log finite.
        let clamped = self.smoothed.clamp(-0.999, 0.999);
        let fisher = 0.5 * ((1.0 + clamped) / (1.0 - clamped)).ln();
        self.last_fisher = Some(fisher);
        Some(fisher)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.smoothed = 0.0;
        self.last_fisher = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.last_fisher.is_some()
    }

    fn name(&self) -> &'static str {
        "FisherTransform"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(FisherTransform::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let mut ft = FisherTransform::new(10).unwrap();
        assert_eq!(ft.period(), 10);
        assert_eq!(ft.warmup_period(), 10);
        assert_eq!(ft.name(), "FisherTransform");
        assert!(ft.value().is_none());
        for i in 1..=10 {
            ft.update(f64::from(i));
        }
        assert!(ft.value().is_some());
        assert!(ft.is_ready());
    }

    #[test]
    fn warmup_returns_none_until_seed() {
        let mut ft = FisherTransform::new(5).unwrap();
        for i in 1..=4 {
            assert_eq!(ft.update(f64::from(i)), None);
        }
        assert!(ft.update(5.0).is_some());
    }

    #[test]
    fn constant_series_zero_range_yields_zero() {
        let mut ft = FisherTransform::new(5).unwrap();
        let out = ft.batch(&[42.0_f64; 30]);
        for x in out.iter().skip(5).flatten() {
            assert!(x.abs() < 1e-6, "expected near-zero, got {x}");
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..60)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 8.0)
            .collect();
        let mut a = FisherTransform::new(10).unwrap();
        let mut b = FisherTransform::new(10).unwrap();
        let batch = a.batch(&prices);
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut ft = FisherTransform::new(5).unwrap();
        ft.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let before = ft.value();
        assert!(before.is_some());
        assert_eq!(ft.update(f64::NAN), before);
        assert_eq!(ft.update(f64::INFINITY), before);
    }

    #[test]
    fn reset_clears_state() {
        let mut ft = FisherTransform::new(5).unwrap();
        ft.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        assert!(ft.is_ready());
        ft.reset();
        assert!(!ft.is_ready());
        assert_eq!(ft.update(1.0), None);
    }
}
