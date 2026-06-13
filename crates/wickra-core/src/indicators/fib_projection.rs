//! Fibonacci Projection — a measured move from the last three swing pivots.

use crate::indicators::pattern_swing::{SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// The four canonical projection ratios, in ascending order. Each scales the
/// A→B leg and projects it from C; `1.0` is the classic AB=CD measured move.
const RATIOS: [f64; 4] = [0.618, 1.0, 1.618, 2.618];

/// Fibonacci Projection levels (the C→D target zone of a measured move).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FibProjectionOutput {
    /// 61.8% projection of the A→B leg from C.
    pub level_618: f64,
    /// 100% projection — the AB=CD measured move.
    pub level_1000: f64,
    /// 161.8% projection.
    pub level_1618: f64,
    /// 261.8% projection.
    pub level_2618: f64,
}

/// Fibonacci Projection (`FibProjection`).
///
/// Reads the last three confirmed swing pivots as the points A, B and C of a
/// measured move and projects the A→B leg from C at the canonical ratios — the
/// price targets for the C→D leg.
///
/// Parameter-free; construction is infallible. Returns `None` until three
/// pivots have confirmed.
///
/// See `crates/wickra-core/src/indicators/fib_projection.rs`.
/// # Example
///
/// ```
/// use wickra_core::{FibProjection, Candle, Indicator};
///
/// let mut indicator = FibProjection::new();
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
pub struct FibProjection {
    swing: SwingTracker,
}

impl FibProjection {
    /// Construct a new Fibonacci Projection tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 3),
        }
    }

    fn levels(&self) -> Option<FibProjectionOutput> {
        let pivots = self.swing.pivots();
        let [a, b, c] = [
            pivots.first()?.price,
            pivots.get(1)?.price,
            pivots.get(2)?.price,
        ];
        let project = |p: f64| c + p * (b - a);
        Some(FibProjectionOutput {
            level_618: project(RATIOS[0]),
            level_1000: project(RATIOS[1]),
            level_1618: project(RATIOS[2]),
            level_2618: project(RATIOS[3]),
        })
    }
}

impl Default for FibProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for FibProjection {
    type Input = Candle;
    type Output = FibProjectionOutput;

    fn update(&mut self, candle: Candle) -> Option<FibProjectionOutput> {
        self.swing.update(candle);
        self.levels()
    }

    fn reset(&mut self) {
        self.swing.reset();
    }

    fn warmup_period(&self) -> usize {
        3
    }

    fn is_ready(&self) -> bool {
        self.swing.pivots().len() >= 3
    }

    fn name(&self) -> &'static str {
        "FibProjection"
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
        let indicator = FibProjection::new();
        assert_eq!(indicator.name(), "FibProjection");
        assert_eq!(indicator.warmup_period(), 3);
        assert!(!indicator.is_ready());
        assert!(!FibProjection::default().is_ready());
    }

    #[test]
    fn no_output_before_three_pivots() {
        let mut indicator = FibProjection::new();
        let outputs: Vec<_> = candles_for_pivots(&[200.0, 100.0])
            .into_iter()
            .map(|c| indicator.update(c))
            .collect();
        assert!(outputs.iter().all(Option::is_none));
        assert!(!indicator.is_ready());
    }

    #[test]
    fn measured_move_from_three_pivots() {
        // A = 200 (high), B = 160 (low), C = 190 (high). A->B = -40, projected
        // down from C.
        let mut indicator = FibProjection::new();
        let mut last = None;
        for candle in candles_for_pivots(&[200.0, 160.0, 190.0]) {
            last = indicator.update(candle);
        }
        let v = last.unwrap();
        assert!(indicator.is_ready());
        let (a, b, c) = (200.0, 160.0, 190.0);
        assert_relative_eq!(v.level_618, c + 0.618 * (b - a));
        assert_relative_eq!(v.level_1000, c + (b - a));
        assert_relative_eq!(v.level_1618, c + 1.618 * (b - a));
        assert_relative_eq!(v.level_2618, c + 2.618 * (b - a));
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = FibProjection::new();
        for candle in candles_for_pivots(&[200.0, 160.0, 190.0]) {
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
        let candles = candles_for_pivots(&[200.0, 160.0, 190.0, 150.0]);
        let mut a = FibProjection::new();
        let mut b = FibProjection::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
