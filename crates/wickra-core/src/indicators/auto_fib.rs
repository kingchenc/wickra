//! Auto-Fibonacci — retracement of the most significant recent swing leg.

use crate::indicators::pattern_swing::{SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// How many recent pivots to consider when picking the dominant leg.
const PIVOT_HISTORY: usize = 6;

/// The seven canonical retracement ratios, in ascending order.
const RATIOS: [f64; 7] = [0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0];

/// Auto-Fibonacci retracement levels for the dominant recent swing leg.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoFibOutput {
    /// 0.0% — the dominant leg's end.
    pub level_0: f64,
    /// 23.6% retracement.
    pub level_236: f64,
    /// 38.2% retracement.
    pub level_382: f64,
    /// 50% retracement.
    pub level_500: f64,
    /// 61.8% retracement.
    pub level_618: f64,
    /// 78.6% retracement.
    pub level_786: f64,
    /// 100% — the dominant leg's start.
    pub level_1000: f64,
}

/// Auto-Fibonacci (`AutoFib`).
///
/// Like [`crate::indicators::FibRetracement`], but instead of always using the
/// immediate last leg it scans the last six confirmed pivots and anchors the
/// retracement on the single largest-magnitude leg among them — the dominant
/// swing the market is most likely respecting.
///
/// Parameter-free; construction is infallible. Returns `None` until two pivots
/// have confirmed.
///
/// See `crates/wickra-core/src/indicators/auto_fib.rs`.
/// # Example
///
/// ```
/// use wickra_core::{AutoFib, Candle, Indicator};
///
/// let mut indicator = AutoFib::new();
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
pub struct AutoFib {
    swing: SwingTracker,
}

impl AutoFib {
    /// Construct a new Auto-Fibonacci tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, PIVOT_HISTORY),
        }
    }

    fn levels(&self) -> Option<AutoFibOutput> {
        let dominant = self.swing.pivots().windows(2).max_by(|x, y| {
            (x[0].price - x[1].price)
                .abs()
                .total_cmp(&(y[0].price - y[1].price).abs())
        })?;
        let (start, end) = (dominant[0].price, dominant[1].price);
        let level = |r: f64| end + r * (start - end);
        Some(AutoFibOutput {
            level_0: level(RATIOS[0]),
            level_236: level(RATIOS[1]),
            level_382: level(RATIOS[2]),
            level_500: level(RATIOS[3]),
            level_618: level(RATIOS[4]),
            level_786: level(RATIOS[5]),
            level_1000: level(RATIOS[6]),
        })
    }
}

impl Default for AutoFib {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for AutoFib {
    type Input = Candle;
    type Output = AutoFibOutput;

    fn update(&mut self, candle: Candle) -> Option<AutoFibOutput> {
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
        "AutoFib"
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
        let indicator = AutoFib::new();
        assert_eq!(indicator.name(), "AutoFib");
        assert_eq!(indicator.warmup_period(), 2);
        assert!(!indicator.is_ready());
        assert!(!AutoFib::default().is_ready());
    }

    #[test]
    fn no_output_before_two_pivots() {
        let mut indicator = AutoFib::new();
        let outputs: Vec<_> = candles_for_pivots(&[120.0])
            .into_iter()
            .map(|c| indicator.update(c))
            .collect();
        assert!(outputs.iter().all(Option::is_none));
    }

    #[test]
    fn anchors_on_the_largest_leg() {
        // Pivots: 130 -> 120 (small, 10) -> 220 (large, 100) -> 200 (small, 20).
        // The dominant leg is 120 -> 220; its retracement spans [120, 220].
        let mut indicator = AutoFib::new();
        let mut last = None;
        for candle in candles_for_pivots(&[130.0, 120.0, 220.0, 200.0]) {
            last = indicator.update(candle);
        }
        let v = last.unwrap();
        assert!(indicator.is_ready());
        // Largest leg 120 -> 220: 0% on 220 (end), 100% on 120 (start).
        assert_relative_eq!(v.level_0, 220.0);
        assert_relative_eq!(v.level_1000, 120.0);
        assert_relative_eq!(v.level_500, 170.0);
        assert_relative_eq!(v.level_618, 220.0 + 0.618 * (120.0 - 220.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = AutoFib::new();
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
        let candles = candles_for_pivots(&[130.0, 120.0, 220.0, 200.0]);
        let mut a = AutoFib::new();
        let mut b = AutoFib::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
