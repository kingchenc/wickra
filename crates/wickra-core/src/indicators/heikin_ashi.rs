//! Heikin-Ashi candle transform.
#![allow(clippy::manual_midpoint)]
//!
//! Heikin-Ashi ("average bar" in Japanese) smooths an OHLC candle stream so
//! trends are easier to read at a glance. The transform is purely local except
//! that `ha_open` depends on the *previous* Heikin-Ashi candle, so it remains
//! a streaming O(1) state machine.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// One Heikin-Ashi candle.
///
/// Fields use the same names as the source `Candle` but represent the
/// transformed OHLC.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeikinAshiOutput {
    /// Heikin-Ashi open: midpoint of the previous Heikin-Ashi open and close.
    pub open: f64,
    /// Heikin-Ashi high: `max(real high, ha_open, ha_close)`.
    pub high: f64,
    /// Heikin-Ashi low: `min(real low, ha_open, ha_close)`.
    pub low: f64,
    /// Heikin-Ashi close: average of the real open, high, low, close.
    pub close: f64,
}

/// Streaming Heikin-Ashi transform.
///
/// Emits a [`HeikinAshiOutput`] for every input bar starting with the very
/// first, so `warmup_period` is 1 and `batch` returns `n` outputs for `n`
/// inputs.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, HeikinAshi, Indicator};
///
/// let mut ha = HeikinAshi::new();
/// let c = Candle::new(10.0, 11.0, 9.0, 10.5, 0.0, 0).unwrap();
/// let out = ha.update(c).unwrap();
/// // First bar: ha_open = (open + close) / 2 = 10.25.
/// assert!((out.open - 10.25).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Default)]
pub struct HeikinAshi {
    prev: Option<HeikinAshiOutput>,
}

impl HeikinAshi {
    /// Construct a fresh transform with no prior state.
    #[must_use]
    pub const fn new() -> Self {
        Self { prev: None }
    }

    /// Most recently emitted Heikin-Ashi candle, if any.
    pub const fn value(&self) -> Option<HeikinAshiOutput> {
        self.prev
    }
}

impl Indicator for HeikinAshi {
    type Input = Candle;
    type Output = HeikinAshiOutput;

    fn update(&mut self, candle: Candle) -> Option<HeikinAshiOutput> {
        let ha_close = (candle.open + candle.high + candle.low + candle.close) / 4.0;
        let ha_open = match self.prev {
            Some(p) => f64::midpoint(p.open, p.close),
            // Seed: average of the real open and close.
            None => f64::midpoint(candle.open, candle.close),
        };
        let ha_high = candle.high.max(ha_open).max(ha_close);
        let ha_low = candle.low.min(ha_open).min(ha_close);
        let out = HeikinAshiOutput {
            open: ha_open,
            high: ha_high,
            low: ha_low,
            close: ha_close,
        };
        self.prev = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.prev = None;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.prev.is_some()
    }

    fn name(&self) -> &'static str {
        "HeikinAshi"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn cnd(o: f64, h: f64, l: f64, c: f64) -> Candle {
        Candle::new(o, h, l, c, 0.0, 0).unwrap()
    }

    #[test]
    fn first_bar_seeds_open_from_real_open_close() {
        let mut ha = HeikinAshi::new();
        let out = ha.update(cnd(10.0, 12.0, 9.0, 11.0)).unwrap();
        assert_relative_eq!(out.open, (10.0 + 11.0) / 2.0, epsilon = 1e-12);
        assert_relative_eq!(out.close, (10.0 + 12.0 + 9.0 + 11.0) / 4.0, epsilon = 1e-12);
        // high/low must envelope ha_open & ha_close along with the real H/L.
        assert!(out.high >= out.open);
        assert!(out.high >= out.close);
        assert!(out.low <= out.open);
        assert!(out.low <= out.close);
    }

    #[test]
    fn second_bar_uses_previous_ha_midpoint_as_open() {
        let mut ha = HeikinAshi::new();
        let first = ha.update(cnd(10.0, 12.0, 9.0, 11.0)).unwrap();
        let second = ha.update(cnd(11.5, 13.0, 10.5, 12.0)).unwrap();
        assert_relative_eq!(
            second.open,
            (first.open + first.close) / 2.0,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            second.close,
            (11.5 + 13.0 + 10.5 + 12.0) / 4.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..50)
            .map(|i| {
                let p = 100.0 + f64::from(i);
                cnd(p, p + 1.5, p - 1.5, p + 0.5)
            })
            .collect();
        let mut a = HeikinAshi::new();
        let mut b = HeikinAshi::new();
        let batched = a.batch(&candles);
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batched, streamed);
    }

    #[test]
    fn ready_after_first_update() {
        let mut ha = HeikinAshi::new();
        assert!(!ha.is_ready());
        ha.update(cnd(10.0, 11.0, 9.0, 10.5));
        assert!(ha.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut ha = HeikinAshi::new();
        ha.update(cnd(10.0, 11.0, 9.0, 10.5));
        assert!(ha.is_ready());
        ha.reset();
        assert!(!ha.is_ready());
        assert!(ha.value().is_none());
        // After reset, the next bar re-seeds from real open/close.
        let out = ha.update(cnd(20.0, 22.0, 18.0, 21.0)).unwrap();
        assert_relative_eq!(out.open, (20.0 + 21.0) / 2.0, epsilon = 1e-12);
    }

    #[test]
    fn metadata() {
        let ha = HeikinAshi::new();
        assert_eq!(ha.warmup_period(), 1);
        assert_eq!(ha.name(), "HeikinAshi");
    }

    #[test]
    fn high_envelopes_open_and_close() {
        // Real high below the synthetic ha_open/close still inflates ha_high.
        let mut ha = HeikinAshi::new();
        // Bar 1 sets a baseline.
        ha.update(cnd(100.0, 101.0, 99.0, 100.5));
        // Bar 2 with an extreme close — ha_close = (50+50+50+200)/4 = 87.5,
        // ha_open = midpoint of prev open/close — and a real high of 200.
        let out = ha.update(cnd(50.0, 200.0, 50.0, 200.0)).unwrap();
        assert_eq!(out.high, 200.0);
        assert!(out.low <= out.open.min(out.close));
    }
}
