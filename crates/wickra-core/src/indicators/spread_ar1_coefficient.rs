//! AR(1) autoregression coefficient of the spread of two series.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// First-order autoregression coefficient `ρ` of the spread `a − b`.
///
/// Each `update` takes one `(a, b)` price pair and forms the spread
/// `sₜ = aₜ − bₜ`. Over the trailing window of `period` spreads the indicator
/// fits the discrete AR(1) model by ordinary least squares of the level on its
/// own lag:
///
/// ```text
/// sₜ = ρ · sₜ₋₁ + c + εₜ
/// ρ  = cov(sₜ₋₁, sₜ) / var(sₜ₋₁)
/// ```
///
/// `ρ` is the direct measure of cointegration / mean-reversion strength of the
/// pair:
///
/// - `ρ` near `0` — the spread snaps back to its mean almost instantly (very
///   strong mean reversion).
/// - `ρ` near `1` — the spread behaves like a random walk (a unit root: no
///   reliable reversion, the pair is *not* cointegrated).
/// - `ρ > 1` — the spread is explosive (diverging).
///
/// This is the complement of [`OuHalfLife`](crate::OuHalfLife): the OU half-life
/// is `−ln(2) / ln(ρ)` for `0 < ρ < 1`, but `ρ` itself is the raw, unbounded
/// stationarity statistic many pairs-trading screens threshold on directly
/// (e.g. "trade only pairs with `ρ < 0.9`"). When the spread is flat over the
/// window (`var(sₜ₋₁) = 0`) the regression slope is undefined and the indicator
/// returns `0`.
///
/// Each `update` is `O(period)`: the OLS slope is recomputed from the window's
/// running geometry.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, SpreadAr1Coefficient};
///
/// let mut ar1 = SpreadAr1Coefficient::new(40).unwrap();
/// let mut last = None;
/// for t in 0..120 {
///     let b = 100.0 + f64::from(t);
///     // `a` hugs `b` with a fast mean-reverting wobble ⇒ ρ well below 1.
///     let a = b + 2.0 * (f64::from(t) * 0.9).sin();
///     last = ar1.update((a, b));
/// }
/// let rho = last.unwrap();
/// assert!(rho > 0.0 && rho < 1.0);
/// ```
#[derive(Debug, Clone)]
pub struct SpreadAr1Coefficient {
    period: usize,
    window: VecDeque<f64>,
}

impl SpreadAr1Coefficient {
    /// Construct a new AR(1) spread-coefficient estimator.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `period < 3` — the AR(1) regression
    /// needs at least two `(level, next)` observations (a slope and an
    /// intercept).
    pub fn new(period: usize) -> Result<Self> {
        if period < 3 {
            return Err(Error::InvalidPeriod {
                message: "AR(1) spread coefficient needs period >= 3",
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

impl Indicator for SpreadAr1Coefficient {
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
        // OLS slope ρ of the level on its own lag over the window.
        let spreads: Vec<f64> = self.window.iter().copied().collect();
        let count = (spreads.len() - 1) as f64;
        let mut sum_level = 0.0;
        let mut sum_next = 0.0;
        let mut sum_ll = 0.0;
        let mut sum_ln = 0.0;
        for pair in spreads.windows(2) {
            let level = pair[0];
            let next = pair[1];
            sum_level += level;
            sum_next += next;
            sum_ll += level * level;
            sum_ln += level * next;
        }
        let mean_level = sum_level / count;
        let mean_next = sum_next / count;
        let var_level = sum_ll / count - mean_level * mean_level;
        if var_level <= 0.0 {
            // Flat spread: the regression has no defined slope.
            return Some(0.0);
        }
        let cov = sum_ln / count - mean_level * mean_next;
        Some(cov / var_level)
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
        "SpreadAr1Coefficient"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_period_below_three() {
        assert!(SpreadAr1Coefficient::new(2).is_err());
        assert!(SpreadAr1Coefficient::new(3).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let ar1 = SpreadAr1Coefficient::new(30).unwrap();
        assert_eq!(ar1.period(), 30);
        assert_eq!(ar1.warmup_period(), 30);
        assert_eq!(ar1.name(), "SpreadAr1Coefficient");
        assert!(!ar1.is_ready());
    }

    #[test]
    fn warmup_returns_none() {
        let mut ar1 = SpreadAr1Coefficient::new(4).unwrap();
        assert_eq!(ar1.update((1.0, 0.0)), None);
        assert_eq!(ar1.update((2.0, 0.0)), None);
        assert_eq!(ar1.update((3.0, 0.0)), None);
        assert!(ar1.update((4.0, 0.0)).is_some());
        assert!(ar1.is_ready());
    }

    #[test]
    fn mean_reverting_spread_has_rho_below_one() {
        // Fast sinusoidal spread around zero ⇒ stationary ⇒ 0 < ρ < 1.
        let pairs: Vec<(f64, f64)> = (0..120)
            .map(|t| {
                let b = 100.0 + f64::from(t);
                let a = b + 2.0 * (f64::from(t) * 0.9).sin();
                (a, b)
            })
            .collect();
        let last = SpreadAr1Coefficient::new(40)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!(last > 0.0 && last < 1.0, "rho {last}");
    }

    #[test]
    fn random_walk_spread_has_rho_near_one() {
        // Spread = a − b grows by exactly 1 each bar ⇒ next = level + 1 ⇒
        // the OLS slope is exactly 1 (unit root).
        let pairs: Vec<(f64, f64)> = (0..40)
            .map(|t| (2.0 * f64::from(t), f64::from(t)))
            .collect();
        let last = SpreadAr1Coefficient::new(20)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, 1.0, epsilon = 1e-9);
    }

    #[test]
    fn flat_spread_returns_zero() {
        // a − b is constant ⇒ var(level) = 0 ⇒ undefined ⇒ 0.
        let pairs: Vec<(f64, f64)> = (0..30)
            .map(|t| (5.0 + f64::from(t), f64::from(t)))
            .collect();
        let last = SpreadAr1Coefficient::new(10)
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
        let mut ar1 = SpreadAr1Coefficient::new(5).unwrap();
        for t in 0..10 {
            ar1.update((f64::from(t) + (f64::from(t) * 0.7).sin(), f64::from(t)));
        }
        assert!(ar1.is_ready());
        ar1.reset();
        assert!(!ar1.is_ready());
        assert_eq!(ar1.update((1.0, 0.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let pairs: Vec<(f64, f64)> = (0..80)
            .map(|t| {
                let b = 50.0 + 0.5 * f64::from(t);
                (b + (f64::from(t) * 0.6).sin(), b)
            })
            .collect();
        let batch = SpreadAr1Coefficient::new(25).unwrap().batch(&pairs);
        let mut ar1 = SpreadAr1Coefficient::new(25).unwrap();
        let streamed: Vec<_> = pairs.iter().map(|p| ar1.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn non_finite_input_returns_none() {
        let mut ar1 = SpreadAr1Coefficient::new(4).unwrap();
        assert_eq!(ar1.update((f64::NAN, 1.0)), None);
        assert_eq!(ar1.update((1.0, f64::INFINITY)), None);
        // The rejected ticks leave no trace: a fresh window still warms up.
        assert_eq!(ar1.update((1.0, 0.0)), None);
        assert_eq!(ar1.update((2.0, 0.0)), None);
        assert_eq!(ar1.update((3.0, 0.0)), None);
        assert!(ar1.update((4.0, 0.0)).is_some());
    }
}
