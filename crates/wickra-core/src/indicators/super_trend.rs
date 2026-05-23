//! `SuperTrend`.

use crate::error::{Error, Result};
use crate::indicators::atr::Atr;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// `SuperTrend` output: the trailing-stop level and the trend direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuperTrendOutput {
    /// The `SuperTrend` line — the active trailing-stop level for this bar.
    pub value: f64,
    /// Trend direction: `+1.0` in an uptrend (the line sits below price),
    /// `-1.0` in a downtrend (the line sits above price).
    pub direction: f64,
}

/// Previous-bar state carried forward by the `SuperTrend` recurrence.
#[derive(Debug, Clone, Copy)]
struct PrevState {
    final_upper: f64,
    final_lower: f64,
    close: f64,
    direction: f64,
}

/// `SuperTrend` — an ATR-banded trailing stop that flips sides on a close
/// through the band.
///
/// ```text
/// hl2          = (high + low) / 2
/// basic_upper  = hl2 + multiplier · ATR
/// basic_lower  = hl2 − multiplier · ATR
///
/// final_upper  = basic_upper  if basic_upper < prev_final_upper or prev_close > prev_final_upper
///                else prev_final_upper
/// final_lower  = basic_lower  if basic_lower > prev_final_lower or prev_close < prev_final_lower
///                else prev_final_lower
///
/// in a downtrend: stay down while close <= final_upper, else flip up
/// in an uptrend:  stay up   while close >= final_lower, else flip down
/// SuperTrend   = final_lower in an uptrend, final_upper in a downtrend
/// ```
///
/// The final bands ratchet — the upper band only moves down (and the lower
/// band only moves up) until price closes through it, which flips the trend
/// and hands the role of trailing stop to the opposite band. The first
/// ATR-ready bar seeds the trend as up. Wilder's classic configuration is
/// `ATR(10)` with a `3.0` multiplier.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, SuperTrend};
///
/// let mut indicator = SuperTrend::classic();
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
pub struct SuperTrend {
    atr: Atr,
    multiplier: f64,
    atr_period: usize,
    prev: Option<PrevState>,
}

impl SuperTrend {
    /// Construct a `SuperTrend` with an explicit ATR period and band multiplier.
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
            multiplier,
            atr_period,
            prev: None,
        })
    }

    /// Wilder's classic configuration: `ATR(10)` with a `3.0` multiplier.
    pub fn classic() -> Self {
        Self::new(10, 3.0).expect("classic SuperTrend params are valid")
    }

    /// Configured `(atr_period, multiplier)`.
    pub const fn params(&self) -> (usize, f64) {
        (self.atr_period, self.multiplier)
    }
}

impl Indicator for SuperTrend {
    type Input = Candle;
    type Output = SuperTrendOutput;

    fn update(&mut self, candle: Candle) -> Option<SuperTrendOutput> {
        let atr = self.atr.update(candle)?;
        let hl2 = f64::midpoint(candle.high, candle.low);
        let basic_upper = hl2 + self.multiplier * atr;
        let basic_lower = hl2 - self.multiplier * atr;

        let (final_upper, final_lower, direction) = match self.prev {
            None => {
                // First ATR-ready bar: no prior bands, seed the trend as up.
                (basic_upper, basic_lower, 1.0)
            }
            Some(p) => {
                let final_upper = if basic_upper < p.final_upper || p.close > p.final_upper {
                    basic_upper
                } else {
                    p.final_upper
                };
                let final_lower = if basic_lower > p.final_lower || p.close < p.final_lower {
                    basic_lower
                } else {
                    p.final_lower
                };
                let direction = if p.direction < 0.0 {
                    // Previous downtrend — the line was the upper band.
                    if candle.close <= final_upper {
                        -1.0
                    } else {
                        1.0
                    }
                } else {
                    // Previous uptrend — the line was the lower band.
                    if candle.close >= final_lower {
                        1.0
                    } else {
                        -1.0
                    }
                };
                (final_upper, final_lower, direction)
            }
        };

        let value = if direction > 0.0 {
            final_lower
        } else {
            final_upper
        };
        self.prev = Some(PrevState {
            final_upper,
            final_lower,
            close: candle.close,
            direction,
        });
        Some(SuperTrendOutput { value, direction })
    }

    fn reset(&mut self) {
        self.atr.reset();
        self.prev = None;
    }

    fn warmup_period(&self) -> usize {
        self.atr_period
    }

    fn is_ready(&self) -> bool {
        self.prev.is_some()
    }

    fn name(&self) -> &'static str {
        "SuperTrend"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(f64::midpoint(high, low), high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn uptrend_keeps_line_below_price_and_direction_up() {
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                let base = 100.0 + 2.0 * i as f64;
                c(base + 1.0, base - 1.0, base + 0.5, i)
            })
            .collect();
        let mut st = SuperTrend::classic();
        for (o, candle) in st.batch(&candles).into_iter().zip(candles.iter()) {
            if let Some(o) = o {
                assert_eq!(o.direction, 1.0, "a pure uptrend stays in direction +1");
                assert!(o.value < candle.close, "the stop line sits below price");
            }
        }
    }

    #[test]
    fn downtrend_keeps_line_above_price_and_direction_down() {
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                let base = 220.0 - 2.0 * i as f64;
                c(base + 1.0, base - 1.0, base - 0.5, i)
            })
            .collect();
        let mut st = SuperTrend::classic();
        let emitted: Vec<(SuperTrendOutput, f64)> = st
            .batch(&candles)
            .into_iter()
            .zip(candles.iter())
            .filter_map(|(o, c)| o.map(|v| (v, c.close)))
            .collect();
        // The seed bar starts the trend up; a steep decline flips it within a
        // few bars. The settled tail must be a clean downtrend.
        for &(o, close) in emitted.iter().skip(10) {
            assert_eq!(
                o.direction, -1.0,
                "a steep downtrend settles to direction -1"
            );
            assert!(o.value > close, "the stop line sits above price");
        }
    }

    #[test]
    fn trend_flips_when_price_reverses() {
        let mut candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base + 0.5, i)
            })
            .collect();
        candles.extend((0..40).map(|i| {
            let base = 140.0 - i as f64;
            c(base + 1.0, base - 1.0, base - 0.5, 40 + i)
        }));
        let mut st = SuperTrend::classic();
        let dirs: Vec<f64> = st
            .batch(&candles)
            .into_iter()
            .flatten()
            .map(|o| o.direction)
            .collect();
        assert!(dirs.iter().any(|&d| d > 0.0), "expected an uptrend stretch");
        assert!(
            dirs.iter().any(|&d| d < 0.0),
            "expected a downtrend stretch"
        );
    }

    #[test]
    fn first_emission_matches_warmup_period() {
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut st = SuperTrend::classic();
        let out = st.batch(&candles);
        assert_eq!(st.warmup_period(), 10);
        for (i, v) in out.iter().enumerate().take(9) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(out[9].is_some(), "first value lands at warmup_period - 1");
    }

    #[test]
    fn rejects_invalid_params() {
        assert!(SuperTrend::new(0, 3.0).is_err());
        assert!(SuperTrend::new(10, 0.0).is_err());
        assert!(SuperTrend::new(10, -1.0).is_err());
        assert!(SuperTrend::new(10, f64::NAN).is_err());
    }

    /// Cover the const accessor `params` (99-101) and the Indicator-impl
    /// `name` body (176-178). `warmup_period` is exercised elsewhere.
    #[test]
    fn accessors_and_metadata() {
        let st = SuperTrend::new(10, 3.0).unwrap();
        let (p, m) = st.params();
        assert_eq!(p, 10);
        assert!((m - 3.0).abs() < 1e-12);
        assert_eq!(st.name(), "SuperTrend");
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut st = SuperTrend::classic();
        st.batch(&candles);
        assert!(st.is_ready());
        st.reset();
        assert!(!st.is_ready());
        assert_eq!(st.update(candles[0]), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.3).sin() * 8.0;
                c(mid + 1.5, mid - 1.5, mid + 0.5, i)
            })
            .collect();
        let mut a = SuperTrend::classic();
        let mut b = SuperTrend::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
