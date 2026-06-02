//! Upside Gap Three Methods candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Upside Gap Three Methods — a 3-bar bullish continuation. Two white candles
/// advance with an upside body gap between them, then a black candle opens inside
/// the second body and closes inside the first body, partially filling the gap
/// without erasing the prior advance.
///
/// ```text
/// bar1 white, bar2 white
/// upside body gap: open2 > close1   (bar2's body sits entirely above bar1's)
/// bar3 black, opens within bar2's body and closes within bar1's body
/// ```
///
/// Output is `+1.0` when the pattern completes and `0.0` otherwise. Upside Gap
/// Three Methods is a single-direction (bullish-only) continuation, so it never
/// emits `−1.0`; its bearish mirror is [`crate::DownsideGapThreeMethods`]. The
/// first two bars always return `0.0` because the three-bar window is not yet
/// filled. Pattern-shape check only — no trend filter is applied; combine with a
/// trend indicator for actionable signals.
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
/// use wickra_core::{Candle, Indicator, UpsideGapThreeMethods};
///
/// let mut indicator = UpsideGapThreeMethods::new();
/// indicator.update(Candle::new(10.0, 11.2, 9.8, 11.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(12.0, 13.2, 11.9, 13.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(12.5, 12.6, 10.4, 10.5, 1.0, 2).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct UpsideGapThreeMethods {
    c1: Option<Candle>,
    c2: Option<Candle>,
    has_emitted: bool,
}

impl UpsideGapThreeMethods {
    /// Construct a new Upside Gap Three Methods detector.
    pub const fn new() -> Self {
        Self {
            c1: None,
            c2: None,
            has_emitted: false,
        }
    }
}

impl Indicator for UpsideGapThreeMethods {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let bar1 = self.c1;
        let bar2 = self.c2;
        self.c1 = self.c2;
        self.c2 = Some(candle);
        let (Some(bar1), Some(bar2)) = (bar1, bar2) else {
            return Some(0.0);
        };
        // bar1 and bar2 are both white.
        if bar1.close <= bar1.open || bar2.close <= bar2.open {
            return Some(0.0);
        }
        // Upside body gap: bar2's body sits entirely above bar1's.
        if bar2.open <= bar1.close {
            return Some(0.0);
        }
        // bar3 is black.
        if candle.close >= candle.open {
            return Some(0.0);
        }
        // bar3 opens within bar2's body and closes within bar1's body.
        if candle.open > bar2.open
            && candle.open < bar2.close
            && candle.close > bar1.open
            && candle.close < bar1.close
        {
            return Some(1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.c1 = None;
        self.c2 = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        3
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "UpsideGapThreeMethods"
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
        let t = UpsideGapThreeMethods::new();
        assert_eq!(t.name(), "UpsideGapThreeMethods");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
    }

    #[test]
    fn upside_gap_three_methods_is_plus_one() {
        let mut t = UpsideGapThreeMethods::new();
        assert_eq!(t.update(c(10.0, 11.2, 9.8, 11.0, 0)), Some(0.0));
        assert_eq!(t.update(c(12.0, 13.2, 11.9, 13.0, 1)), Some(0.0));
        assert_eq!(t.update(c(12.5, 12.6, 10.4, 10.5, 2)), Some(1.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = UpsideGapThreeMethods::new();
        assert_eq!(t.update(c(10.0, 11.2, 9.8, 11.0, 0)), Some(0.0));
        assert_eq!(t.update(c(12.0, 13.2, 11.9, 13.0, 1)), Some(0.0));
    }

    #[test]
    fn non_white_first_bars_yield_zero() {
        let mut t = UpsideGapThreeMethods::new();
        // bar1 is black.
        t.update(c(11.0, 11.2, 9.8, 10.0, 0));
        t.update(c(12.0, 13.2, 11.9, 13.0, 1));
        assert_eq!(t.update(c(12.5, 12.6, 10.4, 10.5, 2)), Some(0.0));
    }

    #[test]
    fn no_gap_yields_zero() {
        let mut t = UpsideGapThreeMethods::new();
        t.update(c(10.0, 13.2, 9.8, 13.0, 0));
        // bar2 opens below bar1's close -> no upside body gap.
        t.update(c(11.0, 13.2, 10.9, 12.5, 1));
        assert_eq!(t.update(c(12.0, 12.6, 10.4, 10.5, 2)), Some(0.0));
    }

    #[test]
    fn third_bar_not_black_yields_zero() {
        let mut t = UpsideGapThreeMethods::new();
        t.update(c(10.0, 11.2, 9.8, 11.0, 0));
        t.update(c(12.0, 13.2, 11.9, 13.0, 1));
        // bar3 white.
        assert_eq!(t.update(c(10.5, 12.6, 10.4, 12.5, 2)), Some(0.0));
    }

    #[test]
    fn third_bar_outside_bodies_yields_zero() {
        let mut t = UpsideGapThreeMethods::new();
        t.update(c(10.0, 11.2, 9.8, 11.0, 0));
        t.update(c(12.0, 13.2, 11.9, 13.0, 1));
        // bar3 black but closes below bar1's body.
        assert_eq!(t.update(c(12.5, 12.6, 8.9, 9.0, 2)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 5.2, base - 0.1, base + 5.0, i)
            })
            .collect();
        let mut a = UpsideGapThreeMethods::new();
        let mut b = UpsideGapThreeMethods::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = UpsideGapThreeMethods::new();
        t.update(c(10.0, 11.2, 9.8, 11.0, 0));
        t.update(c(12.0, 13.2, 11.9, 13.0, 1));
        t.update(c(12.5, 12.6, 10.4, 10.5, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(10.0, 11.2, 9.8, 11.0, 0)), Some(0.0));
    }
}
