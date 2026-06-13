//! Golden Pocket — the 0.618-0.65 optimal-trade-entry zone of the last swing.

use crate::indicators::pattern_swing::{SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Lower bound of the golden pocket (the 61.8% retracement).
const RATIO_LOW: f64 = 0.618;
/// Upper bound of the golden pocket (the 65% retracement).
const RATIO_HIGH: f64 = 0.65;

/// The golden-pocket zone of the most recent swing leg.
///
/// `low`/`high` bracket the 0.618-0.65 retracement band (sorted, so `low <=
/// high` regardless of swing direction); `mid` is their midpoint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GoldenPocketOutput {
    /// Lower price of the golden-pocket band.
    pub low: f64,
    /// Midpoint of the band.
    pub mid: f64,
    /// Upper price of the golden-pocket band.
    pub high: f64,
}

/// Golden Pocket (`GoldenPocket`).
///
/// The 0.618-0.65 retracement band of the most recent confirmed swing leg — the
/// "optimal trade entry" zone many swing traders watch for continuation.
///
/// Parameter-free; construction is infallible. Returns `None` until the first
/// leg is complete.
///
/// See `crates/wickra-core/src/indicators/golden_pocket.rs`.
/// # Example
///
/// ```
/// use wickra_core::{GoldenPocket, Candle, Indicator};
///
/// let mut indicator = GoldenPocket::new();
/// // `None` during warmup, then `Some(_)` once enough bars are seen.
/// let mut out = None;
/// for i in 0..40i64 {
///     let p = 100.0 + (i as f64 * 0.4).sin() * 5.0;
///     let candle = Candle::new(p, p + 1.5, p - 1.5, p + 0.3, 1_000.0, i).unwrap();
///     out = indicator.update(candle);
/// }
/// let _ = out;
/// ```
#[derive(Debug, Clone)]
pub struct GoldenPocket {
    swing: SwingTracker,
}

impl GoldenPocket {
    /// Construct a new Golden Pocket tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 2),
        }
    }

    fn zone(&self) -> Option<GoldenPocketOutput> {
        let pivots = self.swing.pivots();
        let [start, end] = [pivots.first()?.price, pivots.get(1)?.price];
        let span = start - end;
        let edge_low = end + RATIO_LOW * span;
        let edge_high = end + RATIO_HIGH * span;
        let low = edge_low.min(edge_high);
        let high = edge_low.max(edge_high);
        Some(GoldenPocketOutput {
            low,
            mid: f64::midpoint(low, high),
            high,
        })
    }
}

impl Default for GoldenPocket {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for GoldenPocket {
    type Input = Candle;
    type Output = GoldenPocketOutput;

    fn update(&mut self, candle: Candle) -> Option<GoldenPocketOutput> {
        self.swing.update(candle);
        self.zone()
    }

    fn reset(&mut self) {
        self.swing.reset();
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.swing.pivots().len() >= 2
    }

    fn name(&self) -> &'static str {
        "GoldenPocket"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn accessors_and_metadata() {
        let indicator = GoldenPocket::new();
        assert_eq!(indicator.name(), "GoldenPocket");
        assert_eq!(indicator.warmup_period(), 2);
        assert!(!indicator.is_ready());
        assert!(!GoldenPocket::default().is_ready());
    }

    #[test]
    fn no_output_before_two_pivots() {
        let mut indicator = GoldenPocket::new();
        let outputs: Vec<_> = candles_for_pivots(&[120.0])
            .into_iter()
            .map(|c| indicator.update(c))
            .collect();
        assert!(outputs.iter().all(Option::is_none));
    }

    #[test]
    fn zone_of_a_down_leg() {
        // Leg 200 (high) -> 100 (low), span = 100.
        let mut indicator = GoldenPocket::new();
        let mut last = None;
        for candle in candles_for_pivots(&[200.0, 100.0]) {
            last = indicator.update(candle);
        }
        let v = last.unwrap();
        assert!(indicator.is_ready());
        // 61.8% = 161.8, 65% = 165 → sorted band [161.8, 165], mid 163.4.
        assert_relative_eq!(v.low, 161.8);
        assert_relative_eq!(v.high, 165.0);
        assert_relative_eq!(v.mid, 163.4);
    }

    #[test]
    fn band_is_sorted_for_an_up_leg() {
        // Latest leg 100 (low) -> 250 (high): span negative, edges flip, but
        // low <= high must still hold.
        let mut indicator = GoldenPocket::new();
        let mut last = None;
        for candle in candles_for_pivots(&[200.0, 100.0, 250.0]) {
            last = indicator.update(candle);
        }
        let v = last.unwrap();
        assert!(v.low <= v.high);
        assert_relative_eq!(v.mid, f64::midpoint(v.low, v.high));
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = GoldenPocket::new();
        for candle in candles_for_pivots(&[200.0, 100.0]) {
            let _ = indicator.update(candle);
        }
        indicator.reset();
        assert!(!indicator.is_ready());
        let c = Candle::new(99.5, 100.0, 99.5, 99.5, 1.0, 0).unwrap();
        assert!(indicator.update(c).is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = candles_for_pivots(&[200.0, 100.0, 150.0]);
        let mut a = GoldenPocket::new();
        let mut b = GoldenPocket::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
