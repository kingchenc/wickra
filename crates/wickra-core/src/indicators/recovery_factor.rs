//! Recovery Factor — cumulative net return over max drawdown.

use crate::traits::Indicator;

/// Recovery Factor.
///
/// Input is treated as an equity-curve sample (e.g. total account equity).
/// The indicator tracks the running all-time peak and the deepest drawdown
/// seen so far, plus the cumulative net return relative to the *first*
/// observation:
///
/// ```text
/// peak       = max(equity since start)
/// trough_dd  = max((peak − equity) / peak)
/// net_return = (equity_last / equity_first) − 1
/// Recovery   = net_return / trough_dd
/// ```
///
/// `Recovery > 1` means the strategy has earned more than it ever lost on
/// the way. A pure up-trend has no drawdown and the indicator reports `0.0`
/// (the ratio is undefined; zero by convention).
///
/// Cumulative-from-start rather than rolling-windowed: the user resets to
/// re-start the count. Each `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, RecoveryFactor};
///
/// let mut r = RecoveryFactor::new();
/// // Equity climbs, drops 20%, recovers and exceeds original peak.
/// for v in [100.0, 110.0, 105.0, 95.0, 88.0, 100.0, 120.0, 130.0] {
///     r.update(v);
/// }
/// assert!(r.value().unwrap() > 0.0);
/// ```
#[derive(Debug, Clone, Default)]
pub struct RecoveryFactor {
    first: f64,
    last: f64,
    peak: f64,
    max_dd: f64,
    seen: bool,
}

impl RecoveryFactor {
    /// Construct a new Recovery Factor tracker.
    pub const fn new() -> Self {
        Self {
            first: 0.0,
            last: 0.0,
            peak: f64::NEG_INFINITY,
            max_dd: 0.0,
            seen: false,
        }
    }

    /// Current value if available.
    pub fn value(&self) -> Option<f64> {
        if !self.seen || self.first == 0.0 {
            return None;
        }
        if self.max_dd == 0.0 {
            return Some(0.0);
        }
        let net_return = (self.last / self.first) - 1.0;
        Some(net_return / self.max_dd)
    }
}

impl Indicator for RecoveryFactor {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.value();
        }
        if self.seen {
            if input > self.peak {
                self.peak = input;
            }
            if self.peak > 0.0 {
                let dd = (self.peak - input) / self.peak;
                if dd > self.max_dd {
                    self.max_dd = dd;
                }
            }
        } else {
            self.first = input;
            self.peak = input;
            self.seen = true;
        }
        self.last = input;
        self.value()
    }

    fn reset(&mut self) {
        self.first = 0.0;
        self.last = 0.0;
        self.peak = f64::NEG_INFINITY;
        self.max_dd = 0.0;
        self.seen = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.seen && self.first != 0.0
    }

    fn name(&self) -> &'static str {
        "RecoveryFactor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn accessors_and_metadata() {
        let r = RecoveryFactor::new();
        assert_eq!(r.name(), "RecoveryFactor");
        assert_eq!(r.warmup_period(), 1);
        assert_eq!(r.value(), None);
    }

    #[test]
    fn pure_uptrend_yields_zero() {
        let mut r = RecoveryFactor::new();
        for v in 1..=10 {
            r.update(f64::from(v));
        }
        // max_dd == 0 -> 0 by convention.
        assert_eq!(r.value(), Some(0.0));
    }

    #[test]
    fn reference_value() {
        // Start 100, peak 110, trough 88 -> max_dd = 0.2.
        // End 130 -> net_return = 0.3 -> Recovery = 1.5.
        let mut r = RecoveryFactor::new();
        let out = r.batch(&[100.0, 110.0, 105.0, 95.0, 88.0, 100.0, 120.0, 130.0]);
        let last = out.last().copied().unwrap().unwrap();
        assert_relative_eq!(last, 0.30 / 0.20, epsilon = 1e-9);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut r = RecoveryFactor::new();
        r.update(100.0);
        r.update(90.0);
        let v = r.value();
        assert_eq!(r.update(f64::NAN), v);
        assert_eq!(r.update(f64::INFINITY), v);
    }

    #[test]
    fn first_value_alone_yields_zero() {
        // First update: max_dd is still 0 -> 0 by convention; value defined.
        let mut r = RecoveryFactor::new();
        assert_eq!(r.update(100.0), Some(0.0));
    }

    #[test]
    fn first_zero_equity_keeps_value_none() {
        // first == 0 means net-return division would be 0/0; indicator stays
        // not-ready until a non-zero baseline is reset in.
        let mut r = RecoveryFactor::new();
        assert_eq!(r.update(0.0), None);
        assert!(!r.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut r = RecoveryFactor::new();
        r.batch(&[100.0, 90.0, 80.0]);
        assert!(r.is_ready());
        r.reset();
        assert!(!r.is_ready());
        assert_eq!(r.update(100.0), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..40)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 8.0)
            .collect();
        let batch = RecoveryFactor::new().batch(&prices);
        let mut s = RecoveryFactor::new();
        let streamed: Vec<_> = prices.iter().map(|p| s.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn non_positive_peak_skips_drawdown_calc() {
        // All inputs <= 0 keep `peak` non-positive, so the guarded drawdown
        // computation is skipped on every step. Exercises the `else` branch
        // of `if self.peak > 0.0`.
        let mut r = RecoveryFactor::new();
        assert_eq!(r.update(-1.0), Some(0.0));
        assert_eq!(r.update(-2.0), Some(0.0));
        assert_eq!(r.update(-0.5), Some(0.0));
        assert!(r.is_ready());
    }
}
