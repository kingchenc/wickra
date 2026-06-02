//! Downside Gap Three Methods candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Downside Gap Three Methods — a 3-bar bearish continuation. Two black candles
/// decline with a downside body gap between them, then a white candle opens inside
/// the second body and closes inside the first body, partially filling the gap
/// without erasing the prior decline.
///
/// ```text
/// bar1 black, bar2 black
/// downside body gap: open2 < close1   (bar2's body sits entirely below bar1's)
/// bar3 white, opens within bar2's body and closes within bar1's body
/// ```
///
/// Output is `−1.0` when the pattern completes and `0.0` otherwise. Downside Gap
/// Three Methods is a single-direction (bearish-only) continuation, so it never
/// emits `+1.0`; its bullish mirror is [`crate::UpsideGapThreeMethods`]. The first
/// two bars always return `0.0` because the three-bar window is not yet filled.
/// Pattern-shape check only — no trend filter is applied; combine with a trend
/// indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `−1.0` bearish, `0.0` no pattern — so it drops straight into
/// a machine-learning feature matrix as a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, DownsideGapThreeMethods, Indicator};
///
/// let mut indicator = DownsideGapThreeMethods::new();
/// indicator.update(Candle::new(13.0, 13.2, 11.8, 12.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(11.0, 11.1, 9.8, 10.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(10.5, 12.6, 10.4, 12.5, 1.0, 2).unwrap());
/// assert_eq!(out, Some(-1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct DownsideGapThreeMethods {
    c1: Option<Candle>,
    c2: Option<Candle>,
    has_emitted: bool,
}

impl DownsideGapThreeMethods {
    /// Construct a new Downside Gap Three Methods detector.
    pub const fn new() -> Self {
        Self {
            c1: None,
            c2: None,
            has_emitted: false,
        }
    }
}

impl Indicator for DownsideGapThreeMethods {
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
        // bar1 and bar2 are both black.
        if bar1.close >= bar1.open || bar2.close >= bar2.open {
            return Some(0.0);
        }
        // Downside body gap: bar2's body sits entirely below bar1's.
        if bar2.open >= bar1.close {
            return Some(0.0);
        }
        // bar3 is white.
        if candle.close <= candle.open {
            return Some(0.0);
        }
        // bar3 opens within bar2's body and closes within bar1's body.
        if candle.open > bar2.close
            && candle.open < bar2.open
            && candle.close > bar1.close
            && candle.close < bar1.open
        {
            return Some(-1.0);
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
        "DownsideGapThreeMethods"
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
        let t = DownsideGapThreeMethods::new();
        assert_eq!(t.name(), "DownsideGapThreeMethods");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
    }

    #[test]
    fn downside_gap_three_methods_is_minus_one() {
        let mut t = DownsideGapThreeMethods::new();
        assert_eq!(t.update(c(13.0, 13.2, 11.8, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(11.0, 11.1, 9.8, 10.0, 1)), Some(0.0));
        assert_eq!(t.update(c(10.5, 12.6, 10.4, 12.5, 2)), Some(-1.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = DownsideGapThreeMethods::new();
        assert_eq!(t.update(c(13.0, 13.2, 11.8, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(11.0, 11.1, 9.8, 10.0, 1)), Some(0.0));
    }

    #[test]
    fn non_black_first_bars_yield_zero() {
        let mut t = DownsideGapThreeMethods::new();
        // bar1 is white.
        t.update(c(11.0, 13.2, 10.8, 13.0, 0));
        t.update(c(11.0, 11.1, 9.8, 10.0, 1));
        assert_eq!(t.update(c(10.5, 12.6, 10.4, 12.5, 2)), Some(0.0));
    }

    #[test]
    fn no_gap_yields_zero() {
        let mut t = DownsideGapThreeMethods::new();
        t.update(c(13.0, 13.2, 11.8, 12.0, 0));
        // bar2 opens above bar1's close -> no downside body gap.
        t.update(c(12.5, 12.6, 11.4, 11.5, 1));
        assert_eq!(t.update(c(11.5, 12.6, 11.4, 12.0, 2)), Some(0.0));
    }

    #[test]
    fn third_bar_not_white_yields_zero() {
        let mut t = DownsideGapThreeMethods::new();
        t.update(c(13.0, 13.2, 11.8, 12.0, 0));
        t.update(c(11.0, 11.1, 9.8, 10.0, 1));
        // bar3 black.
        assert_eq!(t.update(c(12.5, 12.6, 10.4, 10.5, 2)), Some(0.0));
    }

    #[test]
    fn third_bar_outside_bodies_yields_zero() {
        let mut t = DownsideGapThreeMethods::new();
        t.update(c(13.0, 13.2, 11.8, 12.0, 0));
        t.update(c(11.0, 11.1, 9.8, 10.0, 1));
        // bar3 white but closes above bar1's body.
        assert_eq!(t.update(c(10.5, 14.0, 10.4, 13.5, 2)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 200.0 - i as f64;
                c(base, base + 0.1, base - 5.2, base - 5.0, i)
            })
            .collect();
        let mut a = DownsideGapThreeMethods::new();
        let mut b = DownsideGapThreeMethods::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = DownsideGapThreeMethods::new();
        t.update(c(13.0, 13.2, 11.8, 12.0, 0));
        t.update(c(11.0, 11.1, 9.8, 10.0, 1));
        t.update(c(10.5, 12.6, 10.4, 12.5, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(13.0, 13.2, 11.8, 12.0, 0)), Some(0.0));
    }
}
