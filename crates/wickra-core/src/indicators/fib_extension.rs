//! Fibonacci Extension of the most recent confirmed swing leg.

use crate::indicators::pattern_swing::{SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// The five canonical extension ratios, in ascending order. Each is a multiple
/// of the swing leg measured from its origin, so `1.0` sits on the leg's end and
/// every ratio here projects further in the direction of the move.
const RATIOS: [f64; 5] = [1.272, 1.414, 1.618, 2.0, 2.618];

/// Fibonacci Extension levels for the most recent swing leg.
///
/// Each field is the price reached if the move continues to the matching
/// multiple of the leg, measured from the leg's start.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FibExtensionOutput {
    /// 127.2% extension.
    pub level_1272: f64,
    /// 141.4% extension.
    pub level_1414: f64,
    /// 161.8% extension — the "golden" extension.
    pub level_1618: f64,
    /// 200% extension.
    pub level_2000: f64,
    /// 261.8% extension.
    pub level_2618: f64,
}

/// Fibonacci Extension (`FibExtension`).
///
/// Tracks confirmed swing pivots with a baked-in 5% reversal threshold and, once
/// two pivots exist, projects the leg between them to the canonical extension
/// ratios — the price targets a continuation of the move would reach.
///
/// Parameter-free; construction is infallible. Returns `None` until the first
/// leg is complete.
///
/// See `crates/wickra-core/src/indicators/fib_extension.rs`.
/// # Example
///
/// ```
/// use wickra_core::{FibExtension, Candle, Indicator};
///
/// let mut indicator = FibExtension::new();
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
pub struct FibExtension {
    swing: SwingTracker,
}

impl FibExtension {
    /// Construct a new Fibonacci Extension tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 2),
        }
    }

    /// Extension price at ratio `e` for a leg from `start` to `end`: the total
    /// move is `e` times the leg, measured from `start`.
    fn level(start: f64, end: f64, e: f64) -> f64 {
        start + e * (end - start)
    }

    fn levels(&self) -> Option<FibExtensionOutput> {
        let pivots = self.swing.pivots();
        let [start, end] = [pivots.first()?.price, pivots.get(1)?.price];
        Some(FibExtensionOutput {
            level_1272: Self::level(start, end, RATIOS[0]),
            level_1414: Self::level(start, end, RATIOS[1]),
            level_1618: Self::level(start, end, RATIOS[2]),
            level_2000: Self::level(start, end, RATIOS[3]),
            level_2618: Self::level(start, end, RATIOS[4]),
        })
    }
}

impl Default for FibExtension {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for FibExtension {
    type Input = Candle;
    type Output = FibExtensionOutput;

    fn update(&mut self, candle: Candle) -> Option<FibExtensionOutput> {
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
        "FibExtension"
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
        let indicator = FibExtension::new();
        assert_eq!(indicator.name(), "FibExtension");
        assert_eq!(indicator.warmup_period(), 2);
        assert!(!indicator.is_ready());
        assert!(!FibExtension::default().is_ready());
    }

    #[test]
    fn no_output_before_two_pivots() {
        let mut indicator = FibExtension::new();
        let outputs: Vec<_> = candles_for_pivots(&[120.0])
            .into_iter()
            .map(|c| indicator.update(c))
            .collect();
        assert!(outputs.iter().all(Option::is_none));
    }

    #[test]
    fn extension_levels_of_a_down_leg() {
        // Leg start = 200 (high), end = 100 (low): a 100-point drop continued.
        let mut indicator = FibExtension::new();
        let mut last = None;
        for candle in candles_for_pivots(&[200.0, 100.0]) {
            last = indicator.update(candle);
        }
        let v = last.unwrap();
        assert!(indicator.is_ready());
        // 161.8% extension projects 1.618 * (-100) below the 200 origin.
        assert_relative_eq!(v.level_1272, 200.0 - 127.2);
        assert_relative_eq!(v.level_1414, 200.0 - 141.4);
        assert_relative_eq!(v.level_1618, 200.0 - 161.8);
        assert_relative_eq!(v.level_2000, 0.0);
        assert_relative_eq!(v.level_2618, 200.0 - 261.8);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = FibExtension::new();
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
        let mut a = FibExtension::new();
        let mut b = FibExtension::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
