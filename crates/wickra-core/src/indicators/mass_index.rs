//! Mass Index.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

use super::Ema;

/// Mass Index — Donald Dorsey's range-expansion indicator.
///
/// The Mass Index watches the high–low range, not direction. It smooths the
/// range with an EMA, smooths that again, takes the ratio of the two, and sums
/// the ratio over a window:
///
/// ```text
/// range_t  = high_t − low_t
/// ratio_t  = EMA(range, ema_period) / EMA(EMA(range, ema_period), ema_period)
/// MassIndex = Σ ratio over sum_period
/// ```
///
/// When the range widens, the single EMA pulls ahead of the double EMA, the
/// ratio rises above `1`, and the sum climbs. Dorsey's "reversal bulge" is the
/// Mass Index rising above `27` and then falling back below `26.5` — a sign
/// that a range expansion is about to resolve into a trend reversal. With the
/// conventional `(ema_period = 9, sum_period = 25)` a flat-range market sits at
/// `25`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, MassIndex};
///
/// let mut indicator = MassIndex::new(9, 25).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + i as f64;
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct MassIndex {
    ema_period: usize,
    sum_period: usize,
    ema1: Ema,
    ema2: Ema,
    /// Rolling window of the last `sum_period` EMA ratios.
    window: VecDeque<f64>,
    sum: f64,
    last: Option<f64>,
}

impl MassIndex {
    /// Construct a new Mass Index with the EMA smoothing period and the sum
    /// window length.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if either period is `0`.
    pub fn new(ema_period: usize, sum_period: usize) -> Result<Self> {
        if ema_period == 0 || sum_period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            ema_period,
            sum_period,
            ema1: Ema::new(ema_period)?,
            ema2: Ema::new(ema_period)?,
            window: VecDeque::with_capacity(sum_period),
            sum: 0.0,
            last: None,
        })
    }

    /// The `(ema_period, sum_period)` pair.
    pub const fn periods(&self) -> (usize, usize) {
        (self.ema_period, self.sum_period)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for MassIndex {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let range = candle.high - candle.low;
        let single = self.ema1.update(range)?;
        let double = self.ema2.update(single)?;
        let ratio = if double == 0.0 {
            // A zero-range market: no expansion, neutral ratio.
            1.0
        } else {
            single / double
        };
        if self.window.len() == self.sum_period {
            self.sum -= self.window.pop_front().expect("window is non-empty");
        }
        self.window.push_back(ratio);
        self.sum += ratio;
        if self.window.len() < self.sum_period {
            return None;
        }
        self.last = Some(self.sum);
        Some(self.sum)
    }

    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
        self.window.clear();
        self.sum = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // ema1 seeds at `ema_period`, ema2 at `2·ema_period − 1`, then the sum
        // window needs `sum_period` ratios.
        2 * self.ema_period + self.sum_period - 2
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "MassIndex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    /// A candle with a fixed high–low range `span` centred on `mid`.
    fn candle(mid: f64, span: f64, ts: i64) -> Candle {
        Candle::new(mid, mid + span / 2.0, mid - span / 2.0, mid, 1.0, ts).unwrap()
    }

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(MassIndex::new(0, 25), Err(Error::PeriodZero)));
        assert!(matches!(MassIndex::new(9, 0), Err(Error::PeriodZero)));
    }

    /// Cover the const accessors `periods` / `value` (80-87) and the
    /// Indicator-impl `name` body (134-136). `warmup_period` is already
    /// covered by `warmup_period_formula`.
    #[test]
    fn accessors_and_metadata() {
        let mut mi = MassIndex::new(9, 25).unwrap();
        assert_eq!(mi.periods(), (9, 25));
        assert_eq!(mi.name(), "MassIndex");
        assert_eq!(mi.value(), None);
        for i in 0..mi.warmup_period() {
            mi.update(candle(100.0, 2.0, i64::try_from(i).unwrap()));
        }
        assert!(mi.value().is_some());
    }

    #[test]
    fn warmup_period_formula() {
        let mi = MassIndex::new(9, 25).unwrap();
        assert_eq!(mi.warmup_period(), 2 * 9 + 25 - 2);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut mi = MassIndex::new(3, 4).unwrap();
        let warmup = mi.warmup_period(); // 2*3 + 4 - 2 = 8
        assert_eq!(warmup, 8);
        let candles: Vec<Candle> = (0..20).map(|i| candle(100.0 + i as f64, 2.0, i)).collect();
        let out = mi.batch(&candles);
        for v in out.iter().take(warmup - 1) {
            assert!(v.is_none());
        }
        assert!(out[warmup - 1].is_some());
    }

    #[test]
    fn constant_range_sums_to_sum_period() {
        // A constant high–low range makes both EMAs converge to the same
        // value, so every ratio is 1 and the Mass Index equals `sum_period`.
        let mut mi = MassIndex::new(3, 4).unwrap();
        let candles: Vec<Candle> = (0..40).map(|i| candle(100.0 + i as f64, 2.0, i)).collect();
        for v in mi.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 4.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn zero_range_market_sums_to_sum_period() {
        let mut mi = MassIndex::new(3, 4).unwrap();
        let candles: Vec<Candle> = (0..40).map(|i| candle(100.0, 0.0, i)).collect();
        for v in mi.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 4.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut mi = MassIndex::new(3, 4).unwrap();
        let candles: Vec<Candle> = (0..20).map(|i| candle(100.0 + i as f64, 2.0, i)).collect();
        mi.batch(&candles);
        assert!(mi.is_ready());
        mi.reset();
        assert!(!mi.is_ready());
        assert_eq!(mi.update(candles[0]), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..120)
            .map(|i| {
                let span = 2.0 + (i as f64 * 0.3).sin().abs() * 3.0;
                candle(100.0 + (i as f64 * 0.2).cos() * 5.0, span, i)
            })
            .collect();
        let batch = MassIndex::new(9, 25).unwrap().batch(&candles);
        let mut b = MassIndex::new(9, 25).unwrap();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }
}
