//! MACD with fixed 12/26 periods (MACDFIX).

use crate::error::Result;
use crate::indicators::macd::{MacdIndicator, MacdOutput};
use crate::traits::Indicator;

/// MACD Fix (`MACDFIX`): the classic MACD with the fast and slow EMAs fixed at
/// 12 and 26, leaving only the signal period configurable.
///
/// This is TA-Lib's `MACDFIX` — identical output to
/// [`MacdIndicator::new(12, 26, signal)`](crate::MacdIndicator), packaged as a
/// single-parameter constructor for the common case. The output is the usual
/// [`MacdOutput`] triple `{ macd, signal, histogram }`.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, MacdFix};
///
/// let mut indicator = MacdFix::new(9).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct MacdFix {
    inner: MacdIndicator,
}

impl MacdFix {
    /// Construct a MACDFIX with fast = 12, slow = 26 and the given signal period.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`](crate::Error::PeriodZero) if `signal == 0`.
    pub fn new(signal: usize) -> Result<Self> {
        Ok(Self {
            inner: MacdIndicator::new(12, 26, signal)?,
        })
    }

    /// Configured signal period.
    pub fn signal_period(&self) -> usize {
        self.inner.periods().2
    }
}

impl Indicator for MacdFix {
    type Input = f64;
    type Output = MacdOutput;

    fn update(&mut self, value: f64) -> Option<MacdOutput> {
        self.inner.update(value)
    }

    fn reset(&mut self) {
        self.inner.reset();
    }

    fn warmup_period(&self) -> usize {
        self.inner.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

    fn name(&self) -> &'static str {
        "MACDFIX"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    #[test]
    fn rejects_zero_signal() {
        assert!(MacdFix::new(0).is_err());
    }

    #[test]
    fn accessors_report_config() {
        let m = MacdFix::new(9).unwrap();
        assert_eq!(m.signal_period(), 9);
        assert_eq!(m.name(), "MACDFIX");
        assert!(!m.is_ready());
        assert_eq!(
            m.warmup_period(),
            MacdIndicator::new(12, 26, 9).unwrap().warmup_period()
        );
    }

    #[test]
    fn matches_macd_with_fixed_periods() {
        let prices: Vec<f64> = (0..80)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        let fix: Vec<Option<MacdOutput>> = MacdFix::new(9).unwrap().batch(&prices);
        let classic: Vec<Option<MacdOutput>> =
            MacdIndicator::new(12, 26, 9).unwrap().batch(&prices);
        assert_eq!(fix, classic);
        assert!(fix.iter().any(Option::is_some));
    }

    #[test]
    fn reset_clears_state() {
        let prices: Vec<f64> = (0..80).map(|i| 100.0 + f64::from(i)).collect();
        let mut m = MacdFix::new(9).unwrap();
        let _ = m.batch(&prices);
        assert!(m.is_ready());
        m.reset();
        assert!(!m.is_ready());
    }
}
