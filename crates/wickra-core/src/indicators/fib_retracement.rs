//! Fibonacci Retracement of the most recent confirmed swing leg.

use crate::indicators::pattern_swing::{SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// The seven canonical retracement ratios, in ascending order. `0.0` marks the
/// most recent swing extreme (the end of the leg) and `1.0` the swing origin
/// (its start); the interior ratios are the classic Fibonacci pullbacks.
const RATIOS: [f64; 7] = [0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0];

/// Fibonacci Retracement levels for the most recent swing leg.
///
/// Each field is the price at the matching retracement ratio, measured from the
/// leg's end (`level_0`, the latest confirmed extreme) back toward its start
/// (`level_1000`, the prior pivot).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FibRetracementOutput {
    /// 0.0% — the most recent confirmed swing extreme.
    pub level_0: f64,
    /// 23.6% retracement.
    pub level_236: f64,
    /// 38.2% retracement.
    pub level_382: f64,
    /// 50% retracement (not a Fibonacci ratio, but conventionally drawn).
    pub level_500: f64,
    /// 61.8% retracement — the "golden ratio" pullback.
    pub level_618: f64,
    /// 78.6% retracement.
    pub level_786: f64,
    /// 100% — the swing origin.
    pub level_1000: f64,
}

/// Fibonacci Retracement (`FibRetracement`).
///
/// Tracks confirmed swing pivots with a baked-in 5% reversal threshold (the
/// same non-repainting logic as [`crate::indicators::ZigZag`]) and, once two
/// pivots exist, reports the seven retracement levels of the leg between them.
///
/// The levels are recomputed each time a new pivot confirms; between
/// confirmations [`Indicator::update`] returns the locked levels of the current
/// leg. Before the first leg is complete it returns `None`.
///
/// Parameter-free: the threshold is a compile-time constant, mirroring the
/// chart- and harmonic-pattern detectors, so construction is infallible.
///
/// See `crates/wickra-core/src/indicators/fib_retracement.rs`.
/// # Example
///
/// ```
/// use wickra_core::{FibRetracement, Candle, Indicator};
///
/// let mut indicator = FibRetracement::new();
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
pub struct FibRetracement {
    swing: SwingTracker,
}

impl FibRetracement {
    /// Construct a new Fibonacci Retracement tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 2),
        }
    }

    /// Retracement price at ratio `r` for a leg from `start` to `end`: `0.0`
    /// sits on `end`, `1.0` on `start`.
    fn level(start: f64, end: f64, r: f64) -> f64 {
        end + r * (start - end)
    }

    fn levels(&self) -> Option<FibRetracementOutput> {
        let pivots = self.swing.pivots();
        let [start, end] = [pivots.first()?.price, pivots.get(1)?.price];
        Some(FibRetracementOutput {
            level_0: Self::level(start, end, RATIOS[0]),
            level_236: Self::level(start, end, RATIOS[1]),
            level_382: Self::level(start, end, RATIOS[2]),
            level_500: Self::level(start, end, RATIOS[3]),
            level_618: Self::level(start, end, RATIOS[4]),
            level_786: Self::level(start, end, RATIOS[5]),
            level_1000: Self::level(start, end, RATIOS[6]),
        })
    }
}

impl Default for FibRetracement {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for FibRetracement {
    type Input = Candle;
    type Output = FibRetracementOutput;

    fn update(&mut self, candle: Candle) -> Option<FibRetracementOutput> {
        self.swing.update(candle);
        self.levels()
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
        "FibRetracement"
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
        let indicator = FibRetracement::new();
        assert_eq!(indicator.name(), "FibRetracement");
        assert_eq!(indicator.warmup_period(), 2);
        assert!(!indicator.is_ready());
        assert!(!FibRetracement::default().is_ready());
    }

    #[test]
    fn no_output_before_two_pivots() {
        let mut indicator = FibRetracement::new();
        // A single confirmed pivot is not enough to define a leg.
        let candles = candles_for_pivots(&[120.0]);
        let outputs: Vec<_> = candles.into_iter().map(|c| indicator.update(c)).collect();
        assert!(outputs.iter().all(Option::is_none));
        assert!(!indicator.is_ready());
    }

    #[test]
    fn retracement_levels_of_a_down_leg() {
        // Leg start = 200 (high), end = 100 (low): a 100-point drop.
        let mut indicator = FibRetracement::new();
        let mut last = None;
        for candle in candles_for_pivots(&[200.0, 100.0]) {
            last = indicator.update(candle);
        }
        let v = last.unwrap();
        assert!(indicator.is_ready());
        // 0% on the low (end), 100% on the high (start).
        assert_relative_eq!(v.level_0, 100.0);
        assert_relative_eq!(v.level_1000, 200.0);
        // 61.8% retracement of a 100-point drop, measured up from the low.
        assert_relative_eq!(v.level_618, 161.8);
        assert_relative_eq!(v.level_500, 150.0);
        assert_relative_eq!(v.level_382, 138.2);
        assert_relative_eq!(v.level_236, 123.6);
        assert_relative_eq!(v.level_786, 178.6);
    }

    #[test]
    fn levels_refresh_on_a_new_leg() {
        // Four pivots, cap = 2: once the third and fourth confirm, the reported
        // leg shifts to the latest pair (130 high -> 90 low).
        let mut indicator = FibRetracement::new();
        let mut last = None;
        for candle in candles_for_pivots(&[200.0, 100.0, 130.0, 90.0]) {
            last = indicator.update(candle);
        }
        let v = last.unwrap();
        assert_relative_eq!(v.level_0, 90.0);
        assert_relative_eq!(v.level_1000, 130.0);
        assert_relative_eq!(v.level_618, 90.0 + 0.618 * 40.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = FibRetracement::new();
        for candle in candles_for_pivots(&[200.0, 100.0]) {
            let _ = indicator.update(candle);
        }
        assert!(indicator.is_ready());
        indicator.reset();
        assert!(!indicator.is_ready());
        let c = Candle::new(99.5, 100.0, 99.5, 99.5, 1.0, 0).unwrap();
        assert!(indicator.update(c).is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = candles_for_pivots(&[200.0, 100.0, 150.0]);
        let mut a = FibRetracement::new();
        let mut b = FibRetracement::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
