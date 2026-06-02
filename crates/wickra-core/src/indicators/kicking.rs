//! Kicking candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Kicking — a 2-bar reversal of two opposite-coloured marubozu separated by a
/// gap. A shadowless candle is "kicked" the other way by a shadowless candle of
/// the opposite colour that gaps clear of it — a violent change of control. It is
/// trend-agnostic: the gap direction alone defines the signal.
///
/// ```text
/// marubozu = |close − open| >= 0.95 * (high − low)   (no meaningful shadows)
/// bullish (+1.0): black marubozu, then a white marubozu gapping UP   (low2 > high1)
/// bearish (−1.0): white marubozu, then a black marubozu gapping DOWN (high2 < low1)
/// ```
///
/// Output is `+1.0` (bullish) or `−1.0` (bearish) when the pattern completes and
/// `0.0` otherwise. The first bar always returns `0.0` because the two-bar window
/// is not yet filled. The marubozu threshold follows the geometric house style
/// rather than TA-Lib's rolling averages. Pattern-shape check only — no trend
/// filter is applied; combine with a trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` bullish, `−1.0` bearish, `0.0` no pattern — so it
/// drops straight into a machine-learning feature matrix where the two directions
/// occupy a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Kicking};
///
/// let mut indicator = Kicking::new();
/// indicator.update(Candle::new(12.0, 12.0, 10.0, 10.0, 1.0, 0).unwrap());
/// let out = indicator
///     .update(Candle::new(14.0, 16.0, 14.0, 16.0, 1.0, 1).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Kicking {
    prev: Option<Candle>,
    has_emitted: bool,
}

impl Kicking {
    /// Construct a new Kicking detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            has_emitted: false,
        }
    }
}

/// Whether `candle` is a marubozu (body fills at least 95 % of its range).
fn is_marubozu(candle: &Candle) -> bool {
    let range = candle.high - candle.low;
    range > 0.0 && (candle.close - candle.open).abs() >= 0.95 * range
}

impl Indicator for Kicking {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let prev = self.prev;
        self.prev = Some(candle);
        let Some(bar1) = prev else {
            return Some(0.0);
        };
        if !is_marubozu(&bar1) || !is_marubozu(&candle) {
            return Some(0.0);
        }
        // Bullish: black marubozu kicked up by a white marubozu gapping above it.
        if bar1.close < bar1.open && candle.close > candle.open && candle.low > bar1.high {
            return Some(1.0);
        }
        // Bearish: white marubozu kicked down by a black marubozu gapping below it.
        if bar1.close > bar1.open && candle.close < candle.open && candle.high < bar1.low {
            return Some(-1.0);
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
        "Kicking"
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
        let t = Kicking::new();
        assert_eq!(t.name(), "Kicking");
        assert_eq!(t.warmup_period(), 2);
        assert!(!t.is_ready());
    }

    #[test]
    fn bullish_kicking_is_plus_one() {
        let mut t = Kicking::new();
        assert_eq!(t.update(c(12.0, 12.0, 10.0, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(14.0, 16.0, 14.0, 16.0, 1)), Some(1.0));
    }

    #[test]
    fn bearish_kicking_is_minus_one() {
        let mut t = Kicking::new();
        assert_eq!(t.update(c(10.0, 12.0, 10.0, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(8.0, 8.0, 6.0, 6.0, 1)), Some(-1.0));
    }

    #[test]
    fn not_marubozu_yields_zero() {
        let mut t = Kicking::new();
        // bar1 has long shadows -> not a marubozu.
        t.update(c(12.0, 14.0, 8.0, 10.0, 0));
        assert_eq!(t.update(c(14.0, 16.0, 14.0, 16.0, 1)), Some(0.0));
    }

    #[test]
    fn no_gap_yields_zero() {
        let mut t = Kicking::new();
        t.update(c(12.0, 12.0, 10.0, 10.0, 0));
        // White marubozu but it overlaps bar1 (no gap up).
        assert_eq!(t.update(c(11.0, 13.0, 11.0, 13.0, 1)), Some(0.0));
    }

    #[test]
    fn first_bar_returns_zero() {
        let mut t = Kicking::new();
        assert_eq!(t.update(c(12.0, 12.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64 * 5.0;
                if i % 2 == 0 {
                    c(base + 2.0, base + 2.0, base, base, i)
                } else {
                    c(base + 3.0, base + 5.0, base + 3.0, base + 5.0, i)
                }
            })
            .collect();
        let mut a = Kicking::new();
        let mut b = Kicking::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = Kicking::new();
        t.update(c(12.0, 12.0, 10.0, 10.0, 0));
        t.update(c(14.0, 16.0, 14.0, 16.0, 1));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(12.0, 12.0, 10.0, 10.0, 0)), Some(0.0));
    }
}
