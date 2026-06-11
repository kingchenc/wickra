//! Ornstein–Uhlenbeck half-life of mean reversion for the spread of two series.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Half-life of mean reversion of the spread `a − b`, from an Ornstein–Uhlenbeck
/// fit.
///
/// Each `update` takes one `(a, b)` price pair and forms the spread
/// `sₜ = aₜ − bₜ`. Over the trailing window of `period` spreads the indicator
/// fits the discrete Ornstein–Uhlenbeck (mean-reverting AR(1)) model by
/// ordinary least squares of the change on the level:
///
/// ```text
/// Δsₜ = λ · sₜ₋₁ + c + εₜ
/// half_life = −ln(2) / λ        (only when λ < 0)
/// ```
///
/// `λ` is the speed of mean reversion: a more negative `λ` pulls the spread back
/// to its mean faster. The **half-life** is the number of bars for a deviation
/// to decay by half — the single most useful number for sizing a pairs trade's
/// holding period and look-back. When the spread is not mean-reverting
/// (`λ ≥ 0`, a random walk or a trend) or the regression is degenerate (a flat
/// spread), the indicator returns `0`, meaning "no finite half-life".
///
/// Each `update` is `O(period)`: the OLS slope is recomputed from the window's
/// running geometry. Output is in bars and is always `≥ 0`.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, OuHalfLife};
///
/// let mut hl = OuHalfLife::new(40).unwrap();
/// let mut last = None;
/// for t in 0..120 {
///     let b = 100.0 + f64::from(t);
///     // `a` hugs `b` with a fast mean-reverting wobble ⇒ short half-life.
///     let a = b + 2.0 * (f64::from(t) * 0.9).sin();
///     last = hl.update((a, b));
/// }
/// let half_life = last.unwrap();
/// assert!(half_life > 0.0 && half_life < 40.0);
/// ```
#[derive(Debug, Clone)]
pub struct OuHalfLife {
    period: usize,
    window: VecDeque<f64>,
}

impl OuHalfLife {
    /// Construct a new Ornstein–Uhlenbeck half-life estimator.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `period < 3` — the AR(1) regression
    /// needs at least two observations (a slope and an intercept).
    pub fn new(period: usize) -> Result<Self> {
        if period < 3 {
            return Err(Error::InvalidPeriod {
                message: "OU half-life needs period >= 3",
            });
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
        })
    }

    /// Configured look-back window of spreads.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for OuHalfLife {
    type Input = (f64, f64);
    type Output = f64;

    fn update(&mut self, input: (f64, f64)) -> Option<f64> {
        let (a, b) = input;
        if !a.is_finite() || !b.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(a - b);
        if self.window.len() < self.period {
            return None;
        }
        // OLS slope λ of Δsₜ on sₜ₋₁ over the window.
        let spreads: Vec<f64> = self.window.iter().copied().collect();
        let count = (spreads.len() - 1) as f64;
        let mut sum_level = 0.0;
        let mut sum_delta = 0.0;
        let mut sum_ll = 0.0;
        let mut sum_ld = 0.0;
        for pair in spreads.windows(2) {
            let level = pair[0];
            let delta = pair[1] - pair[0];
            sum_level += level;
            sum_delta += delta;
            sum_ll += level * level;
            sum_ld += level * delta;
        }
        let mean_level = sum_level / count;
        let mean_delta = sum_delta / count;
        let var_level = sum_ll / count - mean_level * mean_level;
        if var_level <= 0.0 {
            // Flat spread: the regression has no defined slope.
            return Some(0.0);
        }
        let cov = sum_ld / count - mean_level * mean_delta;
        let lambda = cov / var_level;
        if lambda >= 0.0 {
            // Not mean-reverting (random walk or diverging): no finite half-life.
            return Some(0.0);
        }
        Some(-std::f64::consts::LN_2 / lambda)
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
        "OuHalfLife"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    #[test]
    fn rejects_period_below_three() {
        assert!(OuHalfLife::new(2).is_err());
        assert!(OuHalfLife::new(3).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let hl = OuHalfLife::new(30).unwrap();
        assert_eq!(hl.period(), 30);
        assert_eq!(hl.warmup_period(), 30);
        assert_eq!(hl.name(), "OuHalfLife");
        assert!(!hl.is_ready());
    }

    #[test]
    fn warmup_returns_none() {
        let mut hl = OuHalfLife::new(4).unwrap();
        assert_eq!(hl.update((1.0, 0.0)), None);
        assert_eq!(hl.update((2.0, 0.0)), None);
        assert_eq!(hl.update((3.0, 0.0)), None);
        assert!(hl.update((4.0, 0.0)).is_some());
        assert!(hl.is_ready());
    }

    #[test]
    fn mean_reverting_spread_has_positive_half_life() {
        // Fast sinusoidal spread around zero ⇒ strong mean reversion.
        let pairs: Vec<(f64, f64)> = (0..120)
            .map(|t| {
                let b = 100.0 + f64::from(t);
                let a = b + 2.0 * (f64::from(t) * 0.9).sin();
                (a, b)
            })
            .collect();
        let last = OuHalfLife::new(40)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!(last > 0.0 && last < 40.0, "half-life {last}");
    }

    #[test]
    fn trending_spread_has_zero_half_life() {
        // Spread = a − b grows monotonically (λ ≥ 0) ⇒ no finite half-life.
        let pairs: Vec<(f64, f64)> = (0..40)
            .map(|t| (2.0 * f64::from(t), f64::from(t)))
            .collect();
        let last = OuHalfLife::new(20)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_eq!(last, 0.0);
    }

    #[test]
    fn flat_spread_returns_zero() {
        // a − b is constant ⇒ var(level) = 0 ⇒ undefined ⇒ 0.
        let pairs: Vec<(f64, f64)> = (0..30)
            .map(|t| (5.0 + f64::from(t), f64::from(t)))
            .collect();
        let last = OuHalfLife::new(10)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_eq!(last, 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut hl = OuHalfLife::new(5).unwrap();
        for t in 0..10 {
            hl.update((f64::from(t) + (f64::from(t) * 0.7).sin(), f64::from(t)));
        }
        assert!(hl.is_ready());
        hl.reset();
        assert!(!hl.is_ready());
        assert_eq!(hl.update((1.0, 0.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let pairs: Vec<(f64, f64)> = (0..80)
            .map(|t| {
                let b = 50.0 + 0.5 * f64::from(t);
                (b + (f64::from(t) * 0.6).sin(), b)
            })
            .collect();
        let batch = OuHalfLife::new(25).unwrap().batch(&pairs);
        let mut hl = OuHalfLife::new(25).unwrap();
        let streamed: Vec<_> = pairs.iter().map(|p| hl.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn non_finite_input_returns_none() {
        let mut hl = OuHalfLife::new(4).unwrap();
        assert_eq!(hl.update((f64::NAN, 1.0)), None);
        assert_eq!(hl.update((1.0, f64::INFINITY)), None);
        // The rejected ticks leave no trace: a fresh window still warms up.
        assert_eq!(hl.update((1.0, 0.0)), None);
        assert_eq!(hl.update((2.0, 0.0)), None);
        assert_eq!(hl.update((3.0, 0.0)), None);
        assert!(hl.update((4.0, 0.0)).is_some());
    }
}
