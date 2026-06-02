//! Evening Doji Star candlestick pattern.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Evening Doji Star — a 3-bar bearish top reversal. A long white bar extends the
/// advance, a doji gaps up above it (the star of indecision), then a black bar
/// gaps back down and closes deep into the first body, confirming the turn.
///
/// ```text
/// long body = |close − open| >= 0.5 * (high − low)
/// doji      = |close − open| <= 0.1 * (high − low)
/// bar1 white & long
/// bar2 doji, body gaps UP above bar1 body       (min(o2,c2) > close1)
/// bar3 black, body gaps DOWN below the doji      (max(o3,c3) < min(o2,c2))
/// bar3 closes deep into bar1 body                (close3 < close1 − penetration·body1)
/// ```
///
/// Output is `−1.0` when the pattern completes and `0.0` otherwise. Evening Doji
/// Star is a single-direction (bearish-only) reversal, so it never emits `+1.0`.
/// The first two bars always return `0.0` because the three-bar window is not yet
/// filled. `penetration` is how far into the first body the third bar must close;
/// it defaults to `0.3` (TA-Lib's `CDLEVENINGDOJISTAR` default) and must lie in
/// `[0, 1)`. Body and doji thresholds follow the geometric house style rather than
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
/// use wickra_core::{Candle, EveningDojiStar, Indicator};
///
/// let mut indicator = EveningDojiStar::new();
/// indicator.update(Candle::new(10.0, 15.1, 9.9, 15.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(17.0, 17.1, 16.9, 17.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(16.0, 16.1, 11.9, 12.0, 1.0, 2).unwrap());
/// assert_eq!(out, Some(-1.0));
/// ```
#[derive(Debug, Clone)]
pub struct EveningDojiStar {
    penetration: f64,
    prev: Option<Candle>,
    prev_prev: Option<Candle>,
    has_emitted: bool,
}

impl Default for EveningDojiStar {
    fn default() -> Self {
        Self::new()
    }
}

impl EveningDojiStar {
    /// Construct an Evening Doji Star detector with the default 0.3 penetration.
    pub const fn new() -> Self {
        Self {
            penetration: 0.3,
            prev: None,
            prev_prev: None,
            has_emitted: false,
        }
    }

    /// Construct an Evening Doji Star detector with a custom penetration fraction.
    ///
    /// `penetration` must lie in `[0, 1)`.
    pub fn with_penetration(penetration: f64) -> Result<Self> {
        if !(0.0..1.0).contains(&penetration) {
            return Err(Error::InvalidPeriod {
                message: "evening doji star penetration must lie in [0, 1)",
            });
        }
        Ok(Self {
            penetration,
            prev: None,
            prev_prev: None,
            has_emitted: false,
        })
    }

    /// Configured penetration fraction.
    pub fn penetration(&self) -> f64 {
        self.penetration
    }
}

impl Indicator for EveningDojiStar {
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
        let range1 = bar1.high - bar1.low;
        let range2 = bar2.high - bar2.low;
        if range1 <= 0.0 || range2 <= 0.0 {
            return Some(0.0);
        }
        let body1 = bar1.close - bar1.open;
        if body1 < 0.5 * range1 {
            return Some(0.0); // bar1 must be a long white body
        }
        if (bar2.close - bar2.open).abs() > 0.1 * range2 {
            return Some(0.0); // bar2 must be a doji
        }
        let star_bottom = bar2.open.min(bar2.close);
        let bar3_top = candle.open.max(candle.close);
        if star_bottom > bar1.close
            && candle.close < candle.open
            && bar3_top < star_bottom
            && candle.close < bar1.close - self.penetration * body1
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
        "EveningDojiStar"
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
    fn rejects_invalid_penetration() {
        assert!(EveningDojiStar::with_penetration(-0.01).is_err());
        assert!(EveningDojiStar::with_penetration(1.0).is_err());
    }

    #[test]
    fn accepts_valid_penetration() {
        let t = EveningDojiStar::with_penetration(0.5).unwrap();
        assert!((t.penetration() - 0.5).abs() < 1e-12);
    }

    #[test]
    fn accessors_and_metadata() {
        let t = EveningDojiStar::default();
        assert_eq!(t.name(), "EveningDojiStar");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
        assert!((t.penetration() - 0.3).abs() < 1e-12);
    }

    #[test]
    fn evening_doji_star_is_minus_one() {
        let mut t = EveningDojiStar::new();
        assert_eq!(t.update(c(10.0, 15.1, 9.9, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(17.0, 17.1, 16.9, 17.0, 1)), Some(0.0));
        assert_eq!(t.update(c(16.0, 16.1, 11.9, 12.0, 2)), Some(-1.0));
    }

    #[test]
    fn middle_not_doji_yields_zero() {
        let mut t = EveningDojiStar::new();
        t.update(c(10.0, 15.1, 9.9, 15.0, 0));
        // Wide-bodied star, not a doji.
        t.update(c(16.0, 18.1, 15.9, 18.0, 1));
        assert_eq!(t.update(c(16.0, 16.1, 11.9, 12.0, 2)), Some(0.0));
    }

    #[test]
    fn shallow_close_yields_zero() {
        let mut t = EveningDojiStar::new();
        t.update(c(10.0, 15.1, 9.9, 15.0, 0));
        t.update(c(17.0, 17.1, 16.9, 17.0, 1));
        // bar3 black but closes at 14.0 -> only 1.0 into the 5.0 body (< 0.3·5).
        assert_eq!(t.update(c(16.0, 16.1, 13.9, 14.0, 2)), Some(0.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = EveningDojiStar::new();
        assert_eq!(t.update(c(10.0, 15.1, 9.9, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(17.0, 17.1, 16.9, 17.0, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 5.2, base - 0.1, base + 5.0, i)
            })
            .collect();
        let mut a = EveningDojiStar::new();
        let mut b = EveningDojiStar::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = EveningDojiStar::new();
        t.update(c(10.0, 15.1, 9.9, 15.0, 0));
        t.update(c(17.0, 17.1, 16.9, 17.0, 1));
        t.update(c(16.0, 16.1, 11.9, 12.0, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(10.0, 15.1, 9.9, 15.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = EveningDojiStar::new();
        // Flat first bar (range1 == 0) -> rejected.
        t.update(c(10.0, 10.0, 10.0, 10.0, 0));
        t.update(c(17.0, 17.1, 16.9, 17.0, 1));
        assert_eq!(t.update(c(16.0, 16.1, 11.9, 12.0, 2)), Some(0.0));
    }

    #[test]
    fn short_first_body_yields_zero() {
        let mut t = EveningDojiStar::new();
        // bar1 has a wide range but a tiny body -> not a long white body.
        t.update(c(10.0, 16.0, 9.0, 10.5, 0));
        t.update(c(17.0, 17.1, 16.9, 17.0, 1));
        assert_eq!(t.update(c(16.0, 16.1, 11.9, 12.0, 2)), Some(0.0));
    }
}
