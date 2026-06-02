//! Opening Marubozu candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Opening Marubozu — a single-bar strong-momentum candle with a long body and no
/// shadow on the *open* end. A white opening marubozu opens right at the low (no
/// lower shadow) and may carry a closing shadow above; a black one opens right at
/// the high (no upper shadow) and may carry a closing shadow below. The shaved
/// open end shows the move took off from the bell without hesitation.
///
/// ```text
/// range = high − low
/// long body: |close − open| >= 0.7 * range
/// white: close > open and open − low  <= 0.05 * range   (open at the low)
/// black: close < open and high − open <= 0.05 * range   (open at the high)
/// ```
///
/// Output is `+1.0` for a white opening marubozu, `−1.0` for a black one, and
/// `0.0` otherwise. Body and shadow thresholds follow the geometric house style
/// rather than TA-Lib's rolling averages. TA-Lib has no direct equivalent; this
/// completes the pair with [`crate::ClosingMarubozu`], which shaves the close end.
/// Pattern-shape check only — no trend filter is applied; combine with a trend
/// indicator for actionable signals.
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
/// use wickra_core::{Candle, Indicator, OpeningMarubozu};
///
/// let mut indicator = OpeningMarubozu::new();
/// // White: opens at the low, small closing shadow above.
/// let candle = Candle::new(10.0, 15.0, 10.0, 14.5, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct OpeningMarubozu {
    has_emitted: bool,
}

impl OpeningMarubozu {
    /// Construct a new Opening Marubozu detector.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for OpeningMarubozu {
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
        if body > 0.0 && candle.open - candle.low <= tol {
            return Some(1.0);
        }
        if body < 0.0 && candle.high - candle.open <= tol {
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
        "OpeningMarubozu"
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
        let t = OpeningMarubozu::new();
        assert_eq!(t.name(), "OpeningMarubozu");
        assert_eq!(t.warmup_period(), 1);
        assert!(!t.is_ready());
    }

    #[test]
    fn white_opening_marubozu_is_plus_one() {
        let mut t = OpeningMarubozu::new();
        // Opens at the low, closing shadow above.
        assert_eq!(t.update(c(10.0, 15.0, 10.0, 14.5, 0)), Some(1.0));
    }

    #[test]
    fn black_opening_marubozu_is_minus_one() {
        let mut t = OpeningMarubozu::new();
        // Opens at the high, closing shadow below.
        assert_eq!(t.update(c(15.0, 15.0, 10.0, 10.5, 0)), Some(-1.0));
    }

    #[test]
    fn white_with_lower_shadow_yields_zero() {
        let mut t = OpeningMarubozu::new();
        // Long white body but a clear lower shadow -> open is not at the low.
        assert_eq!(t.update(c(11.0, 15.0, 10.0, 15.0, 0)), Some(0.0));
    }

    #[test]
    fn black_with_upper_shadow_yields_zero() {
        let mut t = OpeningMarubozu::new();
        // Long black body but a clear upper shadow -> open is not at the high.
        assert_eq!(t.update(c(14.0, 16.0, 10.0, 10.5, 0)), Some(0.0));
    }

    #[test]
    fn short_body_yields_zero() {
        let mut t = OpeningMarubozu::new();
        // Body is short relative to range.
        assert_eq!(t.update(c(10.0, 15.0, 10.0, 12.5, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = OpeningMarubozu::new();
        assert_eq!(t.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 5.0, base, base + 4.5, i)
            })
            .collect();
        let mut a = OpeningMarubozu::new();
        let mut b = OpeningMarubozu::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = OpeningMarubozu::new();
        t.update(c(10.0, 15.0, 10.0, 14.5, 0));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
    }
}
