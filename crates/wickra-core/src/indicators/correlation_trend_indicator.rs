//! Ehlers Correlation Trend Indicator (CTI) — Pearson correlation of price vs. time.
#![allow(clippy::doc_markdown)]

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Ehlers' **Correlation Trend Indicator** (CTI) — the Pearson correlation
/// coefficient between price and a perfectly straight ramp over the lookback.
///
/// ```text
/// CTI = corr( price over the window , [0, 1, …, period−1] )
/// ```
///
/// John Ehlers' CTI asks "how closely does recent price track a straight line?"
/// by correlating the windowed price against the time index itself. A reading near
/// `+1` means price is rising in a near-perfect line (strong uptrend); near `−1`
/// means a clean downtrend; near `0` means no linear trend (a range or choppy
/// market). Because correlation is scale- and offset-invariant, the slope's
/// steepness does not matter — only how *linear* the move is — which makes CTI an
/// unusually clean trend/range classifier. It differs from
/// [`Autocorrelation`](crate::Autocorrelation), which correlates price with a
/// *lagged copy of itself* rather than with time.
///
/// The output is in `[−1, +1]`; a flat window (zero price variance) returns `0`.
/// The first value lands after `period` inputs; each `update` recomputes the
/// correlation over the window in O(`period`).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, CorrelationTrendIndicator};
///
/// let mut indicator = CorrelationTrendIndicator::new(20).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = indicator.update(100.0 + f64::from(i)); // a clean uptrend
/// }
/// assert!((last.unwrap() - 1.0).abs() < 1e-9);
/// ```
#[derive(Debug, Clone)]
pub struct CorrelationTrendIndicator {
    period: usize,
    window: VecDeque<f64>,
    last: Option<f64>,
}

impl CorrelationTrendIndicator {
    /// Construct a CTI over `period` bars.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPeriod`] if `period < 2` (a correlation needs two
    /// points).
    pub fn new(period: usize) -> Result<Self> {
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "CTI needs period >= 2",
            });
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
            last: None,
        })
    }

    /// Configured lookback period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }

    fn compute(&self) -> f64 {
        let n = self.period as f64;
        let mut sum_x = 0.0;
        let mut sum_xx = 0.0;
        let mut sum_xt = 0.0;
        for (i, &x) in self.window.iter().enumerate() {
            let t = i as f64;
            sum_x += x;
            sum_xx += x * x;
            sum_xt += x * t;
        }
        // Time index 0..n-1 has closed-form sums.
        let sum_t = n * (n - 1.0) / 2.0;
        let sum_tt = (n - 1.0) * n * (2.0 * n - 1.0) / 6.0;
        let cov = n * sum_xt - sum_x * sum_t;
        let var_x = n * sum_xx - sum_x * sum_x;
        let var_t = n * sum_tt - sum_t * sum_t;
        let denom = (var_x * var_t).sqrt();
        if denom == 0.0 {
            0.0
        } else {
            (cov / denom).clamp(-1.0, 1.0)
        }
    }
}

impl Indicator for CorrelationTrendIndicator {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.last;
        }
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(input);
        if self.window.len() < self.period {
            return None;
        }
        let out = self.compute();
        self.last = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "CorrelationTrendIndicator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_period_below_two() {
        assert!(matches!(
            CorrelationTrendIndicator::new(1),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(CorrelationTrendIndicator::new(2).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let cti = CorrelationTrendIndicator::new(20).unwrap();
        assert_eq!(cti.period(), 20);
        assert_eq!(cti.warmup_period(), 20);
        assert_eq!(cti.name(), "CorrelationTrendIndicator");
        assert!(!cti.is_ready());
        assert_eq!(cti.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut cti = CorrelationTrendIndicator::new(4).unwrap();
        let out = cti.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        for v in out.iter().take(3) {
            assert!(v.is_none());
        }
        assert!(out[3].is_some());
    }

    #[test]
    fn clean_uptrend_is_one() {
        let mut cti = CorrelationTrendIndicator::new(10).unwrap();
        let last = cti
            .batch(&(0..40).map(f64::from).collect::<Vec<_>>())
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, 1.0, epsilon = 1e-9);
    }

    #[test]
    fn clean_downtrend_is_minus_one() {
        let mut cti = CorrelationTrendIndicator::new(10).unwrap();
        let last = cti
            .batch(&(0..40).map(|i| 100.0 - f64::from(i)).collect::<Vec<_>>())
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, -1.0, epsilon = 1e-9);
    }

    #[test]
    fn flat_window_is_zero() {
        let mut cti = CorrelationTrendIndicator::new(8).unwrap();
        let last = cti.batch(&[7.0; 16]).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn output_in_range() {
        let mut cti = CorrelationTrendIndicator::new(20).unwrap();
        for v in cti
            .batch(
                &(0..200)
                    .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 10.0)
                    .collect::<Vec<_>>(),
            )
            .into_iter()
            .flatten()
        {
            assert!((-1.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn ignores_non_finite() {
        let mut cti = CorrelationTrendIndicator::new(4).unwrap();
        let ready = cti
            .batch(&[1.0, 2.0, 3.0, 4.0])
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_eq!(cti.update(f64::NAN), Some(ready));
    }

    #[test]
    fn reset_clears_state() {
        let mut cti = CorrelationTrendIndicator::new(4).unwrap();
        cti.batch(&[1.0, 2.0, 3.0, 4.0]);
        assert!(cti.is_ready());
        cti.reset();
        assert!(!cti.is_ready());
        assert_eq!(cti.value(), None);
        assert_eq!(cti.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let xs: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 9.0)
            .collect();
        let batch = CorrelationTrendIndicator::new(20).unwrap().batch(&xs);
        let mut b = CorrelationTrendIndicator::new(20).unwrap();
        let streamed: Vec<_> = xs.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
