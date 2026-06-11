//! Cointegration — rolling Engle–Granger hedge ratio plus an ADF stationarity test.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Output of [`Cointegration`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CointegrationOutput {
    /// Engle–Granger hedge ratio `β`: the rolling OLS slope of `a` on `b`.
    pub hedge_ratio: f64,
    /// The current spread (regression residual) `a − (α + β·b)`.
    pub spread: f64,
    /// Augmented Dickey–Fuller `t`-statistic on the spread. **More negative**
    /// means more strongly mean-reverting (cointegrated); compare against the
    /// usual ADF/MacKinnon critical values (e.g. roughly `−2.9` at 5%). `0`
    /// when the test is undefined (a degenerate, zero-variance spread).
    pub adf_stat: f64,
}

/// Rolling cointegration test for a pair of assets (Engle–Granger two-step).
///
/// Each `update` receives one `(a, b)` pair (price levels, or log-levels if you
/// prefer). Over the trailing window of `period` pairs the indicator:
///
/// 1. fits the **hedge ratio** `β` (and intercept `α`) by ordinary least
///    squares of `a` on `b`, and forms the **spread** `eₜ = aₜ − (α + β·bₜ)`;
/// 2. runs an **augmented Dickey–Fuller** test (no constant, no trend, with
///    `adf_lags` lagged differences) on the spread series and reports its
///    `t`-statistic.
///
/// A strongly negative ADF statistic means the spread reverts to its mean — the
/// pair is cointegrated and the spread is tradeable. A statistic near zero
/// means the spread wanders like a random walk (no cointegration). This is the
/// classic pairs-trading screen: `β` tells you the hedge size, the spread is
/// what you trade, and the ADF statistic tells you whether it is worth trading.
///
/// Each `update` is `O(period + adf_lags³)`: the hedge ratio is maintained from
/// running sums, while the spread series and the small ADF regression are
/// recomputed over the window — both bounded by the fixed parameters, not the
/// series length.
///
/// # Example
///
/// ```
/// use wickra_core::{Cointegration, Indicator};
///
/// let mut c = Cointegration::new(30, 1).unwrap();
/// let mut last = None;
/// for t in 0..60 {
///     let b = 100.0 + f64::from(t);
///     // `a` tracks 2·b with a small mean-reverting wobble ⇒ cointegrated.
///     let a = 2.0 * b + 5.0 + 0.5 * (f64::from(t) * 0.7).sin();
///     last = c.update((a, b));
/// }
/// let out = last.unwrap();
/// assert!((out.hedge_ratio - 2.0).abs() < 0.1);
/// assert!(out.adf_stat < 0.0); // mean-reverting spread
/// ```
#[derive(Debug, Clone)]
pub struct Cointegration {
    period: usize,
    adf_lags: usize,
    window: VecDeque<(f64, f64)>,
    sum_a: f64,
    sum_b: f64,
    sum_bb: f64,
    sum_ab: f64,
}

impl Cointegration {
    /// Construct a new rolling cointegration test.
    ///
    /// `period` is the look-back window; `adf_lags` is the number of lagged
    /// differences in the augmented Dickey–Fuller regression (`0` is the plain
    /// Dickey–Fuller test).
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `period < 2·adf_lags + 4`, which is
    /// the smallest window that leaves the ADF regression at least one degree
    /// of freedom.
    pub fn new(period: usize, adf_lags: usize) -> Result<Self> {
        let min_period = 2 * adf_lags + 4;
        if period < min_period {
            return Err(Error::InvalidPeriod {
                message: "cointegration needs period >= 2*adf_lags + 4",
            });
        }
        Ok(Self {
            period,
            adf_lags,
            window: VecDeque::with_capacity(period),
            sum_a: 0.0,
            sum_b: 0.0,
            sum_bb: 0.0,
            sum_ab: 0.0,
        })
    }

    /// Look-back window length.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Number of lagged differences in the ADF regression.
    pub const fn adf_lags(&self) -> usize {
        self.adf_lags
    }
}

impl Indicator for Cointegration {
    /// `(a, b)` price pair.
    type Input = (f64, f64);
    type Output = CointegrationOutput;

    fn update(&mut self, input: (f64, f64)) -> Option<CointegrationOutput> {
        let (a, b) = input;
        if !a.is_finite() || !b.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            let (oa, ob) = self.window.pop_front().expect("non-empty");
            self.sum_a -= oa;
            self.sum_b -= ob;
            self.sum_bb -= ob * ob;
            self.sum_ab -= oa * ob;
        }
        self.window.push_back((a, b));
        self.sum_a += a;
        self.sum_b += b;
        self.sum_bb += b * b;
        self.sum_ab += a * b;
        if self.window.len() < self.period {
            return None;
        }
        let n = self.period as f64;
        let mean_a = self.sum_a / n;
        let mean_b = self.sum_b / n;
        let var_b = (self.sum_bb / n - mean_b * mean_b).max(0.0);
        let (hedge_ratio, intercept) = if var_b == 0.0 {
            // A flat `b` window has no defined slope; fall back to a level shift.
            (0.0, mean_a)
        } else {
            let cov = self.sum_ab / n - mean_a * mean_b;
            let beta = cov / var_b;
            (beta, mean_a - beta * mean_b)
        };
        // Build the spread (residual) series over the window, oldest → newest.
        let spreads: Vec<f64> = self
            .window
            .iter()
            .map(|&(ai, bi)| ai - (intercept + hedge_ratio * bi))
            .collect();
        let spread = *spreads.last().expect("window is full");
        let adf_stat = adf_no_constant(&spreads, self.adf_lags);
        Some(CointegrationOutput {
            hedge_ratio,
            spread,
            adf_stat,
        })
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum_a = 0.0;
        self.sum_b = 0.0;
        self.sum_bb = 0.0;
        self.sum_ab = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "Cointegration"
    }
}

/// Solve the linear system `mat·x = rhs` for a small square system by Gaussian
/// elimination, returning `None` if the matrix is (numerically) singular.
///
/// `mat` is row-major and consumed; `rhs` is the right-hand side.
fn solve(mut mat: Vec<Vec<f64>>, mut rhs: Vec<f64>) -> Option<Vec<f64>> {
    let dim = rhs.len();
    for col in 0..dim {
        let pivot = mat[col][col];
        if pivot.abs() < 1e-12 {
            return None;
        }
        let pivot_row = mat[col].clone();
        for row in (col + 1)..dim {
            let factor = mat[row][col] / pivot;
            for (cell, &above) in mat[row].iter_mut().zip(&pivot_row).skip(col) {
                *cell -= factor * above;
            }
            rhs[row] -= factor * rhs[col];
        }
    }
    let mut sol = vec![0.0; dim];
    for row in (0..dim).rev() {
        let known: f64 = mat[row]
            .iter()
            .zip(&sol)
            .skip(row + 1)
            .map(|(coeff, value)| coeff * value)
            .sum();
        sol[row] = (rhs[row] - known) / mat[row][row];
    }
    Some(sol)
}

/// Augmented Dickey–Fuller `t`-statistic on `series`, with `lags` lagged
/// differences and **no** constant or trend term (the Engle–Granger residual
/// form). Returns `0.0` when the regression is degenerate.
///
/// The regression is `Δeₜ = ρ·eₜ₋₁ + Σ γᵢ·Δeₜ₋ᵢ + εₜ`; the reported statistic
/// is `ρ̂ / se(ρ̂)`.
fn adf_no_constant(series: &[f64], lags: usize) -> f64 {
    let len = series.len();
    let num_reg = lags + 1; // regressors: eₜ₋₁ plus `lags` lagged differences
    let first = lags + 1; // first usable observation index
    if len <= first {
        return 0.0;
    }
    let num_obs = len - first;
    if num_obs <= num_reg {
        return 0.0; // need at least one residual degree of freedom
    }
    let regressors = |idx: usize| -> Vec<f64> {
        let mut row = vec![0.0; num_reg];
        row[0] = series[idx - 1];
        for lag in 1..=lags {
            row[lag] = series[idx - lag] - series[idx - lag - 1];
        }
        row
    };
    let mut xtx = vec![vec![0.0; num_reg]; num_reg];
    let mut xty = vec![0.0; num_reg];
    for idx in first..len {
        let diff = series[idx] - series[idx - 1];
        let row = regressors(idx);
        for (ri, &left) in row.iter().enumerate() {
            xty[ri] += left * diff;
            for (ci, &right) in row.iter().enumerate() {
                xtx[ri][ci] += left * right;
            }
        }
    }
    let Some(theta) = solve(xtx.clone(), xty) else {
        return 0.0;
    };
    let rho = theta[0];
    let mut rss = 0.0;
    for idx in first..len {
        let diff = series[idx] - series[idx - 1];
        let pred: f64 = regressors(idx)
            .iter()
            .zip(&theta)
            .map(|(coeff, value)| coeff * value)
            .sum();
        let resid = diff - pred;
        rss += resid * resid;
    }
    let dof = (num_obs - num_reg) as f64;
    let sigma2 = rss / dof;
    // (XᵀX)⁻¹₀₀ from solving XᵀX·x = e₀. `xtx` is the same matrix the first
    // solve already factored successfully, so this one cannot be singular.
    let mut unit = vec![0.0; num_reg];
    unit[0] = 1.0;
    let inverse = solve(xtx, unit).expect("xtx is non-singular: the coefficient solve succeeded");
    let var_rho = sigma2 * inverse[0];
    if var_rho <= 0.0 {
        return 0.0;
    }
    rho / var_rho.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_too_small_period() {
        // period must be >= 2*lags + 4.
        assert!(Cointegration::new(3, 0).is_err()); // needs >= 4
        assert!(Cointegration::new(4, 0).is_ok());
        assert!(Cointegration::new(5, 1).is_err()); // needs >= 6
        assert!(Cointegration::new(6, 1).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let c = Cointegration::new(30, 2).unwrap();
        assert_eq!(c.period(), 30);
        assert_eq!(c.adf_lags(), 2);
        assert_eq!(c.warmup_period(), 30);
        assert_eq!(c.name(), "Cointegration");
    }

    #[test]
    fn adf_guards_and_degenerate_spread() {
        // Series too short for any observation ⇒ 0.
        assert_eq!(adf_no_constant(&[1.0], 1), 0.0);
        // Long enough but too few degrees of freedom ⇒ 0.
        assert_eq!(adf_no_constant(&[1.0, 2.0, 3.0], 1), 0.0);
        // A perfect deterministic AR(1) spread (eₜ = 0.5·eₜ₋₁) is fit exactly,
        // so the residual variance — and hence the t-statistic — is 0.
        let geom: Vec<f64> = (0..8).map(|t| 0.5_f64.powi(t)).collect();
        assert_eq!(adf_no_constant(&geom, 0), 0.0);
    }

    #[test]
    fn recovers_hedge_ratio() {
        // a = 2·b + 5 + small wobble ⇒ β ≈ 2.
        let pairs: Vec<(f64, f64)> = (0..60)
            .map(|t| {
                let b = 100.0 + f64::from(t);
                let a = 2.0 * b + 5.0 + 0.4 * (f64::from(t) * 0.9).sin();
                (a, b)
            })
            .collect();
        let out = Cointegration::new(30, 1)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!(
            (out.hedge_ratio - 2.0).abs() < 0.1,
            "beta {}",
            out.hedge_ratio
        );
    }

    #[test]
    fn stationary_spread_is_strongly_negative() {
        // A clean mean-reverting (sinusoidal) spread ⇒ very negative ADF.
        let pairs: Vec<(f64, f64)> = (0..80)
            .map(|t| {
                let b = 50.0 + 0.5 * f64::from(t);
                let a = 2.0 * b + 1.0 + 0.5 * (f64::from(t) * 0.6).sin();
                (a, b)
            })
            .collect();
        let out = Cointegration::new(40, 1)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!(out.adf_stat < -2.0, "adf {}", out.adf_stat);
    }

    #[test]
    fn perfect_cointegration_has_zero_spread_and_defined_ratio() {
        // a = 2·b + 5 exactly ⇒ residuals all zero ⇒ ADF degenerate ⇒ 0.
        let pairs: Vec<(f64, f64)> = (0..40)
            .map(|t| {
                let b = 100.0 + f64::from(t);
                (2.0 * b + 5.0, b)
            })
            .collect();
        let out = Cointegration::new(20, 1)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(out.hedge_ratio, 2.0, epsilon = 1e-9);
        assert_relative_eq!(out.spread, 0.0, epsilon = 1e-6);
        assert_relative_eq!(out.adf_stat, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn flat_b_falls_back_to_level() {
        // Constant b ⇒ no slope ⇒ hedge ratio 0, spread = a − mean(a).
        let pairs: Vec<(f64, f64)> = (0..20)
            .map(|t| (10.0 + 0.3 * (f64::from(t) * 0.5).sin(), 7.0))
            .collect();
        let out = Cointegration::new(10, 0)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(out.hedge_ratio, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn plain_dickey_fuller_lags_zero() {
        // Exercise the lags = 0 path (1×1 ADF system).
        let pairs: Vec<(f64, f64)> = (0..40)
            .map(|t| {
                let b = 20.0 + 0.4 * f64::from(t);
                let a = 1.5 * b + 0.6 * (f64::from(t) * 0.7).sin();
                (a, b)
            })
            .collect();
        let out = Cointegration::new(20, 0)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!((out.hedge_ratio - 1.5).abs() < 0.1);
        assert!(out.adf_stat < 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut c = Cointegration::new(10, 1).unwrap();
        for t in 0..20 {
            let b = 100.0 + f64::from(t);
            c.update((2.0 * b + (f64::from(t) * 0.5).sin(), b));
        }
        assert!(c.is_ready());
        c.reset();
        assert!(!c.is_ready());
        assert_eq!(c.update((1.0, 1.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let pairs: Vec<(f64, f64)> = (0..80)
            .map(|t| {
                let b = 30.0 + 0.7 * f64::from(t);
                let a = 1.8 * b + 2.0 + 0.5 * (f64::from(t) * 0.4).sin();
                (a, b)
            })
            .collect();
        let batch = Cointegration::new(25, 2).unwrap().batch(&pairs);
        let mut c = Cointegration::new(25, 2).unwrap();
        let streamed: Vec<_> = pairs.iter().map(|p| c.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn non_finite_input_returns_none() {
        let mut c = Cointegration::new(4, 0).unwrap();
        assert_eq!(c.update((f64::NAN, 1.0)), None);
        assert_eq!(c.update((1.0, f64::INFINITY)), None);
        // The rejected ticks leave no trace: a fresh window still warms up.
        assert_eq!(c.update((1.0, 2.0)), None);
        assert_eq!(c.update((2.0, 5.0)), None);
        assert_eq!(c.update((3.0, 7.0)), None);
        assert!(c.update((4.0, 11.0)).is_some());
    }
}
