//! Three Stars in the South candlestick pattern.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Three Stars in the South — a rare 3-bar bullish reversal: three shrinking red
/// candles where each session carves out a higher low and contracts toward a
/// tiny black marubozu, signalling exhausted selling at the bottom of a decline.
///
/// ```text
/// tol           = tolerance * max(|bar3.high|, |bar3.low|)
/// all three red                                        (close < open)
/// bar1 long lower shadow   (bar1.close − bar1.low) >= (bar1.open − bar1.close)
/// bar2 opens inside bar1's body, higher low, smaller body, closes above bar1.close
/// bar3 small black marubozu (upper & lower shadow <= tol) inside bar2's range
/// ```
///
/// Output is `+1.0` when the pattern completes and `0.0` otherwise. Three Stars
/// in the South is a single-direction (bullish-only) pattern, so it never emits
/// `−1.0`. The first two bars always return `0.0` because the three-bar window
/// is not yet filled. `tolerance` defaults to `0.001` (10 bps relative) and must
/// lie in `[0, 1)`. Pattern-shape check only — no trend filter is applied;
/// combine with a trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` bullish, `0.0` no pattern — so it drops straight into
/// a machine-learning feature matrix as a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, ThreeStarsInSouth};
///
/// let mut indicator = ThreeStarsInSouth::new();
/// indicator.update(Candle::new(20.0, 20.1, 8.0, 15.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(18.0, 18.1, 12.0, 16.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(15.0, 15.0, 14.0, 14.0, 1.0, 2).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone)]
pub struct ThreeStarsInSouth {
    tolerance: f64,
    prev: Option<Candle>,
    prev_prev: Option<Candle>,
    has_emitted: bool,
}

impl Default for ThreeStarsInSouth {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreeStarsInSouth {
    /// Construct a detector with the default relative tolerance (1e-3).
    pub const fn new() -> Self {
        Self {
            tolerance: 0.001,
            prev: None,
            prev_prev: None,
            has_emitted: false,
        }
    }

    /// Construct a detector with a custom relative tolerance.
    ///
    /// `tolerance` must lie in `[0, 1)`.
    pub fn with_tolerance(tolerance: f64) -> Result<Self> {
        if !(0.0..1.0).contains(&tolerance) {
            return Err(Error::InvalidPeriod {
                message: "three stars in the south tolerance must lie in [0, 1)",
            });
        }
        Ok(Self {
            tolerance,
            prev: None,
            prev_prev: None,
            has_emitted: false,
        })
    }

    /// Configured relative tolerance.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
}

impl Indicator for ThreeStarsInSouth {
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
        let tol = self.tolerance * candle.high.abs().max(candle.low.abs());
        let bar1_body = bar1.open - bar1.close;
        let bar1_lower_shadow = bar1.close - bar1.low;
        let bar2_body = bar2.open - bar2.close;
        let bar3_upper_shadow = candle.high - candle.open;
        let bar3_lower_shadow = candle.close - candle.low;
        if bar1.close < bar1.open
            && bar2.close < bar2.open
            && candle.close < candle.open
            && bar1_lower_shadow >= bar1_body
            && bar2.open <= bar1.open
            && bar2.open >= bar1.close
            && bar2.low > bar1.low
            && bar2.close > bar1.close
            && bar2_body < bar1_body
            && bar3_upper_shadow <= tol
            && bar3_lower_shadow <= tol
            && candle.high < bar2.high
            && candle.low > bar2.low
        {
            return Some(1.0);
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
        "ThreeStarsInSouth"
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
        assert!(ThreeStarsInSouth::with_tolerance(-0.01).is_err());
        assert!(ThreeStarsInSouth::with_tolerance(1.0).is_err());
    }

    #[test]
    fn accepts_valid_tolerance() {
        let t = ThreeStarsInSouth::with_tolerance(0.0).unwrap();
        assert!((t.tolerance() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn accessors_and_metadata() {
        let t = ThreeStarsInSouth::default();
        assert_eq!(t.name(), "ThreeStarsInSouth");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
        assert!((t.tolerance() - 0.001).abs() < 1e-12);
    }

    #[test]
    fn three_stars_in_south_is_plus_one() {
        let mut t = ThreeStarsInSouth::new();
        assert_eq!(t.update(c(20.0, 20.1, 8.0, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(18.0, 18.1, 12.0, 16.0, 1)), Some(0.0));
        assert_eq!(t.update(c(15.0, 15.0, 14.0, 14.0, 2)), Some(1.0));
    }

    #[test]
    fn third_with_shadow_yields_zero() {
        let mut t = ThreeStarsInSouth::new();
        t.update(c(20.0, 20.1, 8.0, 15.0, 0));
        t.update(c(18.0, 18.1, 12.0, 16.0, 1));
        // bar3 has a long lower shadow -> not a marubozu.
        assert_eq!(t.update(c(15.0, 15.0, 12.5, 14.0, 2)), Some(0.0));
    }

    #[test]
    fn lower_low_yields_zero() {
        let mut t = ThreeStarsInSouth::new();
        t.update(c(20.0, 20.1, 8.0, 15.0, 0));
        // bar2 dips below bar1's low -> no higher low.
        assert_eq!(t.update(c(18.0, 18.1, 7.0, 16.0, 1)), Some(0.0));
        assert_eq!(t.update(c(15.0, 15.0, 14.0, 14.0, 2)), Some(0.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = ThreeStarsInSouth::new();
        assert_eq!(t.update(c(20.0, 20.1, 8.0, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(18.0, 18.1, 12.0, 16.0, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 2.0, base + 2.1, base - 3.0, base, i)
            })
            .collect();
        let mut a = ThreeStarsInSouth::new();
        let mut b = ThreeStarsInSouth::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = ThreeStarsInSouth::new();
        t.update(c(20.0, 20.1, 8.0, 15.0, 0));
        t.update(c(18.0, 18.1, 12.0, 16.0, 1));
        t.update(c(15.0, 15.0, 14.0, 14.0, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(20.0, 20.1, 8.0, 15.0, 0)), Some(0.0));
    }
}
