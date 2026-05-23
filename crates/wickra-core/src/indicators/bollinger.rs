//! Bollinger Bands.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Bollinger Bands output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BollingerOutput {
    /// Upper band: `middle + multiplier * stddev`.
    pub upper: f64,
    /// Middle band: SMA over the window.
    pub middle: f64,
    /// Lower band: `middle − multiplier * stddev`.
    pub lower: f64,
    /// Sample standard deviation (denominator `period`, population stddev) used to build
    /// the bands. Reported separately because some callers compute their own bands.
    pub stddev: f64,
}

/// Bollinger Bands with SMA middle band and population standard deviation envelopes.
///
/// Standard parameters are `period = 20`, `multiplier = 2.0`. Bollinger's original
/// publication uses population (not sample) standard deviation, which matches every
/// reference implementation (TA-Lib, pandas-ta, etc.).
///
/// The running `sum` and `sum_sq` are reseeded from the live window every
/// `16 · period` updates to cap floating-point drift on long streams. This is
/// amortised O(1), preserves bit-equivalence with the previous behaviour on
/// inputs that did not drift, and is particularly important for `sum_sq`,
/// where catastrophic cancellation between large add/subtract pairs can drive
/// the computed variance negative (the `.max(0.0)` clamp below is the
/// safety-net for the rare cases where the reseed has not happened yet).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, BollingerBands};
///
/// let mut indicator = BollingerBands::new(5, 2.0).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct BollingerBands {
    period: usize,
    multiplier: f64,
    window: VecDeque<f64>,
    sum: f64,
    sum_sq: f64,
    /// Number of finite updates since the running sums were last reseeded
    /// from the live window. See [`RECOMPUTE_EVERY`] below.
    updates_since_recompute: usize,
}

/// How often (in finite updates) the incremental `sum` / `sum_sq` are reseeded
/// from the live window. The multiplier `16` keeps the amortised cost flat and
/// caps any cancellation drift to roughly `16 · period · ULP · max(|x|²)` —
/// negligible on real-world price scales.
const RECOMPUTE_EVERY: usize = 16;

impl BollingerBands {
    /// Construct a new Bollinger Bands indicator.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] for `period == 0` and
    /// [`Error::NonPositiveMultiplier`] for `multiplier <= 0`.
    pub fn new(period: usize, multiplier: f64) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            period,
            multiplier,
            window: VecDeque::with_capacity(period),
            sum: 0.0,
            sum_sq: 0.0,
            updates_since_recompute: 0,
        })
    }

    /// Classic configuration: `period = 20`, `multiplier = 2.0`.
    pub fn classic() -> Self {
        Self::new(20, 2.0).expect("classic Bollinger parameters are valid")
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Configured multiplier.
    pub const fn multiplier(&self) -> f64 {
        self.multiplier
    }

    fn current(&self) -> Option<BollingerOutput> {
        if self.window.len() != self.period {
            return None;
        }
        let n = self.period as f64;
        let mean = self.sum / n;
        // Population variance: E[x^2] - (E[x])^2. Clamp small negative values that arise
        // from catastrophic cancellation on near-constant inputs.
        let var = (self.sum_sq / n - mean * mean).max(0.0);
        let stddev = var.sqrt();
        Some(BollingerOutput {
            upper: mean + self.multiplier * stddev,
            middle: mean,
            lower: mean - self.multiplier * stddev,
            stddev,
        })
    }
}

impl Indicator for BollingerBands {
    type Input = f64;
    type Output = BollingerOutput;

    fn update(&mut self, input: f64) -> Option<BollingerOutput> {
        if !input.is_finite() {
            return self.current();
        }
        if self.window.len() == self.period {
            let old = self.window.pop_front().expect("non-empty");
            self.sum -= old;
            self.sum_sq -= old * old;
        }
        self.window.push_back(input);
        self.sum += input;
        self.sum_sq += input * input;
        self.updates_since_recompute += 1;
        if self.updates_since_recompute >= RECOMPUTE_EVERY * self.period {
            self.sum = self.window.iter().copied().sum();
            self.sum_sq = self.window.iter().copied().map(|x| x * x).sum();
            self.updates_since_recompute = 0;
        }
        self.current()
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum = 0.0;
        self.sum_sq = 0.0;
        self.updates_since_recompute = 0;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "BollingerBands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn naive(prices: &[f64], period: usize, mult: f64) -> Option<BollingerOutput> {
        if prices.len() < period {
            return None;
        }
        let w = &prices[prices.len() - period..];
        let mean = w.iter().sum::<f64>() / period as f64;
        let var = w.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period as f64;
        let s = var.sqrt();
        Some(BollingerOutput {
            upper: mean + mult * s,
            middle: mean,
            lower: mean - mult * s,
            stddev: s,
        })
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(
            BollingerBands::new(0, 2.0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn rejects_non_positive_multiplier() {
        assert!(matches!(
            BollingerBands::new(20, 0.0),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            BollingerBands::new(20, -1.0),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            BollingerBands::new(20, f64::NAN),
            Err(Error::NonPositiveMultiplier)
        ));
    }

    #[test]
    fn warmup_returns_none() {
        let mut bb = BollingerBands::new(5, 2.0).unwrap();
        for v in [1.0, 2.0, 3.0, 4.0] {
            assert!(bb.update(v).is_none());
        }
        assert!(bb.update(5.0).is_some());
    }

    #[test]
    fn constant_series_yields_zero_stddev() {
        let mut bb = BollingerBands::new(10, 2.0).unwrap();
        let out = bb.batch(&[5.0_f64; 30]);
        let last = out.iter().rev().flatten().next().unwrap();
        assert_relative_eq!(last.middle, 5.0, epsilon = 1e-12);
        assert_relative_eq!(last.stddev, 0.0, epsilon = 1e-12);
        assert_relative_eq!(last.upper, 5.0, epsilon = 1e-12);
        assert_relative_eq!(last.lower, 5.0, epsilon = 1e-12);
    }

    #[test]
    fn matches_naive_definition() {
        let prices: Vec<f64> = (1..=60)
            .map(|i| (f64::from(i) * 0.3).sin() * 10.0 + 50.0)
            .collect();
        let mut bb = BollingerBands::new(20, 2.0).unwrap();
        let out = bb.batch(&prices);
        for i in 19..prices.len() {
            let got = out[i].unwrap();
            let want = naive(&prices[..=i], 20, 2.0).unwrap();
            assert_relative_eq!(got.middle, want.middle, epsilon = 1e-9);
            assert_relative_eq!(got.stddev, want.stddev, epsilon = 1e-9);
            assert_relative_eq!(got.upper, want.upper, epsilon = 1e-9);
            assert_relative_eq!(got.lower, want.lower, epsilon = 1e-9);
        }
    }

    #[test]
    fn upper_above_middle_above_lower() {
        let prices: Vec<f64> = (1..=100).map(f64::from).collect();
        let mut bb = BollingerBands::new(20, 2.0).unwrap();
        for o in bb.batch(&prices).into_iter().flatten() {
            assert!(o.upper >= o.middle);
            assert!(o.middle >= o.lower);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=50).map(|i| f64::from(i) * 0.7).collect();
        let mut a = BollingerBands::new(10, 2.0).unwrap();
        let mut b = BollingerBands::new(10, 2.0).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut bb = BollingerBands::new(5, 2.0).unwrap();
        bb.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(bb.is_ready());
        bb.reset();
        assert!(!bb.is_ready());
    }

    /// Long-running stability check. After several recompute cycles the
    /// reported Bollinger bands must still equal a fresh from-scratch
    /// computation over the live window — even on inputs designed to cause
    /// catastrophic cancellation in the `sum_sq` accumulator (alternating
    /// between two very different magnitudes).
    #[test]
    fn long_stream_drift_stays_bounded() {
        let period = 20;
        let mult = 2.0;
        let mut bb = BollingerBands::new(period, mult).unwrap();
        let mut window: VecDeque<f64> = VecDeque::with_capacity(period);
        // Forces the periodic reseed to fire 5+ times.
        let n_updates = 16 * period * 5;
        let mut last = None;
        for i in 0..n_updates {
            let v = if i.is_multiple_of(2) { 1e6 } else { 1.0 };
            last = bb.update(v);
            if window.len() == period {
                window.pop_front();
            }
            window.push_back(v);
        }
        let scratch =
            naive(&window.iter().copied().collect::<Vec<_>>(), period, mult).expect("warmed up");
        let got = last.expect("warmed up");
        assert!(
            (got.middle - scratch.middle).abs() < 1e-3,
            "middle drift: got={}, scratch={}",
            got.middle,
            scratch.middle,
        );
        assert!(
            (got.stddev - scratch.stddev).abs() < 1e-3,
            "stddev drift: got={}, scratch={}",
            got.stddev,
            scratch.stddev,
        );
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut bb = BollingerBands::new(5, 2.0).unwrap();
        let ready = bb.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let last = ready.last().unwrap().unwrap();
        // Non-finite inputs return the current bands without mutating the window.
        assert_eq!(bb.update(f64::NAN).unwrap(), last);
        assert_eq!(bb.update(f64::INFINITY).unwrap(), last);
        // The window still holds 1..=5, so a real input slides it to 2..=6.
        let after = bb.update(6.0).unwrap();
        assert_relative_eq!(
            after.middle,
            (2.0 + 3.0 + 4.0 + 5.0 + 6.0) / 5.0,
            epsilon = 1e-12
        );
    }
}
