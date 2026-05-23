//! Kaufman's Adaptive Moving Average (KAMA).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Kaufman's Adaptive Moving Average.
///
/// KAMA adapts its smoothing constant to volatility: efficient (trending) markets
/// get a fast smoothing constant, choppy markets get a slow one. Parameters are
/// the efficiency-ratio lookback (`er_period`, default 10), the fast EMA period
/// (`fast`, default 2) and the slow EMA period (`slow`, default 30).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Kama};
///
/// let mut indicator = Kama::new(10, 2, 30).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Kama {
    er_period: usize,
    fast_sc: f64,
    slow_sc: f64,
    window: VecDeque<f64>,
    state: Option<f64>,
}

impl Kama {
    /// # Errors
    /// Returns [`Error::PeriodZero`] / [`Error::InvalidPeriod`] for bad parameters.
    pub fn new(er_period: usize, fast: usize, slow: usize) -> Result<Self> {
        if er_period == 0 || fast == 0 || slow == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "KAMA fast period must be strictly less than slow",
            });
        }
        let fast_sc = 2.0 / (fast as f64 + 1.0);
        let slow_sc = 2.0 / (slow as f64 + 1.0);
        Ok(Self {
            er_period,
            fast_sc,
            slow_sc,
            window: VecDeque::with_capacity(er_period + 1),
            state: None,
        })
    }

    /// Classic Kaufman parameters: (10, 2, 30).
    pub fn classic() -> Self {
        Self::new(10, 2, 30).expect("classic KAMA parameters are valid")
    }

    /// Configured `(er_period, fast, slow)` periods.
    pub fn periods(&self) -> (usize, f64, f64) {
        (self.er_period, self.fast_sc, self.slow_sc)
    }
}

impl Indicator for Kama {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.state;
        }
        if self.window.len() == self.er_period + 1 {
            self.window.pop_front();
        }
        self.window.push_back(input);

        if self.window.len() < self.er_period + 1 {
            return None;
        }

        let first = *self.window.front().expect("non-empty");
        let last = *self.window.back().expect("non-empty");
        let direction = (last - first).abs();
        let volatility: f64 = self
            .window
            .iter()
            .zip(self.window.iter().skip(1))
            .map(|(a, b)| (b - a).abs())
            .sum();

        let er = if volatility == 0.0 {
            0.0
        } else {
            direction / volatility
        };
        let sc = (er * (self.fast_sc - self.slow_sc) + self.slow_sc).powi(2);

        let prev = self.state.unwrap_or(first);
        let new = prev + sc * (input - prev);
        self.state = Some(new);
        Some(new)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.state = None;
    }

    fn warmup_period(&self) -> usize {
        self.er_period + 1
    }

    fn is_ready(&self) -> bool {
        self.state.is_some()
    }

    fn name(&self) -> &'static str {
        "KAMA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    /// Cover the `periods` accessor (65-67) and the Indicator-impl
    /// `warmup_period` (115-117) + `name` (123-125). Existing tests
    /// inspect KAMA output but never query the metadata.
    #[test]
    fn accessors_and_metadata() {
        let k = Kama::classic();
        let (er, fast, slow) = k.periods();
        assert_eq!(er, 10);
        assert!((fast - 2.0 / (2.0 + 1.0)).abs() < 1e-12);
        assert!((slow - 2.0 / (30.0 + 1.0)).abs() < 1e-12);
        assert_eq!(k.warmup_period(), 11);
        assert_eq!(k.name(), "KAMA");
    }

    #[test]
    fn constant_series_yields_constant_kama() {
        let mut k = Kama::classic();
        let out = k.batch(&[100.0_f64; 100]);
        let last = out.iter().rev().flatten().next().unwrap();
        assert_relative_eq!(*last, 100.0, epsilon = 1e-9);
    }

    #[test]
    fn rejects_invalid_periods() {
        assert!(Kama::new(0, 2, 30).is_err());
        assert!(Kama::new(10, 30, 2).is_err()); // fast >= slow
        assert!(Kama::new(10, 2, 2).is_err()); // fast == slow
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=120)
            .map(|i| (f64::from(i) * 0.2).sin() * 5.0 + f64::from(i) * 0.1)
            .collect();
        let mut a = Kama::classic();
        let mut b = Kama::classic();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut k = Kama::classic();
        k.batch(&(1..=50).map(f64::from).collect::<Vec<_>>());
        assert!(k.is_ready());
        k.reset();
        assert!(!k.is_ready());
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut k = Kama::classic();
        k.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        let before = k.update(41.0);
        assert!(before.is_some());
        // Non-finite inputs return the last state without sliding the window.
        assert_eq!(k.update(f64::NAN), before);
        assert_eq!(k.update(f64::INFINITY), before);
    }
}
