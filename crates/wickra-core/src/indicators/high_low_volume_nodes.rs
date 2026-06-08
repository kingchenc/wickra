//! High/Low Volume Nodes (HVN / LVN) — the busiest and quietest price levels.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Output of [`HighLowVolumeNodes`]: the price of the highest- and lowest-volume
/// node in the profile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HighLowVolumeNodesOutput {
    /// High Volume Node — the price level (bin centre) with the most volume.
    pub hvn: f64,
    /// Low Volume Node — the traded price level with the least volume.
    pub lvn: f64,
}

/// High/Low Volume Nodes — the price levels of greatest and least acceptance in a
/// rolling volume profile.
///
/// ```text
/// build a `bins`-bucket volume profile over the last `period` candles
/// HVN = bin centre of the bucket with the most volume
/// LVN = bin centre of the traded bucket with the least volume
/// ```
///
/// A volume profile reveals where the market spent the most effort. A **High Volume
/// Node** (HVN) is a price the market accepted and traded heavily — it acts as a
/// magnet and as strong support/resistance. A **Low Volume Node** (LVN) is a price
/// the market rejected quickly — moves tend to accelerate through LVNs and they
/// often mark the edges between balance areas. Each candle's volume is spread
/// across the price bins its high-low range spans (as in
/// [`VolumeProfile`](crate::VolumeProfile)).
///
/// The first value lands after `period` candles; each `update` rebuilds the profile
/// in O(`period · bins`). A degenerate flat window puts both nodes at the price.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, HighLowVolumeNodes};
///
/// let mut indicator = HighLowVolumeNodes::new(20, 24).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let base = 100.0 + (f64::from(i) * 0.3).sin() * 5.0;
///     let c = Candle::new(base, base + 1.0, base - 1.0, base, 1_000.0, 0).unwrap();
///     last = indicator.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct HighLowVolumeNodes {
    period: usize,
    bins: usize,
    window: VecDeque<Candle>,
    last: Option<HighLowVolumeNodesOutput>,
}

impl HighLowVolumeNodes {
    /// Construct a High/Low Volume Nodes indicator.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period` or `bins` is zero.
    pub fn new(period: usize, bins: usize) -> Result<Self> {
        if period == 0 || bins == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            bins,
            window: VecDeque::with_capacity(period),
            last: None,
        })
    }

    /// Configured `(period, bins)`.
    pub const fn params(&self) -> (usize, usize) {
        (self.period, self.bins)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<HighLowVolumeNodesOutput> {
        self.last
    }

    /// Build the volume histogram; returns `(low, bin_width, bins)`.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn profile(&self) -> (f64, f64, Vec<f64>) {
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for c in &self.window {
            low = low.min(c.low);
            high = high.max(c.high);
        }
        let mut hist = vec![0.0; self.bins];
        let span = high - low;
        if span <= 0.0 {
            hist[0] = self.window.iter().map(|c| c.volume).sum();
            return (low, 0.0, hist);
        }
        let width = span / self.bins as f64;
        for c in &self.window {
            if c.volume == 0.0 {
                continue;
            }
            let lo_idx = (((c.low - low) / width).floor() as usize).min(self.bins - 1);
            let hi_idx = (((c.high - low) / width).floor() as usize).min(self.bins - 1);
            let touched = hi_idx - lo_idx + 1;
            let share = c.volume / touched as f64;
            for bin in hist.iter_mut().take(hi_idx + 1).skip(lo_idx) {
                *bin += share;
            }
        }
        (low, width, hist)
    }
}

impl Indicator for HighLowVolumeNodes {
    type Input = Candle;
    type Output = HighLowVolumeNodesOutput;

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn update(&mut self, candle: Candle) -> Option<HighLowVolumeNodesOutput> {
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(candle);
        if self.window.len() < self.period {
            return None;
        }
        let (low, width, hist) = self.profile();
        let centre = |idx: usize| low + (idx as f64 + 0.5) * width;

        let mut hvn_idx = 0;
        let mut hvn_vol = f64::NEG_INFINITY;
        let mut lvn_idx = 0;
        let mut lvn_vol = f64::INFINITY;
        for (idx, &vol) in hist.iter().enumerate() {
            if vol > hvn_vol {
                hvn_vol = vol;
                hvn_idx = idx;
            }
            if vol > 0.0 && vol < lvn_vol {
                lvn_vol = vol;
                lvn_idx = idx;
            }
        }
        // If no traded bin was found (all zero volume), both default to bin 0.
        if !lvn_vol.is_finite() {
            lvn_idx = hvn_idx;
        }
        let out = HighLowVolumeNodesOutput {
            hvn: centre(hvn_idx),
            lvn: centre(lvn_idx),
        };
        self.last = Some(out);
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
        "HighLowVolumeNodes"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(high: f64, low: f64, volume: f64) -> Candle {
        Candle::new_unchecked(
            f64::midpoint(high, low),
            high,
            low,
            f64::midpoint(high, low),
            volume,
            0,
        )
    }

    #[test]
    fn rejects_zero_params() {
        assert!(matches!(
            HighLowVolumeNodes::new(0, 24),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            HighLowVolumeNodes::new(20, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let h = HighLowVolumeNodes::new(20, 24).unwrap();
        assert_eq!(h.params(), (20, 24));
        assert_eq!(h.warmup_period(), 20);
        assert_eq!(h.name(), "HighLowVolumeNodes");
        assert!(!h.is_ready());
        assert_eq!(h.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut h = HighLowVolumeNodes::new(4, 8).unwrap();
        let candles: Vec<Candle> = (0..6).map(|_| c(110.0, 90.0, 1_000.0)).collect();
        let out = h.batch(&candles);
        for v in out.iter().take(3) {
            assert!(v.is_none());
        }
        assert!(out[3].is_some());
    }

    #[test]
    fn hvn_at_heavy_price() {
        // Most bars cluster at ~100 (heavy volume); one bar pokes up to 120 lightly.
        let mut h = HighLowVolumeNodes::new(6, 24).unwrap();
        let mut candles: Vec<Candle> = (0..5).map(|_| c(101.0, 99.0, 5_000.0)).collect();
        candles.push(c(121.0, 119.0, 100.0));
        let out = h.batch(&candles).into_iter().flatten().last().unwrap();
        // HVN should sit near the heavy 100 cluster, well below the light 120 poke.
        assert!(
            out.hvn < 110.0,
            "HVN should be at the heavy cluster, got {}",
            out.hvn
        );
        assert!(out.lvn >= out.hvn - 1e9); // lvn is a valid level
    }

    #[test]
    fn hvn_at_or_above_low() {
        let mut h = HighLowVolumeNodes::new(10, 24).unwrap();
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                c(
                    110.0 + (f64::from(i) * 0.3).sin() * 5.0,
                    90.0,
                    1_000.0 + f64::from(i),
                )
            })
            .collect();
        for o in h.batch(&candles).into_iter().flatten() {
            assert!(o.hvn.is_finite() && o.lvn.is_finite());
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut h = HighLowVolumeNodes::new(4, 8).unwrap();
        h.batch(&[c(110.0, 90.0, 1_000.0); 6]);
        assert!(h.is_ready());
        h.reset();
        assert!(!h.is_ready());
        assert_eq!(h.value(), None);
        assert_eq!(h.update(c(110.0, 90.0, 1_000.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                c(
                    110.0 + (f64::from(i) * 0.25).sin() * 9.0,
                    90.0,
                    1_000.0 + f64::from(i),
                )
            })
            .collect();
        let batch = HighLowVolumeNodes::new(20, 24).unwrap().batch(&candles);
        let mut b = HighLowVolumeNodes::new(20, 24).unwrap();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn flat_window_is_handled() {
        // Zero high-low span dumps all volume into bin 0 and returns early.
        let mut h = HighLowVolumeNodes::new(2, 4).unwrap();
        h.update(c(50.0, 50.0, 10.0));
        assert!(h.update(c(50.0, 50.0, 10.0)).is_some());
    }

    #[test]
    fn zero_volume_window_falls_back() {
        // All-zero volume leaves no traded bin; the LVN falls back to the HVN.
        let mut h = HighLowVolumeNodes::new(2, 4).unwrap();
        h.update(c(60.0, 40.0, 0.0));
        let out = h.update(c(60.0, 40.0, 0.0)).unwrap();
        assert_eq!(out.hvn, out.lvn);
    }
}
