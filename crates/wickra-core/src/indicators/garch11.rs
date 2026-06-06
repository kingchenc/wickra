//! GARCH(1,1) — conditional volatility with a long-run-variance anchor.

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// GARCH(1,1) conditional volatility — the square root of the
/// generalized-autoregressive-conditional-heteroskedasticity variance recursion.
///
/// ```text
/// r_t  = ln(price_t / price_{t−1})
/// σ²_t = ω + α · r²_{t−1} + β · σ²_{t−1}
/// out  = √σ²_t
/// ```
///
/// GARCH(1,1) (Bollerslev 1986) generalizes the
/// [`EwmaVolatility`](crate::EwmaVolatility) recursion by adding a constant `ω`,
/// which pins the process to a finite long-run (unconditional) variance
/// `ω / (1 − α − β)`. The `α` term gives weight to the latest squared return
/// (the "ARCH" shock) and `β` to the previous variance (the "GARCH"
/// persistence). When `ω = 0` and `α + β = 1` the model degenerates to EWMA; a
/// proper GARCH keeps `ω > 0` and `α + β < 1` so volatility mean-reverts rather
/// than drifting.
///
/// The recursion is seeded with the unconditional variance (`σ²₁ = ω / (1 − α −
/// β)`) and emits from the first log return onward. Unlike EWMA — which decays to
/// zero on a flat series — a flat series here mean-reverts toward `ω / (1 − β)`
/// (the `α`-term vanishes but the `ω` floor and the `β` carry remain), so the
/// output is always strictly positive. Each `update` is O(1).
///
/// Non-finite and non-positive prices are ignored (the log return would be
/// undefined): the tick is dropped, state is left untouched, and the last value
/// is returned.
///
/// # Example
///
/// ```
/// use wickra_core::{Garch11, Indicator};
///
/// // Typical equity daily estimate.
/// let mut indicator = Garch11::new(0.000_002, 0.10, 0.88).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Garch11 {
    omega: f64,
    alpha: f64,
    beta: f64,
    unconditional: f64,
    prev_price: Option<f64>,
    /// `(σ²_{t−1}, r²_{t−1})` — previous variance and previous squared return.
    state: Option<(f64, f64)>,
    last: Option<f64>,
}

impl Garch11 {
    /// Construct a new GARCH(1,1) indicator from its three parameters.
    ///
    /// `omega` (`ω`) is the constant variance floor, `alpha` (`α`) the weight on
    /// the latest squared return, and `beta` (`β`) the persistence of the
    /// previous variance.
    ///
    /// # Errors
    /// Returns [`Error::InvalidParameter`] unless every parameter is finite,
    /// `omega > 0`, `alpha >= 0`, `beta >= 0`, and `alpha + beta < 1` (the
    /// covariance-stationarity condition that gives a finite long-run variance).
    pub fn new(omega: f64, alpha: f64, beta: f64) -> Result<Self> {
        if !omega.is_finite() || !alpha.is_finite() || !beta.is_finite() {
            return Err(Error::InvalidParameter {
                message: "GARCH(1,1) parameters must be finite",
            });
        }
        if omega <= 0.0 {
            return Err(Error::InvalidParameter {
                message: "GARCH(1,1) omega must be > 0",
            });
        }
        if alpha < 0.0 || beta < 0.0 {
            return Err(Error::InvalidParameter {
                message: "GARCH(1,1) alpha and beta must be >= 0",
            });
        }
        if alpha + beta >= 1.0 {
            return Err(Error::InvalidParameter {
                message: "GARCH(1,1) requires alpha + beta < 1 (covariance stationarity)",
            });
        }
        Ok(Self {
            omega,
            alpha,
            beta,
            unconditional: omega / (1.0 - alpha - beta),
            prev_price: None,
            state: None,
            last: None,
        })
    }

    /// Configured `(omega, alpha, beta)`.
    pub const fn params(&self) -> (f64, f64, f64) {
        (self.omega, self.alpha, self.beta)
    }

    /// Long-run (unconditional) variance `ω / (1 − α − β)`.
    pub const fn unconditional_variance(&self) -> f64 {
        self.unconditional
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for Garch11 {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        // Non-finite / non-positive prices are skipped: `ln(input / prev)` is
        // undefined, so the tick must not enter the variance recursion.
        if !input.is_finite() || input <= 0.0 {
            return self.last;
        }
        let Some(prev) = self.prev_price else {
            self.prev_price = Some(input);
            return None;
        };
        self.prev_price = Some(input);
        // `prev` came from `self.prev_price`, gated by the guard above, so it is
        // finite and positive — the log return is always well-defined.
        let r = (input / prev).ln();
        let r_sq = r * r;
        let var = match self.state {
            // Seed the recursion with the unconditional variance.
            None => self.unconditional,
            Some((prev_var, prev_r_sq)) => {
                self.omega + self.alpha * prev_r_sq + self.beta * prev_var
            }
        };
        self.state = Some((var, r_sq));
        // `var` is `omega (> 0) + non-negative terms`, so it is strictly
        // positive — the square root is always well-defined.
        let vol = var.sqrt();
        self.last = Some(vol);
        Some(vol)
    }

    fn reset(&mut self) {
        self.prev_price = None;
        self.state = None;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // The first log return needs a previous price; the estimate is seeded
        // with the unconditional variance and emitted on that first return.
        2
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "Garch11"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_invalid_params() {
        assert!(matches!(
            Garch11::new(0.0, 0.1, 0.8),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            Garch11::new(-1.0, 0.1, 0.8),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            Garch11::new(0.001, -0.1, 0.8),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            Garch11::new(0.001, 0.1, -0.8),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            Garch11::new(0.001, 0.5, 0.5),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            Garch11::new(f64::NAN, 0.1, 0.8),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            Garch11::new(0.001, f64::INFINITY, 0.8),
            Err(Error::InvalidParameter { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let g = Garch11::new(0.001, 0.1, 0.85).unwrap();
        assert_eq!(g.params(), (0.001, 0.1, 0.85));
        assert_relative_eq!(g.unconditional_variance(), 0.001 / 0.05, epsilon = 1e-12);
        assert_eq!(g.warmup_period(), 2);
        assert_eq!(g.name(), "Garch11");
        assert!(!g.is_ready());
        assert_eq!(g.value(), None);
    }

    #[test]
    fn first_emission_is_unconditional() {
        // The first log return emits the seed = sqrt(unconditional variance),
        // independent of the return value.
        let g = Garch11::new(0.002, 0.1, 0.85);
        let mut g = g.unwrap();
        assert_eq!(g.update(100.0), None);
        let out = g.update(110.0).unwrap();
        assert_relative_eq!(out, (0.002_f64 / 0.05).sqrt(), epsilon = 1e-12);
    }

    #[test]
    fn known_value() {
        // σ²₁ = uncond; σ²₂ = ω + α·r1² + β·uncond.
        let (omega, alpha, beta) = (0.002, 0.1, 0.85);
        let mut g = Garch11::new(omega, alpha, beta).unwrap();
        let out = g.batch(&[100.0, 110.0, 99.0]);
        let uncond = omega / (1.0 - alpha - beta);
        let r1 = (110.0_f64 / 100.0).ln();
        assert_relative_eq!(out[1].unwrap(), uncond.sqrt(), epsilon = 1e-12);
        let var2 = omega + alpha * r1 * r1 + beta * uncond;
        assert_relative_eq!(out[2].unwrap(), var2.sqrt(), epsilon = 1e-12);
    }

    #[test]
    fn flat_series_converges_to_long_run() {
        // With zero returns the alpha term vanishes; the variance mean-reverts
        // to the fixed point ω / (1 − β), NOT to zero (the key GARCH/EWMA
        // distinction).
        let (omega, beta) = (0.002, 0.85);
        let mut g = Garch11::new(omega, 0.10, beta).unwrap();
        let out = g.batch(&[100.0; 400]);
        let fixed_point = (omega / (1.0 - beta)).sqrt();
        assert_relative_eq!(out.last().unwrap().unwrap(), fixed_point, epsilon = 1e-9);
    }

    #[test]
    fn output_is_strictly_positive() {
        let mut g = Garch11::new(0.000_002, 0.1, 0.88).unwrap();
        let prices: Vec<f64> = (1..=200)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 12.0)
            .collect();
        for v in g.batch(&prices).into_iter().flatten() {
            assert!(
                v > 0.0,
                "GARCH volatility must be strictly positive, got {v}"
            );
        }
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut g = Garch11::new(0.001, 0.1, 0.85).unwrap();
        let out = g.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        let last = *out.last().unwrap();
        assert!(last.is_some());
        assert_eq!(g.update(f64::NAN), last);
        assert_eq!(g.update(f64::INFINITY), last);
    }

    #[test]
    fn skips_non_positive_prices() {
        let mut g = Garch11::new(0.001, 0.1, 0.85).unwrap();
        let warmup = g.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        let baseline = warmup.last().copied().flatten().expect("warmed up");
        assert_eq!(g.update(-5.0), Some(baseline));
        assert_eq!(g.update(0.0), Some(baseline));
        // State untouched: a clone advanced by the same real tick agrees.
        let mut control = g.clone();
        let after = g.update(21.0).expect("ready");
        assert_eq!(control.update(21.0).expect("ready"), after);
    }

    #[test]
    fn skips_non_positive_before_first_price() {
        let mut g = Garch11::new(0.001, 0.1, 0.85).unwrap();
        assert_eq!(g.update(0.0), None);
        assert_eq!(g.update(f64::NAN), None);
        assert_eq!(g.update(100.0), None);
        assert!(g.update(110.0).is_some());
    }

    #[test]
    fn reset_clears_state() {
        let mut g = Garch11::new(0.001, 0.1, 0.85).unwrap();
        g.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        assert!(g.is_ready());
        g.reset();
        assert!(!g.is_ready());
        assert_eq!(g.value(), None);
        assert_eq!(g.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 9.0)
            .collect();
        let batch = Garch11::new(0.000_002, 0.1, 0.88).unwrap().batch(&prices);
        let mut b = Garch11::new(0.000_002, 0.1, 0.88).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
