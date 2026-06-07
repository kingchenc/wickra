//! Better Volume (VSA) — a streaming effort-versus-result oscillator.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Better Volume — a Volume-Spread-Analysis (VSA) "effort versus result"
/// oscillator: how much volume (effort) a bar spent relative to the price range
/// (result) it achieved, both normalised against their own recent averages.
///
/// ```text
/// range_t   = high_t − low_t
/// rel_vol   = volume_t / SMA(volume, period)
/// rel_range = range_t  / SMA(range,  period)
/// BetterVol = rel_vol − rel_range
/// ```
///
/// Volume-Spread Analysis (Wyckoff, popularised by Tom Williams) reads markets
/// through the relationship between **effort** (volume) and **result** (the bar's
/// spread). A bar with heavy volume but a narrow range — `rel_vol` high while
/// `rel_range` low, so the oscillator is **positive** — is *churn*: large effort
/// produced little movement, the hallmark of absorption (supply meeting demand at
/// a top, or vice versa at a bottom). A bar that travels far on light volume —
/// negative oscillator — shows *ease of movement*, a trend meeting no resistance.
///
/// Both legs are normalised by their `period` simple moving averages (including
/// the current bar), so the output is centred near `0` and self-scales to the
/// instrument. A degenerate average of `0` makes its leg `0` rather than dividing
/// by zero. The first value lands after `period` inputs. Each `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, BetterVolume};
///
/// let mut indicator = BetterVolume::new(20).unwrap();
/// let mut last = None;
/// for i in 0..60 {
///     let base = 100.0 + f64::from(i);
///     let c = Candle::new(base, base + 2.0, base - 2.0, base + 0.5, 1_000.0, 0).unwrap();
///     last = indicator.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct BetterVolume {
    period: usize,
    volumes: VecDeque<f64>,
    ranges: VecDeque<f64>,
    vol_sum: f64,
    range_sum: f64,
    last: Option<f64>,
}

impl BetterVolume {
    /// Construct a new Better Volume oscillator with the given averaging `period`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            volumes: VecDeque::with_capacity(period),
            ranges: VecDeque::with_capacity(period),
            vol_sum: 0.0,
            range_sum: 0.0,
            last: None,
        })
    }

    /// Configured averaging period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for BetterVolume {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let range = candle.high - candle.low;
        if self.volumes.len() == self.period {
            self.vol_sum -= self.volumes.pop_front().expect("non-empty");
            self.range_sum -= self.ranges.pop_front().expect("non-empty");
        }
        self.volumes.push_back(candle.volume);
        self.ranges.push_back(range);
        self.vol_sum += candle.volume;
        self.range_sum += range;
        if self.volumes.len() < self.period {
            return None;
        }
        let n = self.period as f64;
        let sma_vol = self.vol_sum / n;
        let sma_range = self.range_sum / n;
        let rel_vol = if sma_vol > 0.0 {
            candle.volume / sma_vol
        } else {
            0.0
        };
        let rel_range = if sma_range > 0.0 {
            range / sma_range
        } else {
            0.0
        };
        let out = rel_vol - rel_range;
        self.last = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.volumes.clear();
        self.ranges.clear();
        self.vol_sum = 0.0;
        self.range_sum = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "BetterVolume"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(high: f64, low: f64, volume: f64) -> Candle {
        Candle::new_unchecked(low, high, low, high, volume, 0)
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(BetterVolume::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let bv = BetterVolume::new(20).unwrap();
        assert_eq!(bv.period(), 20);
        assert_eq!(bv.warmup_period(), 20);
        assert_eq!(bv.name(), "BetterVolume");
        assert!(!bv.is_ready());
        assert_eq!(bv.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut bv = BetterVolume::new(3).unwrap();
        let candles: Vec<Candle> = (0..6).map(|_| candle(102.0, 100.0, 1_000.0)).collect();
        let out = bv.batch(&candles);
        for v in out.iter().take(2) {
            assert!(v.is_none());
        }
        assert!(out[2].is_some());
    }

    #[test]
    fn steady_bars_are_neutral() {
        // Identical volume and range every bar -> rel_vol = rel_range = 1 -> 0.
        let mut bv = BetterVolume::new(4).unwrap();
        let candles: Vec<Candle> = (0..10).map(|_| candle(102.0, 100.0, 1_000.0)).collect();
        let last = bv.batch(&candles).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn churn_bar_is_positive() {
        // Three normal bars, then a high-volume narrow-range bar -> positive.
        let mut bv = BetterVolume::new(4).unwrap();
        let mut candles: Vec<Candle> = (0..3).map(|_| candle(105.0, 100.0, 1_000.0)).collect();
        candles.push(candle(100.5, 100.0, 5_000.0)); // huge volume, tiny range
        let last = bv.batch(&candles).into_iter().flatten().last().unwrap();
        assert!(last > 0.0, "churn bar should be positive, got {last}");
    }

    #[test]
    fn ease_of_movement_bar_is_negative() {
        // Three normal bars, then a wide-range light-volume bar -> negative.
        let mut bv = BetterVolume::new(4).unwrap();
        let mut candles: Vec<Candle> = (0..3).map(|_| candle(101.0, 100.0, 5_000.0)).collect();
        candles.push(candle(115.0, 100.0, 500.0)); // wide range, tiny volume
        let last = bv.batch(&candles).into_iter().flatten().last().unwrap();
        assert!(
            last < 0.0,
            "ease-of-movement bar should be negative, got {last}"
        );
    }

    #[test]
    fn zero_everything_is_zero() {
        // Zero volume and zero range -> both legs guarded to 0.
        let mut bv = BetterVolume::new(3).unwrap();
        let candles: Vec<Candle> = (0..6).map(|_| candle(100.0, 100.0, 0.0)).collect();
        for v in bv.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut bv = BetterVolume::new(3).unwrap();
        bv.batch(
            &(0..6)
                .map(|_| candle(102.0, 100.0, 1_000.0))
                .collect::<Vec<_>>(),
        );
        assert!(bv.is_ready());
        bv.reset();
        assert!(!bv.is_ready());
        assert_eq!(bv.value(), None);
        assert_eq!(bv.update(candle(102.0, 100.0, 1_000.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..120)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.25).sin() * 9.0;
                candle(
                    base + 2.0,
                    base - 1.5,
                    1_000.0 + (f64::from(i) * 0.5).cos() * 400.0,
                )
            })
            .collect();
        let batch = BetterVolume::new(20).unwrap().batch(&candles);
        let mut b = BetterVolume::new(20).unwrap();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }
}
