//! Pair Spread Z-Score — the standardised log-spread of two cointegrated assets.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Z-score of the log-spread `ln(a) − β·ln(b)` between two assets.
///
/// This is the canonical mean-reversion / statistical-arbitrage signal for a
/// pair. Each `update` receives one `(a, b)` pair of raw **prices** and the
/// indicator does two things:
///
/// 1. **Hedge ratio.** A rolling ordinary-least-squares regression of
///    `ln(a)` on `ln(b)` over the trailing `beta_period` samples gives the
///    slope `β = cov(ln a, ln b) / var(ln b)`. The instantaneous spread is the
///    residual against the origin, `s = ln(a) − β·ln(b)`.
/// 2. **Standardisation.** The spread is then z-scored over the trailing
///    `z_period` spreads: `z = (s − mean_s) / std_s`.
///
/// A large positive `z` means `a` is rich relative to `b` (sell the spread); a
/// large negative `z` means `a` is cheap (buy the spread); `z` near zero means
/// the pair is at its typical relationship. The two windows are independent:
/// `beta_period` controls how much history the hedge ratio adapts over, and
/// `z_period` controls the look-back for the mean and dispersion of the spread.
///
/// Each `update` is O(1): five running sums maintain the rolling OLS and two
/// more maintain the rolling spread mean/variance. A flat `ln(b)` window has
/// zero variance and the hedge ratio is undefined; `β` is then taken as `0`,
/// reducing the spread to `ln(a)`. A flat spread window (zero dispersion)
/// yields a z-score of `0` rather than `NaN`.
///
/// Prices must be strictly positive and finite for the logarithm to be
/// defined; a non-positive or non-finite price is skipped (it does not enter
/// either window), exactly as a real feed would discard a bad tick.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, PairSpreadZScore};
///
/// let mut zs = PairSpreadZScore::new(2, 2).unwrap();
/// // A flat benchmark gives hedge ratio 0, so the spread is just ln(a); with
/// // a 2-sample z-window the z-score collapses to the sign of the last move.
/// let mut last = None;
/// for a in [100.0, 100.0, 110.0, 120.0] {
///     last = zs.update((a, 100.0));
/// }
/// assert!((last.unwrap() - 1.0).abs() < 1e-9);
/// ```
#[derive(Debug, Clone)]
pub struct PairSpreadZScore {
    beta_period: usize,
    z_period: usize,
    // Rolling OLS of y = ln(a) on x = ln(b).
    reg: VecDeque<(f64, f64)>,
    sum_x: f64,
    sum_y: f64,
    sum_xx: f64,
    sum_xy: f64,
    // Rolling mean/variance of the spread.
    spreads: VecDeque<f64>,
    sum_s: f64,
    sum_ss: f64,
}

impl PairSpreadZScore {
    /// Construct a new pair spread z-score.
    ///
    /// `beta_period` is the look-back for the rolling hedge ratio; `z_period`
    /// is the look-back for standardising the spread.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if either period is below `2`
    /// (variance needs at least two points).
    pub fn new(beta_period: usize, z_period: usize) -> Result<Self> {
        if beta_period < 2 {
            return Err(Error::InvalidPeriod {
                message: "pair spread z-score needs beta_period >= 2",
            });
        }
        if z_period < 2 {
            return Err(Error::InvalidPeriod {
                message: "pair spread z-score needs z_period >= 2",
            });
        }
        Ok(Self {
            beta_period,
            z_period,
            reg: VecDeque::with_capacity(beta_period),
            sum_x: 0.0,
            sum_y: 0.0,
            sum_xx: 0.0,
            sum_xy: 0.0,
            spreads: VecDeque::with_capacity(z_period),
            sum_s: 0.0,
            sum_ss: 0.0,
        })
    }

    /// Look-back of the rolling hedge-ratio regression.
    pub const fn beta_period(&self) -> usize {
        self.beta_period
    }

    /// Look-back of the rolling spread standardisation.
    pub const fn z_period(&self) -> usize {
        self.z_period
    }

    /// The current hedge ratio `β`, or `None` while the regression is warming
    /// up. A flat `ln(b)` window reports `0`.
    fn hedge_ratio(&self) -> Option<f64> {
        if self.reg.len() < self.beta_period {
            return None;
        }
        let n = self.beta_period as f64;
        let mean_x = self.sum_x / n;
        let mean_y = self.sum_y / n;
        let var_x = (self.sum_xx / n - mean_x * mean_x).max(0.0);
        if var_x == 0.0 {
            return Some(0.0);
        }
        let cov = self.sum_xy / n - mean_x * mean_y;
        Some(cov / var_x)
    }

    fn push_spread(&mut self, s: f64) -> Option<f64> {
        if self.spreads.len() == self.z_period {
            let old = self.spreads.pop_front().expect("non-empty");
            self.sum_s -= old;
            self.sum_ss -= old * old;
        }
        self.spreads.push_back(s);
        self.sum_s += s;
        self.sum_ss += s * s;
        if self.spreads.len() < self.z_period {
            return None;
        }
        let m = self.z_period as f64;
        let mean_s = self.sum_s / m;
        let var_s = (self.sum_ss / m - mean_s * mean_s).max(0.0);
        let std_s = var_s.sqrt();
        if std_s == 0.0 {
            // A flat spread window has no dispersion to standardise against.
            return Some(0.0);
        }
        Some((s - mean_s) / std_s)
    }
}

impl Indicator for PairSpreadZScore {
    /// `(a, b)` price pair.
    type Input = (f64, f64);
    type Output = f64;

    fn update(&mut self, input: (f64, f64)) -> Option<f64> {
        let (a, b) = input;
        if !(a > 0.0 && b > 0.0 && a.is_finite() && b.is_finite()) {
            // Bad tick: skip it without disturbing either window.
            return None;
        }
        let x = b.ln();
        let y = a.ln();
        if self.reg.len() == self.beta_period {
            let (ox, oy) = self.reg.pop_front().expect("non-empty");
            self.sum_x -= ox;
            self.sum_y -= oy;
            self.sum_xx -= ox * ox;
            self.sum_xy -= ox * oy;
        }
        self.reg.push_back((x, y));
        self.sum_x += x;
        self.sum_y += y;
        self.sum_xx += x * x;
        self.sum_xy += x * y;
        let beta = self.hedge_ratio()?;
        let spread = y - beta * x;
        self.push_spread(spread)
    }

    fn reset(&mut self) {
        self.reg.clear();
        self.sum_x = 0.0;
        self.sum_y = 0.0;
        self.sum_xx = 0.0;
        self.sum_xy = 0.0;
        self.spreads.clear();
        self.sum_s = 0.0;
        self.sum_ss = 0.0;
    }

    fn warmup_period(&self) -> usize {
        // `beta_period` samples to define the hedge ratio (and the first
        // spread), then `z_period − 1` more to fill the spread window.
        self.beta_period + self.z_period - 1
    }

    fn is_ready(&self) -> bool {
        self.spreads.len() == self.z_period
    }

    fn name(&self) -> &'static str {
        "PairSpreadZScore"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_periods_below_two() {
        assert!(PairSpreadZScore::new(1, 5).is_err());
        assert!(PairSpreadZScore::new(5, 1).is_err());
        assert!(PairSpreadZScore::new(2, 2).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let z = PairSpreadZScore::new(10, 20).unwrap();
        assert_eq!(z.beta_period(), 10);
        assert_eq!(z.z_period(), 20);
        assert_eq!(z.warmup_period(), 29);
        assert_eq!(z.name(), "PairSpreadZScore");
    }

    #[test]
    fn flat_benchmark_two_sample_window_is_sign_of_move() {
        // Flat b ⇒ β = 0 ⇒ spread = ln(a); z_period = 2 ⇒ z = sign of last move.
        let mut z = PairSpreadZScore::new(2, 2).unwrap();
        assert_eq!(z.update((100.0, 100.0)), None);
        assert_eq!(z.update((100.0, 100.0)), None);
        // The ±1 result is exact in real arithmetic; the variance is computed
        // via Σs²−mean² so a few ulps of cancellation error remain.
        assert_relative_eq!(z.update((110.0, 100.0)).unwrap(), 1.0, epsilon = 1e-9);
        assert_relative_eq!(z.update((105.0, 100.0)).unwrap(), -1.0, epsilon = 1e-9);
        assert_relative_eq!(z.update((130.0, 100.0)).unwrap(), 1.0, epsilon = 1e-9);
    }

    #[test]
    fn constant_spread_yields_zero() {
        // Both legs flat ⇒ spread constant ⇒ zero dispersion ⇒ z = 0.
        let pairs: Vec<(f64, f64)> = (0..10).map(|_| (50.0, 100.0)).collect();
        let last = PairSpreadZScore::new(3, 4)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn bad_tick_is_skipped() {
        let mut z = PairSpreadZScore::new(2, 2).unwrap();
        // A non-positive or non-finite price never enters the windows.
        assert_eq!(z.update((0.0, 100.0)), None);
        assert_eq!(z.update((100.0, f64::NAN)), None);
        assert!(!z.is_ready());
        // Valid ticks then warm the indicator normally.
        z.update((100.0, 100.0));
        z.update((100.0, 100.0));
        z.update((110.0, 100.0));
        assert!(z.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut z = PairSpreadZScore::new(3, 3).unwrap();
        for i in 0..10 {
            let b = 100.0 + 5.0 * f64::from(i).sin();
            z.update((b * 1.5, b));
        }
        assert!(z.is_ready());
        z.reset();
        assert!(!z.is_ready());
        assert_eq!(z.update((100.0, 100.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let pairs: Vec<(f64, f64)> = (0..80)
            .map(|i| {
                let t = f64::from(i);
                let b = 100.0 + 10.0 * (t * 0.2).sin();
                let a = b * (1.0 + 0.05 * (t * 0.5).cos());
                (a, b)
            })
            .collect();
        let batch = PairSpreadZScore::new(14, 10).unwrap().batch(&pairs);
        let mut z = PairSpreadZScore::new(14, 10).unwrap();
        let streamed: Vec<_> = pairs.iter().map(|p| z.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
