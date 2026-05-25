//! Absolute Price Oscillator (APO).

use crate::error::{Error, Result};
use crate::indicators::ema::Ema;
use crate::traits::Indicator;

/// Absolute Price Oscillator — the raw difference between a fast and a slow
/// `EMA`. This is MACD's line without the signal-EMA — useful when only the
/// momentum-direction reading is needed.
///
/// ```text
/// APO_t = EMA(close, fast)_t − EMA(close, slow)_t
/// ```
///
/// Default parameters mirror MACD: `(fast = 12, slow = 26)`. `fast` must be
/// strictly less than `slow`.
///
/// # Example
///
/// ```
/// use wickra_core::{Apo, Indicator};
///
/// let mut apo = Apo::new(12, 26).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = apo.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Apo {
    fast_period: usize,
    slow_period: usize,
    fast: Ema,
    slow: Ema,
}

impl Apo {
    /// # Errors
    /// - [`Error::PeriodZero`] if either period is zero.
    /// - [`Error::InvalidPeriod`] if `fast >= slow`.
    pub fn new(fast: usize, slow: usize) -> Result<Self> {
        if fast == 0 || slow == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "APO fast period must be strictly less than slow",
            });
        }
        Ok(Self {
            fast_period: fast,
            slow_period: slow,
            fast: Ema::new(fast)?,
            slow: Ema::new(slow)?,
        })
    }

    /// MACD-style defaults: `(fast = 12, slow = 26)`.
    pub fn classic() -> Self {
        Self::new(12, 26).expect("classic APO parameters are valid")
    }

    /// Configured `(fast, slow)`.
    pub const fn periods(&self) -> (usize, usize) {
        (self.fast_period, self.slow_period)
    }
}

impl Indicator for Apo {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        // Feed both EMAs on every input so the slow one warms in parallel.
        let f = self.fast.update(input);
        let s = self.slow.update(input);
        Some(f? - s?)
    }

    fn reset(&mut self) {
        self.fast.reset();
        self.slow.reset();
    }

    fn warmup_period(&self) -> usize {
        // Slow EMA dominates; both EMAs emit at their `period` th input.
        self.slow_period
    }

    fn is_ready(&self) -> bool {
        self.slow.is_ready()
    }

    fn name(&self) -> &'static str {
        "APO"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(Apo::new(0, 26), Err(Error::PeriodZero)));
        assert!(matches!(Apo::new(12, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn rejects_fast_geq_slow() {
        assert!(matches!(Apo::new(26, 12), Err(Error::InvalidPeriod { .. })));
        assert!(matches!(Apo::new(12, 12), Err(Error::InvalidPeriod { .. })));
    }

    #[test]
    fn accessors_and_metadata() {
        let apo = Apo::classic();
        assert_eq!(apo.periods(), (12, 26));
        assert_eq!(apo.warmup_period(), 26);
        assert_eq!(apo.name(), "APO");
    }

    #[test]
    fn classic_factory() {
        assert_eq!(Apo::classic().periods(), (12, 26));
    }

    #[test]
    fn constant_series_converges_to_zero() {
        // Both EMAs reproduce the constant exactly, so APO is 0.
        let mut apo = Apo::new(3, 5).unwrap();
        let out = apo.batch(&[42.0_f64; 30]);
        for v in out.iter().skip(4).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn warmup_emits_first_value_at_slow_period() {
        let mut apo = Apo::new(2, 4).unwrap();
        assert_eq!(apo.warmup_period(), 4);
        for i in 1..=3 {
            assert_eq!(apo.update(f64::from(i)), None);
        }
        assert!(apo.update(4.0).is_some());
    }

    #[test]
    fn pure_uptrend_is_positive() {
        // Fast EMA leads the slow EMA on an uptrend, so APO > 0.
        let mut apo = Apo::classic();
        let prices: Vec<f64> = (1..=200).map(f64::from).collect();
        let out = apo.batch(&prices);
        let last = out.iter().rev().flatten().next().unwrap();
        assert!(*last > 0.0, "APO on uptrend should be positive: {last}");
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 5.0)
            .collect();
        let mut a = Apo::classic();
        let mut b = Apo::classic();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut apo = Apo::classic();
        apo.batch(&(1..=80).map(f64::from).collect::<Vec<_>>());
        assert!(apo.is_ready());
        apo.reset();
        assert!(!apo.is_ready());
        assert_eq!(apo.update(1.0), None);
    }
}
