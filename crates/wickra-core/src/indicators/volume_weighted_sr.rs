//! Volume-Weighted Support/Resistance — a volume-weighted high/low band.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Output of [`VolumeWeightedSr`]: the volume-weighted support and resistance
/// levels over the lookback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumeWeightedSrOutput {
    /// Volume-weighted average low — the support level.
    pub support: f64,
    /// Volume-weighted average high — the resistance level.
    pub resistance: f64,
}

/// Volume-Weighted Support/Resistance — a band whose edges are the
/// **volume-weighted** average of the recent highs (resistance) and lows
/// (support), so the levels gravitate toward the prices where trading actually
/// happened.
///
/// ```text
/// support    = Σ(low_i  · volume_i) / Σ volume_i      over the window
/// resistance = Σ(high_i · volume_i) / Σ volume_i      over the window
/// ```
///
/// Plain high/low channels (e.g. [`Donchian`](crate::Donchian)) weight every bar
/// equally, so a thin spike sets the boundary. Volume-weighting pulls the support
/// and resistance toward the highs and lows that carried real volume — the prices
/// the market agreed mattered — giving levels that tend to hold better. The
/// distance between the two is a volume-aware range estimate. If the window's
/// volume is all zero the band falls back to the equal-weighted average high and
/// low.
///
/// The first value lands after `period` inputs; each `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, VolumeWeightedSr};
///
/// let mut indicator = VolumeWeightedSr::new(20).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let base = 100.0 + (f64::from(i) * 0.3).sin() * 5.0;
///     let c = Candle::new(base, base + 2.0, base - 2.0, base, 1_000.0 + f64::from(i), 0).unwrap();
///     last = indicator.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct VolumeWeightedSr {
    period: usize,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    volumes: VecDeque<f64>,
    sum_hv: f64,
    sum_lv: f64,
    sum_v: f64,
    sum_h: f64,
    sum_l: f64,
    last: Option<VolumeWeightedSrOutput>,
}

impl VolumeWeightedSr {
    /// Construct a volume-weighted S/R band over `period` bars.
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
            highs: VecDeque::with_capacity(period),
            lows: VecDeque::with_capacity(period),
            volumes: VecDeque::with_capacity(period),
            sum_hv: 0.0,
            sum_lv: 0.0,
            sum_v: 0.0,
            sum_h: 0.0,
            sum_l: 0.0,
            last: None,
        })
    }

    /// Configured lookback period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<VolumeWeightedSrOutput> {
        self.last
    }
}

impl Indicator for VolumeWeightedSr {
    type Input = Candle;
    type Output = VolumeWeightedSrOutput;

    fn update(&mut self, candle: Candle) -> Option<VolumeWeightedSrOutput> {
        if self.highs.len() == self.period {
            let h = self.highs.pop_front().expect("non-empty");
            let l = self.lows.pop_front().expect("non-empty");
            let v = self.volumes.pop_front().expect("non-empty");
            self.sum_hv -= h * v;
            self.sum_lv -= l * v;
            self.sum_v -= v;
            self.sum_h -= h;
            self.sum_l -= l;
        }
        self.highs.push_back(candle.high);
        self.lows.push_back(candle.low);
        self.volumes.push_back(candle.volume);
        self.sum_hv += candle.high * candle.volume;
        self.sum_lv += candle.low * candle.volume;
        self.sum_v += candle.volume;
        self.sum_h += candle.high;
        self.sum_l += candle.low;
        if self.highs.len() < self.period {
            return None;
        }
        let n = self.period as f64;
        let (support, resistance) = if self.sum_v > 0.0 {
            (self.sum_lv / self.sum_v, self.sum_hv / self.sum_v)
        } else {
            (self.sum_l / n, self.sum_h / n)
        };
        let out = VolumeWeightedSrOutput {
            support,
            resistance,
        };
        self.last = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.highs.clear();
        self.lows.clear();
        self.volumes.clear();
        self.sum_hv = 0.0;
        self.sum_lv = 0.0;
        self.sum_v = 0.0;
        self.sum_h = 0.0;
        self.sum_l = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "VolumeWeightedSr"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(high: f64, low: f64, volume: f64) -> Candle {
        Candle::new_unchecked(low, high, low, f64::midpoint(high, low), volume, 0)
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(VolumeWeightedSr::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let v = VolumeWeightedSr::new(20).unwrap();
        assert_eq!(v.period(), 20);
        assert_eq!(v.warmup_period(), 20);
        assert_eq!(v.name(), "VolumeWeightedSr");
        assert!(!v.is_ready());
        assert_eq!(v.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut v = VolumeWeightedSr::new(4).unwrap();
        let candles: Vec<Candle> = (0..6).map(|_| c(102.0, 98.0, 1_000.0)).collect();
        let out = v.batch(&candles);
        for o in out.iter().take(3) {
            assert!(o.is_none());
        }
        assert!(out[3].is_some());
    }

    #[test]
    fn support_below_resistance() {
        let mut v = VolumeWeightedSr::new(10).unwrap();
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                c(
                    110.0 + (f64::from(i) * 0.3).sin() * 5.0,
                    90.0 + (f64::from(i) * 0.3).cos() * 5.0,
                    1_000.0 + f64::from(i),
                )
            })
            .collect();
        for o in v.batch(&candles).into_iter().flatten() {
            assert!(o.support <= o.resistance);
        }
    }

    #[test]
    fn weights_toward_high_volume_bars() {
        // Three low-volume bars at [98,102] and one heavy bar at [108,112]; the
        // resistance should be pulled toward the heavy bar's high.
        let mut v = VolumeWeightedSr::new(4).unwrap();
        let candles = [
            c(102.0, 98.0, 100.0),
            c(102.0, 98.0, 100.0),
            c(102.0, 98.0, 100.0),
            c(112.0, 108.0, 9_000.0),
        ];
        let out = v.batch(&candles).into_iter().flatten().last().unwrap();
        // Volume-weighted resistance sits much closer to 112 than the simple mean (104.5).
        assert!(
            out.resistance > 108.0,
            "resistance {} should lean to the heavy bar",
            out.resistance
        );
    }

    #[test]
    fn zero_volume_falls_back_to_equal_weight() {
        let mut v = VolumeWeightedSr::new(3).unwrap();
        let candles = [
            c(102.0, 98.0, 0.0),
            c(104.0, 96.0, 0.0),
            c(106.0, 94.0, 0.0),
        ];
        let out = v.batch(&candles).into_iter().flatten().last().unwrap();
        // Equal-weight averages: high mean = 104, low mean = 96.
        assert_relative_eq!(out.resistance, 104.0, epsilon = 1e-9);
        assert_relative_eq!(out.support, 96.0, epsilon = 1e-9);
    }

    #[test]
    fn reset_clears_state() {
        let mut v = VolumeWeightedSr::new(4).unwrap();
        v.batch(&(0..6).map(|_| c(102.0, 98.0, 1_000.0)).collect::<Vec<_>>());
        assert!(v.is_ready());
        v.reset();
        assert!(!v.is_ready());
        assert_eq!(v.value(), None);
        assert_eq!(v.update(c(102.0, 98.0, 1_000.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..120)
            .map(|i| {
                c(
                    110.0 + (f64::from(i) * 0.25).sin() * 9.0,
                    90.0 + (f64::from(i) * 0.25).cos() * 9.0,
                    1_000.0 + f64::from(i),
                )
            })
            .collect();
        let batch = VolumeWeightedSr::new(20).unwrap().batch(&candles);
        let mut b = VolumeWeightedSr::new(20).unwrap();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
