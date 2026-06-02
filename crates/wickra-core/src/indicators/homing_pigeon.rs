//! Homing Pigeon candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Homing Pigeon — a 2-bar bullish reversal. Two black candles in a decline, the
/// second a small body sitting entirely inside the first body (a same-colour
/// harami). The shrinking range signals selling pressure is fading.
///
/// ```text
/// bar1 black (close < open)
/// bar2 black & its body sits inside bar1's body
///      (open2 <= open1  &&  close2 >= close1)
/// bar2 body is smaller than bar1's
/// ```
///
/// Output is `+1.0` when the pattern completes and `0.0` otherwise. Homing Pigeon
/// is a single-direction (bullish-only) reversal, so it never emits `−1.0`. The
/// first bar always returns `0.0` because the two-bar window is not yet filled.
/// Pattern-shape check only — no trend filter is applied; combine with a trend
/// indicator for actionable signals.
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
/// use wickra_core::{Candle, HomingPigeon, Indicator};
///
/// let mut indicator = HomingPigeon::new();
/// indicator.update(Candle::new(15.0, 15.1, 9.9, 10.0, 1.0, 0).unwrap());
/// let out = indicator
///     .update(Candle::new(14.0, 14.1, 10.9, 11.0, 1.0, 1).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct HomingPigeon {
    prev: Option<Candle>,
    has_emitted: bool,
}

impl HomingPigeon {
    /// Construct a new Homing Pigeon detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for HomingPigeon {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let prev = self.prev;
        self.prev = Some(candle);
        let Some(bar1) = prev else {
            return Some(0.0);
        };
        // Both bars black, bar2's body inside bar1's body and smaller.
        if bar1.close < bar1.open
            && candle.close < candle.open
            && candle.open <= bar1.open
            && candle.close >= bar1.close
            && (candle.open - candle.close) < (bar1.open - bar1.close)
        {
            return Some(1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.prev = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "HomingPigeon"
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
        let t = HomingPigeon::new();
        assert_eq!(t.name(), "HomingPigeon");
        assert_eq!(t.warmup_period(), 2);
        assert!(!t.is_ready());
    }

    #[test]
    fn homing_pigeon_is_plus_one() {
        let mut t = HomingPigeon::new();
        assert_eq!(t.update(c(15.0, 15.1, 9.9, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(14.0, 14.1, 10.9, 11.0, 1)), Some(1.0));
    }

    #[test]
    fn second_bar_white_yields_zero() {
        let mut t = HomingPigeon::new();
        t.update(c(15.0, 15.1, 9.9, 10.0, 0));
        // bar2 white -> not a homing pigeon.
        assert_eq!(t.update(c(11.0, 14.1, 10.9, 14.0, 1)), Some(0.0));
    }

    #[test]
    fn second_body_not_inside_yields_zero() {
        let mut t = HomingPigeon::new();
        t.update(c(15.0, 15.1, 9.9, 10.0, 0));
        // bar2 opens above bar1's open -> body not contained.
        assert_eq!(t.update(c(16.0, 16.1, 10.9, 11.0, 1)), Some(0.0));
    }

    #[test]
    fn first_bar_returns_zero() {
        let mut t = HomingPigeon::new();
        assert_eq!(t.update(c(15.0, 15.1, 9.9, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 5.0, base + 5.1, base - 0.1, base, i)
            })
            .collect();
        let mut a = HomingPigeon::new();
        let mut b = HomingPigeon::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = HomingPigeon::new();
        t.update(c(15.0, 15.1, 9.9, 10.0, 0));
        t.update(c(14.0, 14.1, 10.9, 11.0, 1));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(15.0, 15.1, 9.9, 10.0, 0)), Some(0.0));
    }
}
