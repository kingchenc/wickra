//! Maximum Drawdown over a rolling window.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Rolling Maximum Drawdown — the deepest peak-to-trough decline within the
/// trailing window.
///
/// The input is treated as an equity-curve sample (or any non-negative value
/// series). For each bar the indicator computes the largest fractional decline
/// from any prior peak inside the trailing `period`-bar window:
///
/// ```text
/// drawdown_t = (equity_t − peak_t) / peak_t        (a negative number)
/// MaxDrawdown = min(drawdown_t over window)        (most-negative value)
/// ```
///
/// Output is the magnitude of the worst drawdown as a non-negative fraction
/// (`0.20` = 20 % drop from peak). A monotonically rising equity curve has a
/// max drawdown of `0`. Setting `period` greater than or equal to the number of
/// bars you will ever feed makes the metric effectively *cumulative* — the
/// indicator never forgets the global peak.
///
/// Each `update` is amortised O(1): the running peak is tracked with a
/// monotonically-decreasing deque.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, MaxDrawdown};
///
/// let mut mdd = MaxDrawdown::new(10).unwrap();
/// // Equity peaks at 110 then drops to 88 — a 20% drawdown.
/// for v in [100.0, 110.0, 100.0, 95.0, 88.0, 90.0, 92.0, 95.0, 100.0, 105.0] {
///     mdd.update(v);
/// }
/// assert!((mdd.update(106.0).unwrap() - 0.20).abs() < 1e-9);
/// ```
#[derive(Debug, Clone)]
pub struct MaxDrawdown {
    period: usize,
    count: u64,
    /// Monotonically-decreasing deque of `(index, value)` over the trailing
    /// window. Front is the trailing peak in O(1).
    peak_dq: VecDeque<(u64, f64)>,
    window: VecDeque<f64>,
    last: Option<f64>,
}

impl MaxDrawdown {
    /// Construct a new rolling Max Drawdown.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            count: 0,
            peak_dq: VecDeque::with_capacity(period),
            window: VecDeque::with_capacity(period),
            last: None,
        })
    }

    /// Configured rolling-window length.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for MaxDrawdown {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.last;
        }
        self.count += 1;
        // Drop tail entries dominated by the new value (running peak from the
        // back side of the window).
        while let Some(&(_, back)) = self.peak_dq.back() {
            if back <= input {
                self.peak_dq.pop_back();
            } else {
                break;
            }
        }
        self.peak_dq.push_back((self.count, input));
        // Window slide.
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(input);
        let window_lo = self.count.saturating_sub(self.period as u64 - 1);
        while let Some(&(idx, _)) = self.peak_dq.front() {
            if idx < window_lo {
                self.peak_dq.pop_front();
            } else {
                break;
            }
        }
        if self.window.len() < self.period {
            return None;
        }
        // Scan the window for the deepest drawdown vs running peak so far.
        let mut peak = f64::NEG_INFINITY;
        let mut worst = 0.0_f64;
        for &v in &self.window {
            if v > peak {
                peak = v;
            }
            if peak > 0.0 {
                let dd = (peak - v) / peak;
                if dd > worst {
                    worst = dd;
                }
            }
        }
        self.last = Some(worst);
        Some(worst)
    }

    fn reset(&mut self) {
        self.count = 0;
        self.peak_dq.clear();
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
        "MaxDrawdown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(MaxDrawdown::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let mut mdd = MaxDrawdown::new(10).unwrap();
        assert_eq!(mdd.period(), 10);
        assert_eq!(mdd.name(), "MaxDrawdown");
        assert_eq!(mdd.value(), None);
        assert_eq!(mdd.warmup_period(), 10);
        for v in 1..=10 {
            mdd.update(f64::from(v));
        }
        assert!(mdd.value().is_some());
    }

    #[test]
    fn pure_uptrend_yields_zero() {
        let mut mdd = MaxDrawdown::new(5).unwrap();
        let out = mdd.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        for v in out.into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn reference_drawdown() {
        // Window [100, 120, 90]: peak 120, trough 90 -> 25% drawdown.
        let mut mdd = MaxDrawdown::new(3).unwrap();
        let out = mdd.batch(&[100.0, 120.0, 90.0]);
        assert_eq!(out[0], None);
        assert_eq!(out[1], None);
        assert_relative_eq!(out[2].unwrap(), 0.25, epsilon = 1e-12);
    }

    #[test]
    fn constant_series_yields_zero() {
        let mut mdd = MaxDrawdown::new(4).unwrap();
        let out = mdd.batch(&[50.0; 12]);
        for v in out.into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut mdd = MaxDrawdown::new(3).unwrap();
        mdd.batch(&[100.0, 90.0, 80.0]);
        let last = mdd.value();
        assert_eq!(mdd.update(f64::NAN), last);
        assert_eq!(mdd.update(f64::INFINITY), last);
    }

    #[test]
    fn reset_clears_state() {
        let mut mdd = MaxDrawdown::new(3).unwrap();
        mdd.batch(&[100.0, 90.0, 80.0]);
        assert!(mdd.is_ready());
        mdd.reset();
        assert!(!mdd.is_ready());
        assert_eq!(mdd.update(100.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..60)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 10.0)
            .collect();
        let batch = MaxDrawdown::new(10).unwrap().batch(&prices);
        let mut s = MaxDrawdown::new(10).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| s.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn non_positive_peak_yields_zero() {
        // All-zero stream: peak is 0, division skipped, result stays 0.
        let mut mdd = MaxDrawdown::new(3).unwrap();
        let out = mdd.batch(&[0.0_f64; 6]);
        for v in out.into_iter().flatten() {
            assert_eq!(v, 0.0);
        }
    }
}
