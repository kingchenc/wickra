//! Trend Intensity Index (TII).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::indicators::sma::Sma;
use crate::traits::Indicator;

/// M.H. Pee's Trend Intensity Index — a `[0, 100]` oscillator that measures
/// what fraction of the recent SMA deviations are positive.
///
/// First, compute an `SMA(close, sma_period)` (canonical `sma_period = 60`).
/// On each bar `t` that the SMA is defined, compute the deviation
/// `dev_t = close_t − SMA_t`. Then, over the most recent `dev_period`
/// deviations (canonical `dev_period = 30`, i.e. `sma_period / 2`), sum the
/// positive and negative magnitudes separately:
///
/// ```text
/// SD_pos = Σ_{i ∈ window, dev_i > 0}  dev_i
/// SD_neg = Σ_{i ∈ window, dev_i < 0}  |dev_i|
/// TII    = 100 · SD_pos / (SD_pos + SD_neg)
/// ```
///
/// `TII` is bounded in `[0, 100]`: high readings (`> 80`) signal a sustained
/// uptrend (most recent closes above the SMA), low readings (`< 20`) a
/// sustained downtrend. A perfectly flat window produces `50` (every deviation
/// is zero, so the indicator falls back to its neutral mid-point).
///
/// The first output is emitted once both the SMA is ready (`sma_period`
/// inputs) and the deviation ring is full (`dev_period − 1` more inputs):
/// warmup = `sma_period + dev_period − 1`.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Tii};
///
/// let mut indicator = Tii::new(20, 10).unwrap();
/// let mut last = None;
/// for i in 0..60 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Tii {
    sma_period: usize,
    dev_period: usize,
    sma: Sma,
    /// Rolling window of the most recent `dev_period` deviations.
    window: VecDeque<f64>,
    sum_pos: f64,
    sum_neg: f64,
    last: Option<f64>,
}

impl Tii {
    /// Construct a new TII with the SMA period and the deviation window length.
    ///
    /// The canonical Pee parameters are `(sma_period = 60, dev_period = 30)`;
    /// expose them as the Python defaults.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if either period is `0`.
    pub fn new(sma_period: usize, dev_period: usize) -> Result<Self> {
        if sma_period == 0 || dev_period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            sma_period,
            dev_period,
            sma: Sma::new(sma_period)?,
            window: VecDeque::with_capacity(dev_period),
            sum_pos: 0.0,
            sum_neg: 0.0,
            last: None,
        })
    }

    /// Configured `(sma_period, dev_period)`.
    pub const fn periods(&self) -> (usize, usize) {
        (self.sma_period, self.dev_period)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for Tii {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        let sma_value = self.sma.update(input)?;
        let dev = input - sma_value;

        if self.window.len() == self.dev_period {
            let old = self.window.pop_front().expect("ring is non-empty");
            if old > 0.0 {
                self.sum_pos -= old;
            } else if old < 0.0 {
                self.sum_neg -= -old;
            }
        }
        self.window.push_back(dev);
        if dev > 0.0 {
            self.sum_pos += dev;
        } else if dev < 0.0 {
            self.sum_neg += -dev;
        }

        if self.window.len() < self.dev_period {
            return None;
        }

        let denom = self.sum_pos + self.sum_neg;
        let tii = if denom <= 0.0 {
            // A perfectly flat window — every deviation is zero. By
            // convention we return the neutral mid-point, matching
            // pandas-ta's implementation. The `<=` also catches the rare
            // case where rolling-subtraction rounding leaves the
            // accumulator slightly negative; the indicator is then
            // mathematically undefined and we again fall back to the
            // neutral mid-point.
            50.0
        } else {
            // Clamp to [0, 100]: by construction the ratio lives in this
            // interval, but the rolling sum_pos / sum_neg subtractions
            // accumulate floating-point error and can produce a result
            // a few ULP outside the bound on long histories.
            (100.0 * self.sum_pos / denom).clamp(0.0, 100.0)
        };
        self.last = Some(tii);
        Some(tii)
    }

    fn reset(&mut self) {
        self.sma.reset();
        self.window.clear();
        self.sum_pos = 0.0;
        self.sum_neg = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // SMA emits its first value at input `sma_period`; the deviation ring
        // then needs `dev_period − 1` more inputs to fill, so first TII lands
        // at `sma_period + dev_period − 1`.
        self.sma_period + self.dev_period - 1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "TII"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(Tii::new(0, 10), Err(Error::PeriodZero)));
        assert!(matches!(Tii::new(10, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let mut t = Tii::new(60, 30).unwrap();
        assert_eq!(t.periods(), (60, 30));
        assert_eq!(t.warmup_period(), 89);
        assert_eq!(t.name(), "TII");
        assert!(t.value().is_none());
        let prices: Vec<f64> = (1..=100).map(|i| 100.0 + f64::from(i)).collect();
        for &p in &prices {
            t.update(p);
        }
        assert!(t.value().is_some());
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let prices: Vec<f64> = (1..=30)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        let mut t = Tii::new(5, 4).unwrap();
        let out = t.batch(&prices);
        let warmup = 5 + 4 - 1; // 8
        for v in out.iter().take(warmup - 1) {
            assert!(v.is_none());
        }
        assert!(out[warmup - 1].is_some());
    }

    #[test]
    fn pure_uptrend_saturates_at_100() {
        // Strictly increasing series: the SMA always lags, so every close
        // sits above the SMA → every deviation positive → TII = 100.
        let prices: Vec<f64> = (1..=80).map(|i| 100.0 + f64::from(i)).collect();
        let mut t = Tii::new(10, 5).unwrap();
        let last = t.batch(&prices).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 100.0, epsilon = 1e-9);
    }

    #[test]
    fn pure_downtrend_falls_to_zero() {
        let prices: Vec<f64> = (1..=80).rev().map(|i| 100.0 + f64::from(i)).collect();
        let mut t = Tii::new(10, 5).unwrap();
        let last = t.batch(&prices).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn constant_series_yields_neutral_50() {
        // Every deviation is zero; the `denom == 0` guard returns the
        // neutral mid-point.
        let mut t = Tii::new(5, 4).unwrap();
        let last = t
            .batch(&[10.0_f64; 30])
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_eq!(last, 50.0);
    }

    #[test]
    fn output_bounded_in_unit_interval() {
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 6.0 + (f64::from(i) * 0.07).cos() * 3.0)
            .collect();
        let mut t = Tii::new(20, 10).unwrap();
        for v in t.batch(&prices).into_iter().flatten() {
            assert!((0.0..=100.0).contains(&v));
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 5.0)
            .collect();
        let mut a = Tii::new(20, 10).unwrap();
        let mut b = Tii::new(20, 10).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = Tii::new(5, 4).unwrap();
        t.batch(&(1..=30).map(f64::from).collect::<Vec<_>>());
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(1.0), None);
    }
}
