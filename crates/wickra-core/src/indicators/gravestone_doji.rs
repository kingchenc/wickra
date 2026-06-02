//! Gravestone Doji candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Gravestone Doji — a single-bar bearish reversal. Open, close, and low sit at
/// the bottom of the bar while a long upper shadow shows price was pushed up hard
/// and then sold all the way back to the open — sellers rejecting the highs.
///
/// ```text
/// range = high − low
/// doji          = |close − open| <= 0.1 * range
/// no lower wick = min(open, close) − low   <= 0.1 * range
/// long upper    = high − max(open, close)  >= 0.5 * range
/// ```
///
/// Output is `−1.0` when the gravestone prints and `0.0` otherwise. Gravestone
/// Doji is a single-direction (bearish-only) shape, so it never emits `+1.0`.
/// Body and shadow thresholds follow the geometric house style (fixed fractions
/// of the bar range) rather than TA-Lib's rolling averages. Pattern-shape check
/// only — no trend filter is applied; combine with a trend indicator for
/// actionable signals.
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
/// use wickra_core::{Candle, GravestoneDoji, Indicator};
///
/// let mut indicator = GravestoneDoji::new();
/// // Body at the bottom, long upper shadow.
/// let candle = Candle::new(10.0, 14.0, 9.95, 10.0, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(-1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct GravestoneDoji {
    has_emitted: bool,
}

impl GravestoneDoji {
    /// Construct a new Gravestone Doji detector.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for GravestoneDoji {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let range = candle.high - candle.low;
        if range <= 0.0 {
            return Some(0.0);
        }
        if (candle.close - candle.open).abs() > 0.1 * range {
            return Some(0.0);
        }
        let upper = candle.high - candle.open.max(candle.close);
        let lower = candle.open.min(candle.close) - candle.low;
        if lower <= 0.1 * range && upper >= 0.5 * range {
            return Some(-1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "GravestoneDoji"
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
        let t = GravestoneDoji::new();
        assert_eq!(t.name(), "GravestoneDoji");
        assert_eq!(t.warmup_period(), 1);
        assert!(!t.is_ready());
    }

    #[test]
    fn gravestone_is_minus_one() {
        let mut t = GravestoneDoji::new();
        assert_eq!(t.update(c(10.0, 14.0, 9.95, 10.0, 0)), Some(-1.0));
    }

    #[test]
    fn lower_shadow_yields_zero() {
        let mut t = GravestoneDoji::new();
        // Long lower shadow -> not a gravestone (this is a dragonfly shape).
        assert_eq!(t.update(c(10.0, 10.05, 6.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn short_upper_shadow_yields_zero() {
        let mut t = GravestoneDoji::new();
        // Body at the bottom but the upper shadow is too short.
        assert_eq!(t.update(c(10.0, 10.4, 9.95, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn non_doji_yields_zero() {
        let mut t = GravestoneDoji::new();
        assert_eq!(t.update(c(10.0, 14.0, 9.5, 13.5, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = GravestoneDoji::new();
        assert_eq!(t.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 4.0, base - 0.05, base, i)
            })
            .collect();
        let mut a = GravestoneDoji::new();
        let mut b = GravestoneDoji::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = GravestoneDoji::new();
        t.update(c(10.0, 14.0, 9.95, 10.0, 0));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
    }
}
