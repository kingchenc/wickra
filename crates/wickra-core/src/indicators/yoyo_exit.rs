//! Yo-Yo Exit.

use crate::error::{Error, Result};
use crate::indicators::atr::Atr;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Yo-Yo Exit — an ATR-based long-only trailing stop that "yo-yos" in and out
/// of the market: when price closes below the trail it exits, and when price
/// recovers `multiplier · ATR` above the same trail it re-enters long. The
/// emitted level is always the *trail itself* (not a flip-to-short stop), so a
/// consumer reads a single line on the chart and toggles the position
/// depending on which side of it the close sits.
///
/// ```text
/// band = multiplier · ATR
/// in-trade:  trail_t = max(trail_{t−1}, close − band)
///            exit when close < trail
/// out:       trail held flat at the last in-trade level
///            re-enter when close > trail + band
/// ```
///
/// Unlike [`AtrTrailingStop`](crate::AtrTrailingStop) — which always flips to
/// the opposite side — the Yo-Yo only takes longs and treats the off-period
/// as a "wait until price proves itself again" phase. A common configuration
/// is `ATR(14)` with a `2.0` multiplier.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, YoyoExit};
///
/// let mut indicator = YoyoExit::new(14, 2.0).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct YoyoExit {
    atr: Atr,
    atr_period: usize,
    multiplier: f64,
    trail: Option<f64>,
    /// `true` while the trail is being ratcheted by new closes; `false` while
    /// the strategy is sidelined waiting for a re-entry.
    in_trade: bool,
}

impl YoyoExit {
    /// Construct a Yo-Yo Exit with an explicit ATR period and band multiplier.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `atr_period == 0` and
    /// [`Error::NonPositiveMultiplier`] if `multiplier` is not strictly
    /// positive and finite.
    pub fn new(atr_period: usize, multiplier: f64) -> Result<Self> {
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            atr: Atr::new(atr_period)?,
            atr_period,
            multiplier,
            trail: None,
            in_trade: true,
        })
    }

    /// A common configuration: `ATR(14)` with a `2.0` multiplier.
    pub fn classic() -> Self {
        Self::new(14, 2.0).expect("classic Yo-Yo Exit params are valid")
    }

    /// Configured `(atr_period, multiplier)`.
    pub const fn params(&self) -> (usize, f64) {
        (self.atr_period, self.multiplier)
    }

    /// `true` while the strategy is currently long, `false` while sidelined.
    pub const fn in_trade(&self) -> bool {
        self.in_trade
    }
}

impl Indicator for YoyoExit {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let atr = self.atr.update(candle)?;
        let band = self.multiplier * atr;
        let close = candle.close;

        let trail = match self.trail {
            Some(prev) => {
                if self.in_trade {
                    if close < prev {
                        // Stopped out — sideline, keep the trail flat.
                        self.in_trade = false;
                        prev
                    } else {
                        // Ratchet up only.
                        prev.max(close - band)
                    }
                } else if close > prev + band {
                    // Re-entry trigger — start a new trail anchored on this close.
                    self.in_trade = true;
                    close - band
                } else {
                    prev
                }
            }
            // First ATR-ready bar starts a fresh long.
            None => close - band,
        };
        self.trail = Some(trail);
        Some(trail)
    }

    fn reset(&mut self) {
        self.atr.reset();
        self.trail = None;
        self.in_trade = true;
    }

    fn warmup_period(&self) -> usize {
        self.atr_period
    }

    fn is_ready(&self) -> bool {
        self.trail.is_some()
    }

    fn name(&self) -> &'static str {
        "YoyoExit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(f64::midpoint(high, low), high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn rejects_invalid_params() {
        assert!(YoyoExit::new(0, 2.0).is_err());
        assert!(YoyoExit::new(14, 0.0).is_err());
        assert!(YoyoExit::new(14, -1.0).is_err());
        assert!(YoyoExit::new(14, f64::NAN).is_err());
    }

    #[test]
    fn accessors_and_metadata() {
        let s = YoyoExit::classic();
        let (p, m) = s.params();
        assert_eq!(p, 14);
        assert_relative_eq!(m, 2.0, epsilon = 1e-12);
        assert_eq!(s.warmup_period(), 14);
        assert_eq!(s.name(), "YoyoExit");
        assert!(s.in_trade());
    }

    #[test]
    fn first_emission_matches_warmup() {
        let candles: Vec<Candle> = (0..20)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut s = YoyoExit::new(8, 2.0).unwrap();
        let out = s.batch(&candles);
        for (i, v) in out.iter().enumerate().take(7) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(out[7].is_some());
    }

    #[test]
    fn reference_values_flat_market() {
        // ATR = 2; band = 4; trail starts at close - band = 10 - 4 = 6 and stays there.
        let candles: Vec<Candle> = (0..20).map(|i| c(11.0, 9.0, 10.0, i)).collect();
        let mut s = YoyoExit::new(5, 2.0).unwrap();
        for v in s.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 6.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn uptrend_trail_ratchets_up() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut s = YoyoExit::new(14, 3.0).unwrap();
        let emitted: Vec<f64> = s.batch(&candles).into_iter().flatten().collect();
        for w in emitted.windows(2) {
            assert!(w[1] >= w[0] - 1e-9, "trail must not loosen in an uptrend");
        }
    }

    #[test]
    fn reentry_after_stop_out() {
        // Up-leg sets the trail, big drop stops out, recovery re-enters.
        let mut candles: Vec<Candle> = (0..30)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        candles.push(c(60.0, 40.0, 50.0, 30)); // stop-out
        candles.push(c(60.0, 50.0, 55.0, 31)); // still out
        candles.push(c(200.0, 100.0, 200.0, 32)); // strong rally -> re-entry
        let mut s = YoyoExit::new(14, 3.0).unwrap();
        // Drive to completion; we just need it to not panic and to flip in_trade
        // back to true once a re-entry trigger fires.
        for c in &candles {
            let _ = s.update(*c);
        }
        assert!(s.is_ready());
        // Final candle's close (200) is way above the trail, so we're back in.
        assert!(s.in_trade());
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut s = YoyoExit::classic();
        s.batch(&candles);
        assert!(s.is_ready());
        s.reset();
        assert!(!s.is_ready());
        assert!(s.in_trade());
        assert_eq!(s.update(candles[0]), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.3).sin() * 8.0;
                c(mid + 1.5, mid - 1.5, mid + 0.5, i)
            })
            .collect();
        let mut a = YoyoExit::classic();
        let mut b = YoyoExit::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
