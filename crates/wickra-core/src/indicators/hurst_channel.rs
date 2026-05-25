//! Hurst Channel (Brian Millard / Hurst-cycle channel).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::indicators::sma::Sma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Hurst Channel output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HurstChannelOutput {
    /// Upper channel: `middle + multiplier · (highest_high − lowest_low)`.
    pub upper: f64,
    /// Middle line: SMA of close over the period.
    pub middle: f64,
    /// Lower channel: `middle − multiplier · (highest_high − lowest_low)`.
    pub lower: f64,
}

/// Hurst Channel: an SMA centerline wrapped by a rolling high-low range.
///
/// ```text
/// middle = SMA(close, period)
/// range  = max(high, period) − min(low, period)
/// upper  = middle + multiplier · range
/// lower  = middle − multiplier · range
/// ```
///
/// The Hurst Channel sizes its envelope by the *realised* high-low range of
/// the window — a simpler, range-based volatility proxy than Bollinger's
/// rolling stddev or Keltner's ATR. With a `multiplier` of `0.5` the channel
/// reduces to a centerline that hugs the midpoint of the Donchian envelope;
/// chart vendors that follow Hurst's cycle work commonly use `period = 10` and
/// `multiplier = 0.5` for the "inner" channel.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, HurstChannel, Indicator};
///
/// let mut indicator = HurstChannel::new(10, 0.5).unwrap();
/// let mut last = None;
/// for i in 0..30 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct HurstChannel {
    period: usize,
    multiplier: f64,
    sma: Sma,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
}

impl HurstChannel {
    /// # Errors
    /// Returns [`Error::PeriodZero`] / [`Error::NonPositiveMultiplier`] on
    /// invalid inputs.
    pub fn new(period: usize, multiplier: f64) -> Result<Self> {
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            period,
            multiplier,
            sma: Sma::new(period)?,
            highs: VecDeque::with_capacity(period),
            lows: VecDeque::with_capacity(period),
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Configured range multiplier.
    pub const fn multiplier(&self) -> f64 {
        self.multiplier
    }
}

impl Indicator for HurstChannel {
    type Input = Candle;
    type Output = HurstChannelOutput;

    fn update(&mut self, candle: Candle) -> Option<HurstChannelOutput> {
        if self.highs.len() == self.period {
            self.highs.pop_front();
            self.lows.pop_front();
        }
        self.highs.push_back(candle.high);
        self.lows.push_back(candle.low);

        let middle = self.sma.update(candle.close)?;
        let hi = self.highs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let lo = self.lows.iter().copied().fold(f64::INFINITY, f64::min);
        let range = hi - lo;
        Some(HurstChannelOutput {
            upper: middle + self.multiplier * range,
            middle,
            lower: middle - self.multiplier * range,
        })
    }

    fn reset(&mut self) {
        self.sma.reset();
        self.highs.clear();
        self.lows.clear();
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.sma.is_ready()
    }

    fn name(&self) -> &'static str {
        "HurstChannel"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(h: f64, l: f64, cl: f64) -> Candle {
        Candle::new(cl, h, l, cl, 1.0, 0).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(HurstChannel::new(0, 0.5), Err(Error::PeriodZero)));
    }

    #[test]
    fn rejects_non_positive_multiplier() {
        assert!(matches!(
            HurstChannel::new(10, 0.0),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            HurstChannel::new(10, -0.5),
            Err(Error::NonPositiveMultiplier)
        ));
        assert!(matches!(
            HurstChannel::new(10, f64::NAN),
            Err(Error::NonPositiveMultiplier)
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let h = HurstChannel::new(10, 0.5).unwrap();
        assert_eq!(h.period(), 10);
        assert_relative_eq!(h.multiplier(), 0.5, epsilon = 1e-12);
        assert_eq!(h.warmup_period(), 10);
        assert_eq!(h.name(), "HurstChannel");
    }

    #[test]
    fn flat_market_collapses_bands() {
        let candles: Vec<Candle> = (0..20).map(|_| c(10.0, 10.0, 10.0)).collect();
        let mut h = HurstChannel::new(5, 0.5).unwrap();
        let last = h.batch(&candles).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last.upper, 10.0, epsilon = 1e-9);
        assert_relative_eq!(last.middle, 10.0, epsilon = 1e-9);
        assert_relative_eq!(last.lower, 10.0, epsilon = 1e-9);
    }

    #[test]
    fn upper_above_middle_above_lower() {
        let candles: Vec<Candle> = (0..50)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.2).sin() * 5.0;
                c(m + 1.0, m - 1.0, m)
            })
            .collect();
        let mut h = HurstChannel::new(10, 0.5).unwrap();
        for o in h.batch(&candles).into_iter().flatten() {
            assert!(o.upper >= o.middle);
            assert!(o.middle >= o.lower);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| c(f64::from(i) + 2.0, f64::from(i), f64::from(i) + 1.0))
            .collect();
        let mut a = HurstChannel::new(10, 0.5).unwrap();
        let mut b = HurstChannel::new(10, 0.5).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..10)
            .map(|i| c(f64::from(i) + 1.0, f64::from(i) - 1.0, f64::from(i)))
            .collect();
        let mut h = HurstChannel::new(5, 0.5).unwrap();
        h.batch(&candles);
        assert!(h.is_ready());
        h.reset();
        assert!(!h.is_ready());
        assert_eq!(h.update(candles[0]), None);
    }

    /// Reference: five identical candles `(high=12, low=8, close=10)`:
    /// SMA(close, 5) = 10, range = 12 − 8 = 4, multiplier = 0.5
    /// upper = 10 + 0.5·4 = 12, lower = 10 − 0.5·4 = 8.
    #[test]
    fn reference_values() {
        let candles: Vec<Candle> = (0..5).map(|_| c(12.0, 8.0, 10.0)).collect();
        let mut h = HurstChannel::new(5, 0.5).unwrap();
        let out = h.batch(&candles);
        assert!(out[0].is_none() && out[3].is_none());
        let v = out[4].unwrap();
        assert_relative_eq!(v.middle, 10.0, epsilon = 1e-9);
        assert_relative_eq!(v.upper, 12.0, epsilon = 1e-9);
        assert_relative_eq!(v.lower, 8.0, epsilon = 1e-9);
    }
}
