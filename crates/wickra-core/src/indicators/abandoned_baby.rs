//! Abandoned Baby candlestick pattern.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Abandoned Baby — a strong 3-bar reversal where a doji is "abandoned" by price
/// gaps on both sides, isolating it from the candles before and after.
///
/// ```text
/// tol           = tolerance * max(|bar2.open|, |bar2.close|)
/// bar2 doji                                   (|bar2.close − bar2.open| <= tol)
///
/// bullish  (+1.0): bar1 red, bar2 gaps fully below bar1 (bar2.high < bar1.low),
///                  bar3 green and gaps fully above bar2 (bar3.low > bar2.high)
/// bearish  (−1.0): bar1 green, bar2 gaps fully above bar1 (bar2.low > bar1.high),
///                  bar3 red and gaps fully below bar2 (bar3.high < bar2.low)
/// ```
///
/// Output is `0.0` otherwise. The first two bars always return `0.0` because the
/// three-bar window is not yet filled. `tolerance` defaults to `0.001` (10 bps
/// relative) and bounds how flat the middle candle must be to count as a doji; it
/// must lie in `[0, 1)`. Pattern-shape check only — no trend filter is applied;
/// combine with a trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` bullish, `−1.0` bearish, `0.0` no pattern — so it
/// drops straight into a machine-learning feature matrix where the bullish and
/// bearish variants occupy a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{AbandonedBaby, Candle, Indicator};
///
/// let mut indicator = AbandonedBaby::new();
/// indicator.update(Candle::new(20.0, 20.1, 14.9, 15.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(13.0, 13.1, 12.9, 13.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(16.0, 18.1, 15.9, 18.0, 1.0, 2).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone)]
pub struct AbandonedBaby {
    tolerance: f64,
    prev: Option<Candle>,
    prev_prev: Option<Candle>,
    has_emitted: bool,
}

impl Default for AbandonedBaby {
    fn default() -> Self {
        Self::new()
    }
}

impl AbandonedBaby {
    /// Construct a detector with the default relative doji tolerance (1e-3).
    pub const fn new() -> Self {
        Self {
            tolerance: 0.001,
            prev: None,
            prev_prev: None,
            has_emitted: false,
        }
    }

    /// Construct a detector with a custom relative doji tolerance.
    ///
    /// `tolerance` must lie in `[0, 1)`.
    pub fn with_tolerance(tolerance: f64) -> Result<Self> {
        if !(0.0..1.0).contains(&tolerance) {
            return Err(Error::InvalidPeriod {
                message: "abandoned baby tolerance must lie in [0, 1)",
            });
        }
        Ok(Self {
            tolerance,
            prev: None,
            prev_prev: None,
            has_emitted: false,
        })
    }

    /// Configured relative doji tolerance.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
}

impl Indicator for AbandonedBaby {
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
        let tol = self.tolerance * bar2.open.abs().max(bar2.close.abs());
        let bar2_is_doji = (bar2.close - bar2.open).abs() <= tol;
        if !bar2_is_doji {
            return Some(0.0);
        }
        // Bullish: red bar1, doji gaps below, green bar3 gaps above.
        if bar1.close < bar1.open
            && bar2.high < bar1.low
            && candle.close > candle.open
            && candle.low > bar2.high
        {
            return Some(1.0);
        }
        // Bearish: green bar1, doji gaps above, red bar3 gaps below.
        if bar1.close > bar1.open
            && bar2.low > bar1.high
            && candle.close < candle.open
            && candle.high < bar2.low
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
        "AbandonedBaby"
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
    fn rejects_invalid_tolerance() {
        assert!(AbandonedBaby::with_tolerance(-0.01).is_err());
        assert!(AbandonedBaby::with_tolerance(1.0).is_err());
    }

    #[test]
    fn accepts_valid_tolerance() {
        let t = AbandonedBaby::with_tolerance(0.0).unwrap();
        assert!((t.tolerance() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn accessors_and_metadata() {
        let t = AbandonedBaby::default();
        assert_eq!(t.name(), "AbandonedBaby");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
        assert!((t.tolerance() - 0.001).abs() < 1e-12);
    }

    #[test]
    fn bullish_abandoned_baby_is_plus_one() {
        let mut t = AbandonedBaby::new();
        assert_eq!(t.update(c(20.0, 20.1, 14.9, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(13.0, 13.1, 12.9, 13.0, 1)), Some(0.0));
        assert_eq!(t.update(c(16.0, 18.1, 15.9, 18.0, 2)), Some(1.0));
    }

    #[test]
    fn bearish_abandoned_baby_is_minus_one() {
        let mut t = AbandonedBaby::new();
        assert_eq!(t.update(c(15.0, 20.1, 14.9, 20.0, 0)), Some(0.0));
        assert_eq!(t.update(c(22.0, 22.1, 21.9, 22.0, 1)), Some(0.0));
        assert_eq!(t.update(c(19.0, 19.1, 16.9, 17.0, 2)), Some(-1.0));
    }

    #[test]
    fn middle_not_doji_yields_zero() {
        let mut t = AbandonedBaby::new();
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        // Middle bar has a wide body -> not a doji.
        assert_eq!(t.update(c(13.0, 14.0, 11.0, 11.5, 1)), Some(0.0));
        assert_eq!(t.update(c(16.0, 18.1, 15.9, 18.0, 2)), Some(0.0));
    }

    #[test]
    fn no_gap_yields_zero() {
        let mut t = AbandonedBaby::new();
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        // Doji overlaps bar1's range -> no gap.
        assert_eq!(t.update(c(15.0, 15.1, 14.9, 15.0, 1)), Some(0.0));
        assert_eq!(t.update(c(16.0, 18.1, 15.9, 18.0, 2)), Some(0.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = AbandonedBaby::new();
        assert_eq!(t.update(c(20.0, 20.1, 14.9, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(13.0, 13.1, 12.9, 13.0, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + (i as f64 * 0.3).sin() * 5.0;
                c(base, base + 1.0, base - 1.0, base + 0.5, i)
            })
            .collect();
        let mut a = AbandonedBaby::new();
        let mut b = AbandonedBaby::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = AbandonedBaby::new();
        t.update(c(20.0, 20.1, 14.9, 15.0, 0));
        t.update(c(13.0, 13.1, 12.9, 13.0, 1));
        t.update(c(16.0, 18.1, 15.9, 18.0, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(20.0, 20.1, 14.9, 15.0, 0)), Some(0.0));
    }
}
