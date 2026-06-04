//! TTM Trend — John Carter's bar-coloring trend filter.

use crate::error::Result;
use crate::indicators::sma::Sma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// TTM Trend: compares the current close to the simple moving average of the
/// recent median prices `(high + low) / 2`. A close above that reference colors
/// the bar as an uptrend (`+1.0`); a close at or below it as a downtrend
/// (`-1.0`).
///
/// ```text
/// reference = SMA((high + low) / 2, period)
/// TTM Trend = +1  if close > reference
///             -1  otherwise
/// ```
///
/// The classic TTM Trend uses the trailing six bars. The signal is a regime
/// label rather than a level: it stays `None` during warmup and then emits
/// `±1.0` on every bar.
///
/// Reference: John Carter, *Mastering the Trade*, 2005.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, TtmTrend};
///
/// let mut indicator = TtmTrend::new(6).unwrap();
/// let mut last = None;
/// for i in 0..20 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 1.0, base - 1.0, base + 0.5, 1.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert_eq!(last, Some(1.0));
/// ```
#[derive(Debug, Clone)]
pub struct TtmTrend {
    period: usize,
    sma: Sma,
}

impl TtmTrend {
    /// Construct a TTM Trend over the given lookback.
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

    /// Configured lookback period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for TtmTrend {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let median = f64::midpoint(candle.high, candle.low);
        let reference = self.sma.update(median)?;
        Some(if candle.close > reference { 1.0 } else { -1.0 })
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
        "TtmTrend"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::traits::BatchExt;

    fn candle(high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(f64::midpoint(high, low), high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(TtmTrend::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let t = TtmTrend::new(6).unwrap();
        assert_eq!(t.period(), 6);
        assert_eq!(t.warmup_period(), 6);
        assert_eq!(t.name(), "TtmTrend");
        assert!(!t.is_ready());
    }

    #[test]
    fn warmup_then_emits() {
        let mut t = TtmTrend::new(3).unwrap();
        let candles: Vec<Candle> = (0..3).map(|i| candle(13.0, 9.0, 12.0, i)).collect();
        let out = t.batch(&candles);
        assert!(out[0].is_none());
        assert!(out[1].is_none());
        assert!(out[2].is_some());
    }

    #[test]
    fn close_above_reference_is_uptrend() {
        // Close (12) sits above the median reference (13 + 9) / 2 = 11 -> +1.
        let mut t = TtmTrend::new(3).unwrap();
        let candles: Vec<Candle> = (0..6).map(|i| candle(13.0, 9.0, 12.0, i)).collect();
        assert_eq!(t.batch(&candles).last().unwrap().unwrap(), 1.0);
    }

    #[test]
    fn close_at_or_below_reference_is_downtrend() {
        // Constant median 10, close equal to the reference -> not strictly above -> -1.
        let mut t = TtmTrend::new(3).unwrap();
        let candles: Vec<Candle> = (0..6).map(|i| candle(11.0, 9.0, 10.0, i)).collect();
        assert_eq!(t.batch(&candles).last().unwrap().unwrap(), -1.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut t = TtmTrend::new(3).unwrap();
        let candles: Vec<Candle> = (0..6).map(|i| candle(13.0, 9.0, 12.0, i)).collect();
        t.batch(&candles);
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40_i64)
            .map(|i| {
                let base = 100.0 + (i as f64 * 0.25).sin() * 4.0;
                candle(base + 1.0, base - 1.0, base + (i as f64 * 0.5).cos(), i)
            })
            .collect();
        let mut a = TtmTrend::new(6).unwrap();
        let mut b = TtmTrend::new(6).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|c| b.update(*c)).collect::<Vec<_>>()
        );
    }
}
