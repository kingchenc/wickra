//! Elder Ray — Bull Power and Bear Power.

use crate::error::Result;
use crate::indicators::ema::Ema;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// One Elder Ray reading: the bull and bear power for a bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElderRayOutput {
    /// `high − EMA(close)`: how far buyers pushed price above the trend mean.
    pub bull_power: f64,
    /// `low − EMA(close)`: how far sellers pushed price below the trend mean
    /// (negative in a normal market).
    pub bear_power: f64,
}

/// Elder Ray — Alexander Elder's Bull Power / Bear Power oscillator.
///
/// An EMA of the close marks the market's consensus of value; the bar's high and
/// low relative to it measure how far the bulls and bears could push price away
/// from that consensus:
///
/// ```text
/// ema       = EMA(close, period)
/// BullPower = high - ema
/// BearPower = low  - ema
/// ```
///
/// Bull Power is normally positive (the high prints above the mean) and Bear
/// Power normally negative (the low prints below it). Their behaviour relative
/// to zero and to the EMA's slope drives Elder's signals: e.g. in an uptrend
/// (rising EMA), a bounce in a negative-but-rising Bear Power is a buy setup.
///
/// The first reading lands once the inner EMA is seeded, at bar `period`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, ElderRay, Indicator};
///
/// let mut er = ElderRay::new(13).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let base = 100.0 + f64::from(i);
///     let c = Candle::new(base, base + 2.0, base - 2.0, base + 0.5, 1.0, i64::from(i)).unwrap();
///     last = er.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct ElderRay {
    period: usize,
    ema: Ema,
}

impl ElderRay {
    /// Construct an Elder Ray with the given EMA period.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            period,
            ema: Ema::new(period)?,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for ElderRay {
    type Input = Candle;
    type Output = ElderRayOutput;

    fn update(&mut self, candle: Candle) -> Option<ElderRayOutput> {
        let ema = self.ema.update(candle.close)?;
        Some(ElderRayOutput {
            bull_power: candle.high - ema,
            bear_power: candle.low - ema,
        })
    }

    fn reset(&mut self) {
        self.ema.reset();
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.ema.is_ready()
    }

    fn name(&self) -> &'static str {
        "ElderRay"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(high: f64, low: f64, close: f64) -> Candle {
        Candle::new(close, high, low, close, 1.0, 0).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(ElderRay::new(0).is_err());
    }

    /// Cover the const accessor `period` and the Indicator-impl `warmup_period`
    /// + `name`.
    #[test]
    fn accessors_and_metadata() {
        let er = ElderRay::new(13).unwrap();
        assert_eq!(er.period(), 13);
        assert_eq!(er.warmup_period(), 13);
        assert_eq!(er.name(), "ElderRay");
    }

    #[test]
    fn warmup_then_known_value() {
        // EMA(3) seeds at bar 3 with SMA([10,12,14]) = 12 (closes).
        // bar 3: high 16, low 13 -> bull = 16 - 12 = 4, bear = 13 - 12 = 1.
        let mut er = ElderRay::new(3).unwrap();
        assert_eq!(er.update(candle(11.0, 9.0, 10.0)), None);
        assert_eq!(er.update(candle(13.0, 11.0, 12.0)), None);
        let v = er.update(candle(16.0, 13.0, 14.0)).unwrap();
        assert_relative_eq!(v.bull_power, 4.0, epsilon = 1e-12);
        assert_relative_eq!(v.bear_power, 1.0, epsilon = 1e-12);
    }

    #[test]
    fn matches_manual_ema() {
        let bars: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.3).sin() * 5.0;
                candle(base + 2.0, base - 2.0, base)
            })
            .collect();
        let mut er = ElderRay::new(13).unwrap();
        let mut ema = Ema::new(13).unwrap();
        for (i, c) in bars.iter().enumerate() {
            let got = er.update(*c);
            let want = ema.update(c.close).map(|e| (c.high - e, c.low - e));
            assert_eq!(got.is_some(), want.is_some(), "readiness mismatch at {i}");
            if let (Some(g), Some((b, be))) = (got, want) {
                assert_relative_eq!(g.bull_power, b, epsilon = 1e-9);
                assert_relative_eq!(g.bear_power, be, epsilon = 1e-9);
            }
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut er = ElderRay::new(5).unwrap();
        er.batch(
            &(0..20)
                .map(|i| candle(f64::from(i) + 1.0, f64::from(i) - 1.0, f64::from(i)))
                .collect::<Vec<_>>(),
        );
        assert!(er.is_ready());
        er.reset();
        assert!(!er.is_ready());
        assert_eq!(er.update(candle(2.0, 0.0, 1.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let bars: Vec<Candle> = (0..30)
            .map(|i| {
                let base = 50.0 + f64::from(i);
                candle(base + 1.5, base - 1.5, base)
            })
            .collect();
        let mut a = ElderRay::new(7).unwrap();
        let mut b = ElderRay::new(7).unwrap();
        assert_eq!(
            a.batch(&bars),
            bars.iter().map(|c| b.update(*c)).collect::<Vec<_>>()
        );
    }
}
