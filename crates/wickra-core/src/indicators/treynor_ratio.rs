//! Rolling Treynor Ratio.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Rolling Treynor Ratio.
///
/// Each `update` receives one `(asset_return, benchmark_return)` pair. Over
/// the trailing window of `period` pairs:
///
/// ```text
/// cov_ab = (1/n) · Σ a·b − ā·b̄
/// var_b  = (1/n) · Σ b² − b̄²
/// Beta   = cov_ab / var_b
/// Treynor = (mean(asset) − risk_free) / Beta
/// ```
///
/// Treynor is Sharpe's market-risk cousin: it divides excess return by the
/// asset's sensitivity to the benchmark (Beta) rather than by the asset's
/// own volatility. Useful for diversified portfolios where idiosyncratic
/// volatility has been mostly diversified away and the dominant remaining
/// risk is systematic / market exposure.
///
/// A flat benchmark window has zero variance and the indicator returns
/// `0.0` rather than `NaN`. A near-zero `Beta` makes the ratio explode by
/// construction; callers should treat extreme values with the usual care.
///
/// Each `update` is O(1) — running sums maintain `Σa`, `Σb`, `Σb²`, `Σa·b`
/// as the window slides.
#[derive(Debug, Clone)]
pub struct TreynorRatio {
    period: usize,
    risk_free: f64,
    window: VecDeque<(f64, f64)>,
    sum_a: f64,
    sum_b: f64,
    sum_bb: f64,
    sum_ab: f64,
}

impl TreynorRatio {
    /// Construct a new rolling Treynor Ratio.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `period < 2`.
    pub fn new(period: usize, risk_free: f64) -> Result<Self> {
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "treynor ratio needs period >= 2",
            });
        }
        Ok(Self {
            period,
            risk_free,
            window: VecDeque::with_capacity(period),
            sum_a: 0.0,
            sum_b: 0.0,
            sum_bb: 0.0,
            sum_ab: 0.0,
        })
    }

    /// Configured window length.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Configured per-period risk-free rate.
    pub const fn risk_free(&self) -> f64 {
        self.risk_free
    }
}

impl Indicator for TreynorRatio {
    type Input = (f64, f64);
    type Output = f64;

    fn update(&mut self, input: (f64, f64)) -> Option<f64> {
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
        let var_b = (self.sum_bb / n) - mean_b * mean_b;
        if var_b <= 0.0 {
            return Some(0.0);
        }
        let cov_ab = (self.sum_ab / n) - mean_a * mean_b;
        let beta = cov_ab / var_b;
        if beta == 0.0 {
            return Some(0.0);
        }
        Some((mean_a - self.risk_free) / beta)
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
        "TreynorRatio"
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
            TreynorRatio::new(1, 0.0),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let t = TreynorRatio::new(20, 0.001).unwrap();
        assert_eq!(t.period(), 20);
        assert_relative_eq!(t.risk_free(), 0.001, epsilon = 1e-12);
        assert_eq!(t.name(), "TreynorRatio");
        assert_eq!(t.warmup_period(), 20);
    }

    #[test]
    fn reference_beta_two_payoff() {
        // a_i = 2 * b_i with non-zero mean.
        // Beta should be 2; mean_a = 2 * mean_b; Treynor = mean_b.
        let mut t = TreynorRatio::new(20, 0.0).unwrap();
        let inputs: Vec<(f64, f64)> = (1..=20)
            .map(|i| (2.0 * f64::from(i) * 0.01, f64::from(i) * 0.01))
            .collect();
        let out = t.batch(&inputs);
        let last = out[19].unwrap();
        let expected = inputs.iter().map(|(_, b)| *b).sum::<f64>() / 20.0;
        assert_relative_eq!(last, expected, epsilon = 1e-9);
    }

    #[test]
    fn flat_benchmark_yields_zero() {
        // Benchmark all 0 -> var_b = 0 -> indicator returns 0.0.
        let mut t = TreynorRatio::new(4, 0.0).unwrap();
        let out = t.batch(&[(0.01, 0.0), (0.02, 0.0), (-0.01, 0.0), (0.03, 0.0)]);
        assert_eq!(out[3], Some(0.0));
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut t = TreynorRatio::new(3, 0.0).unwrap();
        assert_eq!(t.update((f64::NAN, 0.0)), None);
        assert_eq!(t.update((0.0, f64::INFINITY)), None);
    }

    #[test]
    fn reset_clears_state() {
        let mut t = TreynorRatio::new(3, 0.0).unwrap();
        t.batch(&[(0.01, 0.005), (0.02, 0.01), (-0.01, -0.005)]);
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update((0.01, 0.005)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let inputs: Vec<(f64, f64)> = (0..50)
            .map(|i| {
                let b = (f64::from(i) * 0.2).sin() * 0.01;
                (1.5 * b + 0.001, b)
            })
            .collect();
        let batch = TreynorRatio::new(10, 0.0).unwrap().batch(&inputs);
        let mut s = TreynorRatio::new(10, 0.0).unwrap();
        let streamed: Vec<_> = inputs.iter().map(|x| s.update(*x)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn zero_beta_returns_zero() {
        // Constant asset returns vs varying benchmark force cov(a,b) = 0,
        // hence beta = 0 — the explicit zero-beta short-circuit.
        let mut t = TreynorRatio::new(4, 0.0).unwrap();
        let pairs: [(f64, f64); 4] = [(0.01, 0.005), (0.01, -0.002), (0.01, 0.001), (0.01, 0.003)];
        let mut last = None;
        for p in pairs {
            last = t.update(p);
        }
        assert_eq!(last, Some(0.0));
    }
}
