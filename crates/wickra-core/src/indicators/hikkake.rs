//! Hikkake candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Hikkake — a 3-bar trap. An inside bar (bar2 fully contained by bar1) sets up a
/// breakout that immediately fails on bar3, trapping breakout traders and pointing
/// the opposite way.
///
/// ```text
/// inside bar : bar2.high < bar1.high  &&  bar2.low > bar1.low
/// bullish (+1.0): bar3 makes a LOWER high AND LOWER low than bar2
///                 (a false downside break -> expect a move up)
/// bearish (−1.0): bar3 makes a HIGHER high AND HIGHER low than bar2
///                 (a false upside break -> expect a move down)
/// ```
///
/// Output is `+1.0` (bullish setup), `−1.0` (bearish setup), or `0.0` otherwise.
/// The detector fires when the three-bar setup completes on bar3; it does not
/// separately flag the optional later confirmation bar. The first two bars always
/// return `0.0` because the window is not yet filled. Pattern-shape check only —
/// no trend filter is applied; combine with a trend indicator for actionable
/// signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` bullish, `−1.0` bearish, `0.0` no pattern — so it
/// drops straight into a machine-learning feature matrix where the bullish and
/// bearish setups occupy a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Hikkake, Indicator};
///
/// let mut indicator = Hikkake::new();
/// indicator.update(Candle::new(10.0, 15.0, 5.0, 12.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(11.0, 13.0, 8.0, 12.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(9.0, 12.0, 6.0, 7.0, 1.0, 2).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Hikkake {
    prev: Option<Candle>,
    prev_prev: Option<Candle>,
    has_emitted: bool,
}

impl Hikkake {
    /// Construct a new Hikkake detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            prev_prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for Hikkake {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let bar1 = self.prev_prev;
        let bar2 = self.prev;
        self.prev_prev = self.prev;
        self.prev = Some(candle);
        let (Some(bar1), Some(bar2)) = (bar1, bar2) else {
            return Some(0.0);
        };
        // bar2 must be an inside bar of bar1.
        if !(bar2.high < bar1.high && bar2.low > bar1.low) {
            return Some(0.0);
        }
        // Bullish: bar3 breaks below the inside bar (lower high and lower low).
        if candle.high < bar2.high && candle.low < bar2.low {
            return Some(1.0);
        }
        // Bearish: bar3 breaks above the inside bar (higher high and higher low).
        if candle.high > bar2.high && candle.low > bar2.low {
            return Some(-1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.prev = None;
        self.prev_prev = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        3
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "Hikkake"
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
        let t = Hikkake::new();
        assert_eq!(t.name(), "Hikkake");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
    }

    #[test]
    fn bullish_hikkake_is_plus_one() {
        let mut t = Hikkake::new();
        assert_eq!(t.update(c(10.0, 15.0, 5.0, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(11.0, 13.0, 8.0, 12.0, 1)), Some(0.0));
        assert_eq!(t.update(c(9.0, 12.0, 6.0, 7.0, 2)), Some(1.0));
    }

    #[test]
    fn bearish_hikkake_is_minus_one() {
        let mut t = Hikkake::new();
        assert_eq!(t.update(c(10.0, 15.0, 5.0, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(11.0, 13.0, 8.0, 12.0, 1)), Some(0.0));
        assert_eq!(t.update(c(12.0, 14.0, 9.0, 13.0, 2)), Some(-1.0));
    }

    #[test]
    fn not_inside_bar_yields_zero() {
        let mut t = Hikkake::new();
        t.update(c(10.0, 15.0, 5.0, 12.0, 0));
        // bar2 is not contained by bar1 (higher high).
        t.update(c(11.0, 16.0, 8.0, 12.0, 1));
        assert_eq!(t.update(c(9.0, 12.0, 6.0, 7.0, 2)), Some(0.0));
    }

    #[test]
    fn outside_bar3_yields_zero() {
        let mut t = Hikkake::new();
        t.update(c(10.0, 15.0, 5.0, 12.0, 0));
        t.update(c(11.0, 13.0, 8.0, 12.0, 1));
        // bar3 engulfs bar2 (higher high and lower low) -> neither direction.
        assert_eq!(t.update(c(11.0, 14.0, 7.0, 9.0, 2)), Some(0.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = Hikkake::new();
        assert_eq!(t.update(c(10.0, 15.0, 5.0, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(11.0, 13.0, 8.0, 12.0, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                match i % 3 {
                    0 => c(base, base + 6.0, base - 6.0, base, i),
                    1 => c(base, base + 2.0, base - 2.0, base, i),
                    _ => c(base, base + 1.0, base - 5.0, base - 4.0, i),
                }
            })
            .collect();
        let mut a = Hikkake::new();
        let mut b = Hikkake::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = Hikkake::new();
        t.update(c(10.0, 15.0, 5.0, 12.0, 0));
        t.update(c(11.0, 13.0, 8.0, 12.0, 1));
        t.update(c(9.0, 12.0, 6.0, 7.0, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(10.0, 15.0, 5.0, 12.0, 0)), Some(0.0));
    }
}
