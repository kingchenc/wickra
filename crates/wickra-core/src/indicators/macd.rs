//! Moving Average Convergence Divergence (MACD).

use crate::error::{Error, Result};
use crate::indicators::ema::Ema;
use crate::traits::Indicator;

/// MACD output: the three classic series at a given step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacdOutput {
    /// Fast EMA − slow EMA.
    pub macd: f64,
    /// EMA of `macd` over the signal period.
    pub signal: f64,
    /// `macd − signal`.
    pub histogram: f64,
}

/// MACD = EMA(fast) − EMA(slow), with a signal EMA on top.
///
/// Standard parameters are `fast = 12`, `slow = 26`, `signal = 9`. The signal EMA
/// is seeded from the first `signal` raw MACD values, so the first full
/// [`MacdOutput`] is emitted after `slow + signal − 1` inputs (assuming the
/// slow EMA seeded by then).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, MacdIndicator};
///
/// let mut indicator = MacdIndicator::new(3, 6, 3).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct MacdIndicator {
    fast: Ema,
    slow: Ema,
    signal_ema: Ema,
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    last: Option<MacdOutput>,
}

impl MacdIndicator {
    /// Construct a MACD with the given periods.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if any period is zero, and
    /// [`Error::InvalidPeriod`] if `fast >= slow`.
    pub fn new(fast: usize, slow: usize, signal: usize) -> Result<Self> {
        if fast == 0 || slow == 0 || signal == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "fast period must be strictly less than slow period",
            });
        }
        Ok(Self {
            fast: Ema::new(fast)?,
            slow: Ema::new(slow)?,
            signal_ema: Ema::new(signal)?,
            fast_period: fast,
            slow_period: slow,
            signal_period: signal,
            last: None,
        })
    }

    /// Default `(12, 26, 9)` configuration, matching every classical chart package.
    pub fn classic() -> Self {
        Self::new(12, 26, 9).expect("classic MACD periods are valid")
    }

    /// Configured periods as `(fast, slow, signal)`.
    pub const fn periods(&self) -> (usize, usize, usize) {
        (self.fast_period, self.slow_period, self.signal_period)
    }

    /// Most recent fully-computed output if available.
    pub const fn value(&self) -> Option<MacdOutput> {
        self.last
    }
}

impl Indicator for MacdIndicator {
    type Input = f64;
    type Output = MacdOutput;

    fn update(&mut self, input: f64) -> Option<MacdOutput> {
        if !input.is_finite() {
            return self.last;
        }

        let fast = self.fast.update(input);
        let slow = self.slow.update(input);

        match (fast, slow) {
            (Some(f), Some(s)) => {
                let macd = f - s;
                let signal = self.signal_ema.update(macd)?;
                let out = MacdOutput {
                    macd,
                    signal,
                    histogram: macd - signal,
                };
                self.last = Some(out);
                Some(out)
            }
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.fast.reset();
        self.slow.reset();
        self.signal_ema.reset();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // Slow EMA needs `slow` inputs to seed; signal EMA needs another `signal - 1`.
        self.slow_period + self.signal_period - 1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "MACD"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_fast_geq_slow() {
        assert!(matches!(
            MacdIndicator::new(26, 12, 9),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            MacdIndicator::new(12, 12, 9),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn rejects_zero_periods() {
        assert!(matches!(
            MacdIndicator::new(0, 26, 9),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            MacdIndicator::new(12, 0, 9),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            MacdIndicator::new(12, 26, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn first_emission_matches_warmup_period() {
        let prices: Vec<f64> = (1..=60).map(f64::from).collect();
        let mut macd = MacdIndicator::classic();
        let out = macd.batch(&prices);
        let warmup = macd.warmup_period();
        // Indices 0..warmup-1 are None, index warmup-1 might be Some or might still need
        // the signal EMA's seeding. Our warmup_period is the index at which the first
        // signal value appears: slow + signal - 1.
        for x in out.iter().take(warmup - 1) {
            assert!(x.is_none(), "expected None within warmup");
        }
        assert!(
            out[warmup - 1].is_some(),
            "expected first emission at warmup_period - 1 ({warmup} idx)"
        );
    }

    #[test]
    fn histogram_equals_macd_minus_signal() {
        let prices: Vec<f64> = (1..=80).map(|i| f64::from(i) * 0.5).collect();
        let mut macd = MacdIndicator::classic();
        for v in macd.batch(&prices).into_iter().flatten() {
            assert_relative_eq!(v.histogram, v.macd - v.signal, epsilon = 1e-12);
        }
    }

    #[test]
    fn constant_series_yields_zero_macd_eventually() {
        let mut macd = MacdIndicator::classic();
        let out = macd.batch(&[100.0_f64; 200]);
        // Both EMAs converge to 100, so MACD must approach 0.
        let last = out.iter().rev().flatten().next().expect("emits a value");
        assert_relative_eq!(last.macd, 0.0, epsilon = 1e-9);
        assert_relative_eq!(last.signal, 0.0, epsilon = 1e-9);
        assert_relative_eq!(last.histogram, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn rising_series_macd_positive_then_signal_catches_up() {
        let prices: Vec<f64> = (1..=200).map(f64::from).collect();
        let mut macd = MacdIndicator::classic();
        let out = macd.batch(&prices);
        let last = out.iter().rev().flatten().next().unwrap();
        assert!(last.macd > 0.0, "rising series must yield positive MACD");
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=100)
            .map(|i| (f64::from(i) * 0.4).cos() * 10.0)
            .collect();
        let mut a = MacdIndicator::classic();
        let mut b = MacdIndicator::classic();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut macd = MacdIndicator::classic();
        macd.batch(&(1..=80).map(f64::from).collect::<Vec<_>>());
        assert!(macd.is_ready());
        macd.reset();
        assert!(!macd.is_ready());
        assert_eq!(macd.update(1.0), None);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut macd = MacdIndicator::classic();
        macd.batch(&(1..=80).map(f64::from).collect::<Vec<_>>());
        let before = macd.value();
        assert!(before.is_some());
        // Non-finite inputs return the last value without advancing any EMA.
        assert_eq!(macd.update(f64::NAN), before);
        assert_eq!(macd.update(f64::INFINITY), before);
        assert_eq!(macd.value(), before);
    }
}
