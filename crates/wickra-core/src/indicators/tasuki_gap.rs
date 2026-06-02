//! Tasuki Gap candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Tasuki Gap — a 3-bar continuation. Two same-coloured candles open a body gap in
/// the trend direction, then an opposite-coloured candle opens inside the second
/// body and closes back *into* the gap without filling it — the gap holds, so the
/// trend is expected to continue.
///
/// ```text
/// Upside (bullish, +1):
///   bar1 white, bar2 white with an upside body gap (open2 > close1)
///   bar3 black, opens within bar2's body, closes inside the gap
///   (close1 < close3 < open2)
/// Downside (bearish, −1): the mirror image with black candles and a downside gap
/// ```
///
/// Output is `+1.0` for an upside Tasuki gap, `−1.0` for a downside one, and `0.0`
/// otherwise. The first two bars always return `0.0` because the three-bar window
/// is not yet filled. Thresholds follow the geometric house style rather than
/// TA-Lib's rolling averages. Pattern-shape check only — no trend filter is
/// applied; combine with a trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` bullish, `−1.0` bearish, `0.0` no pattern — so it drops
/// straight into a machine-learning feature matrix where the bullish and bearish
/// variants occupy a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, TasukiGap};
///
/// let mut indicator = TasukiGap::new();
/// indicator.update(Candle::new(10.0, 11.2, 9.8, 11.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(12.0, 14.0, 11.9, 13.5, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(13.0, 13.1, 11.4, 11.5, 1.0, 2).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct TasukiGap {
    c1: Option<Candle>,
    c2: Option<Candle>,
    has_emitted: bool,
}

impl TasukiGap {
    /// Construct a new Tasuki Gap detector.
    pub const fn new() -> Self {
        Self {
            c1: None,
            c2: None,
            has_emitted: false,
        }
    }
}

impl Indicator for TasukiGap {
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

        let up = bar1.close > bar1.open && bar2.close > bar2.open;
        let down = bar1.close < bar1.open && bar2.close < bar2.open;
        if up {
            if bar2.open <= bar1.close {
                return Some(0.0); // no upside body gap
            }
            if candle.close >= candle.open {
                return Some(0.0); // bar3 must be black
            }
            if candle.open <= bar2.open || candle.open >= bar2.close {
                return Some(0.0); // bar3 must open within bar2's body
            }
            if candle.close < bar2.open && candle.close > bar1.close {
                return Some(1.0); // bar3 closes inside the gap
            }
            return Some(0.0);
        }
        if down {
            if bar2.open >= bar1.close {
                return Some(0.0); // no downside body gap
            }
            if candle.close <= candle.open {
                return Some(0.0); // bar3 must be white
            }
            if candle.open >= bar2.open || candle.open <= bar2.close {
                return Some(0.0); // bar3 must open within bar2's body
            }
            if candle.close > bar2.open && candle.close < bar1.close {
                return Some(-1.0); // bar3 closes inside the gap
            }
            return Some(0.0);
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
        "TasukiGap"
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
        let t = TasukiGap::new();
        assert_eq!(t.name(), "TasukiGap");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
    }

    #[test]
    fn upside_tasuki_gap_is_plus_one() {
        let mut t = TasukiGap::new();
        assert_eq!(t.update(c(10.0, 11.2, 9.8, 11.0, 0)), Some(0.0));
        assert_eq!(t.update(c(12.0, 14.0, 11.9, 13.5, 1)), Some(0.0));
        assert_eq!(t.update(c(13.0, 13.1, 11.4, 11.5, 2)), Some(1.0));
    }

    #[test]
    fn downside_tasuki_gap_is_minus_one() {
        let mut t = TasukiGap::new();
        assert_eq!(t.update(c(13.0, 13.2, 11.8, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(11.0, 11.1, 9.5, 10.0, 1)), Some(0.0));
        assert_eq!(t.update(c(10.5, 11.6, 10.4, 11.5, 2)), Some(-1.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = TasukiGap::new();
        assert_eq!(t.update(c(10.0, 11.2, 9.8, 11.0, 0)), Some(0.0));
        assert_eq!(t.update(c(12.0, 14.0, 11.9, 13.5, 1)), Some(0.0));
    }

    #[test]
    fn up_no_gap_yields_zero() {
        let mut t = TasukiGap::new();
        t.update(c(10.0, 11.2, 9.8, 11.0, 0));
        // bar2 white but opens below bar1's close -> no upside gap.
        t.update(c(10.5, 13.1, 10.4, 13.0, 1));
        assert_eq!(t.update(c(12.5, 12.6, 10.9, 11.0, 2)), Some(0.0));
    }

    #[test]
    fn up_third_not_black_yields_zero() {
        let mut t = TasukiGap::new();
        t.update(c(10.0, 11.2, 9.8, 11.0, 0));
        t.update(c(12.0, 14.0, 11.9, 13.5, 1));
        // bar3 white.
        assert_eq!(t.update(c(12.5, 13.1, 12.4, 13.0, 2)), Some(0.0));
    }

    #[test]
    fn up_third_open_outside_body_yields_zero() {
        let mut t = TasukiGap::new();
        t.update(c(10.0, 11.2, 9.8, 11.0, 0));
        t.update(c(12.0, 14.0, 11.9, 13.5, 1));
        // bar3 black but opens above bar2's body.
        assert_eq!(t.update(c(14.0, 14.1, 11.4, 11.5, 2)), Some(0.0));
    }

    #[test]
    fn up_third_close_not_in_gap_yields_zero() {
        let mut t = TasukiGap::new();
        t.update(c(10.0, 11.2, 9.8, 11.0, 0));
        t.update(c(12.0, 14.0, 11.9, 13.5, 1));
        // bar3 black, opens in body, but closes below the gap (under bar1's close).
        assert_eq!(t.update(c(13.0, 13.1, 10.4, 10.5, 2)), Some(0.0));
    }

    #[test]
    fn down_no_gap_yields_zero() {
        let mut t = TasukiGap::new();
        t.update(c(13.0, 13.2, 11.8, 12.0, 0));
        // bar2 black but opens above bar1's close -> no downside gap.
        t.update(c(12.5, 12.6, 10.4, 10.5, 1));
        assert_eq!(t.update(c(11.0, 12.6, 10.9, 12.0, 2)), Some(0.0));
    }

    #[test]
    fn down_third_not_white_yields_zero() {
        let mut t = TasukiGap::new();
        t.update(c(13.0, 13.2, 11.8, 12.0, 0));
        t.update(c(11.0, 11.1, 9.5, 10.0, 1));
        // bar3 black.
        assert_eq!(t.update(c(11.5, 11.6, 10.4, 10.5, 2)), Some(0.0));
    }

    #[test]
    fn down_third_open_outside_body_yields_zero() {
        let mut t = TasukiGap::new();
        t.update(c(13.0, 13.2, 11.8, 12.0, 0));
        t.update(c(11.0, 11.1, 9.5, 10.0, 1));
        // bar3 white but opens below bar2's body.
        assert_eq!(t.update(c(9.5, 11.6, 9.4, 11.5, 2)), Some(0.0));
    }

    #[test]
    fn down_third_close_not_in_gap_yields_zero() {
        let mut t = TasukiGap::new();
        t.update(c(13.0, 13.2, 11.8, 12.0, 0));
        t.update(c(11.0, 11.1, 9.5, 10.0, 1));
        // bar3 white, opens in body, but closes above the gap (over bar1's close).
        assert_eq!(t.update(c(10.5, 13.0, 10.4, 12.5, 2)), Some(0.0));
    }

    #[test]
    fn mixed_colours_yield_zero() {
        let mut t = TasukiGap::new();
        // bar1 white, bar2 black -> neither an upside nor downside setup.
        t.update(c(10.0, 11.2, 9.8, 11.0, 0));
        t.update(c(13.0, 13.2, 11.0, 11.5, 1));
        assert_eq!(t.update(c(12.0, 12.6, 10.9, 11.0, 2)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 5.2, base - 0.1, base + 5.0, i)
            })
            .collect();
        let mut a = TasukiGap::new();
        let mut b = TasukiGap::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = TasukiGap::new();
        t.update(c(10.0, 11.2, 9.8, 11.0, 0));
        t.update(c(12.0, 14.0, 11.9, 13.5, 1));
        t.update(c(13.0, 13.1, 11.4, 11.5, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(10.0, 11.2, 9.8, 11.0, 0)), Some(0.0));
    }
}
