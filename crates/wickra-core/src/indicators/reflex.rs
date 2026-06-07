//! Ehlers Reflex — a zero-lag cycle oscillator built on a SuperSmoother prefilter.
#![allow(clippy::doc_markdown)]

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::indicators::super_smoother::SuperSmoother;
use crate::traits::Indicator;

/// Ehlers' **Reflex** — a near-zero-lag oscillator that measures how far the
/// smoothed price has deviated from the straight line connecting its endpoints
/// over the lookback.
///
/// From John Ehlers, "Reflex: A New Zero-Lag Indicator" (*Stocks & Commodities*,
/// Feb 2020):
///
/// ```text
/// Filt   = SuperSmoother(price, period)
/// slope  = (Filt[period] − Filt[0]) / period          (line over the window)
/// sum    = mean over i=1..period of ( Filt[0] + i·slope − Filt[i] )
/// ms     = 0.04·sum² + 0.96·ms[−1]                     (adaptive normaliser)
/// Reflex = sum / sqrt(ms)                              (0 if ms == 0)
/// ```
///
/// Reflex fits a straight line across the SuperSmoothed price over `period` bars
/// and averages the deviation of the curve from that line. Because the line uses
/// both endpoints, the measure has almost no lag — it crosses zero essentially at
/// the cycle turns. The adaptive mean-square normaliser rescales the output to a
/// roughly `±3` range regardless of price, so the same thresholds work on any
/// instrument. Its sibling [`Trendflex`](crate::Trendflex) uses the deviation from
/// the *current* value instead of the line, making it trend- rather than
/// cycle-sensitive.
///
/// The first value lands after `period + 1` SuperSmoothed samples. Each `update`
/// is O(`period`).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Reflex};
///
/// let mut indicator = Reflex::new(20).unwrap();
/// let mut last = None;
/// for i in 0..120 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Reflex {
    period: usize,
    smoother: SuperSmoother,
    filt: VecDeque<f64>,
    ms: f64,
    last: Option<f64>,
}

impl Reflex {
    /// Construct a Reflex with the given lookback `period`.
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

impl Indicator for Reflex {
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
        // Newest at index `period`, oldest (period bars ago) at index 0.
        let newest = self.filt[self.period];
        let oldest = self.filt[0];
        let slope = (oldest - newest) / self.period as f64;
        let mut sum = 0.0;
        for i in 1..=self.period {
            sum += (newest + i as f64 * slope) - self.filt[self.period - i];
        }
        sum /= self.period as f64;
        self.ms = 0.04 * sum * sum + 0.96 * self.ms;
        let reflex = if self.ms > 0.0 {
            sum / self.ms.sqrt()
        } else {
            0.0
        };
        self.last = Some(reflex);
        Some(reflex)
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
        "Reflex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(Reflex::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let r = Reflex::new(20).unwrap();
        assert_eq!(r.period(), 20);
        assert_eq!(r.warmup_period(), 21);
        assert_eq!(r.name(), "Reflex");
        assert!(!r.is_ready());
        assert_eq!(r.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut r = Reflex::new(5).unwrap();
        let xs: Vec<f64> = (0..12)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 3.0)
            .collect();
        let out = r.batch(&xs);
        for v in out.iter().take(5) {
            assert!(v.is_none());
        }
        assert!(out[5].is_some());
    }

    #[test]
    fn constant_input_is_zero() {
        // A flat price is exactly its own straight line -> zero deviation -> 0.
        let mut r = Reflex::new(10).unwrap();
        for v in r.batch(&[50.0; 100]).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn cyclic_input_oscillates_around_zero() {
        let mut r = Reflex::new(20).unwrap();
        let xs: Vec<f64> = (0..400)
            .map(|i| 100.0 + (std::f64::consts::TAU * f64::from(i) / 20.0).sin() * 5.0)
            .collect();
        let out: Vec<f64> = r.batch(&xs).into_iter().flatten().skip(100).collect();
        assert!(out.iter().any(|&v| v > 0.5));
        assert!(out.iter().any(|&v| v < -0.5));
    }

    #[test]
    fn ignores_non_finite() {
        let mut r = Reflex::new(10).unwrap();
        r.batch(
            &(0..40)
                .map(|i| 100.0 + (f64::from(i) * 0.3).sin())
                .collect::<Vec<_>>(),
        );
        let before = r.value();
        assert_eq!(r.update(f64::NAN), before);
    }

    #[test]
    fn reset_clears_state() {
        let mut r = Reflex::new(10).unwrap();
        r.batch(
            &(0..40)
                .map(|i| 100.0 + (f64::from(i) * 0.3).sin())
                .collect::<Vec<_>>(),
        );
        assert!(r.is_ready());
        r.reset();
        assert!(!r.is_ready());
        assert_eq!(r.value(), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let xs: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 9.0)
            .collect();
        let batch = Reflex::new(20).unwrap().batch(&xs);
        let mut b = Reflex::new(20).unwrap();
        let streamed: Vec<_> = xs.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
