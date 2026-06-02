//! Volume Profile — the full per-bin volume distribution over a rolling window.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Volume Profile output: the price domain plus the per-bin volume histogram.
///
/// `bins[i]` holds the volume attributed to the price bucket
/// `[price_low + i * w, price_low + (i + 1) * w)` where
/// `w = (price_high - price_low) / bins.len()`. The histogram sums to the total
/// volume in the rolling window (within floating-point tolerance).
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeProfileOutput {
    /// Lowest price in the window — the lower edge of bin 0.
    pub price_low: f64,
    /// Highest price in the window — the upper edge of the last bin.
    pub price_high: f64,
    /// Per-bin volume, lowest price bucket first. Length equals `bin_count`.
    pub bins: Vec<f64>,
}

/// Rolling Volume Profile over the last `period` candles.
///
/// Where [`crate::ValueArea`] reduces the same volume distribution to its
/// summary levels (Point of Control, Value Area High / Low), Volume Profile
/// exposes the **full histogram** so callers can inspect, render or post-process
/// the raw distribution. Each candle's volume is spread uniformly across the
/// bins its `[low, high]` range touches; a single-print bar (`low == high`)
/// drops its whole volume into one bin. The histogram domain spans the window's
/// lowest low to its highest high.
///
/// A window whose bars are all single-print at one price (`price_high == price_low`)
/// is degenerate: the entire volume lands in bin 0 and both edges collapse to
/// that price.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, VolumeProfile};
///
/// let mut vp = VolumeProfile::new(5, 10).unwrap();
/// let mut last = None;
/// for i in 0..10 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base, 10.0, i64::from(i)).unwrap();
///     last = vp.update(candle);
/// }
/// let profile = last.unwrap();
/// assert_eq!(profile.bins.len(), 10);
/// ```
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone)]
pub struct VolumeProfile {
    period: usize,
    bin_count: usize,
    window: VecDeque<Candle>,
    last: Option<VolumeProfileOutput>,
}

impl VolumeProfile {
    /// Construct a Volume Profile indicator.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period` or `bin_count` is zero.
    pub fn new(period: usize, bin_count: usize) -> Result<Self> {
        if period == 0 || bin_count == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            bin_count,
            window: VecDeque::with_capacity(period),
            last: None,
        })
    }

    /// Classic Volume Profile: 20-bar rolling window, 50 bins.
    pub fn classic() -> Self {
        Self::new(20, 50).expect("classic VolumeProfile params are valid")
    }

    /// Configured `(period, bin_count)`.
    pub const fn params(&self) -> (usize, usize) {
        (self.period, self.bin_count)
    }

    /// Most recent profile if available.
    pub fn value(&self) -> Option<&VolumeProfileOutput> {
        self.last.as_ref()
    }

    fn price_to_bin(&self, price: f64, win_low: f64, bin_width: f64) -> usize {
        let raw = ((price - win_low) / bin_width).floor();
        let max = (self.bin_count - 1) as f64;
        raw.clamp(0.0, max) as usize
    }

    fn compute(&self) -> VolumeProfileOutput {
        let mut win_low = f64::INFINITY;
        let mut win_high = f64::NEG_INFINITY;
        for candle in &self.window {
            if candle.low < win_low {
                win_low = candle.low;
            }
            if candle.high > win_high {
                win_high = candle.high;
            }
        }
        let span = win_high - win_low;
        let mut bins = vec![0.0_f64; self.bin_count];

        if span <= 0.0 {
            // All bars are single-print at the same price.
            let total: f64 = self.window.iter().map(|candle| candle.volume).sum();
            bins[0] = total;
            return VolumeProfileOutput {
                price_low: win_low,
                price_high: win_low,
                bins,
            };
        }

        let bin_width = span / self.bin_count as f64;
        for candle in &self.window {
            if candle.volume == 0.0 {
                continue;
            }
            if candle.high <= candle.low {
                let idx = self.price_to_bin(candle.low, win_low, bin_width);
                bins[idx] += candle.volume;
                continue;
            }
            let lo_idx = self.price_to_bin(candle.low, win_low, bin_width);
            let hi_idx = self.price_to_bin(candle.high, win_low, bin_width);
            let touched = hi_idx - lo_idx + 1;
            let share = candle.volume / touched as f64;
            for bin in bins.iter_mut().take(hi_idx + 1).skip(lo_idx) {
                *bin += share;
            }
        }

        VolumeProfileOutput {
            price_low: win_low,
            price_high: win_high,
            bins,
        }
    }
}

impl Indicator for VolumeProfile {
    type Input = Candle;
    type Output = VolumeProfileOutput;

    fn update(&mut self, candle: Candle) -> Option<VolumeProfileOutput> {
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(candle);
        if self.window.len() < self.period {
            return None;
        }
        let out = self.compute();
        self.last = Some(out.clone());
        Some(out)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "VolumeProfile"
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
        assert!(matches!(VolumeProfile::new(0, 50), Err(Error::PeriodZero)));
    }

    #[test]
    fn rejects_zero_bin_count() {
        assert!(matches!(VolumeProfile::new(20, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let vp = VolumeProfile::new(20, 50).unwrap();
        assert_eq!(vp.name(), "VolumeProfile");
        assert_eq!(vp.warmup_period(), 20);
        assert_eq!(vp.params(), (20, 50));
        assert!(vp.value().is_none());
        assert!(!vp.is_ready());
    }

    #[test]
    fn classic_params() {
        let vp = VolumeProfile::classic();
        assert_eq!(vp.params(), (20, 50));
    }

    #[test]
    fn warms_up_over_period() {
        let mut vp = VolumeProfile::new(3, 4).unwrap();
        assert!(vp.update(c(10.0, 11.0, 9.0, 10.0, 5.0, 0)).is_none());
        assert!(vp.update(c(10.0, 11.0, 9.0, 10.0, 5.0, 1)).is_none());
        assert!(vp.update(c(10.0, 11.0, 9.0, 10.0, 5.0, 2)).is_some());
        assert!(vp.is_ready());
    }

    #[test]
    fn reference_distribution() {
        // Window of 2 candles, 4 bins.
        // bar0: single print at 10, vol 100 -> bin 0 gets 100.
        // bar1: 10..14, vol 80, spans 4 bins -> 20 each.
        // domain: low=10, high=14, width=1 -> bins = [120, 20, 20, 20].
        let mut vp = VolumeProfile::new(2, 4).unwrap();
        assert!(vp.update(c(10.0, 10.0, 10.0, 10.0, 100.0, 0)).is_none());
        let out = vp.update(c(10.0, 14.0, 10.0, 12.0, 80.0, 1)).unwrap();
        assert_relative_eq!(out.price_low, 10.0, epsilon = 1e-12);
        assert_relative_eq!(out.price_high, 14.0, epsilon = 1e-12);
        assert_eq!(out.bins.len(), 4);
        assert_relative_eq!(out.bins[0], 120.0, epsilon = 1e-9);
        assert_relative_eq!(out.bins[1], 20.0, epsilon = 1e-9);
        assert_relative_eq!(out.bins[2], 20.0, epsilon = 1e-9);
        assert_relative_eq!(out.bins[3], 20.0, epsilon = 1e-9);
    }

    #[test]
    fn conserves_total_volume() {
        let mut vp = VolumeProfile::new(4, 8).unwrap();
        let candles = [
            c(10.0, 12.0, 9.0, 11.0, 30.0, 0),
            c(11.0, 13.0, 10.0, 12.0, 40.0, 1),
            c(12.0, 14.0, 11.0, 13.0, 50.0, 2),
            c(13.0, 15.0, 12.0, 14.0, 60.0, 3),
        ];
        let out = vp.batch(&candles).pop().unwrap().unwrap();
        let total: f64 = out.bins.iter().sum();
        assert_relative_eq!(total, 180.0, epsilon = 1e-9);
    }

    #[test]
    fn degenerate_single_price_window() {
        // All bars single-print at 50 -> domain collapses, all volume in bin 0.
        let mut vp = VolumeProfile::new(2, 4).unwrap();
        vp.update(c(50.0, 50.0, 50.0, 50.0, 10.0, 0));
        let out = vp.update(c(50.0, 50.0, 50.0, 50.0, 20.0, 1)).unwrap();
        assert_relative_eq!(out.price_low, 50.0, epsilon = 1e-12);
        assert_relative_eq!(out.price_high, 50.0, epsilon = 1e-12);
        assert_relative_eq!(out.bins[0], 30.0, epsilon = 1e-9);
        assert_relative_eq!(out.bins[1], 0.0, epsilon = 1e-12);
    }

    #[test]
    fn zero_volume_bars_are_skipped() {
        let mut vp = VolumeProfile::new(2, 4).unwrap();
        vp.update(c(10.0, 14.0, 10.0, 12.0, 0.0, 0));
        let out = vp.update(c(10.0, 14.0, 10.0, 12.0, 40.0, 1)).unwrap();
        let total: f64 = out.bins.iter().sum();
        assert_relative_eq!(total, 40.0, epsilon = 1e-9);
    }

    #[test]
    fn rolling_window_drops_oldest() {
        let mut vp = VolumeProfile::new(2, 4).unwrap();
        vp.update(c(100.0, 100.0, 100.0, 100.0, 99.0, 0));
        vp.update(c(10.0, 14.0, 10.0, 12.0, 40.0, 1));
        // Third bar evicts the price-100 bar; domain is now 10..14 only.
        let out = vp.update(c(10.0, 14.0, 10.0, 12.0, 40.0, 2)).unwrap();
        assert_relative_eq!(out.price_high, 14.0, epsilon = 1e-12);
        let total: f64 = out.bins.iter().sum();
        assert_relative_eq!(total, 80.0, epsilon = 1e-9);
    }

    #[test]
    fn reset_clears_state() {
        let mut vp = VolumeProfile::new(2, 4).unwrap();
        vp.update(c(10.0, 11.0, 9.0, 10.0, 5.0, 0));
        vp.update(c(10.0, 11.0, 9.0, 10.0, 5.0, 1));
        assert!(vp.is_ready());
        vp.reset();
        assert!(!vp.is_ready());
        assert!(vp.value().is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                let base = 100.0 + f64::from(i % 7);
                c(
                    base,
                    base + 2.0,
                    base - 2.0,
                    base,
                    10.0 + f64::from(i),
                    i64::from(i),
                )
            })
            .collect();
        let mut a = VolumeProfile::new(10, 16).unwrap();
        let mut b = VolumeProfile::new(10, 16).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
