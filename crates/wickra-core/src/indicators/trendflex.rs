//! Ehlers Trendflex — a trend-sensitive sibling of Reflex.
#![allow(clippy::doc_markdown)]

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::indicators::super_smoother::SuperSmoother;
use crate::traits::Indicator;

/// Ehlers' **Trendflex** — the trend-sensitive companion to
/// [`Reflex`](crate::Reflex): it averages how far the SuperSmoothed price sits
/// above or below its values over the lookback, then self-normalises.
///
/// From John Ehlers, "Reflex: A New Zero-Lag Indicator" (*Stocks & Commodities*,
/// Feb 2020):
///
/// ```text
/// Filt      = SuperSmoother(price, period)
/// sum       = mean over i=1..period of ( Filt[0] − Filt[i] )
/// ms        = 0.04·sum² + 0.96·ms[−1]                (adaptive normaliser)
/// Trendflex = sum / sqrt(ms)                         (0 if ms == 0)
/// ```
///
/// Where Reflex measures deviation from the straight *line* across the window
/// (cycle sensitive, near zero lag), Trendflex measures deviation from the
/// window's *values* (trend sensitive). It stays pinned to one side of zero
/// during a trend and oscillates through zero in a range, so it doubles as a
/// trend/range gauge. The adaptive mean-square normaliser keeps the output near a
/// `±3` band on any instrument.
///
/// The first value lands after `period + 1` SuperSmoothed samples. Each `update`
/// is O(`period`).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Trendflex};
///
/// let mut indicator = Trendflex::new(20).unwrap();
/// let mut last = None;
/// for i in 0..120 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Trendflex {
    period: usize,
    smoother: SuperSmoother,
    filt: VecDeque<f64>,
    ms: f64,
    last: Option<f64>,
}

impl Trendflex {
    /// Construct a Trendflex with the given lookback `period`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            smoother: SuperSmoother::new(period)?,
            filt: VecDeque::with_capacity(period + 1),
            ms: 0.0,
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
}

impl Indicator for Trendflex {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, price: f64) -> Option<f64> {
        if !price.is_finite() {
            return self.last;
        }
        let filt = self.smoother.update(price)?;
        if self.filt.len() == self.period + 1 {
            self.filt.pop_front();
        }
        self.filt.push_back(filt);
        if self.filt.len() < self.period + 1 {
            return None;
        }
        let newest = self.filt[self.period];
        let mut sum = 0.0;
        for i in 1..=self.period {
            sum += newest - self.filt[self.period - i];
        }
        sum /= self.period as f64;
        self.ms = 0.04 * sum * sum + 0.96 * self.ms;
        let trendflex = if self.ms > 0.0 {
            sum / self.ms.sqrt()
        } else {
            0.0
        };
        self.last = Some(trendflex);
        Some(trendflex)
    }

    fn reset(&mut self) {
        self.smoother.reset();
        self.filt.clear();
        self.ms = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "Trendflex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(Trendflex::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let t = Trendflex::new(20).unwrap();
        assert_eq!(t.period(), 20);
        assert_eq!(t.warmup_period(), 21);
        assert_eq!(t.name(), "Trendflex");
        assert!(!t.is_ready());
        assert_eq!(t.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut t = Trendflex::new(5).unwrap();
        let xs: Vec<f64> = (0..12).map(f64::from).collect();
        let out = t.batch(&xs);
        for v in out.iter().take(5) {
            assert!(v.is_none());
        }
        assert!(out[5].is_some());
    }

    #[test]
    fn constant_input_is_zero() {
        let mut t = Trendflex::new(10).unwrap();
        for v in t.batch(&[50.0; 100]).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn uptrend_is_positive() {
        // A steady rise keeps the current filtered value above its past values.
        let mut t = Trendflex::new(10).unwrap();
        let out: Vec<f64> = t
            .batch(&(0..200).map(f64::from).collect::<Vec<_>>())
            .into_iter()
            .flatten()
            .skip(100)
            .collect();
        for v in out {
            assert!(v > 0.0, "uptrend should be positive, got {v}");
        }
    }

    #[test]
    fn downtrend_is_negative() {
        let mut t = Trendflex::new(10).unwrap();
        let out: Vec<f64> = t
            .batch(&(0..200).map(|i| 200.0 - f64::from(i)).collect::<Vec<_>>())
            .into_iter()
            .flatten()
            .skip(100)
            .collect();
        for v in out {
            assert!(v < 0.0, "downtrend should be negative, got {v}");
        }
    }

    #[test]
    fn ignores_non_finite() {
        let mut t = Trendflex::new(10).unwrap();
        t.batch(&(0..40).map(f64::from).collect::<Vec<_>>());
        let before = t.value();
        assert_eq!(t.update(f64::NAN), before);
    }

    #[test]
    fn reset_clears_state() {
        let mut t = Trendflex::new(10).unwrap();
        t.batch(&(0..40).map(f64::from).collect::<Vec<_>>());
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.value(), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let xs: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 9.0)
            .collect();
        let batch = Trendflex::new(20).unwrap().batch(&xs);
        let mut b = Trendflex::new(20).unwrap();
        let streamed: Vec<_> = xs.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
