//! Coppock Curve.

use crate::error::{Error, Result};
use crate::traits::Indicator;

use super::{Roc, Wma};

/// Coppock Curve — Edwin Coppock's long-term momentum indicator.
///
/// The Coppock Curve is a weighted moving average of the sum of two rates of
/// change:
///
/// ```text
/// Coppock = WMA( ROC(long) + ROC(short), wma_period )
/// ```
///
/// Coppock designed it (1962) as a long-horizon buy signal for stock indices:
/// on a monthly chart with the conventional `(long = 14, short = 11,
/// wma_period = 10)`, a turn upward from below zero has historically marked
/// the start of a new bull phase. The two ROCs blend a slightly longer and a
/// slightly shorter momentum horizon; the WMA smooths the result.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Coppock};
///
/// let mut indicator = Coppock::new(14, 11, 10).unwrap();
/// let mut last = None;
/// for i in 0..120 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Coppock {
    roc_long_period: usize,
    roc_short_period: usize,
    wma_period: usize,
    roc_long: Roc,
    roc_short: Roc,
    wma: Wma,
    current: Option<f64>,
}

impl Coppock {
    /// Construct a new Coppock Curve with the two ROC periods and the WMA period.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if any period is `0`.
    pub fn new(roc_long_period: usize, roc_short_period: usize, wma_period: usize) -> Result<Self> {
        if roc_long_period == 0 || roc_short_period == 0 || wma_period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            roc_long_period,
            roc_short_period,
            wma_period,
            roc_long: Roc::new(roc_long_period)?,
            roc_short: Roc::new(roc_short_period)?,
            wma: Wma::new(wma_period)?,
            current: None,
        })
    }

    /// The `(roc_long, roc_short, wma)` periods.
    pub const fn periods(&self) -> (usize, usize, usize) {
        (self.roc_long_period, self.roc_short_period, self.wma_period)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.current
    }
}

impl Indicator for Coppock {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            // Non-finite input is ignored; no component is advanced.
            return self.current;
        }
        let long = self.roc_long.update(input);
        let short = self.roc_short.update(input);
        let result = match (long, short) {
            (Some(l), Some(s)) => self.wma.update(l + s),
            _ => None,
        };
        if result.is_some() {
            self.current = result;
        }
        result
    }

    fn reset(&mut self) {
        self.roc_long.reset();
        self.roc_short.reset();
        self.wma.reset();
        self.current = None;
    }

    fn warmup_period(&self) -> usize {
        // Let `L = max(roc_long_period, roc_short_period)` and `W = wma_period`.
        // Both ROCs need `period + 1` inputs to emit; the slower one therefore
        // first emits at **0-based index L** (= the `(L + 1)`-th input). From
        // that bar onward both ROCs feed the WMA in lock-step, so the WMA
        // sees its `W`-th input at 0-based index `L + W − 1` — the first bar
        // it emits. `warmup_period` is the 1-based count of inputs needed for
        // the first `Some` value, which is `(L + W − 1) + 1 = L + W`.
        //
        // Worked example for `Coppock::new(6, 4, 3)`:
        //   - ROC(6).first_some at index 6 (the 7th input).
        //   - ROC(4).first_some at index 4 (the 5th input). Both available
        //     from index 6 onward.
        //   - WMA(3) consumes 3 inputs at indices 6, 7, 8 → first WMA `Some`
        //     at index 8 (the 9th input). `warmup_period() == 9`.
        self.roc_long_period.max(self.roc_short_period) + self.wma_period
    }

    fn is_ready(&self) -> bool {
        self.current.is_some()
    }

    fn name(&self) -> &'static str {
        "Coppock"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(Coppock::new(0, 11, 10), Err(Error::PeriodZero)));
        assert!(matches!(Coppock::new(14, 0, 10), Err(Error::PeriodZero)));
        assert!(matches!(Coppock::new(14, 11, 0), Err(Error::PeriodZero)));
    }

    /// Cover the const accessors `periods` / `value` (lines 68-75) and the
    /// Indicator-impl `name` body (128-130). Existing tests inspect numeric
    /// output and `warmup_period` but never query the configured periods,
    /// the current cached value, or the indicator name.
    #[test]
    fn accessors_and_metadata() {
        let mut c = Coppock::new(14, 11, 10).unwrap();
        assert_eq!(c.periods(), (14, 11, 10));
        assert_eq!(c.name(), "Coppock");
        assert_eq!(c.value(), None);
        // Drive past warmup so value() flips to Some.
        for i in 1..=u32::try_from(c.warmup_period()).unwrap() {
            c.update(100.0 + f64::from(i));
        }
        assert!(c.value().is_some());
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut c = Coppock::new(6, 4, 3).unwrap();
        assert_eq!(c.warmup_period(), 9);
        let out = c.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        for v in out.iter().take(8) {
            assert!(v.is_none());
        }
        assert!(out[8].is_some());
    }

    /// `warmup_period()` equals the 1-based index of the first emitted
    /// `Some` for every legal parameter combination — including the
    /// parameter set `(roc_long=4, roc_short=2, wma=3)` that an external
    /// audit claimed would prove the formula off by one. It does not: the
    /// slower ROC first emits at 0-based index 4, the WMA needs 3 such inputs
    /// and emits at 0-based index 6 (the 7th input), which is what
    /// `roc_long.max(roc_short) + wma = max(4, 2) + 3 = 7` reports.
    #[test]
    fn warmup_period_matches_first_some_for_every_parameter_set() {
        let prices: Vec<f64> = (1..=80).map(|i| 100.0 + f64::from(i)).collect();
        for &(long, short, wma) in &[(6, 4, 3), (14, 11, 10), (4, 2, 3), (10, 3, 5), (3, 3, 3)] {
            let mut c = Coppock::new(long, short, wma).unwrap();
            let warmup = c.warmup_period();
            let out = c.batch(&prices);
            for (i, v) in out.iter().enumerate().take(warmup - 1) {
                assert!(
                    v.is_none(),
                    "Coppock({long}, {short}, {wma}): index {i} expected None during warmup, got {v:?}"
                );
            }
            assert!(
                out[warmup - 1].is_some(),
                "Coppock({long}, {short}, {wma}): warmup_period() = {warmup} but the warmup index is None",
            );
        }
    }

    #[test]
    fn constant_series_yields_zero() {
        // Both ROCs are 0 on a flat series, so the WMA of zeros is 0.
        let mut c = Coppock::new(6, 4, 3).unwrap();
        let out = c.batch(&[100.0; 40]);
        for v in out.iter().skip(c.warmup_period() - 1).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn uptrend_is_positive() {
        // A steady uptrend has positive ROCs, so the Coppock Curve is positive.
        let mut c = Coppock::new(14, 11, 10).unwrap();
        let prices: Vec<f64> = (1..=120).map(|i| 100.0 * 1.01_f64.powi(i)).collect();
        let out = c.batch(&prices);
        let last = out.iter().rev().flatten().next().unwrap();
        assert!(
            *last > 0.0,
            "uptrend Coppock should be positive, got {last}"
        );
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut c = Coppock::new(6, 4, 3).unwrap();
        let out = c.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        let last = *out.last().unwrap();
        assert!(last.is_some());
        assert_eq!(c.update(f64::NAN), last);
        assert_eq!(c.update(f64::INFINITY), last);
    }

    #[test]
    fn reset_clears_state() {
        let mut c = Coppock::new(6, 4, 3).unwrap();
        c.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        assert!(c.is_ready());
        c.reset();
        assert!(!c.is_ready());
        assert_eq!(c.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 10.0)
            .collect();
        let batch = Coppock::new(14, 11, 10).unwrap().batch(&prices);
        let mut b = Coppock::new(14, 11, 10).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
