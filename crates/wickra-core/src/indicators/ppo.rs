//! Percentage Price Oscillator.

use crate::error::{Error, Result};
use crate::traits::Indicator;

use super::Ema;

/// Percentage Price Oscillator — MACD expressed as a percentage.
///
/// PPO is the gap between a fast and a slow EMA, divided by the slow EMA and
/// scaled to a percentage:
///
/// ```text
/// PPO = 100 · (EMA_fast − EMA_slow) / EMA_slow
/// ```
///
/// Dividing by the slow EMA makes PPO **scale-free**: a `PPO` of `1.5` means
/// "the fast EMA is 1.5 % above the slow EMA" on any instrument, so PPO
/// readings *are* comparable across assets — unlike the raw price-unit
/// [`MacdIndicator`](crate::MacdIndicator). The classic PPO **signal line** is
/// a 9-period EMA of this PPO line; compose it with [`Chain`](crate::Chain)
/// and an [`Ema`] if you need it.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Ppo};
///
/// let mut indicator = Ppo::new(12, 26).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Ppo {
    fast: usize,
    slow: usize,
    ema_fast: Ema,
    ema_slow: Ema,
    current: Option<f64>,
}

impl Ppo {
    /// Construct a new PPO with the `fast` and `slow` EMA periods.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if either period is `0`, or
    /// [`Error::InvalidPeriod`] if `fast >= slow`.
    pub fn new(fast: usize, slow: usize) -> Result<Self> {
        if fast == 0 || slow == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "PPO fast period must be < slow period",
            });
        }
        Ok(Self {
            fast,
            slow,
            ema_fast: Ema::new(fast)?,
            ema_slow: Ema::new(slow)?,
            current: None,
        })
    }

    /// The `(fast, slow)` periods.
    pub const fn periods(&self) -> (usize, usize) {
        (self.fast, self.slow)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.current
    }
}

impl Indicator for Ppo {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            // Non-finite input is ignored; the EMAs are not advanced.
            return self.current;
        }
        let fast = self.ema_fast.update(input);
        let slow = self.ema_slow.update(input);
        match (fast, slow) {
            (Some(f), Some(s)) => {
                let ppo = if s == 0.0 {
                    // Undefined ratio against a zero slow EMA: report flat.
                    0.0
                } else {
                    100.0 * (f - s) / s
                };
                self.current = Some(ppo);
                Some(ppo)
            }
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.ema_fast.reset();
        self.ema_slow.reset();
        self.current = None;
    }

    fn warmup_period(&self) -> usize {
        // The slow EMA is the last to seed.
        self.slow
    }

    fn is_ready(&self) -> bool {
        self.current.is_some()
    }

    fn name(&self) -> &'static str {
        "PPO"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(Ppo::new(0, 26), Err(Error::PeriodZero)));
        assert!(matches!(Ppo::new(12, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn new_rejects_fast_not_less_than_slow() {
        assert!(matches!(Ppo::new(26, 12), Err(Error::InvalidPeriod { .. })));
        assert!(matches!(Ppo::new(12, 12), Err(Error::InvalidPeriod { .. })));
    }

    /// Cover the const accessors `periods` / `value` (lines 71-78) and the
    /// Indicator-impl `name` body (122-124). `warmup_period` is already
    /// covered by `first_emission_at_warmup_period`.
    #[test]
    fn accessors_and_metadata() {
        let mut ppo = Ppo::new(12, 26).unwrap();
        assert_eq!(ppo.periods(), (12, 26));
        assert_eq!(ppo.name(), "PPO");
        assert_eq!(ppo.value(), None);
        for i in 1..=26 {
            ppo.update(f64::from(i));
        }
        assert!(ppo.value().is_some());
    }

    /// Cover the `s == 0.0` defensive branch (line 96). PPO divides by
    /// the slow EMA; existing tests use prices ≈ 100, so the slow EMA
    /// is never 0. Feed a stream of zeros — both EMAs converge to 0.0
    /// and the indicator must emit exactly 0.0 (flat-momentum fallback)
    /// rather than NaN.
    #[test]
    fn zero_slow_ema_yields_zero_ppo() {
        let mut ppo = Ppo::new(3, 6).unwrap();
        let out = ppo.batch(&[0.0_f64; 20]);
        let last = out.into_iter().flatten().last().expect("emits");
        assert_eq!(last, 0.0);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut ppo = Ppo::new(3, 6).unwrap();
        assert_eq!(ppo.warmup_period(), 6);
        let out = ppo.batch(&(1..=30).map(f64::from).collect::<Vec<_>>());
        for v in out.iter().take(5) {
            assert!(v.is_none());
        }
        assert!(out[5].is_some());
    }

    #[test]
    fn constant_series_yields_zero() {
        // Both EMAs converge to the constant, so their gap is zero.
        let mut ppo = Ppo::new(3, 6).unwrap();
        let out = ppo.batch(&[100.0; 60]);
        for v in out.iter().skip(5).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn uptrend_is_positive() {
        // In a rising series the fast EMA leads the slow EMA, so PPO > 0.
        let mut ppo = Ppo::new(5, 12).unwrap();
        let out = ppo.batch(&(1..=80).map(f64::from).collect::<Vec<_>>());
        let last = out.iter().rev().flatten().next().unwrap();
        assert!(*last > 0.0, "uptrend PPO should be positive, got {last}");
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut ppo = Ppo::new(3, 6).unwrap();
        let out = ppo.batch(&(1..=30).map(f64::from).collect::<Vec<_>>());
        let last = *out.last().unwrap();
        assert!(last.is_some());
        assert_eq!(ppo.update(f64::NAN), last);
        assert_eq!(ppo.update(f64::INFINITY), last);
    }

    #[test]
    fn reset_clears_state() {
        let mut ppo = Ppo::new(3, 6).unwrap();
        ppo.batch(&(1..=30).map(f64::from).collect::<Vec<_>>());
        assert!(ppo.is_ready());
        ppo.reset();
        assert!(!ppo.is_ready());
        assert_eq!(ppo.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 9.0)
            .collect();
        let batch = Ppo::new(12, 26).unwrap().batch(&prices);
        let mut b = Ppo::new(12, 26).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
