//! Price Momentum Oscillator (`DecisionPoint`).

use crate::error::{Error, Result};
use crate::traits::Indicator;

use super::Ema;

/// Price Momentum Oscillator — Carl Swenlin's `DecisionPoint` PMO line.
///
/// PMO is a doubly-smoothed rate of change. The 1-bar percentage change is
/// smoothed once, scaled by `10`, then smoothed again:
///
/// ```text
/// roc_t       = (price_t / price_{t−1} − 1) · 100
/// smoothed_t  = customEMA(roc, smoothing1)_t
/// PMO_t       = customEMA(10 · smoothed, smoothing2)_t
/// ```
///
/// `customEMA` is the `DecisionPoint` smoothing: an exponential average whose
/// smoothing constant is `2 / period` (not the textbook `2 / (period + 1)`),
/// seeded from the very first value. The conventional periods are `35` and
/// `20`. The classic PMO **signal line** is simply a 10-period EMA of this
/// PMO line — compose it with [`Chain`](crate::Chain) and an [`Ema`] if you
/// need it.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Pmo};
///
/// let mut indicator = Pmo::new(35, 20).unwrap();
/// let mut last = None;
/// for i in 0..120 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Pmo {
    smoothing1: usize,
    smoothing2: usize,
    prev_price: Option<f64>,
    ema1: Ema,
    ema2: Ema,
    current: Option<f64>,
}

impl Pmo {
    /// Construct a new PMO with the two smoothing periods.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if either period is `0`, or
    /// [`Error::InvalidPeriod`] if either is `1` (the smoothing constant
    /// `2 / period` must not exceed `1`).
    pub fn new(smoothing1: usize, smoothing2: usize) -> Result<Self> {
        if smoothing1 == 0 || smoothing2 == 0 {
            return Err(Error::PeriodZero);
        }
        if smoothing1 < 2 || smoothing2 < 2 {
            return Err(Error::InvalidPeriod {
                message: "PMO smoothing periods must be >= 2",
            });
        }
        Ok(Self {
            smoothing1,
            smoothing2,
            prev_price: None,
            ema1: Ema::with_alpha(2.0 / smoothing1 as f64)?,
            ema2: Ema::with_alpha(2.0 / smoothing2 as f64)?,
            current: None,
        })
    }

    /// The `(smoothing1, smoothing2)` periods.
    pub const fn periods(&self) -> (usize, usize) {
        (self.smoothing1, self.smoothing2)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.current
    }
}

impl Indicator for Pmo {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            // Non-finite input is ignored; state is left untouched.
            return self.current;
        }
        let Some(prev) = self.prev_price else {
            self.prev_price = Some(input);
            return None;
        };
        self.prev_price = Some(input);

        let roc = if prev == 0.0 {
            // Undefined ratio against a zero price: treat momentum as flat.
            0.0
        } else {
            (input / prev - 1.0) * 100.0
        };
        let smoothed = self.ema1.update(roc)?;
        let pmo = self.ema2.update(10.0 * smoothed)?;
        self.current = Some(pmo);
        Some(pmo)
    }

    fn reset(&mut self) {
        self.prev_price = None;
        self.ema1.reset();
        self.ema2.reset();
        self.current = None;
    }

    fn warmup_period(&self) -> usize {
        // The first ROC needs a previous price; both customEMAs seed from
        // their first input, so the first PMO lands on the second update.
        2
    }

    fn is_ready(&self) -> bool {
        self.current.is_some()
    }

    fn name(&self) -> &'static str {
        "PMO"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(Pmo::new(0, 20), Err(Error::PeriodZero)));
        assert!(matches!(Pmo::new(35, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn new_rejects_period_one() {
        assert!(matches!(Pmo::new(1, 20), Err(Error::InvalidPeriod { .. })));
        assert!(matches!(Pmo::new(35, 1), Err(Error::InvalidPeriod { .. })));
    }

    /// Cover the const accessors `periods` / `value` (lines 76-83) and the
    /// Indicator-impl `name` body (130-132). `warmup_period` is already
    /// covered by `first_emission_at_second_update`.
    #[test]
    fn accessors_and_metadata() {
        let mut pmo = Pmo::new(35, 20).unwrap();
        assert_eq!(pmo.periods(), (35, 20));
        assert_eq!(pmo.name(), "PMO");
        assert_eq!(pmo.value(), None);
        pmo.update(100.0);
        pmo.update(101.0);
        assert!(pmo.value().is_some());
    }

    /// Cover the `prev == 0.0` defensive branch (line 103). The PMO ROC
    /// divides by the previous price; existing tests use prices ≈ 100, so
    /// the divide-by-zero guard never fired. Feed a single zero price
    /// followed by a positive price and assert the first emitted PMO is
    /// the flat-momentum value (the wrapping `customEMA` of `0.0` is 0.0
    /// regardless of smoothing factor on its first input).
    #[test]
    fn zero_previous_price_treats_roc_as_flat() {
        let mut pmo = Pmo::new(2, 2).unwrap();
        // Seed prev_price = 0.
        assert_eq!(pmo.update(0.0), None);
        // Next bar: prev == 0 hits the fallback returning roc = 0.0; the
        // doubly-smoothed PMO seeds at 0.0 (10 * 0 = 0 through both EMAs).
        let out = pmo.update(50.0).expect("emits");
        assert_eq!(out, 0.0);
    }

    #[test]
    fn first_emission_at_second_update() {
        let mut pmo = Pmo::new(35, 20).unwrap();
        assert_eq!(pmo.warmup_period(), 2);
        assert_eq!(pmo.update(100.0), None);
        assert!(pmo.update(101.0).is_some());
    }

    #[test]
    fn constant_series_yields_zero() {
        // Flat prices -> ROC is always 0 -> both smoothings stay at 0.
        let mut pmo = Pmo::new(35, 20).unwrap();
        let out = pmo.batch(&[100.0; 60]);
        for v in out.iter().skip(2).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn steady_uptrend_is_positive() {
        let mut pmo = Pmo::new(35, 20).unwrap();
        let prices: Vec<f64> = (1..=120).map(|i| 100.0 * 1.01_f64.powi(i)).collect();
        let out = pmo.batch(&prices);
        let last = out.iter().rev().flatten().next().unwrap();
        assert!(
            *last > 0.0,
            "steady uptrend PMO should be positive, got {last}"
        );
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut pmo = Pmo::new(35, 20).unwrap();
        let out = pmo.batch(&(1..=60).map(f64::from).collect::<Vec<_>>());
        let last = *out.last().unwrap();
        assert!(last.is_some());
        assert_eq!(pmo.update(f64::NAN), last);
        assert_eq!(pmo.update(f64::INFINITY), last);
    }

    #[test]
    fn reset_clears_state() {
        let mut pmo = Pmo::new(35, 20).unwrap();
        pmo.batch(&(1..=60).map(f64::from).collect::<Vec<_>>());
        assert!(pmo.is_ready());
        pmo.reset();
        assert!(!pmo.is_ready());
        assert_eq!(pmo.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 8.0)
            .collect();
        let batch = Pmo::new(35, 20).unwrap().batch(&prices);
        let mut b = Pmo::new(35, 20).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
