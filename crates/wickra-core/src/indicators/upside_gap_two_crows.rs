//! Upside Gap Two Crows candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Upside Gap Two Crows — a 3-bar bearish reversal that appears after an
/// advance. Two black candles gap up above a long white candle; the second
/// black candle engulfs the first crow yet still closes above the white body,
/// leaving the upside gap open.
///
/// ```text
/// bar1 green (long white)
/// bar2 red   & its body gaps up above bar1's body  (bar2.close > bar1.close)
/// bar3 red   & opens above bar2's open              (bar3.open  > bar2.open)
///            & closes below bar2's close            (bar3.close < bar2.close)
///            & closes above bar1's close            (bar3.close > bar1.close)
/// ```
///
/// Output is `−1.0` when the pattern completes and `0.0` otherwise. Upside Gap
/// Two Crows is a single-direction (bearish-only) pattern, so it never emits
/// `+1.0`. The first two bars always return `0.0` because the three-bar window
/// is not yet filled. Pattern-shape check only — no trend filter is applied;
/// combine with a trend indicator for actionable signals.
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
/// use wickra_core::{Candle, Indicator, UpsideGapTwoCrows};
///
/// let mut indicator = UpsideGapTwoCrows::new();
/// indicator.update(Candle::new(10.0, 12.2, 9.9, 12.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(14.0, 14.2, 12.9, 13.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(15.0, 15.2, 12.4, 12.5, 1.0, 2).unwrap());
/// assert_eq!(out, Some(-1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct UpsideGapTwoCrows {
    prev: Option<Candle>,
    prev_prev: Option<Candle>,
    has_emitted: bool,
}

impl UpsideGapTwoCrows {
    /// Construct a new Upside Gap Two Crows detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            prev_prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for UpsideGapTwoCrows {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let pp = self.prev_prev;
        let p = self.prev;
        self.prev_prev = self.prev;
        self.prev = Some(candle);
        let (Some(bar1), Some(bar2)) = (pp, p) else {
            return Some(0.0);
        };
        if bar1.close > bar1.open
            && bar2.close < bar2.open
            && bar2.close > bar1.close
            && candle.close < candle.open
            && candle.open > bar2.open
            && candle.close < bar2.close
            && candle.close > bar1.close
        {
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
        "UpsideGapTwoCrows"
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
        let u = UpsideGapTwoCrows::new();
        assert_eq!(u.name(), "UpsideGapTwoCrows");
        assert_eq!(u.warmup_period(), 3);
        assert!(!u.is_ready());
    }

    #[test]
    fn upside_gap_two_crows_is_minus_one() {
        let mut u = UpsideGapTwoCrows::new();
        // bar1 green 10->12; bar2 red 14->13 gapping up; bar3 red opens 15
        // (above bar2 open) and closes 12.5 (below bar2 close, above bar1 close).
        assert_eq!(u.update(c(10.0, 12.2, 9.9, 12.0, 0)), Some(0.0));
        assert_eq!(u.update(c(14.0, 14.2, 12.9, 13.0, 1)), Some(0.0));
        assert_eq!(u.update(c(15.0, 15.2, 12.4, 12.5, 2)), Some(-1.0));
    }

    #[test]
    fn closing_the_first_gap_yields_zero() {
        let mut u = UpsideGapTwoCrows::new();
        u.update(c(10.0, 12.2, 9.9, 12.0, 0));
        u.update(c(14.0, 14.2, 12.9, 13.0, 1));
        // bar3 closes 11.5, below bar1's close (12) -> gap closed, not the pattern.
        assert_eq!(u.update(c(15.0, 15.2, 11.4, 11.5, 2)), Some(0.0));
    }

    #[test]
    fn no_gap_up_yields_zero() {
        let mut u = UpsideGapTwoCrows::new();
        u.update(c(10.0, 12.2, 9.9, 12.0, 0));
        // bar2's body does not gap above bar1's body.
        u.update(c(11.5, 12.0, 10.4, 11.0, 1));
        assert_eq!(u.update(c(12.0, 12.2, 10.9, 11.5, 2)), Some(0.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut u = UpsideGapTwoCrows::new();
        assert_eq!(u.update(c(10.0, 12.2, 9.9, 12.0, 0)), Some(0.0));
        assert_eq!(u.update(c(14.0, 14.2, 12.9, 13.0, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                if i % 3 == 0 {
                    c(base, base + 0.5, base - 1.0, base + 0.4, i)
                } else {
                    c(base + 1.5, base + 1.7, base - 0.2, base + 0.6, i)
                }
            })
            .collect();
        let mut a = UpsideGapTwoCrows::new();
        let mut b = UpsideGapTwoCrows::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut u = UpsideGapTwoCrows::new();
        u.update(c(10.0, 12.2, 9.9, 12.0, 0));
        u.update(c(14.0, 14.2, 12.9, 13.0, 1));
        u.update(c(15.0, 15.2, 12.4, 12.5, 2));
        assert!(u.is_ready());
        u.reset();
        assert!(!u.is_ready());
        assert_eq!(u.update(c(10.0, 12.2, 9.9, 12.0, 0)), Some(0.0));
    }
}
