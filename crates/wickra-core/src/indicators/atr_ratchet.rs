//! ATR Ratchet (Kaufman) — a trailing stop that creeps toward price each bar.

use crate::error::{Error, Result};
use crate::indicators::atr::Atr;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Output of [`AtrRatchet`]: the active stop level and the trend direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtrRatchetOutput {
    /// The ratchet stop level — below price when long, above price when short.
    pub value: f64,
    /// Trend direction: `+1.0` long, `-1.0` short.
    pub direction: f64,
}

/// ATR Ratchet — Perry Kaufman's time-based volatility stop that tightens by a
/// fixed fraction of ATR **every bar**, whether or not price moves.
///
/// ```text
/// on entry (long):   stop = close − start_mult · ATR
/// each later bar:     stop = stop + increment · ATR    (ratchets toward price)
/// flip to short when  close < stop, reseeding stop = close + start_mult · ATR
/// ```
///
/// Most trailing stops only move when price makes a new extreme. Kaufman's ratchet
/// instead advances the stop a little each bar — `increment · ATR` — so a trade
/// that stalls is squeezed out over time even in a flat market. The initial
/// distance (`start_mult · ATR`) gives the position room to breathe; the per-bar
/// `increment` controls how aggressively the leash shortens. When price closes
/// through the stop the system reverses and reseeds at the full initial distance.
///
/// The first stop lands once ATR is ready (`atr_period` inputs). Each `update` is
/// O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, AtrRatchet};
///
/// let mut indicator = AtrRatchet::new(14, 4.0, 0.1).unwrap();
/// let mut last = None;
/// for i in 0..60 {
///     let base = 100.0 + f64::from(i);
///     let c = Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 1_000.0, 0).unwrap();
///     last = indicator.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct AtrRatchet {
    atr: Atr,
    atr_period: usize,
    start_mult: f64,
    increment: f64,
    direction: f64,
    stop: f64,
    last: Option<AtrRatchetOutput>,
}

impl AtrRatchet {
    /// Construct an ATR Ratchet stop.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `atr_period == 0` and
    /// [`Error::NonPositiveMultiplier`] if `start_mult` or `increment` is not
    /// finite and positive.
    pub fn new(atr_period: usize, start_mult: f64, increment: f64) -> Result<Self> {
        if !start_mult.is_finite()
            || start_mult <= 0.0
            || !increment.is_finite()
            || increment <= 0.0
        {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            atr: Atr::new(atr_period)?,
            atr_period,
            start_mult,
            increment,
            direction: 0.0,
            stop: 0.0,
            last: None,
        })
    }

    /// Configured `(atr_period, start_mult, increment)`.
    pub const fn params(&self) -> (usize, f64, f64) {
        (self.atr_period, self.start_mult, self.increment)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<AtrRatchetOutput> {
        self.last
    }
}

impl Indicator for AtrRatchet {
    type Input = Candle;
    type Output = AtrRatchetOutput;

    fn update(&mut self, candle: Candle) -> Option<AtrRatchetOutput> {
        let atr = self.atr.update(candle)?;
        let close = candle.close;

        if self.direction == 0.0 {
            self.direction = 1.0;
            self.stop = close - self.start_mult * atr;
        } else if self.direction > 0.0 {
            self.stop += self.increment * atr;
            if close < self.stop {
                self.direction = -1.0;
                self.stop = close + self.start_mult * atr;
            }
        } else {
            self.stop -= self.increment * atr;
            if close > self.stop {
                self.direction = 1.0;
                self.stop = close - self.start_mult * atr;
            }
        }

        let out = AtrRatchetOutput {
            value: self.stop,
            direction: self.direction,
        };
        self.last = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.atr.reset();
        self.direction = 0.0;
        self.stop = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.atr_period
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "AtrRatchet"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(high: f64, low: f64, close: f64) -> Candle {
        Candle::new_unchecked(f64::midpoint(high, low), high, low, close, 1_000.0, 0)
    }

    #[test]
    fn rejects_invalid_params() {
        assert!(matches!(
            AtrRatchet::new(0, 4.0, 0.1),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            AtrRatchet::new(14, 0.0, 0.1),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            AtrRatchet::new(14, 4.0, 0.0),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            AtrRatchet::new(14, 4.0, f64::NAN),
            Err(Error::NonPositiveMultiplier)
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let r = AtrRatchet::new(14, 4.0, 0.1).unwrap();
        assert_eq!(r.params(), (14, 4.0, 0.1));
        assert_eq!(r.warmup_period(), 14);
        assert_eq!(r.name(), "AtrRatchet");
        assert!(!r.is_ready());
        assert_eq!(r.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut r = AtrRatchet::new(5, 4.0, 0.1).unwrap();
        let candles: Vec<Candle> = (0..12)
            .map(|i| {
                let base = 100.0 + f64::from(i);
                c(base + 1.0, base - 1.0, base)
            })
            .collect();
        let out = r.batch(&candles);
        for v in out.iter().take(4) {
            assert!(v.is_none());
        }
        assert!(out[4].is_some());
    }

    #[test]
    fn uptrend_keeps_stop_below_price() {
        let mut r = AtrRatchet::new(5, 4.0, 0.05).unwrap();
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                let base = 100.0 + 2.0 * f64::from(i);
                c(base + 1.0, base - 1.0, base + 0.5)
            })
            .collect();
        for (o, candle) in r.batch(&candles).into_iter().zip(candles.iter()) {
            if let Some(o) = o {
                assert_eq!(o.direction, 1.0);
                assert!(o.value < candle.close);
            }
        }
    }

    #[test]
    fn stall_eventually_triggers_flip() {
        // A long trend then a long flat stretch: the ratchet creeps up each bar
        // and eventually overtakes the flat close, flipping to short.
        let mut r = AtrRatchet::new(5, 2.0, 0.5).unwrap();
        let mut candles: Vec<Candle> = (0..20)
            .map(|i| {
                let base = 100.0 + f64::from(i);
                c(base + 1.0, base - 1.0, base + 0.5)
            })
            .collect();
        // Flat stretch at the last price.
        candles.extend((0..40).map(|_| c(120.6, 118.6, 119.5)));
        let dirs: Vec<f64> = r
            .batch(&candles)
            .into_iter()
            .flatten()
            .map(|o| o.direction)
            .collect();
        assert!(
            dirs.iter().any(|&d| d < 0.0),
            "the ratchet should eventually flip short"
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut r = AtrRatchet::new(5, 4.0, 0.1).unwrap();
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + f64::from(i);
                c(base + 1.0, base - 1.0, base + 0.5)
            })
            .collect();
        r.batch(&candles);
        assert!(r.is_ready());
        r.reset();
        assert!(!r.is_ready());
        assert_eq!(r.value(), None);
        assert_eq!(r.update(candles[0]), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..120)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.25).sin() * 9.0;
                c(base + 2.0, base - 1.5, base + 0.5)
            })
            .collect();
        let batch = AtrRatchet::new(14, 4.0, 0.1).unwrap().batch(&candles);
        let mut b = AtrRatchet::new(14, 4.0, 0.1).unwrap();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }
}
