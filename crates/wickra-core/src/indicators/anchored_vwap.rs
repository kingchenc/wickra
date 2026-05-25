//! Anchored Volume-Weighted Average Price.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Anchored VWAP — a cumulative VWAP whose accumulation begins at a
/// user-chosen anchor bar rather than the session open.
///
/// ```text
/// AVWAP_t = Σ_{i ≥ anchor} (typical_price_i · volume_i) / Σ_{i ≥ anchor} volume_i
/// ```
///
/// The indicator emits `None` until the first anchored bar has been ingested.
/// Calling [`AnchoredVwap::set_anchor`] re-anchors at the **next** bar that
/// arrives, clearing the running sums; this is the conventional behaviour for
/// "click to anchor" trader workflows where the anchor is set on the close of
/// a swing point and the next bar starts the new accumulation. The cumulative
/// total is unbounded; for finite-memory needs use [`crate::RollingVwap`].
///
/// Bars where the running volume is still zero (only happens if every anchored
/// bar so far carried zero volume) return `None` to avoid a zero-division.
///
/// # Example
///
/// ```
/// use wickra_core::{AnchoredVwap, Candle, Indicator};
///
/// let mut indicator = AnchoredVwap::new();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     // Re-anchor at bar 40 (e.g. a major swing low).
///     if i == 40 {
///         indicator.set_anchor();
///     }
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct AnchoredVwap {
    sum_pv: f64,
    sum_v: f64,
    has_emitted: bool,
    pending_anchor: bool,
}

impl AnchoredVwap {
    /// Construct a fresh Anchored VWAP. The first bar to arrive is the anchor.
    pub const fn new() -> Self {
        Self {
            sum_pv: 0.0,
            sum_v: 0.0,
            has_emitted: false,
            pending_anchor: false,
        }
    }

    /// Mark a re-anchor: the **next** [`Indicator::update`] call clears the
    /// running sums before adding its own contribution, effectively starting a
    /// fresh anchored window.
    pub fn set_anchor(&mut self) {
        self.pending_anchor = true;
    }

    /// Current anchored value if at least one bar with non-zero volume has
    /// been observed in the current anchor window.
    pub fn value(&self) -> Option<f64> {
        if self.sum_v == 0.0 {
            None
        } else {
            Some(self.sum_pv / self.sum_v)
        }
    }
}

impl Indicator for AnchoredVwap {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        if self.pending_anchor {
            // Drop the old window before folding in this bar.
            self.sum_pv = 0.0;
            self.sum_v = 0.0;
            self.has_emitted = false;
            self.pending_anchor = false;
        }
        let tp = candle.typical_price();
        self.sum_pv += tp * candle.volume;
        self.sum_v += candle.volume;
        if self.sum_v == 0.0 {
            return None;
        }
        self.has_emitted = true;
        Some(self.sum_pv / self.sum_v)
    }

    fn reset(&mut self) {
        self.sum_pv = 0.0;
        self.sum_v = 0.0;
        self.has_emitted = false;
        self.pending_anchor = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "AnchoredVWAP"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(price: f64, volume: f64, ts: i64) -> Candle {
        Candle::new(price, price, price, price, volume, ts).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let v = AnchoredVwap::new();
        assert_eq!(v.name(), "AnchoredVWAP");
        assert_eq!(v.warmup_period(), 1);
        assert_eq!(v.value(), None);
    }

    #[test]
    fn first_bar_with_zero_volume_returns_none() {
        let mut v = AnchoredVwap::new();
        assert_eq!(v.update(c(50.0, 0.0, 0)), None);
        assert!(!v.is_ready());
        // The next bar with volume still works.
        assert_relative_eq!(v.update(c(10.0, 4.0, 1)).unwrap(), 10.0, epsilon = 1e-12);
    }

    #[test]
    fn equal_volumes_yield_mean_typical_price() {
        // typical_price of a flat OHLC bar equals the price.
        let mut v = AnchoredVwap::new();
        let out = v.batch(&[c(10.0, 1.0, 0), c(20.0, 1.0, 1), c(30.0, 1.0, 2)]);
        assert_relative_eq!(out[2].unwrap(), 20.0, epsilon = 1e-12);
    }

    #[test]
    fn set_anchor_clears_old_window() {
        // Run a few bars at price 10, then re-anchor and pump in price 100.
        // After the re-anchor the running mean must be 100, not the mix.
        let mut v = AnchoredVwap::new();
        v.batch(&[c(10.0, 1.0, 0), c(10.0, 1.0, 1), c(10.0, 1.0, 2)]);
        assert_relative_eq!(v.value().unwrap(), 10.0, epsilon = 1e-12);
        v.set_anchor();
        let after = v.update(c(100.0, 5.0, 3)).unwrap();
        assert_relative_eq!(after, 100.0, epsilon = 1e-12);
    }

    #[test]
    fn set_anchor_before_first_bar_acts_as_normal_first_bar() {
        // Calling set_anchor on an empty indicator should be a no-op effect:
        // the first bar still anchors the window.
        let mut v = AnchoredVwap::new();
        v.set_anchor();
        assert_relative_eq!(v.update(c(42.0, 2.0, 0)).unwrap(), 42.0, epsilon = 1e-12);
    }

    #[test]
    fn weighted_average_reference() {
        // Two bars: 10@1, 20@3 -> (10 + 60) / 4 = 17.5.
        let mut v = AnchoredVwap::new();
        let out = v.batch(&[c(10.0, 1.0, 0), c(20.0, 3.0, 1)]);
        assert_relative_eq!(out[1].unwrap(), 17.5, epsilon = 1e-12);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (1..30).map(|i| c(f64::from(i), 1.0, i.into())).collect();
        let mut a = AnchoredVwap::new();
        let mut b = AnchoredVwap::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut v = AnchoredVwap::new();
        v.batch(&[c(10.0, 1.0, 0), c(20.0, 1.0, 1)]);
        assert!(v.is_ready());
        v.reset();
        assert!(!v.is_ready());
        assert_eq!(v.value(), None);
        // After reset the first bar acts as the new anchor.
        assert_relative_eq!(v.update(c(50.0, 1.0, 2)).unwrap(), 50.0, epsilon = 1e-12);
    }
}
