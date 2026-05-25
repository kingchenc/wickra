//! Demand Index (James Sibbet).

use crate::error::{Error, Result};
use crate::indicators::ema::Ema;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// James Sibbet's Demand Index — a smoothed ratio of buying pressure to
/// selling pressure, classifying each bar's volume by whether the close rose
/// or fell relative to the previous close.
///
/// Sibbet's original 1970s formulation runs the raw buying/selling pressure
/// through several smoothings and yields a number that swings in `[−100, 100]`.
/// This implementation uses the textbook simplified form that captures the same
/// signal in a streaming-friendly shape:
///
/// ```text
/// pressure_t = volume_t · ((close_t − close_{t−1}) / max(close_{t−1}, ε))
///              · (1 + (high_t − low_t) / max(close_{t−1}, ε))
/// DI_t       = EMA(pressure, period)_t
/// ```
///
/// Positive readings mean the smoothed money flow is leaning to the buy side
/// (up-day volume dominates), negative to the sell side. The first candle only
/// establishes the previous close, so the first non-`None` value lands once the
/// EMA has accumulated `period` pressure samples. A previous close of zero
/// contributes no signal (avoids division by zero). The output is unbounded;
/// what matters is the sign and the divergence against price.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, DemandIndex, Indicator};
///
/// let mut indicator = DemandIndex::new(10).unwrap();
/// let mut last = None;
/// for i in 0..120 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 50.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct DemandIndex {
    period: usize,
    ema: Ema,
    prev_close: Option<f64>,
}

impl DemandIndex {
    /// Construct a new Demand Index with the given EMA smoothing period.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            ema: Ema::new(period)?,
            prev_close: None,
        })
    }

    /// Configured EMA smoothing period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for DemandIndex {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev) = self.prev_close else {
            self.prev_close = Some(candle.close);
            return None;
        };
        let pressure = if prev == 0.0 {
            // No prior baseline -> can't normalise; treat as no flow.
            0.0
        } else {
            let ret = (candle.close - prev) / prev;
            let range_norm = (candle.high - candle.low) / prev;
            candle.volume * ret * (1.0 + range_norm)
        };
        self.prev_close = Some(candle.close);
        self.ema.update(pressure)
    }

    fn reset(&mut self) {
        self.ema.reset();
        self.prev_close = None;
    }

    fn warmup_period(&self) -> usize {
        // One seed bar to establish the previous close, then the EMA needs
        // `period` samples to seed.
        self.period + 1
    }

    fn is_ready(&self) -> bool {
        self.ema.is_ready()
    }

    fn name(&self) -> &'static str {
        "DemandIndex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(open: f64, high: f64, low: f64, close: f64, volume: f64, ts: i64) -> Candle {
        Candle::new(open, high, low, close, volume, ts).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(DemandIndex::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let di = DemandIndex::new(10).unwrap();
        assert_eq!(di.period(), 10);
        assert_eq!(di.name(), "DemandIndex");
        assert_eq!(di.warmup_period(), 11);
    }

    #[test]
    fn constant_series_yields_zero() {
        // No close change -> pressure = 0 on every bar -> EMA stays at 0.
        let candles: Vec<Candle> = (0..40)
            .map(|i| c(10.0, 10.0, 10.0, 10.0, 100.0, i))
            .collect();
        let mut di = DemandIndex::new(5).unwrap();
        for v in di.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn rising_series_yields_positive_signal() {
        // Strictly rising closes on constant volume -> pressure is positive every
        // bar -> smoothed DI must end up strictly positive.
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let f = i as f64;
                c(100.0 + f, 101.0 + f, 99.0 + f, 100.5 + f, 100.0, i)
            })
            .collect();
        let mut di = DemandIndex::new(5).unwrap();
        let out = di.batch(&candles);
        let last = out.iter().filter_map(|x| *x).next_back().unwrap();
        assert!(
            last > 0.0,
            "rising series must yield positive DI, got {last}"
        );
    }

    #[test]
    fn falling_series_yields_negative_signal() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let f = i as f64;
                c(200.0 - f, 201.0 - f, 199.0 - f, 199.5 - f, 100.0, i)
            })
            .collect();
        let mut di = DemandIndex::new(5).unwrap();
        let out = di.batch(&candles);
        let last = out.iter().filter_map(|x| *x).next_back().unwrap();
        assert!(
            last < 0.0,
            "falling series must yield negative DI, got {last}"
        );
    }

    #[test]
    fn zero_prev_close_contributes_no_signal() {
        // First two bars: prev close is exactly zero -> pressure clipped to 0.
        // We then continue with a non-zero series and confirm output behaves.
        let mut di = DemandIndex::new(3).unwrap();
        di.update(c(0.0, 0.0, 0.0, 0.0, 100.0, 0));
        // Bar 2 sees prev_close == 0 -> pressure = 0.
        di.update(c(0.0, 1.0, 0.0, 1.0, 100.0, 1));
        // Subsequent bars now have non-zero prev_close.
        di.update(c(1.0, 2.0, 1.0, 2.0, 100.0, 2));
        // Just check that nothing exploded; an EMA(3) needs 3 samples post-seed.
        // The first sample at bar 2 was zero, the second at bar 3 positive.
        let v = di.update(c(2.0, 3.0, 2.0, 3.0, 100.0, 3));
        assert!(v.is_some());
        assert!(v.unwrap().is_finite());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..100i64)
            .map(|i| {
                let f = i as f64;
                let mid = 100.0 + (f * 0.2).sin() * 5.0;
                c(
                    mid,
                    mid + 1.5,
                    mid - 1.5,
                    mid + 0.3,
                    80.0 + (i % 5) as f64,
                    i,
                )
            })
            .collect();
        let mut a = DemandIndex::new(10).unwrap();
        let mut b = DemandIndex::new(10).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let f = i as f64;
                c(100.0 + f, 101.0 + f, 99.0 + f, 100.5 + f, 100.0, i)
            })
            .collect();
        let mut di = DemandIndex::new(5).unwrap();
        di.batch(&candles);
        assert!(di.is_ready());
        di.reset();
        assert!(!di.is_ready());
        assert_eq!(di.update(candles[0]), None);
    }
}
