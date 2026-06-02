//! Kicking-by-Length candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Kicking-by-Length — the [`Kicking`](crate::Kicking) pattern with the signal
/// taken from the *longer* of the two marubozu rather than from the gap direction.
/// When the two shadowless candles differ in size, the bigger one is treated as
/// the dominant force.
///
/// ```text
/// marubozu = |close − open| >= 0.95 * (high − low)
/// setup: two opposite-coloured marubozu separated by a gap
///   black then white gapping UP, or white then black gapping DOWN
/// signal = colour of the LONGER marubozu  (white -> +1.0, black -> −1.0)
/// ```
///
/// Output is `+1.0` or `−1.0` when the kicking setup is present and `0.0`
/// otherwise. Note this can disagree with [`Kicking`](crate::Kicking): a black
/// marubozu kicked up by a *shorter* white marubozu reports `−1.0` here. The first
/// bar always returns `0.0` because the two-bar window is not yet filled. The
/// marubozu threshold follows the geometric house style rather than TA-Lib's
/// rolling averages. Pattern-shape check only — no trend filter is applied;
/// combine with a trend indicator for actionable signals.
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
/// use wickra_core::{Candle, Indicator, KickingByLength};
///
/// let mut indicator = KickingByLength::new();
/// indicator.update(Candle::new(12.0, 12.0, 10.0, 10.0, 1.0, 0).unwrap());
/// // White marubozu gaps up and is the longer body -> +1.
/// let out = indicator
///     .update(Candle::new(14.0, 20.0, 14.0, 20.0, 1.0, 1).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct KickingByLength {
    prev: Option<Candle>,
    has_emitted: bool,
}

impl KickingByLength {
    /// Construct a new Kicking-by-Length detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            has_emitted: false,
        }
    }
}

fn is_marubozu(candle: &Candle) -> bool {
    let range = candle.high - candle.low;
    range > 0.0 && (candle.close - candle.open).abs() >= 0.95 * range
}

impl Indicator for KickingByLength {
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
        let body1 = bar1.close - bar1.open;
        let body2 = candle.close - candle.open;
        let bullish_setup = body1 < 0.0 && body2 > 0.0 && candle.low > bar1.high;
        let bearish_setup = body1 > 0.0 && body2 < 0.0 && candle.high < bar1.low;
        if !(bullish_setup || bearish_setup) {
            return Some(0.0);
        }
        // The longer marubozu's colour is the signal.
        let longer_is_white = if body1.abs() >= body2.abs() {
            body1 > 0.0
        } else {
            body2 > 0.0
        };
        Some(if longer_is_white { 1.0 } else { -1.0 })
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
        "KickingByLength"
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
        let t = KickingByLength::new();
        assert_eq!(t.name(), "KickingByLength");
        assert_eq!(t.warmup_period(), 2);
        assert!(!t.is_ready());
    }

    #[test]
    fn longer_white_is_plus_one() {
        let mut t = KickingByLength::new();
        assert_eq!(t.update(c(12.0, 12.0, 10.0, 10.0, 0)), Some(0.0));
        // White marubozu (length 6) longer than the black one (length 2).
        assert_eq!(t.update(c(14.0, 20.0, 14.0, 20.0, 1)), Some(1.0));
    }

    #[test]
    fn longer_black_is_minus_one() {
        let mut t = KickingByLength::new();
        // Black marubozu (length 6), then a shorter white marubozu (length 2)
        // gapping up -> the longer black body wins, so -1.
        assert_eq!(t.update(c(16.0, 16.0, 10.0, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(18.0, 20.0, 18.0, 20.0, 1)), Some(-1.0));
    }

    #[test]
    fn not_marubozu_yields_zero() {
        let mut t = KickingByLength::new();
        t.update(c(12.0, 14.0, 8.0, 10.0, 0));
        assert_eq!(t.update(c(14.0, 20.0, 14.0, 20.0, 1)), Some(0.0));
    }

    #[test]
    fn no_gap_yields_zero() {
        let mut t = KickingByLength::new();
        t.update(c(12.0, 12.0, 10.0, 10.0, 0));
        assert_eq!(t.update(c(11.0, 13.0, 11.0, 13.0, 1)), Some(0.0));
    }

    #[test]
    fn first_bar_returns_zero() {
        let mut t = KickingByLength::new();
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
        let mut a = KickingByLength::new();
        let mut b = KickingByLength::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = KickingByLength::new();
        t.update(c(12.0, 12.0, 10.0, 10.0, 0));
        t.update(c(14.0, 20.0, 14.0, 20.0, 1));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(12.0, 12.0, 10.0, 10.0, 0)), Some(0.0));
    }
}
