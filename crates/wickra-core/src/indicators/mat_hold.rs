//! Mat Hold candlestick pattern.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Mat Hold — a 5-bar bullish continuation. A long white candle is followed by a
/// brief three-bar pullback that gaps up and then drifts on small bodies *without*
/// surrendering much ground, after which a white candle breaks to a new high and
/// the uptrend resumes.
///
/// ```text
/// long body = |close − open| >= 0.5 * (high − low)
/// bar1 white & long
/// bar2 small body gapping up above bar1   (min(o2,c2) > close1)
/// bar2, bar3, bar4 each small             (|body| <= 0.5 · body1)
/// the pullback holds                       (min low of bars 2..4 > close1 − penetration·body1)
/// bar5 white, closing at a new high        (close5 > max high of bars 1..4)
/// ```
///
/// Output is `+1.0` when the pattern completes and `0.0` otherwise. Mat Hold is a
/// single-direction (bullish-only) continuation, so it never emits `−1.0`. The
/// first four bars always return `0.0` because the five-bar window is not yet
/// filled. `penetration` is how far the pullback may retrace into the first body;
/// it defaults to `0.5` (TA-Lib's `CDLMATHOLD` default) and must lie in `[0, 1)`.
/// Body thresholds follow the geometric house style rather than TA-Lib's rolling
/// averages. Pattern-shape check only — no trend filter is applied; combine with a
/// trend indicator for actionable signals.
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
/// use wickra_core::{Candle, Indicator, MatHold};
///
/// let mut indicator = MatHold::new();
/// indicator.update(Candle::new(10.0, 15.1, 9.9, 15.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(16.0, 16.1, 15.4, 15.5, 1.0, 1).unwrap());
/// indicator.update(Candle::new(15.5, 15.6, 14.9, 15.0, 1.0, 2).unwrap());
/// indicator.update(Candle::new(15.0, 15.1, 14.4, 14.5, 1.0, 3).unwrap());
/// let out = indicator
///     .update(Candle::new(14.5, 17.1, 14.4, 17.0, 1.0, 4).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone)]
pub struct MatHold {
    penetration: f64,
    c1: Option<Candle>,
    c2: Option<Candle>,
    c3: Option<Candle>,
    c4: Option<Candle>,
    has_emitted: bool,
}

impl Default for MatHold {
    fn default() -> Self {
        Self::new()
    }
}

impl MatHold {
    /// Construct a Mat Hold detector with the default 0.5 penetration.
    pub const fn new() -> Self {
        Self {
            penetration: 0.5,
            c1: None,
            c2: None,
            c3: None,
            c4: None,
            has_emitted: false,
        }
    }

    /// Construct a Mat Hold detector with a custom penetration fraction.
    ///
    /// `penetration` must lie in `[0, 1)`.
    pub fn with_penetration(penetration: f64) -> Result<Self> {
        if !(0.0..1.0).contains(&penetration) {
            return Err(Error::InvalidPeriod {
                message: "mat hold penetration must lie in [0, 1)",
            });
        }
        Ok(Self {
            penetration,
            c1: None,
            c2: None,
            c3: None,
            c4: None,
            has_emitted: false,
        })
    }

    /// Configured penetration fraction.
    pub fn penetration(&self) -> f64 {
        self.penetration
    }
}

impl Indicator for MatHold {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let bar1 = self.c1;
        let bar2 = self.c2;
        let bar3 = self.c3;
        let bar4 = self.c4;
        self.c1 = self.c2;
        self.c2 = self.c3;
        self.c3 = self.c4;
        self.c4 = Some(candle);
        let (Some(bar1), Some(bar2), Some(bar3), Some(bar4)) = (bar1, bar2, bar3, bar4) else {
            return Some(0.0);
        };
        let range1 = bar1.high - bar1.low;
        if range1 <= 0.0 {
            return Some(0.0);
        }
        let body1 = bar1.close - bar1.open;
        if body1 < 0.5 * range1 {
            return Some(0.0); // bar1 must be a long white body
        }
        let small = 0.5 * body1;
        if (bar2.close - bar2.open).abs() > small
            || (bar3.close - bar3.open).abs() > small
            || (bar4.close - bar4.open).abs() > small
        {
            return Some(0.0); // the three pullback bars must be small
        }
        // bar2 gaps up above bar1's body.
        if bar2.open.min(bar2.close) <= bar1.close {
            return Some(0.0);
        }
        // The pullback must hold above the penetration line.
        let hold_line = bar1.close - self.penetration * body1;
        if bar2.low.min(bar3.low).min(bar4.low) <= hold_line {
            return Some(0.0);
        }
        // bar5 breaks to a new high on a white body.
        let max_high = bar1.high.max(bar2.high).max(bar3.high).max(bar4.high);
        if candle.close > candle.open && candle.close > max_high {
            return Some(1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.c1 = None;
        self.c2 = None;
        self.c3 = None;
        self.c4 = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        5
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "MatHold"
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
        assert!(MatHold::with_penetration(-0.01).is_err());
        assert!(MatHold::with_penetration(1.0).is_err());
    }

    #[test]
    fn accepts_valid_penetration() {
        let t = MatHold::with_penetration(0.3).unwrap();
        assert!((t.penetration() - 0.3).abs() < 1e-12);
    }

    #[test]
    fn accessors_and_metadata() {
        let t = MatHold::default();
        assert_eq!(t.name(), "MatHold");
        assert_eq!(t.warmup_period(), 5);
        assert!(!t.is_ready());
        assert!((t.penetration() - 0.5).abs() < 1e-12);
    }

    #[test]
    fn mat_hold_is_plus_one() {
        let mut t = MatHold::new();
        assert_eq!(t.update(c(10.0, 15.1, 9.9, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(16.0, 16.1, 15.4, 15.5, 1)), Some(0.0));
        assert_eq!(t.update(c(15.5, 15.6, 14.9, 15.0, 2)), Some(0.0));
        assert_eq!(t.update(c(15.0, 15.1, 14.4, 14.5, 3)), Some(0.0));
        assert_eq!(t.update(c(14.5, 17.1, 14.4, 17.0, 4)), Some(1.0));
    }

    #[test]
    fn pullback_breaks_hold_yields_zero() {
        let mut t = MatHold::new();
        t.update(c(10.0, 15.1, 9.9, 15.0, 0));
        t.update(c(16.0, 16.1, 15.4, 15.5, 1));
        t.update(c(15.5, 15.6, 14.9, 15.0, 2));
        // bar4 dips below the hold line (close1 - 0.5*body1 = 12.5).
        t.update(c(13.0, 13.1, 12.0, 12.4, 3));
        assert_eq!(t.update(c(14.5, 17.1, 12.0, 17.0, 4)), Some(0.0));
    }

    #[test]
    fn no_new_high_yields_zero() {
        let mut t = MatHold::new();
        t.update(c(10.0, 15.1, 9.9, 15.0, 0));
        t.update(c(16.0, 16.1, 15.4, 15.5, 1));
        t.update(c(15.5, 15.6, 14.9, 15.0, 2));
        t.update(c(15.0, 15.1, 14.4, 14.5, 3));
        // bar5 white but closes below the prior max high (16.1).
        assert_eq!(t.update(c(14.5, 16.0, 14.4, 15.9, 4)), Some(0.0));
    }

    #[test]
    fn first_four_bars_return_zero() {
        let mut t = MatHold::new();
        assert_eq!(t.update(c(10.0, 15.1, 9.9, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(16.0, 16.1, 15.4, 15.5, 1)), Some(0.0));
        assert_eq!(t.update(c(15.5, 15.6, 14.9, 15.0, 2)), Some(0.0));
        assert_eq!(t.update(c(15.0, 15.1, 14.4, 14.5, 3)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 5.2, base - 0.1, base + 5.0, i)
            })
            .collect();
        let mut a = MatHold::new();
        let mut b = MatHold::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = MatHold::new();
        t.update(c(10.0, 15.1, 9.9, 15.0, 0));
        t.update(c(16.0, 16.1, 15.4, 15.5, 1));
        t.update(c(15.5, 15.6, 14.9, 15.0, 2));
        t.update(c(15.0, 15.1, 14.4, 14.5, 3));
        t.update(c(14.5, 17.1, 14.4, 17.0, 4));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(10.0, 15.1, 9.9, 15.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_first_bar_yields_zero() {
        let mut t = MatHold::new();
        // Flat first bar (range1 == 0) -> rejected.
        t.update(c(10.0, 10.0, 10.0, 10.0, 0));
        t.update(c(16.0, 16.1, 15.4, 15.5, 1));
        t.update(c(15.5, 15.6, 14.9, 15.0, 2));
        t.update(c(15.0, 15.1, 14.4, 14.5, 3));
        assert_eq!(t.update(c(14.5, 17.1, 14.4, 17.0, 4)), Some(0.0));
    }

    #[test]
    fn short_first_body_yields_zero() {
        let mut t = MatHold::new();
        // bar1 has a wide range but a tiny body -> not a long white body.
        t.update(c(10.0, 16.0, 9.0, 10.5, 0));
        t.update(c(16.0, 16.1, 15.4, 15.5, 1));
        t.update(c(15.5, 15.6, 14.9, 15.0, 2));
        t.update(c(15.0, 15.1, 14.4, 14.5, 3));
        assert_eq!(t.update(c(14.5, 17.1, 14.4, 17.0, 4)), Some(0.0));
    }

    #[test]
    fn no_gap_up_yields_zero() {
        let mut t = MatHold::new();
        // Long white bar1 with small pullbacks, but bar2 fails to gap up above
        // bar1's close.
        t.update(c(10.0, 15.1, 9.9, 15.0, 0));
        t.update(c(14.5, 14.7, 14.3, 14.5, 1));
        t.update(c(14.5, 14.7, 14.3, 14.6, 2));
        t.update(c(14.6, 14.8, 14.4, 14.7, 3));
        assert_eq!(t.update(c(14.7, 17.1, 14.6, 17.0, 4)), Some(0.0));
    }
}
