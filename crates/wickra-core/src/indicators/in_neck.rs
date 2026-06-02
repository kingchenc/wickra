//! In-Neck candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// In-Neck — a 2-bar bearish continuation, slightly stronger than On-Neck. A long
/// black candle in a decline is followed by a white candle that opens below the
/// black bar's low and closes just barely *into* the black body, around its close
/// level. The shallow recovery still favours the sellers.
///
/// ```text
/// long body = |close − open| >= 0.5 * (high − low)
/// bar1 black & long
/// bar2 white, opens below bar1's low      (open2 < low1)
/// bar2 closes just into bar1's body        (close1 <= close2 <= close1 + 0.1 · body1)
/// ```
///
/// Output is `−1.0` when the pattern completes and `0.0` otherwise. In-Neck is a
/// single-direction (bearish-only) continuation, so it never emits `+1.0`. The
/// first bar always returns `0.0` because the two-bar window is not yet filled.
/// Body and neckline thresholds follow the geometric house style rather than
/// TA-Lib's rolling averages. Pattern-shape check only — no trend filter is
/// applied; combine with a trend indicator for actionable signals.
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
/// use wickra_core::{Candle, InNeck, Indicator};
///
/// let mut indicator = InNeck::new();
/// indicator.update(Candle::new(15.0, 15.1, 9.0, 10.0, 1.0, 0).unwrap());
/// let out = indicator
///     .update(Candle::new(7.0, 10.3, 6.9, 10.2, 1.0, 1).unwrap());
/// assert_eq!(out, Some(-1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct InNeck {
    prev: Option<Candle>,
    has_emitted: bool,
}

impl InNeck {
    /// Construct a new In-Neck detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for InNeck {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let prev = self.prev;
        self.prev = Some(candle);
        let Some(bar1) = prev else {
            return Some(0.0);
        };
        let range1 = bar1.high - bar1.low;
        if range1 <= 0.0 {
            return Some(0.0);
        }
        let body1 = bar1.open - bar1.close;
        if bar1.close < bar1.open
            && body1 >= 0.5 * range1
            && candle.close > candle.open
            && candle.open < bar1.low
            && candle.close >= bar1.close
            && candle.close <= bar1.close + 0.1 * body1
        {
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
        "InNeck"
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
        let t = InNeck::new();
        assert_eq!(t.name(), "InNeck");
        assert_eq!(t.warmup_period(), 2);
        assert!(!t.is_ready());
    }

    #[test]
    fn in_neck_is_minus_one() {
        let mut t = InNeck::new();
        assert_eq!(t.update(c(15.0, 15.1, 9.0, 10.0, 0)), Some(0.0));
        assert_eq!(t.update(c(7.0, 10.3, 6.9, 10.2, 1)), Some(-1.0));
    }

    #[test]
    fn close_at_low_yields_zero() {
        let mut t = InNeck::new();
        t.update(c(15.0, 15.1, 9.0, 10.0, 0));
        // Closes at the prior low, not into the body -> on-neck, not in-neck.
        assert_eq!(t.update(c(7.0, 9.1, 6.9, 9.0, 1)), Some(0.0));
    }

    #[test]
    fn close_past_neck_yields_zero() {
        let mut t = InNeck::new();
        t.update(c(15.0, 15.1, 9.0, 10.0, 0));
        // Closes well into the body -> thrusting, not in-neck.
        assert_eq!(t.update(c(7.0, 11.6, 6.9, 11.5, 1)), Some(0.0));
    }

    #[test]
    fn second_bar_black_yields_zero() {
        let mut t = InNeck::new();
        t.update(c(15.0, 15.1, 9.0, 10.0, 0));
        assert_eq!(t.update(c(10.4, 10.5, 6.9, 10.1, 1)), Some(0.0));
    }

    #[test]
    fn first_bar_returns_zero() {
        let mut t = InNeck::new();
        assert_eq!(t.update(c(15.0, 15.1, 9.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 5.0, base + 5.1, base - 1.0, base, i)
            })
            .collect();
        let mut a = InNeck::new();
        let mut b = InNeck::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = InNeck::new();
        t.update(c(15.0, 15.1, 9.0, 10.0, 0));
        t.update(c(7.0, 10.3, 6.9, 10.2, 1));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(15.0, 15.1, 9.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_first_bar_yields_zero() {
        let mut t = InNeck::new();
        // Flat first bar (range1 == 0) -> rejected.
        t.update(c(10.0, 10.0, 10.0, 10.0, 0));
        assert_eq!(t.update(c(9.0, 10.0, 8.0, 9.5, 1)), Some(0.0));
    }
}
