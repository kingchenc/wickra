//! Kalman-filter dynamic hedge ratio between two series.

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Output of [`KalmanHedgeRatio`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KalmanHedgeRatioOutput {
    /// Current hedge ratio `β` — the filtered slope of `a` on `b`.
    pub hedge_ratio: f64,
    /// Current intercept `α` — the filtered level offset.
    pub intercept: f64,
    /// Forecast error `a − (α + β·b)`: how far the latest `a` sits from the
    /// Kalman-predicted relationship. This is the tradeable spread signal.
    pub spread: f64,
}

/// Dynamic hedge ratio between two series, estimated online with a Kalman filter.
///
/// Each `update` takes one `(a, b)` price pair and treats the linear relation
/// `aₜ = αₜ + βₜ·bₜ + noise` as a state-space model whose hidden state
/// `[βₜ, αₜ]` follows a random walk. The filter updates the state from every
/// observation, so the hedge ratio **adapts continuously** instead of being a
/// flat OLS slope over a fixed window:
///
/// ```text
/// state    xₜ = [βₜ, αₜ],   drifts as a random walk with covariance Vw·I
/// observe  aₜ = [bₜ, 1]·xₜ + εₜ,   Var(εₜ) = observation_var
/// Vw = delta / (1 − delta)
/// ```
///
/// `delta` controls how fast the hedge ratio is allowed to move: a larger
/// `delta` tracks regime changes faster but is noisier; a smaller `delta` is
/// smoother but slower. `observation_var` is the measurement-noise variance.
/// The reported `spread` (the filter's forecast error) is the mean-reverting
/// signal a pairs trade fades — the Kalman analogue of the
/// [`crate::Cointegration`] residual, but with a hedge ratio that breathes.
///
/// The filter emits an estimate from the **first** update (warmup of one bar);
/// early estimates are diffuse and settle as observations accumulate. Each
/// `update` is `O(1)` over the fixed 2×2 covariance.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, KalmanHedgeRatio};
///
/// let mut k = KalmanHedgeRatio::new(1e-2, 1e-3).unwrap();
/// let mut last = None;
/// for t in 0..400 {
///     // `b` sweeps a wide range so the slope and intercept are identifiable.
///     let b = 100.0 + (f64::from(t) * 0.5).sin() * 95.0;
///     last = k.update((2.0 * b + 5.0, b)); // a = 2·b + 5
/// }
/// let out = last.unwrap();
/// assert!((out.hedge_ratio - 2.0).abs() < 0.05);
/// assert!(out.spread.abs() < 0.05);
/// ```
#[derive(Debug, Clone)]
pub struct KalmanHedgeRatio {
    delta: f64,
    transition_var: f64,
    observation_var: f64,
    beta: f64,
    alpha: f64,
    // State covariance, row-major 2×2: [[p00, p01], [p10, p11]].
    cov: [[f64; 2]; 2],
    count: usize,
}

impl KalmanHedgeRatio {
    /// Construct a new Kalman hedge-ratio filter.
    ///
    /// `delta` is the state-drift ratio in `(0, 1)`; `observation_var` is the
    /// measurement-noise variance (`> 0`).
    ///
    /// # Errors
    /// Returns [`Error::InvalidParameter`] if `delta` is not in `(0, 1)` or if
    /// `observation_var` is not strictly positive (both must also be finite).
    pub fn new(delta: f64, observation_var: f64) -> Result<Self> {
        if !delta.is_finite() || delta <= 0.0 || delta >= 1.0 {
            return Err(Error::InvalidParameter {
                message: "kalman hedge ratio needs delta in (0, 1)",
            });
        }
        if !observation_var.is_finite() || observation_var <= 0.0 {
            return Err(Error::InvalidParameter {
                message: "kalman hedge ratio needs observation_var > 0",
            });
        }
        Ok(Self {
            delta,
            transition_var: delta / (1.0 - delta),
            observation_var,
            beta: 0.0,
            alpha: 0.0,
            cov: [[0.0; 2]; 2],
            count: 0,
        })
    }

    /// Configured state-drift ratio `delta`.
    pub const fn delta(&self) -> f64 {
        self.delta
    }

    /// Configured measurement-noise variance.
    pub const fn observation_var(&self) -> f64 {
        self.observation_var
    }
}

impl Indicator for KalmanHedgeRatio {
    type Input = (f64, f64);
    type Output = KalmanHedgeRatioOutput;

    fn update(&mut self, input: (f64, f64)) -> Option<KalmanHedgeRatioOutput> {
        let (a, b) = input;
        if !a.is_finite() || !b.is_finite() {
            return None;
        }
        // Predicted state covariance: add the transition noise to the diagonal
        // (the very first observation starts from a zero prior).
        let mut cov_pred = self.cov;
        if self.count > 0 {
            cov_pred[0][0] += self.transition_var;
            cov_pred[1][1] += self.transition_var;
        }
        // Observation row is F = [b, 1].
        let predicted = self.beta * b + self.alpha;
        let innovation = a - predicted;
        // F·cov_pred  (a 1×2 row).
        let fr0 = b * cov_pred[0][0] + cov_pred[1][0];
        let fr1 = b * cov_pred[0][1] + cov_pred[1][1];
        // Innovation variance S = F·cov_pred·Fᵀ + observation_var ≥ observation_var > 0.
        let innovation_var = fr0 * b + fr1 + self.observation_var;
        // Kalman gain = cov_pred·Fᵀ / S.
        let rft0 = cov_pred[0][0] * b + cov_pred[0][1];
        let rft1 = cov_pred[1][0] * b + cov_pred[1][1];
        let gain0 = rft0 / innovation_var;
        let gain1 = rft1 / innovation_var;
        // State update.
        self.beta += gain0 * innovation;
        self.alpha += gain1 * innovation;
        // Covariance update P = cov_pred − gain·(F·cov_pred).
        self.cov[0][0] = cov_pred[0][0] - gain0 * fr0;
        self.cov[0][1] = cov_pred[0][1] - gain0 * fr1;
        self.cov[1][0] = cov_pred[1][0] - gain1 * fr0;
        self.cov[1][1] = cov_pred[1][1] - gain1 * fr1;
        self.count += 1;
        Some(KalmanHedgeRatioOutput {
            hedge_ratio: self.beta,
            intercept: self.alpha,
            spread: innovation,
        })
    }

    fn reset(&mut self) {
        self.beta = 0.0;
        self.alpha = 0.0;
        self.cov = [[0.0; 2]; 2];
        self.count = 0;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.count >= 1
    }

    fn name(&self) -> &'static str {
        "KalmanHedgeRatio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    #[test]
    fn rejects_bad_parameters() {
        assert!(KalmanHedgeRatio::new(0.0, 1.0).is_err());
        assert!(KalmanHedgeRatio::new(1.0, 1.0).is_err());
        assert!(KalmanHedgeRatio::new(-0.1, 1.0).is_err());
        assert!(KalmanHedgeRatio::new(f64::NAN, 1.0).is_err());
        assert!(KalmanHedgeRatio::new(0.001, 0.0).is_err());
        assert!(KalmanHedgeRatio::new(0.001, -1.0).is_err());
        assert!(KalmanHedgeRatio::new(0.001, f64::INFINITY).is_err());
        assert!(KalmanHedgeRatio::new(0.001, 0.001).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let k = KalmanHedgeRatio::new(0.001, 0.01).unwrap();
        assert_eq!(k.delta(), 0.001);
        assert_eq!(k.observation_var(), 0.01);
        assert_eq!(k.warmup_period(), 1);
        assert_eq!(k.name(), "KalmanHedgeRatio");
        assert!(!k.is_ready());
    }

    #[test]
    fn emits_from_first_update() {
        let mut k = KalmanHedgeRatio::new(0.001, 0.001).unwrap();
        let first = k.update((10.0, 5.0)).unwrap();
        // The diffuse prior leaves the first state at the origin.
        assert_eq!(first.hedge_ratio, 0.0);
        assert_eq!(first.intercept, 0.0);
        assert_eq!(first.spread, 10.0);
        assert!(k.is_ready());
    }

    #[test]
    fn converges_to_static_relationship() {
        // a = 2·b + 5 ⇒ the filter should recover β ≈ 2, α ≈ 5, spread ≈ 0.
        // `b` sweeps a wide range so β and α are jointly identifiable.
        let pairs: Vec<(f64, f64)> = (0..500)
            .map(|t| {
                let b = 100.0 + (f64::from(t) * 0.5).sin() * 95.0;
                (2.0 * b + 5.0, b)
            })
            .collect();
        let out = KalmanHedgeRatio::new(1e-2, 1e-3)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!(
            (out.hedge_ratio - 2.0).abs() < 0.05,
            "beta {}",
            out.hedge_ratio
        );
        assert!((out.intercept - 5.0).abs() < 1.0, "alpha {}", out.intercept);
        assert!(out.spread.abs() < 0.05, "spread {}", out.spread);
    }

    #[test]
    fn tracks_a_changing_hedge_ratio() {
        // Hedge ratio steps from 2 to 3 partway through; the filter should move
        // toward the new ratio.
        let mut pairs: Vec<(f64, f64)> = (0..300)
            .map(|t| {
                let b = 100.0 + (f64::from(t) * 0.5).sin() * 95.0;
                (2.0 * b + 5.0, b)
            })
            .collect();
        pairs.extend((0..300).map(|t| {
            let b = 100.0 + (f64::from(t) * 0.5).cos() * 95.0;
            (3.0 * b + 5.0, b)
        }));
        let out = KalmanHedgeRatio::new(1e-2, 1e-3)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!(out.hedge_ratio > 2.5, "beta {}", out.hedge_ratio);
    }

    #[test]
    fn reset_clears_state() {
        let mut k = KalmanHedgeRatio::new(0.001, 0.001).unwrap();
        for t in 0..50 {
            let b = 100.0 + f64::from(t);
            k.update((2.0 * b, b));
        }
        assert!(k.is_ready());
        k.reset();
        assert!(!k.is_ready());
        let first = k.update((10.0, 5.0)).unwrap();
        assert_eq!(first.hedge_ratio, 0.0);
    }

    #[test]
    fn batch_equals_streaming() {
        let pairs: Vec<(f64, f64)> = (0..120)
            .map(|t| {
                let b = 30.0 + 0.7 * f64::from(t);
                (1.8 * b + 2.0 + (f64::from(t) * 0.4).sin(), b)
            })
            .collect();
        let batch = KalmanHedgeRatio::new(1e-3, 1e-2).unwrap().batch(&pairs);
        let mut k = KalmanHedgeRatio::new(1e-3, 1e-2).unwrap();
        let streamed: Vec<_> = pairs.iter().map(|p| k.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
