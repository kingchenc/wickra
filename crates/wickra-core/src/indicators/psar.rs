//! Parabolic SAR (Wilder).

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Trade direction in the SAR state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Trend {
    Up,
    Down,
}

/// Parabolic Stop And Reverse.
///
/// Implementation follows Wilder's original recursion: each step computes a new
/// SAR from the previous SAR, extreme point (EP) and acceleration factor (AF);
/// the trend flips when price crosses the SAR.
#[derive(Debug, Clone)]
pub struct Psar {
    af_start: f64,
    af_step: f64,
    af_max: f64,

    initialised: bool,
    prev_high: f64,
    prev_low: f64,
    trend: Trend,
    sar: f64,
    ep: f64,
    af: f64,
}

impl Psar {
    /// Construct PSAR with explicit acceleration parameters.
    ///
    /// # Errors
    /// Returns [`Error::NonPositiveMultiplier`] / [`Error::InvalidPeriod`] for invalid params.
    pub fn new(af_start: f64, af_step: f64, af_max: f64) -> Result<Self> {
        if !af_start.is_finite() || !af_step.is_finite() || !af_max.is_finite() {
            return Err(Error::NonPositiveMultiplier);
        }
        if af_start <= 0.0 || af_step <= 0.0 || af_max <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        if af_start > af_max {
            return Err(Error::InvalidPeriod {
                message: "af_start must be <= af_max",
            });
        }
        Ok(Self {
            af_start,
            af_step,
            af_max,
            initialised: false,
            prev_high: 0.0,
            prev_low: 0.0,
            trend: Trend::Up,
            sar: 0.0,
            ep: 0.0,
            af: af_start,
        })
    }

    /// Wilder's defaults: `(0.02, 0.02, 0.20)`.
    pub fn classic() -> Self {
        Self::new(0.02, 0.02, 0.20).expect("classic PSAR params are valid")
    }
}

impl Indicator for Psar {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        if !self.initialised {
            // Seed on the first candle; the first SAR is emitted on the second.
            // The initial trend is assumed Up — PSAR's reversal logic flips it
            // within the first few bars if the market is actually falling.
            self.prev_high = candle.high;
            self.prev_low = candle.low;
            self.sar = candle.low;
            self.ep = candle.high;
            self.trend = Trend::Up;
            self.af = self.af_start;
            self.initialised = true;
            return None;
        }

        // Predicted SAR for this period (before clamping to prior two extremes).
        let mut new_sar = self.sar + self.af * (self.ep - self.sar);

        // Wilder rule: SAR cannot penetrate today's or yesterday's range.
        let prev_h = self.prev_high;
        let prev_l = self.prev_low;
        new_sar = match self.trend {
            Trend::Up => new_sar.min(prev_l).min(candle.low),
            Trend::Down => new_sar.max(prev_h).max(candle.high),
        };

        let mut output_sar = new_sar;

        // Check for trend reversal.
        let reversed = match self.trend {
            Trend::Up => candle.low <= new_sar,
            Trend::Down => candle.high >= new_sar,
        };

        if reversed {
            // Flip trend, reset AF and EP, place SAR at prior EP.
            output_sar = self.ep;
            self.trend = match self.trend {
                Trend::Up => Trend::Down,
                Trend::Down => Trend::Up,
            };
            self.ep = match self.trend {
                Trend::Up => candle.high,
                Trend::Down => candle.low,
            };
            self.af = self.af_start;
        } else {
            // Update EP and AF if a new extreme has been reached.
            match self.trend {
                Trend::Up => {
                    if candle.high > self.ep {
                        self.ep = candle.high;
                        self.af = (self.af + self.af_step).min(self.af_max);
                    }
                }
                Trend::Down => {
                    if candle.low < self.ep {
                        self.ep = candle.low;
                        self.af = (self.af + self.af_step).min(self.af_max);
                    }
                }
            }
        }

        self.sar = output_sar;
        self.prev_high = candle.high;
        self.prev_low = candle.low;
        Some(output_sar)
    }

    fn reset(&mut self) {
        // Restore every field to its constructor state. The re-seed on the next
        // `update` would overwrite the trend/extremes anyway, but a complete
        // reset keeps the struct consistent for inspection and future changes.
        self.initialised = false;
        self.prev_high = 0.0;
        self.prev_low = 0.0;
        self.trend = Trend::Up;
        self.sar = 0.0;
        self.ep = 0.0;
        self.af = self.af_start;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.initialised
    }

    fn name(&self) -> &'static str {
        "PSAR"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(h: f64, l: f64, cl: f64) -> Candle {
        Candle::new(cl, h, l, cl, 1.0, 0).unwrap()
    }

    #[test]
    fn first_candle_returns_none() {
        let mut psar = Psar::classic();
        assert_eq!(psar.update(c(11.0, 9.0, 10.0)), None);
    }

    #[test]
    fn pure_uptrend_sar_below_lows() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + f64::from(i);
                c(base + 0.5, base - 0.5, base)
            })
            .collect();
        let mut psar = Psar::classic();
        for (i, sar) in psar.batch(&candles).into_iter().enumerate() {
            if let Some(s) = sar {
                assert!(
                    s <= candles[i].low + 1e-9,
                    "SAR {s} should be <= low {} at i={i}",
                    candles[i].low
                );
            }
        }
    }

    #[test]
    fn pure_downtrend_sar_above_highs() {
        let candles: Vec<Candle> = (0..40)
            .rev()
            .map(|i| {
                let base = 100.0 + f64::from(i);
                c(base + 0.5, base - 0.5, base)
            })
            .collect();
        let mut psar = Psar::classic();
        let outs = psar.batch(&candles);
        // After the trend establishes downward, SAR should sit above highs.
        for (i, sar) in outs.into_iter().enumerate().skip(5) {
            if let Some(s) = sar {
                assert!(s >= candles[i].high - 1e-9);
            }
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.3).sin() * 8.0;
                c(m + 1.0, m - 1.0, m)
            })
            .collect();
        let mut a = Psar::classic();
        let mut b = Psar::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn rejects_invalid_params() {
        assert!(Psar::new(0.0, 0.02, 0.20).is_err());
        assert!(Psar::new(0.02, 0.0, 0.20).is_err());
        assert!(Psar::new(0.30, 0.02, 0.20).is_err());
        assert!(Psar::new(f64::NAN, 0.02, 0.20).is_err());
    }

    #[test]
    fn reset_allows_clean_reuse() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + f64::from(i);
                c(base + 0.5, base - 0.5, base)
            })
            .collect();
        let mut psar = Psar::classic();
        let first = psar.batch(&candles);
        assert!(psar.is_ready());
        psar.reset();
        assert!(!psar.is_ready());
        // A reset instance must reproduce a pristine run bit for bit.
        let second = psar.batch(&candles);
        assert_eq!(first, second);
    }
}
