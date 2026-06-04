//! Ehlers' Adaptive Laguerre Filter.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// John Ehlers' Adaptive Laguerre Filter — a four-stage Laguerre polynomial
/// smoother whose damping factor `gamma` is recomputed every bar from how well
/// the filter is currently tracking price.
///
/// The Laguerre cascade is the same one used by [`LaguerreRsi`](crate::LaguerreRsi),
/// but instead of a fixed `gamma` the filter adapts: it measures the recent
/// absolute error `|price − filter|`, normalises those errors across a window of
/// `period` bars to `[0, 1]`, and takes their **median** as `gamma`. When price
/// is tracking smoothly the errors are small and uniform (low `gamma`, fast
/// response); when price jumps, the spread of errors widens and `gamma` rises,
/// slowing the filter to reject the noise.
///
/// ```text
/// diff_t  = |price_t − filter_{t-1}|
/// over the last `period` diffs:
///   HH = max(diff),  LL = min(diff)
///   norm_i = (diff_i − LL) / (HH − LL)      (0 if HH == LL)
///   gamma  = median(norm)
/// alpha   = 1 − gamma
/// L0_t = alpha·price_t + gamma·L0_{t-1}
/// L1_t = −gamma·L0_t + L0_{t-1} + gamma·L1_{t-1}
/// L2_t = −gamma·L1_t + L1_{t-1} + gamma·L2_{t-1}
/// L3_t = −gamma·L2_t + L2_{t-1} + gamma·L3_{t-1}
/// filter_t = (L0_t + 2·L1_t + 2·L2_t + L3_t) / 6
/// ```
///
/// The output is a smoothed price on the same scale as the input. The first
/// emission lands once the error window holds `period` values.
///
/// Reference: John F. Ehlers, *"Adaptive Laguerre Filter"*, Technical Analysis
/// of Stocks & Commodities, 2007.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, AdaptiveLaguerreFilter};
///
/// let mut indicator = AdaptiveLaguerreFilter::new(13).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct AdaptiveLaguerreFilter {
    period: usize,
    l0: f64,
    l1: f64,
    l2: f64,
    l3: f64,
    /// Previous filter output, or `None` before the first bar.
    filter: Option<f64>,
    /// The last `period` absolute errors `|price − filter|`.
    diffs: VecDeque<f64>,
}

impl AdaptiveLaguerreFilter {
    /// Construct a new adaptive Laguerre filter with the given error-window
    /// length.
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
            l0: 0.0,
            l1: 0.0,
            l2: 0.0,
            l3: 0.0,
            filter: None,
            diffs: VecDeque::with_capacity(period),
        })
    }

    /// Configured error-window length.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if the error window is full.
    pub fn value(&self) -> Option<f64> {
        if self.diffs.len() == self.period {
            self.filter
        } else {
            None
        }
    }

    /// Median of the normalised errors currently in the window. Returns `0.0`
    /// when every error is equal (e.g. during a constant warmup), which makes
    /// the filter maximally fast.
    fn adaptive_gamma(&self) -> f64 {
        let mut hh = f64::MIN;
        let mut ll = f64::MAX;
        for &d in &self.diffs {
            if d > hh {
                hh = d;
            }
            if d < ll {
                ll = d;
            }
        }
        let range = hh - ll;
        if range <= 0.0 {
            return 0.0;
        }
        let mut norm: Vec<f64> = self.diffs.iter().map(|&d| (d - ll) / range).collect();
        // `total_cmp` never panics — under pathological (e.g. overflowing) fuzz
        // inputs a normalised error can be non-finite; a total order keeps the
        // sort sound where `partial_cmp` would return `None`.
        norm.sort_by(f64::total_cmp);
        let mid = norm.len() / 2;
        if norm.len() % 2 == 1 {
            norm[mid]
        } else {
            f64::midpoint(norm[mid - 1], norm[mid])
        }
    }
}

impl Indicator for AdaptiveLaguerreFilter {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, price: f64) -> Option<f64> {
        if !price.is_finite() {
            return self.value();
        }
        // Absolute tracking error against the previous filter (0 on the first
        // bar, where there is no prior filter value).
        let diff = self.filter.map_or(0.0, |f| (price - f).abs());
        if self.diffs.len() == self.period {
            self.diffs.pop_front();
        }
        self.diffs.push_back(diff);

        let gamma = self.adaptive_gamma();
        let alpha = 1.0 - gamma;

        let l0 = alpha * price + gamma * self.l0;
        let l1 = -gamma * l0 + self.l0 + gamma * self.l1;
        let l2 = -gamma * l1 + self.l1 + gamma * self.l2;
        let l3 = -gamma * l2 + self.l2 + gamma * self.l3;
        self.l0 = l0;
        self.l1 = l1;
        self.l2 = l2;
        self.l3 = l3;

        let filter = (l0 + 2.0 * l1 + 2.0 * l2 + l3) / 6.0;
        self.filter = Some(filter);
        self.value()
    }

    fn reset(&mut self) {
        self.l0 = 0.0;
        self.l1 = 0.0;
        self.l2 = 0.0;
        self.l3 = 0.0;
        self.filter = None;
        self.diffs.clear();
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.diffs.len() == self.period
    }

    fn name(&self) -> &'static str {
        "AdaptiveLaguerre"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    /// Independent reference: replays the exact recurrence from scratch.
    fn naive(prices: &[f64], period: usize) -> Vec<Option<f64>> {
        let (mut l0, mut l1, mut l2, mut l3) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
        let mut filter: Option<f64> = None;
        let mut diffs: Vec<f64> = Vec::new();
        let mut out = Vec::with_capacity(prices.len());
        for &price in prices {
            let diff = filter.map_or(0.0, |f: f64| (price - f).abs());
            diffs.push(diff);
            if diffs.len() > period {
                diffs.remove(0);
            }
            let hh = diffs.iter().copied().fold(f64::MIN, f64::max);
            let ll = diffs.iter().copied().fold(f64::MAX, f64::min);
            let range = hh - ll;
            let gamma = if range <= 0.0 {
                0.0
            } else {
                let mut norm: Vec<f64> = diffs.iter().map(|&d| (d - ll) / range).collect();
                norm.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let mid = norm.len() / 2;
                if norm.len() % 2 == 1 {
                    norm[mid]
                } else {
                    f64::midpoint(norm[mid - 1], norm[mid])
                }
            };
            let alpha = 1.0 - gamma;
            let n0 = alpha * price + gamma * l0;
            let n1 = -gamma * n0 + l0 + gamma * l1;
            let n2 = -gamma * n1 + l1 + gamma * l2;
            let n3 = -gamma * n2 + l2 + gamma * l3;
            l0 = n0;
            l1 = n1;
            l2 = n2;
            l3 = n3;
            let f = (n0 + 2.0 * n1 + 2.0 * n2 + n3) / 6.0;
            filter = Some(f);
            out.push(if diffs.len() == period { Some(f) } else { None });
        }
        out
    }

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(
            AdaptiveLaguerreFilter::new(0),
            Err(Error::PeriodZero)
        ));
    }

    /// Cover the const accessor `period` and the Indicator-impl `warmup_period`
    /// + `name`.
    #[test]
    fn accessors_and_metadata() {
        let alf = AdaptiveLaguerreFilter::new(13).unwrap();
        assert_eq!(alf.period(), 13);
        assert_eq!(alf.warmup_period(), 13);
        assert_eq!(alf.name(), "AdaptiveLaguerre");
    }

    #[test]
    fn warmup_returns_none_until_window_full() {
        let mut alf = AdaptiveLaguerreFilter::new(3).unwrap();
        assert_eq!(alf.update(10.0), None);
        assert_eq!(alf.update(11.0), None);
        assert!(alf.update(12.0).is_some());
    }

    #[test]
    fn constant_series_converges_to_constant() {
        // Errors are all zero -> gamma 0 -> the 4-stage delay line fills with
        // the constant and the filter settles on it.
        let mut alf = AdaptiveLaguerreFilter::new(5).unwrap();
        let out = alf.batch(&[42.0_f64; 40]);
        let last = out.iter().rev().flatten().next().unwrap();
        assert_relative_eq!(*last, 42.0, epsilon = 1e-9);
    }

    #[test]
    fn converged_output_stays_within_price_range() {
        // Once the Laguerre cascade has filled (it cold-starts from zero, so the
        // first few post-warmup values ramp up toward price), the filter is a
        // convex blend of recent prices and must stay inside the data range.
        let prices: Vec<f64> = (0..120)
            .map(|i| 50.0 + (f64::from(i) * 0.4).sin() * 10.0)
            .collect();
        let lo = prices.iter().copied().fold(f64::MAX, f64::min);
        let hi = prices.iter().copied().fold(f64::MIN, f64::max);
        let period = 8;
        let mut alf = AdaptiveLaguerreFilter::new(period).unwrap();
        for (i, v) in alf.batch(&prices).into_iter().enumerate() {
            // Skip the cold-start transient (a few multiples of the window).
            if i < 4 * period {
                continue;
            }
            let v = v.expect("filter is ready well past warmup");
            assert!(
                v >= lo - 1e-6 && v <= hi + 1e-6,
                "filter out of range at {i}"
            );
        }
    }

    #[test]
    fn matches_naive_recurrence() {
        let prices: Vec<f64> = (0..80)
            .map(|i| 100.0 + (f64::from(i) * 0.5).sin() * 8.0 + f64::from(i) * 0.1)
            .collect();
        let mut alf = AdaptiveLaguerreFilter::new(10).unwrap();
        let got = alf.batch(&prices);
        let want = naive(&prices, 10);
        for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
            assert_eq!(g.is_some(), w.is_some(), "readiness mismatch at {i}");
            if let (Some(a), Some(b)) = (g, w) {
                assert_relative_eq!(*a, *b, epsilon = 1e-9);
            }
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut alf = AdaptiveLaguerreFilter::new(5).unwrap();
        alf.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        assert!(alf.is_ready());
        alf.reset();
        assert!(!alf.is_ready());
        assert_eq!(alf.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=50).map(|i| f64::from(i) * 0.7).collect();
        let mut a = AdaptiveLaguerreFilter::new(7).unwrap();
        let mut b = AdaptiveLaguerreFilter::new(7).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut alf = AdaptiveLaguerreFilter::new(3).unwrap();
        alf.update(10.0);
        alf.update(11.0);
        let ready = alf.update(12.0).expect("ready after three inputs");
        assert_eq!(alf.update(f64::NAN), Some(ready));
        assert_eq!(alf.update(f64::INFINITY), Some(ready));
    }
}
