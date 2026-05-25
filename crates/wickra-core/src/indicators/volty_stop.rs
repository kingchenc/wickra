//! Volty Stop (Volatility Stop, Kase).

use crate::error::{Error, Result};
use crate::indicators::atr::Atr;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Volty Stop — Cynthia Kase's volatility-anchored trailing stop. The stop is
/// hung off the *extreme close* recorded since the current trade was opened,
/// not off the most recent bar, which keeps it tight without giving back gains
/// when price pulls back inside the trend.
///
/// ```text
/// long:   anchor = max_close_since_long
///         stop_t = anchor − multiplier · ATR
///         flip-to-short on close < stop_t -> anchor = close, stop = close + mult · ATR
/// short:  anchor = min_close_since_short
///         stop_t = anchor + multiplier · ATR
///         flip-to-long  on close > stop_t -> anchor = close, stop = close − mult · ATR
/// ```
///
/// The anchor only ratchets in the trade's favour, so the stop tightens as
/// price reaches new extremes. Compared to the
/// [`AtrTrailingStop`](crate::AtrTrailingStop) — which re-anchors on every
/// bar's close — Volty Stop's extreme-anchor design gives back less on
/// pullbacks while keeping the same ATR-based volatility scaling. A common
/// configuration is `ATR(14)` with a `2.0` multiplier.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, VoltyStop};
///
/// let mut indicator = VoltyStop::new(14, 2.0).unwrap();
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
pub struct VoltyStop {
    atr: Atr,
    atr_period: usize,
    multiplier: f64,
    anchor: Option<f64>,
    long: bool,
}

impl VoltyStop {
    /// Construct a Volty Stop with an explicit ATR period and band multiplier.
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
            anchor: None,
            long: true,
        })
    }

    /// A common configuration: `ATR(14)` with a `2.0` multiplier.
    pub fn classic() -> Self {
        Self::new(14, 2.0).expect("classic Volty Stop params are valid")
    }

    /// Configured `(atr_period, multiplier)`.
    pub const fn params(&self) -> (usize, f64) {
        (self.atr_period, self.multiplier)
    }
}

impl Indicator for VoltyStop {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let atr = self.atr.update(candle)?;
        let band = self.multiplier * atr;
        let close = candle.close;

        let (anchor, long) = match (self.anchor, self.long) {
            (Some(prev_anchor), true) => {
                let stop = prev_anchor - band;
                if close < stop {
                    // Close-through long stop -> flip short, anchor at close.
                    (close, false)
                } else {
                    // Ratchet the anchor up to today's close if higher.
                    (prev_anchor.max(close), true)
                }
            }
            (Some(prev_anchor), false) => {
                let stop = prev_anchor + band;
                if close > stop {
                    (close, true)
                } else {
                    (prev_anchor.min(close), false)
                }
            }
            // First ATR-ready bar seeds a long anchor at the close.
            (None, _) => (close, true),
        };
        self.anchor = Some(anchor);
        self.long = long;
        let stop = if long { anchor - band } else { anchor + band };
        Some(stop)
    }

    fn reset(&mut self) {
        self.atr.reset();
        self.anchor = None;
        self.long = true;
    }

    fn warmup_period(&self) -> usize {
        self.atr_period
    }

    fn is_ready(&self) -> bool {
        self.anchor.is_some()
    }

    fn name(&self) -> &'static str {
        "VoltyStop"
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
        assert!(VoltyStop::new(0, 2.0).is_err());
        assert!(VoltyStop::new(14, 0.0).is_err());
        assert!(VoltyStop::new(14, -1.0).is_err());
        assert!(VoltyStop::new(14, f64::NAN).is_err());
    }

    #[test]
    fn accessors_and_metadata() {
        let s = VoltyStop::classic();
        let (p, m) = s.params();
        assert_eq!(p, 14);
        assert_relative_eq!(m, 2.0, epsilon = 1e-12);
        assert_eq!(s.warmup_period(), 14);
        assert_eq!(s.name(), "VoltyStop");
    }

    #[test]
    fn first_emission_matches_warmup() {
        let candles: Vec<Candle> = (0..20)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut s = VoltyStop::new(8, 2.0).unwrap();
        let out = s.batch(&candles);
        for (i, v) in out.iter().enumerate().take(7) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(out[7].is_some());
    }

    #[test]
    fn reference_values_flat_market() {
        // H=11, L=9, C=10 -> TR=2 -> ATR=2; band = 2·2 = 4; anchor stays at 10; stop = 10-4 = 6.
        let candles: Vec<Candle> = (0..20).map(|i| c(11.0, 9.0, 10.0, i)).collect();
        let mut s = VoltyStop::new(5, 2.0).unwrap();
        for v in s.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 6.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn uptrend_anchor_ratchets_up_with_close() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut s = VoltyStop::new(14, 3.0).unwrap();
        let emitted: Vec<(f64, f64)> = s
            .batch(&candles)
            .into_iter()
            .zip(candles.iter())
            .filter_map(|(o, c)| o.map(|v| (v, c.close)))
            .collect();
        for w in emitted.windows(2) {
            assert!(
                w[1].0 >= w[0].0 - 1e-9,
                "stop must not loosen in an uptrend"
            );
        }
        for &(stop, close) in &emitted {
            assert!(stop < close, "uptrend stop should sit below the close");
        }
    }

    #[test]
    fn stop_flips_on_reversal() {
        let mut candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        candles.extend((0..40).map(|i| {
            let base = 140.0 - 3.0 * i as f64;
            c(base + 1.0, base - 1.0, base, 40 + i)
        }));
        let mut s = VoltyStop::new(14, 3.0).unwrap();
        let paired: Vec<(f64, f64)> = s
            .batch(&candles)
            .into_iter()
            .zip(candles.iter())
            .filter_map(|(o, c)| o.map(|v| (v, c.close)))
            .collect();
        assert!(paired.iter().any(|&(stop, close)| stop < close));
        assert!(paired.iter().any(|&(stop, close)| stop > close));
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut s = VoltyStop::classic();
        s.batch(&candles);
        assert!(s.is_ready());
        s.reset();
        assert!(!s.is_ready());
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
        let mut a = VoltyStop::classic();
        let mut b = VoltyStop::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
