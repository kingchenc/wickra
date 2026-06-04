//! Qstick — Tushar Chande's measure of buying vs. selling pressure.

use crate::error::Result;
use crate::indicators::sma::Sma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Qstick: the simple moving average of the body `close - open` over `period`
/// bars.
///
/// Positive values indicate a run of bars that closed above their open (net
/// buying pressure); negative values indicate net selling pressure. A zero
/// crossing is read as a shift in short-term sentiment.
///
/// ```text
/// Qstick = SMA(close - open, period)
/// ```
///
/// Reference: Tushar Chande, *The New Technical Trader*, 1994.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Qstick};
///
/// let mut indicator = Qstick::new(5).unwrap();
/// let mut last = None;
/// for i in 0..20 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 1.0, base + 1.0, 1.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Qstick {
    period: usize,
    sma: Sma,
}

impl Qstick {
    /// Construct a Qstick with the given averaging period.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`](crate::error::Error::PeriodZero) if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            period,
            sma: Sma::new(period)?,
        })
    }

    /// Configured averaging period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for Qstick {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.sma.update(candle.close - candle.open)
    }

    fn reset(&mut self) {
        self.sma.reset();
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.sma.is_ready()
    }

    fn name(&self) -> &'static str {
        "Qstick"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(open: f64, close: f64, ts: i64) -> Candle {
        let high = open.max(close) + 1.0;
        let low = open.min(close) - 1.0;
        Candle::new(open, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(Qstick::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let q = Qstick::new(5).unwrap();
        assert_eq!(q.period(), 5);
        assert_eq!(q.warmup_period(), 5);
        assert_eq!(q.name(), "Qstick");
        assert!(!q.is_ready());
    }

    #[test]
    fn warmup_emits_first_value_at_period() {
        let mut q = Qstick::new(3).unwrap();
        let candles: Vec<Candle> = (0..3).map(|i| candle(10.0, 11.0, i)).collect();
        let out = q.batch(&candles);
        assert!(out[0].is_none());
        assert!(out[1].is_none());
        assert!(out[2].is_some());
    }

    #[test]
    fn constant_bodies_yield_the_body() {
        // Every bar closes 1.5 above its open -> Qstick converges to 1.5.
        let mut q = Qstick::new(4).unwrap();
        let candles: Vec<Candle> = (0..10).map(|i| candle(10.0, 11.5, i)).collect();
        let out = q.batch(&candles);
        assert_relative_eq!(out.last().unwrap().unwrap(), 1.5, epsilon = 1e-12);
    }

    #[test]
    fn selling_pressure_is_negative() {
        let mut q = Qstick::new(3).unwrap();
        let candles: Vec<Candle> = (0..6).map(|i| candle(11.0, 10.0, i)).collect();
        let last = q.batch(&candles).last().unwrap().unwrap();
        assert!(last < 0.0, "qstick {last} should be negative");
    }

    #[test]
    fn reset_clears_state() {
        let mut q = Qstick::new(3).unwrap();
        let candles: Vec<Candle> = (0..6).map(|i| candle(10.0, 11.0, i)).collect();
        q.batch(&candles);
        assert!(q.is_ready());
        q.reset();
        assert!(!q.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40_i64)
            .map(|i| {
                candle(
                    100.0 + (i as f64 * 0.3).sin(),
                    100.0 + (i as f64 * 0.4).cos(),
                    i,
                )
            })
            .collect();
        let mut a = Qstick::new(7).unwrap();
        let mut b = Qstick::new(7).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|c| b.update(*c)).collect::<Vec<_>>()
        );
    }
}
