//! Modified Hikkake candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Modified Hikkake — a close-confirmed variant of the [`Hikkake`](crate::Hikkake)
/// trap. An inside bar is followed by a bar that breaks out *and is immediately
/// rejected*: it pierces the inside bar's range intrabar but closes back inside,
/// a stronger signal than the plain breakout setup.
///
/// ```text
/// inside bar : bar2.high < bar1.high  &&  bar2.low > bar1.low
/// bullish (+1.0): bar3 makes a lower high AND lower low than bar2,
///                 yet closes back above the inside-bar low   (close3 > bar2.low)
/// bearish (−1.0): bar3 makes a higher high AND higher low than bar2,
///                 yet closes back below the inside-bar high  (close3 < bar2.high)
/// ```
///
/// Output is `+1.0` (bullish), `−1.0` (bearish), or `0.0` otherwise. The extra
/// close-recovery condition is what distinguishes it from the plain Hikkake, which
/// fires on the high/low break alone. The first two bars always return `0.0`
/// because the three-bar window is not yet filled. Pattern-shape check only — no
/// trend filter is applied; combine with a trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` bullish, `−1.0` bearish, `0.0` no pattern — so it
/// drops straight into a machine-learning feature matrix as a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, HikkakeModified, Indicator};
///
/// let mut indicator = HikkakeModified::new();
/// indicator.update(Candle::new(10.0, 15.0, 5.0, 12.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(11.0, 13.0, 8.0, 12.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(9.0, 12.0, 6.0, 9.0, 1.0, 2).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct HikkakeModified {
    prev: Option<Candle>,
    prev_prev: Option<Candle>,
    has_emitted: bool,
}

impl HikkakeModified {
    /// Construct a new Modified Hikkake detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            prev_prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for HikkakeModified {
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
        if !(bar2.high < bar1.high && bar2.low > bar1.low) {
            return Some(0.0);
        }
        // Bullish: false downside break that closes back above the inside-bar low.
        if candle.high < bar2.high && candle.low < bar2.low && candle.close > bar2.low {
            return Some(1.0);
        }
        // Bearish: false upside break that closes back below the inside-bar high.
        if candle.high > bar2.high && candle.low > bar2.low && candle.close < bar2.high {
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
        "HikkakeModified"
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
        let t = HikkakeModified::new();
        assert_eq!(t.name(), "HikkakeModified");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
    }

    #[test]
    fn bullish_modified_hikkake_is_plus_one() {
        let mut t = HikkakeModified::new();
        assert_eq!(t.update(c(10.0, 15.0, 5.0, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(11.0, 13.0, 8.0, 12.0, 1)), Some(0.0));
        assert_eq!(t.update(c(9.0, 12.0, 6.0, 9.0, 2)), Some(1.0));
    }

    #[test]
    fn bearish_modified_hikkake_is_minus_one() {
        let mut t = HikkakeModified::new();
        assert_eq!(t.update(c(10.0, 15.0, 5.0, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(11.0, 13.0, 8.0, 12.0, 1)), Some(0.0));
        assert_eq!(t.update(c(13.0, 14.0, 9.0, 10.0, 2)), Some(-1.0));
    }

    #[test]
    fn break_without_close_recovery_yields_zero() {
        let mut t = HikkakeModified::new();
        t.update(c(10.0, 15.0, 5.0, 12.0, 0));
        t.update(c(11.0, 13.0, 8.0, 12.0, 1));
        // Lower high and lower low, but closes below the inside-bar low -> plain
        // Hikkake break, not the close-confirmed modified version.
        assert_eq!(t.update(c(9.0, 12.0, 6.0, 7.0, 2)), Some(0.0));
    }

    #[test]
    fn not_inside_bar_yields_zero() {
        let mut t = HikkakeModified::new();
        t.update(c(10.0, 15.0, 5.0, 12.0, 0));
        t.update(c(11.0, 16.0, 8.0, 12.0, 1));
        assert_eq!(t.update(c(9.0, 12.0, 6.0, 9.0, 2)), Some(0.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = HikkakeModified::new();
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
                    _ => c(base, base + 1.0, base - 5.0, base, i),
                }
            })
            .collect();
        let mut a = HikkakeModified::new();
        let mut b = HikkakeModified::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = HikkakeModified::new();
        t.update(c(10.0, 15.0, 5.0, 12.0, 0));
        t.update(c(11.0, 13.0, 8.0, 12.0, 1));
        t.update(c(9.0, 12.0, 6.0, 9.0, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(10.0, 15.0, 5.0, 12.0, 0)), Some(0.0));
    }
}
