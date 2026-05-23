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
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Psar};
///
/// let mut indicator = Psar::new(0.02, 0.02, 0.2).unwrap();
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
pub struct Psar {
    af_start: f64,
    af_step: f64,
    af_max: f64,

    /// `true` once the first candle has been observed and the seed values
    /// (`prev_high`, `prev_low`, `sar`, `ep`) are valid. `false` is the
    /// constructor / `reset()` state in which the compute-fields hold
    /// `f64::NAN` sentinels.
    initialised: bool,
    /// `true` once `update` has returned the first `Some(sar)`. Drives
    /// [`Indicator::is_ready`] so it matches the convention of every other
    /// indicator: `is_ready() == true` ↔ the most recent `update` produced
    /// (or could produce) a real value. PSAR's seed candle returns `None`
    /// while `initialised` flips to `true`, which is why `is_ready` cannot
    /// just mirror `initialised`.
    has_emitted: bool,
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
            has_emitted: false,
            // NaN sentinels: any read of these fields before the seed candle
            // overwrites them is a logic bug. The `initialised` flag gates
            // every read, and the `debug_assert!` in `update` makes the
            // invariant explicit so a future refactor cannot silently treat a
            // sentinel as a real price.
            prev_high: f64::NAN,
            prev_low: f64::NAN,
            trend: Trend::Up,
            sar: f64::NAN,
            ep: f64::NAN,
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
            // `has_emitted` stays false — this is the seed bar; the first
            // `Some` lands on the next call.
            return None;
        }

        // After `initialised` flips to `true`, every compute field is guaranteed
        // finite. This guards against a future refactor that changes the seed
        // gate but leaves a NaN sentinel reachable.
        debug_assert!(
            self.prev_high.is_finite()
                && self.prev_low.is_finite()
                && self.sar.is_finite()
                && self.ep.is_finite(),
            "PSAR seed state must be finite once initialised"
        );

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
        self.has_emitted = true;
        Some(output_sar)
    }

    fn reset(&mut self) {
        // Restore every field to its constructor state. The compute fields
        // return to `f64::NAN` sentinels so a future refactor that reads them
        // before re-seeding cannot silently treat `0.0` as a real price.
        self.initialised = false;
        self.has_emitted = false;
        self.prev_high = f64::NAN;
        self.prev_low = f64::NAN;
        self.trend = Trend::Up;
        self.sar = f64::NAN;
        self.ep = f64::NAN;
        self.af = self.af_start;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        // Match the convention of every other indicator: `is_ready` flips to
        // `true` only once a real value has been returned. The previous
        // implementation returned `self.initialised`, which is `true` *after*
        // the seed candle (which itself returns `None`) — so a streaming
        // consumer that wrote `if ind.is_ready() { use(ind.update(c)?) }`
        // would hit a `None` it didn't expect. (Audit finding R6.)
        self.has_emitted
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
        // `all()` with `is_none_or` keeps every reachable arm on the hot path —
        // the previous filter_map / violation-Vec construction had a cold
        // "violation found" tuple branch that was unreachable on a clean
        // uptrend, leaving its line uncovered by Codecov.
        let ok = psar
            .batch(&candles)
            .iter()
            .enumerate()
            .all(|(i, sar)| sar.is_none_or(|s| s <= candles[i].low + 1e-9));
        assert!(ok, "SAR sat above a candle's low on a pure uptrend");
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
        // After the trend establishes downward, SAR should sit above highs.
        // Same `all()` + `is_none_or` shape as `pure_uptrend_sar_below_lows`
        // so the violation-tuple branch never appears as a cold path.
        let ok = psar
            .batch(&candles)
            .iter()
            .enumerate()
            .skip(5)
            .all(|(i, sar)| sar.is_none_or(|s| s >= candles[i].high - 1e-9));
        assert!(ok, "SAR sat below a candle's high on a pure downtrend");
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

    /// Cover the Indicator-impl `warmup_period` (206-208) and `name`
    /// (220-222). PSAR's warmup is the constant 2 (seed candle + first
    /// emitting candle); the name is the literal "PSAR".
    #[test]
    fn accessors_and_metadata() {
        let psar = Psar::classic();
        assert_eq!(psar.warmup_period(), 2);
        assert_eq!(psar.name(), "PSAR");
    }

    #[test]
    fn rejects_invalid_params() {
        assert!(Psar::new(0.0, 0.02, 0.20).is_err());
        assert!(Psar::new(0.02, 0.0, 0.20).is_err());
        assert!(Psar::new(0.30, 0.02, 0.20).is_err());
        assert!(Psar::new(f64::NAN, 0.02, 0.20).is_err());
    }

    #[test]
    fn is_ready_only_after_first_some_value() {
        // Audit R6: the previous implementation flipped `is_ready` to true on
        // the seed candle (which returns `None`), making the convention
        // `is_ready == last_value.is_some()` a lie. The new gate is
        // `has_emitted`, set when `update` returns its first `Some`.
        let mut psar = Psar::classic();
        assert!(!psar.is_ready(), "fresh PSAR must not be ready");
        let first = psar.update(c(11.0, 9.0, 10.0));
        assert!(first.is_none(), "seed candle returns None by design");
        assert!(
            !psar.is_ready(),
            "is_ready must stay false until a Some value is produced"
        );
        let second = psar.update(c(12.0, 10.0, 11.0));
        assert!(second.is_some(), "second candle must emit");
        assert!(
            psar.is_ready(),
            "is_ready must flip to true once a real value has been returned"
        );
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
