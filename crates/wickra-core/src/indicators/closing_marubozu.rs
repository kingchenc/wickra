//! Closing Marubozu candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Closing Marubozu — a single-bar strong-momentum candle with a long body and no
/// shadow on the *close* end. A white closing marubozu closes right at the high
/// (no upper shadow) and may carry an opening shadow below; a black one closes
/// right at the low (no lower shadow) and may carry an opening shadow above. The
/// shaved close end shows the move ran unopposed into the bell.
///
/// ```text
/// range = high − low
/// long body: |close − open| >= 0.7 * range
/// white: close > open and high − close <= 0.05 * range   (close at the high)
/// black: close < open and close − low  <= 0.05 * range   (close at the low)
/// ```
///
/// Output is `+1.0` for a white closing marubozu, `−1.0` for a black one, and
/// `0.0` otherwise. Body and shadow thresholds follow the geometric house style
/// rather than TA-Lib's rolling averages. The opposite shaved end is
/// [`crate::OpeningMarubozu`]. Pattern-shape check only — no trend filter is
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
/// use wickra_core::{Candle, ClosingMarubozu, Indicator};
///
/// let mut indicator = ClosingMarubozu::new();
/// // White: closes at the high, small opening shadow below.
/// let candle = Candle::new(10.5, 15.0, 10.0, 15.0, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ClosingMarubozu {
    has_emitted: bool,
}

impl ClosingMarubozu {
    /// Construct a new Closing Marubozu detector.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for ClosingMarubozu {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let range = candle.high - candle.low;
        if range <= 0.0 {
            return Some(0.0);
        }
        let body = candle.close - candle.open;
        if body.abs() < 0.7 * range {
            return Some(0.0);
        }
        let tol = 0.05 * range;
        if body > 0.0 && candle.high - candle.close <= tol {
            return Some(1.0);
        }
        if body < 0.0 && candle.close - candle.low <= tol {
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
        "ClosingMarubozu"
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
        let t = ClosingMarubozu::new();
        assert_eq!(t.name(), "ClosingMarubozu");
        assert_eq!(t.warmup_period(), 1);
        assert!(!t.is_ready());
    }

    #[test]
    fn white_closing_marubozu_is_plus_one() {
        let mut t = ClosingMarubozu::new();
        // Closes at the high, opening shadow below.
        assert_eq!(t.update(c(10.5, 15.0, 10.0, 15.0, 0)), Some(1.0));
    }

    #[test]
    fn black_closing_marubozu_is_minus_one() {
        let mut t = ClosingMarubozu::new();
        // Closes at the low, opening shadow above.
        assert_eq!(t.update(c(14.5, 15.0, 10.0, 10.0, 0)), Some(-1.0));
    }

    #[test]
    fn white_with_upper_shadow_yields_zero() {
        let mut t = ClosingMarubozu::new();
        // Long white body but a clear upper shadow -> close is not at the high.
        assert_eq!(t.update(c(10.5, 16.0, 10.0, 15.0, 0)), Some(0.0));
    }

    #[test]
    fn black_with_lower_shadow_yields_zero() {
        let mut t = ClosingMarubozu::new();
        // Long black body but a clear lower shadow -> close is not at the low.
        assert_eq!(t.update(c(14.5, 15.0, 9.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn short_body_yields_zero() {
        let mut t = ClosingMarubozu::new();
        // Body is short relative to range.
        assert_eq!(t.update(c(12.0, 15.0, 10.0, 12.5, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = ClosingMarubozu::new();
        assert_eq!(t.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 0.5, base + 5.0, base, base + 5.0, i)
            })
            .collect();
        let mut a = ClosingMarubozu::new();
        let mut b = ClosingMarubozu::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = ClosingMarubozu::new();
        t.update(c(10.5, 15.0, 10.0, 15.0, 0));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
    }
}
