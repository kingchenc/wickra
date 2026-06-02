//! Concealing Baby Swallow candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Returns `true` when `candle` is a black marubozu: a down candle whose body fills
/// the range with negligible shadows on both ends.
fn black_marubozu(candle: Candle) -> bool {
    let range = candle.high - candle.low;
    if range <= 0.0 {
        return false;
    }
    let upper = candle.high - candle.open;
    let lower = candle.close - candle.low;
    candle.open > candle.close && upper <= 0.05 * range && lower <= 0.05 * range
}

/// Concealing Baby Swallow — a rare 4-bar bullish reversal. Two black marubozu lead
/// a steep decline; the third is a black candle that gaps down on the open yet
/// throws a long upper shadow back up into the second body; the fourth is a large
/// black candle that completely engulfs the third, shadows included. The relentless
/// selling that can no longer make ground signals capitulation.
///
/// ```text
/// bar1, bar2 black marubozu (body == range, negligible shadows)
/// bar3 black, opens below bar2's body (open3 < close2) with an upper
///   shadow into it (high3 > close2)
/// bar4 black, engulfs bar3 including shadows: open4 > high3 and close4 < low3
/// ```
///
/// Output is `+1.0` when the pattern completes and `0.0` otherwise. Concealing Baby
/// Swallow is a single-direction (bullish-only) reversal, so it never emits `−1.0`.
/// The first three bars always return `0.0` because the four-bar window is not yet
/// filled. Body and shadow thresholds follow the geometric house style rather than
/// TA-Lib's rolling averages. Pattern-shape check only — no trend filter is applied;
/// combine with a trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` bullish, `0.0` no pattern — so it drops straight into
/// a machine-learning feature matrix as a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, ConcealingBabySwallow, Indicator};
///
/// let mut indicator = ConcealingBabySwallow::new();
/// indicator.update(Candle::new(20.0, 20.1, 14.9, 15.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(16.0, 16.1, 11.9, 12.0, 1.0, 1).unwrap());
/// indicator.update(Candle::new(11.0, 13.0, 9.9, 10.0, 1.0, 2).unwrap());
/// let out = indicator
///     .update(Candle::new(14.0, 14.1, 8.9, 9.0, 1.0, 3).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ConcealingBabySwallow {
    c1: Option<Candle>,
    c2: Option<Candle>,
    c3: Option<Candle>,
    has_emitted: bool,
}

impl ConcealingBabySwallow {
    /// Construct a new Concealing Baby Swallow detector.
    pub const fn new() -> Self {
        Self {
            c1: None,
            c2: None,
            c3: None,
            has_emitted: false,
        }
    }
}

impl Indicator for ConcealingBabySwallow {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let bar1 = self.c1;
        let bar2 = self.c2;
        let bar3 = self.c3;
        self.c1 = self.c2;
        self.c2 = self.c3;
        self.c3 = Some(candle);
        let (Some(bar1), Some(bar2), Some(bar3)) = (bar1, bar2, bar3) else {
            return Some(0.0);
        };
        // bar1 and bar2 are black marubozu.
        if !black_marubozu(bar1) || !black_marubozu(bar2) {
            return Some(0.0);
        }
        // bar3 is black, gaps down on the open, throws an upper shadow into bar2.
        if bar3.open <= bar3.close {
            return Some(0.0);
        }
        if bar3.open >= bar2.close {
            return Some(0.0); // no downside open gap
        }
        if bar3.high <= bar2.close {
            return Some(0.0); // upper shadow does not reach into bar2's body
        }
        // bar4 is black and engulfs bar3 including its shadows.
        if candle.open <= candle.close {
            return Some(0.0);
        }
        if candle.open > bar3.high && candle.close < bar3.low {
            return Some(1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.c1 = None;
        self.c2 = None;
        self.c3 = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        4
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "ConcealingBabySwallow"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(open: f64, high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(open, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let t = ConcealingBabySwallow::new();
        assert_eq!(t.name(), "ConcealingBabySwallow");
        assert_eq!(t.warmup_period(), 4);
        assert!(!t.is_ready());
    }

    #[test]
    fn concealing_baby_swallow_is_plus_one() {
        let mut t = ConcealingBabySwallow::new();
        assert_eq!(t.update(c(20.0, 20.1, 14.9, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(16.0, 16.1, 11.9, 12.0, 1)), Some(0.0));
        assert_eq!(t.update(c(11.0, 13.0, 9.9, 10.0, 2)), Some(0.0));
        assert_eq!(t.update(c(14.0, 14.1, 8.9, 9.0, 3)), Some(1.0));
    }

    #[test]
    fn warmup_returns_zero() {
        let mut t = ConcealingBabySwallow::new();
        assert_eq!(t.update(c(20.0, 20.1, 14.9, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(16.0, 16.1, 11.9, 12.0, 1)), Some(0.0));
        assert_eq!(t.update(c(11.0, 13.0, 9.9, 10.0, 2)), Some(0.0));
    }

    #[test]
    fn first_bar_not_marubozu_yields_zero() {
        let mut t = ConcealingBabySwallow::new();
        // bar1 white.
        t.update(c(15.0, 20.1, 14.9, 20.0, 0));
        t.update(c(16.0, 16.1, 11.9, 12.0, 1));
        t.update(c(11.0, 13.0, 9.9, 10.0, 2));
        assert_eq!(t.update(c(14.0, 14.1, 8.9, 9.0, 3)), Some(0.0));
    }

    #[test]
    fn first_bar_zero_range_yields_zero() {
        let mut t = ConcealingBabySwallow::new();
        // bar1 zero range -> not a marubozu.
        t.update(c(15.0, 15.0, 15.0, 15.0, 0));
        t.update(c(16.0, 16.1, 11.9, 12.0, 1));
        t.update(c(11.0, 13.0, 9.9, 10.0, 2));
        assert_eq!(t.update(c(14.0, 14.1, 8.9, 9.0, 3)), Some(0.0));
    }

    #[test]
    fn second_bar_not_marubozu_yields_zero() {
        let mut t = ConcealingBabySwallow::new();
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        // bar2 white.
        t.update(c(12.0, 16.1, 11.9, 16.0, 1));
        t.update(c(11.0, 13.0, 9.9, 10.0, 2));
        assert_eq!(t.update(c(14.0, 14.1, 8.9, 9.0, 3)), Some(0.0));
    }

    #[test]
    fn third_bar_not_black_yields_zero() {
        let mut t = ConcealingBabySwallow::new();
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        t.update(c(16.0, 16.1, 11.9, 12.0, 1));
        // bar3 white.
        t.update(c(11.0, 13.0, 9.9, 12.5, 2));
        assert_eq!(t.update(c(14.0, 14.1, 8.9, 9.0, 3)), Some(0.0));
    }

    #[test]
    fn third_bar_no_gap_yields_zero() {
        let mut t = ConcealingBabySwallow::new();
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        t.update(c(16.0, 16.1, 11.9, 12.0, 1));
        // bar3 black but opens at/above bar2's close -> no downside gap.
        t.update(c(12.5, 13.0, 9.9, 10.0, 2));
        assert_eq!(t.update(c(14.0, 14.1, 8.9, 9.0, 3)), Some(0.0));
    }

    #[test]
    fn third_bar_no_upper_shadow_yields_zero() {
        let mut t = ConcealingBabySwallow::new();
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        t.update(c(16.0, 16.1, 11.9, 12.0, 1));
        // bar3 black, gaps down, but its high does not reach into bar2's body.
        t.update(c(11.0, 11.5, 9.9, 10.0, 2));
        assert_eq!(t.update(c(14.0, 14.1, 8.9, 9.0, 3)), Some(0.0));
    }

    #[test]
    fn fourth_bar_not_black_yields_zero() {
        let mut t = ConcealingBabySwallow::new();
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        t.update(c(16.0, 16.1, 11.9, 12.0, 1));
        t.update(c(11.0, 13.0, 9.9, 10.0, 2));
        // bar4 white.
        assert_eq!(t.update(c(14.0, 14.1, 8.9, 14.05, 3)), Some(0.0));
    }

    #[test]
    fn fourth_bar_not_engulfing_yields_zero() {
        let mut t = ConcealingBabySwallow::new();
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        t.update(c(16.0, 16.1, 11.9, 12.0, 1));
        t.update(c(11.0, 13.0, 9.9, 10.0, 2));
        // bar4 black but does not engulf bar3's high.
        assert_eq!(t.update(c(12.5, 12.6, 8.9, 9.0, 3)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 200.0 - i as f64;
                c(base, base + 0.05, base - 5.0, base - 5.0, i)
            })
            .collect();
        let mut a = ConcealingBabySwallow::new();
        let mut b = ConcealingBabySwallow::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = ConcealingBabySwallow::new();
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        t.update(c(16.0, 16.1, 11.9, 12.0, 1));
        t.update(c(11.0, 13.0, 9.9, 10.0, 2));
        t.update(c(14.0, 14.1, 8.9, 9.0, 3));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(20.0, 20.1, 14.9, 15.0, 0)), Some(0.0));
    }
}
