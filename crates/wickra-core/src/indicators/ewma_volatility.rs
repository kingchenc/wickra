//! EWMA Volatility — `RiskMetrics` exponentially-weighted volatility.

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// EWMA Volatility — the `RiskMetrics` exponentially-weighted estimate of the
/// volatility of log returns.
///
/// ```text
/// r_t  = ln(price_t / price_{t−1})
/// σ²_t = λ · σ²_{t−1} + (1 − λ) · r²_t
/// EWMA = √σ²_t
/// ```
///
/// Unlike [`HistoricalVolatility`](crate::HistoricalVolatility) — an equally
/// weighted, mean-centred sample standard deviation over a fixed window — the
/// EWMA estimator weights recent squared returns geometrically by the decay
/// factor `λ`. The most recent return carries weight `1 − λ`, the one before it
/// `λ(1 − λ)`, and so on, so the estimate reacts to a volatility shock
/// immediately and then forgets it at rate `λ`. This is the J.P. Morgan
/// `RiskMetrics` one-parameter model; the standard daily decay is `λ = 0.94`
/// (monthly `0.97`). No mean is subtracted: squared returns *are* the variance
/// contribution, which matches the `RiskMetrics` assumption of a zero conditional
/// mean over short horizons.
///
/// The recursion is seeded with the first squared return (`σ²₁ = r²₁`) and emits
/// from the first return onward, so the very first reading is a one-observation
/// estimate that the decay then refines. Each `update` is O(1).
///
/// Non-finite and non-positive prices are ignored (the log return would be
/// undefined): the tick is dropped, state is left untouched, and the last value
/// is returned.
///
/// # Example
///
/// ```
/// use wickra_core::{EwmaVolatility, Indicator};
///
/// let mut indicator = EwmaVolatility::new(0.94).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct EwmaVolatility {
    lambda: f64,
    prev_price: Option<f64>,
    /// Exponentially-weighted variance of log returns; `None` until seeded.
    variance: Option<f64>,
    last: Option<f64>,
}

impl EwmaVolatility {
    /// Construct a new EWMA-volatility indicator.
    ///
    /// `lambda` is the decay factor, strictly between `0` and `1` (`RiskMetrics`
    /// uses `0.94` for daily data). Larger `lambda` means a longer memory and a
    /// smoother estimate.
    ///
    /// # Errors
    /// Returns [`Error::InvalidParameter`] if `lambda` is not finite or not in
    /// the open interval `(0, 1)`.
    pub fn new(lambda: f64) -> Result<Self> {
        if !lambda.is_finite() || lambda <= 0.0 || lambda >= 1.0 {
            return Err(Error::InvalidParameter {
                message: "EWMA volatility lambda must be in the open interval (0, 1)",
            });
        }
        Ok(Self {
            lambda,
            prev_price: None,
            variance: None,
            last: None,
        })
    }

    /// Configured decay factor.
    pub const fn lambda(&self) -> f64 {
        self.lambda
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for EwmaVolatility {
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
        let var = match self.variance {
            // Seed the recursion with the first squared return.
            None => r * r,
            Some(prev_var) => self.lambda * prev_var + (1.0 - self.lambda) * r * r,
        };
        self.variance = Some(var);
        // `var` is a convex combination of non-negative terms, but rounding can
        // leave a tiny negative residual when every return is ~0; clamp first.
        let vol = var.max(0.0).sqrt();
        self.last = Some(vol);
        Some(vol)
    }

    fn reset(&mut self) {
        self.prev_price = None;
        self.variance = None;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // The first log return needs a previous price; the estimate is seeded
        // and emitted on that first return.
        2
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "EwmaVolatility"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_invalid_lambda() {
        for bad in [0.0, 1.0, -0.5, 1.5, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                EwmaVolatility::new(bad),
                Err(Error::InvalidParameter { .. })
            ));
        }
    }

    #[test]
    fn accessors_and_metadata() {
        let ewma = EwmaVolatility::new(0.94).unwrap();
        assert_relative_eq!(ewma.lambda(), 0.94);
        assert_eq!(ewma.warmup_period(), 2);
        assert_eq!(ewma.name(), "EwmaVolatility");
        assert!(!ewma.is_ready());
        assert_eq!(ewma.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut ewma = EwmaVolatility::new(0.94).unwrap();
        assert_eq!(ewma.update(100.0), None);
        let out = ewma.update(110.0);
        assert!(out.is_some());
        assert!(ewma.is_ready());
    }

    #[test]
    fn known_value() {
        // r1 = ln(110/100), r2 = ln(99/110). Seed σ²₁ = r1²; then
        // σ²₂ = λ·r1² + (1−λ)·r2².
        let lambda = 0.94;
        let mut ewma = EwmaVolatility::new(lambda).unwrap();
        let out = ewma.batch(&[100.0, 110.0, 99.0]);
        let r1 = (110.0_f64 / 100.0).ln();
        let r2 = (99.0_f64 / 110.0).ln();
        assert_relative_eq!(out[1].unwrap(), r1.abs(), epsilon = 1e-12);
        let var2 = lambda * r1 * r1 + (1.0 - lambda) * r2 * r2;
        assert_relative_eq!(out[2].unwrap(), var2.sqrt(), epsilon = 1e-12);
    }

    #[test]
    fn constant_series_yields_zero() {
        let mut ewma = EwmaVolatility::new(0.9).unwrap();
        for v in ewma.batch(&[100.0; 40]).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn output_is_non_negative() {
        let mut ewma = EwmaVolatility::new(0.94).unwrap();
        let prices: Vec<f64> = (1..=200)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 12.0)
            .collect();
        for v in ewma.batch(&prices).into_iter().flatten() {
            assert!(v >= 0.0, "EWMA volatility must be non-negative, got {v}");
        }
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut ewma = EwmaVolatility::new(0.94).unwrap();
        let out = ewma.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        let last = *out.last().unwrap();
        assert!(last.is_some());
        assert_eq!(ewma.update(f64::NAN), last);
        assert_eq!(ewma.update(f64::INFINITY), last);
    }

    #[test]
    fn skips_non_positive_prices() {
        let mut ewma = EwmaVolatility::new(0.94).unwrap();
        let warmup = ewma.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        let baseline = warmup.last().copied().flatten().expect("warmed up");
        assert_eq!(ewma.update(-5.0), Some(baseline));
        assert_eq!(ewma.update(0.0), Some(baseline));
        // State untouched: a clone advanced by the same real tick agrees.
        let mut control = ewma.clone();
        let after = ewma.update(21.0).expect("ready");
        assert_eq!(control.update(21.0).expect("ready"), after);
    }

    #[test]
    fn skips_non_positive_before_first_price() {
        // The skip guard fires before any previous price exists.
        let mut ewma = EwmaVolatility::new(0.94).unwrap();
        assert_eq!(ewma.update(0.0), None);
        assert_eq!(ewma.update(f64::NAN), None);
        assert_eq!(ewma.update(100.0), None);
        assert!(ewma.update(110.0).is_some());
    }

    #[test]
    fn reset_clears_state() {
        let mut ewma = EwmaVolatility::new(0.94).unwrap();
        ewma.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        assert!(ewma.is_ready());
        ewma.reset();
        assert!(!ewma.is_ready());
        assert_eq!(ewma.value(), None);
        assert_eq!(ewma.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 9.0)
            .collect();
        let batch = EwmaVolatility::new(0.94).unwrap().batch(&prices);
        let mut b = EwmaVolatility::new(0.94).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
