//! Granger causality F-statistic: does series `b` help predict series `a`?

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Granger causality of `b` on `a` over a rolling window, as an F-statistic.
///
/// Each `update` takes one `(a, b)` pair. Over the trailing window of `period`
/// observations the indicator fits two autoregressions of `a` and compares them
/// with an F-test:
///
/// ```text
/// restricted:    aₜ = c + Σ φᵢ·aₜ₋ᵢ                       (a's own lags only)
/// unrestricted:  aₜ = c + Σ φᵢ·aₜ₋ᵢ + Σ ψᵢ·bₜ₋ᵢ          (+ b's lags)
/// F = ((RSSᵣ − RSSᵤ) / lag) / (RSSᵤ / (n − 2·lag − 1))
/// ```
///
/// If adding `b`'s lags significantly reduces the residual sum of squares, `b`
/// **Granger-causes** `a`: past values of `b` carry information about the future
/// of `a` beyond what `a`'s own past holds. A **larger** F means stronger
/// predictive causality (lead–lag structure a stat-arb model can trade); a
/// value near `0` means `b` adds nothing. Note Granger causality is purely
/// predictive — it is not structural cause and effect.
///
/// The statistic is `0` when a regression is degenerate — a collinear or flat
/// window makes the normal equations singular. The output is always `≥ 0`.
///
/// Each `update` is `O(period · lag² + lag³)`, bounded by the fixed parameters.
///
/// # Example
///
/// ```
/// use wickra_core::{GrangerCausality, Indicator};
///
/// let mut g = GrangerCausality::new(60, 1).unwrap();
/// let mut last = None;
/// for t in 0..120 {
///     let drive = (f64::from(t) * 0.3).sin();
///     // a echoes b's previous value plus noise ⇒ b Granger-causes a.
///     let b = drive;
///     let a = 0.5 * (f64::from(t.max(1) - 1) * 0.3).sin() + 0.1 * (f64::from(t) * 0.9).cos();
///     last = g.update((a, b));
/// }
/// assert!(last.unwrap() >= 0.0);
/// ```
#[derive(Debug, Clone)]
pub struct GrangerCausality {
    period: usize,
    lag: usize,
    window: VecDeque<(f64, f64)>,
}

impl GrangerCausality {
    /// Construct a new Granger causality test.
    ///
    /// `period` is the look-back window; `lag` is the autoregressive order
    /// (number of own/cross lags in each model).
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `lag < 1` or if `period < 3·lag + 2`
    /// (the smallest window that leaves the unrestricted regression at least one
    /// residual degree of freedom).
    pub fn new(period: usize, lag: usize) -> Result<Self> {
        if lag < 1 {
            return Err(Error::InvalidPeriod {
                message: "granger causality needs lag >= 1",
            });
        }
        if period < 3 * lag + 2 {
            return Err(Error::InvalidPeriod {
                message: "granger causality needs period >= 3*lag + 2",
            });
        }
        Ok(Self {
            period,
            lag,
            window: VecDeque::with_capacity(period),
        })
    }

    /// Configured look-back window.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Configured autoregressive order.
    pub const fn lag(&self) -> usize {
        self.lag
    }
}

impl Indicator for GrangerCausality {
    type Input = (f64, f64);
    type Output = f64;

    fn update(&mut self, input: (f64, f64)) -> Option<f64> {
        if !input.0.is_finite() || !input.1.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(input);
        if self.window.len() < self.period {
            return None;
        }
        let lag = self.lag;
        let a: Vec<f64> = self.window.iter().map(|&(av, _)| av).collect();
        let b: Vec<f64> = self.window.iter().map(|&(_, bv)| bv).collect();
        let num_obs = self.period - lag;

        let mut target = Vec::with_capacity(num_obs);
        let mut restricted = Vec::with_capacity(num_obs);
        let mut unrestricted = Vec::with_capacity(num_obs);
        for k in 0..num_obs {
            let now = lag + k;
            target.push(a[now]);
            let mut row_r = Vec::with_capacity(lag + 1);
            row_r.push(1.0);
            for back in 1..=lag {
                row_r.push(a[now - back]);
            }
            let mut row_u = row_r.clone();
            for back in 1..=lag {
                row_u.push(b[now - back]);
            }
            restricted.push(row_r);
            unrestricted.push(row_u);
        }

        let Some(rss_r) = ols_rss(&restricted, &target, lag + 1) else {
            return Some(0.0);
        };
        let Some(rss_u) = ols_rss(&unrestricted, &target, 2 * lag + 1) else {
            return Some(0.0);
        };
        let dof = (num_obs - (2 * lag + 1)) as f64;
        let numerator = (rss_r - rss_u) / lag as f64;
        let denominator = rss_u / dof;
        Some((numerator / denominator).max(0.0))
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
        "GrangerCausality"
    }
}

/// Residual sum of squares of the OLS fit of `target` on the design `rows`
/// (each a length-`num_reg` regressor vector). Returns `None` if the normal
/// equations are singular.
fn ols_rss(rows: &[Vec<f64>], target: &[f64], num_reg: usize) -> Option<f64> {
    let mut xtx = vec![vec![0.0; num_reg]; num_reg];
    let mut xty = vec![0.0; num_reg];
    for (row, &observed) in rows.iter().zip(target) {
        for (ri, &left) in row.iter().enumerate() {
            xty[ri] += left * observed;
            for (ci, &right) in row.iter().enumerate() {
                xtx[ri][ci] += left * right;
            }
        }
    }
    let theta = solve(xtx, xty)?;
    let mut rss = 0.0;
    for (row, &observed) in rows.iter().zip(target) {
        let pred: f64 = row
            .iter()
            .zip(&theta)
            .map(|(coeff, value)| coeff * value)
            .sum();
        let resid = observed - pred;
        rss += resid * resid;
    }
    Some(rss)
}

/// Solve the linear system `mat·x = rhs` by Gaussian elimination, returning
/// `None` if the matrix is (numerically) singular. `mat` is row-major.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    #[test]
    fn rejects_bad_parameters() {
        assert!(GrangerCausality::new(10, 0).is_err()); // lag must be >= 1
        assert!(GrangerCausality::new(4, 1).is_err()); // period must be >= 3*lag + 2
        assert!(GrangerCausality::new(5, 1).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let g = GrangerCausality::new(60, 2).unwrap();
        assert_eq!(g.period(), 60);
        assert_eq!(g.lag(), 2);
        assert_eq!(g.warmup_period(), 60);
        assert_eq!(g.name(), "GrangerCausality");
        assert!(!g.is_ready());
    }

    #[test]
    fn warmup_returns_none() {
        let mut g = GrangerCausality::new(5, 1).unwrap();
        for t in 0..4 {
            assert_eq!(g.update((f64::from(t), f64::from(t) * 0.5)), None);
        }
        assert!(g.update((4.0, 2.0)).is_some());
        assert!(g.is_ready());
    }

    #[test]
    fn b_leading_a_has_positive_statistic() {
        // a[t] is driven by b[t-1] plus a little of its own past ⇒ b helps.
        let mut prev_drive = 0.0;
        let pairs: Vec<(f64, f64)> = (0..120)
            .map(|t| {
                let drive = (f64::from(t) * 0.3).sin() + 0.4 * (f64::from(t) * 0.11).cos();
                let a = 0.8 * prev_drive + 0.05 * (f64::from(t) * 0.7).sin();
                prev_drive = drive;
                (a, drive)
            })
            .collect();
        let last = GrangerCausality::new(60, 1)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!(last > 1.0, "F {last}");
    }

    #[test]
    fn constant_b_is_singular_and_returns_zero() {
        // b is constant ⇒ its lag columns are collinear with the intercept ⇒
        // the unrestricted normal equations are singular ⇒ 0.
        let pairs: Vec<(f64, f64)> = (0..40)
            .map(|t| (f64::from(t) + (f64::from(t) * 0.6).sin(), 3.0))
            .collect();
        let last = GrangerCausality::new(20, 1)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_eq!(last, 0.0);
    }

    #[test]
    fn constant_a_restricted_singular_returns_zero() {
        // a is constant ⇒ its own lag columns are collinear with the intercept
        // ⇒ the restricted normal equations are singular ⇒ 0.
        let pairs: Vec<(f64, f64)> = (0..40).map(|t| (5.0, (f64::from(t) * 0.4).sin())).collect();
        let last = GrangerCausality::new(20, 1)
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
        let mut g = GrangerCausality::new(8, 1).unwrap();
        for t in 0..12 {
            g.update((
                f64::from(t) + (f64::from(t) * 0.7).sin(),
                (f64::from(t) * 0.3).cos(),
            ));
        }
        assert!(g.is_ready());
        g.reset();
        assert!(!g.is_ready());
        assert_eq!(g.update((1.0, 1.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let pairs: Vec<(f64, f64)> = (0..80)
            .map(|t| {
                let b = (f64::from(t) * 0.4).sin();
                (
                    0.6 * (f64::from(t.max(1) - 1) * 0.4).sin() + 0.1 * f64::from(t % 3),
                    b,
                )
            })
            .collect();
        let batch = GrangerCausality::new(30, 2).unwrap().batch(&pairs);
        let mut g = GrangerCausality::new(30, 2).unwrap();
        let streamed: Vec<_> = pairs.iter().map(|p| g.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn non_finite_input_returns_none() {
        let mut g = GrangerCausality::new(5, 1).unwrap();
        assert_eq!(g.update((f64::NAN, 1.0)), None);
        assert_eq!(g.update((1.0, f64::INFINITY)), None);
        // The rejected ticks leave no trace: a fresh window still warms up.
        for t in 0..4 {
            assert_eq!(g.update((f64::from(t), f64::from(t) * 0.5)), None);
        }
        assert!(g.update((4.0, 2.0)).is_some());
    }
}
