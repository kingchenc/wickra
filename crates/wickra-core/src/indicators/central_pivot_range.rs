//! Central Pivot Range (CPR) — the pivot plus its two central levels.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Output of [`CentralPivotRange`]: the pivot and the two central lines that
/// bracket it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CentralPivotRangeOutput {
    /// Pivot point `(high + low + close) / 3`.
    pub pivot: f64,
    /// Top central line — the higher of the two central levels.
    pub tc: f64,
    /// Bottom central line — the lower of the two central levels.
    pub bc: f64,
}

/// Central Pivot Range (CPR) — the classic pivot point flanked by two "central"
/// levels whose separation gauges the day's expected character.
///
/// ```text
/// pivot = (high + low + close) / 3
/// bc'   = (high + low) / 2
/// tc'   = 2·pivot − bc'
/// TC    = max(tc', bc'),   BC = min(tc', bc')
/// ```
///
/// The CPR is computed from the **previous** period's bar (feed it completed
/// daily/weekly bars). The width of the range `TC − BC` is the headline read: a
/// **narrow** CPR signals a likely trending day (price has little balance area to
/// chew through), while a **wide** CPR signals a likely range-bound, balanced
/// day. Price opening above the whole range is bullish, below it bearish, inside
/// it neutral. The `tc'`/`bc'` formulas are symmetric about the pivot; this
/// implementation labels the larger as `TC` and the smaller as `BC` so `TC >= BC`
/// always holds.
///
/// There are no parameters and no warmup — each completed bar yields one CPR.
/// Each `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, CentralPivotRange};
///
/// let mut indicator = CentralPivotRange::new();
/// let prev_day = Candle::new(101.0, 110.0, 90.0, 105.0, 1_000.0, 0).unwrap();
/// let cpr = indicator.update(prev_day).unwrap();
/// assert!(cpr.tc >= cpr.bc);
/// ```
#[derive(Debug, Clone, Default)]
pub struct CentralPivotRange {
    ready: bool,
}

impl CentralPivotRange {
    /// Construct a new Central Pivot Range. The indicator is parameter-free.
    #[must_use]
    pub const fn new() -> Self {
        Self { ready: false }
    }
}

impl Indicator for CentralPivotRange {
    type Input = Candle;
    type Output = CentralPivotRangeOutput;

    fn update(&mut self, candle: Candle) -> Option<CentralPivotRangeOutput> {
        let pivot = (candle.high + candle.low + candle.close) / 3.0;
        let bc_raw = f64::midpoint(candle.high, candle.low);
        let tc_raw = 2.0 * pivot - bc_raw;
        let tc = tc_raw.max(bc_raw);
        let bc = tc_raw.min(bc_raw);
        self.ready = true;
        Some(CentralPivotRangeOutput { pivot, tc, bc })
    }

    fn reset(&mut self) {
        self.ready = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn name(&self) -> &'static str {
        "CentralPivotRange"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(high: f64, low: f64, close: f64) -> Candle {
        Candle::new_unchecked(close, high, low, close, 1_000.0, 0)
    }

    #[test]
    fn accessors_and_metadata() {
        let cpr = CentralPivotRange::new();
        assert_eq!(cpr.warmup_period(), 1);
        assert_eq!(cpr.name(), "CentralPivotRange");
        assert!(!cpr.is_ready());
    }

    #[test]
    fn formula_reference_values() {
        // H=110, L=90, C=105 -> pivot = 305/3; bc' = 100; tc' = 2*pivot - 100.
        let out = CentralPivotRange::new()
            .update(c(110.0, 90.0, 105.0))
            .unwrap();
        let pivot = 305.0 / 3.0;
        let bc_raw = 100.0;
        let tc_raw = 2.0 * pivot - bc_raw;
        assert!((out.pivot - pivot).abs() < 1e-12);
        assert!((out.tc - tc_raw.max(bc_raw)).abs() < 1e-12);
        assert!((out.bc - tc_raw.min(bc_raw)).abs() < 1e-12);
    }

    #[test]
    fn tc_never_below_bc() {
        let out = CentralPivotRange::new()
            .update(c(200.0, 100.0, 150.0))
            .unwrap();
        assert!(out.tc >= out.bc);
    }

    #[test]
    fn constant_bar_collapses_range() {
        // H = L = C -> pivot = bc' = tc' = the price; range collapses.
        let out = CentralPivotRange::new()
            .update(c(50.0, 50.0, 50.0))
            .unwrap();
        assert_eq!(out.pivot, 50.0);
        assert_eq!(out.tc, 50.0);
        assert_eq!(out.bc, 50.0);
    }

    #[test]
    fn ready_after_first_update() {
        let mut cpr = CentralPivotRange::new();
        assert!(!cpr.is_ready());
        cpr.update(c(11.0, 9.0, 10.0));
        assert!(cpr.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut cpr = CentralPivotRange::new();
        cpr.update(c(11.0, 9.0, 10.0));
        assert!(cpr.is_ready());
        cpr.reset();
        assert!(!cpr.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| c(f64::from(i) + 2.0, f64::from(i), f64::from(i) + 1.0))
            .collect();
        let batch = CentralPivotRange::new().batch(&candles);
        let mut b = CentralPivotRange::new();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
