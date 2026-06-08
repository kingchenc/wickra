//! Composite Profile — POC and value area over a long composite window.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Output of [`CompositeProfile`]: the point of control and the value-area bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompositeProfileOutput {
    /// Point of Control — the price (bin centre) with the most volume.
    pub poc: f64,
    /// Value-Area High — top of the band holding `value_area_pct` of volume.
    pub vah: f64,
    /// Value-Area Low — bottom of that band.
    pub val: f64,
}

/// Composite Profile — a multi-session volume profile reduced to its **point of
/// control** and **value area**, built over a long composite window.
///
/// ```text
/// build a `bins`-bucket volume profile over the last `period` candles
/// POC = bin with the most volume
/// expand from the POC, always adding the heavier adjacent bin, until the
///   accumulated volume reaches `value_area_pct` of the total
/// VAH / VAL = the highest / lowest price included
/// ```
///
/// A composite profile merges many sessions into one structure to reveal the
/// dominant value area and control price across a longer horizon — the levels that
/// matter for swing positioning rather than a single day. The point of control is
/// the fairest price (heaviest trade); the value area (classically 70% of volume)
/// brackets where the market spent most of its time. Price inside the value area is
/// "in balance"; acceptance outside it signals a value migration.
///
/// The first value lands after `period` candles; each `update` rebuilds the
/// profile in O(`period · bins`).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, CompositeProfile};
///
/// let mut indicator = CompositeProfile::new(100, 50, 0.70).unwrap();
/// let mut last = None;
/// for i in 0..150 {
///     let base = 100.0 + (f64::from(i) * 0.1).sin() * 8.0;
///     let c = Candle::new(base, base + 1.0, base - 1.0, base, 1_000.0, 0).unwrap();
///     last = indicator.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct CompositeProfile {
    period: usize,
    bins: usize,
    value_area_pct: f64,
    window: VecDeque<Candle>,
    last: Option<CompositeProfileOutput>,
}

impl CompositeProfile {
    /// Construct a Composite Profile.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period` or `bins` is zero, or
    /// [`Error::InvalidParameter`] if `value_area_pct` is not in `(0, 1]`.
    pub fn new(period: usize, bins: usize, value_area_pct: f64) -> Result<Self> {
        if period == 0 || bins == 0 {
            return Err(Error::PeriodZero);
        }
        if !value_area_pct.is_finite() || value_area_pct <= 0.0 || value_area_pct > 1.0 {
            return Err(Error::InvalidParameter {
                message: "value_area_pct must be in (0, 1]",
            });
        }
        Ok(Self {
            period,
            bins,
            value_area_pct,
            window: VecDeque::with_capacity(period),
            last: None,
        })
    }

    /// Configured `(period, bins, value_area_pct)`.
    pub const fn params(&self) -> (usize, usize, f64) {
        (self.period, self.bins, self.value_area_pct)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<CompositeProfileOutput> {
        self.last
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn compute(&self) -> CompositeProfileOutput {
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for c in &self.window {
            low = low.min(c.low);
            high = high.max(c.high);
        }
        let span = high - low;
        if span <= 0.0 {
            return CompositeProfileOutput {
                poc: low,
                vah: low,
                val: low,
            };
        }
        let width = span / self.bins as f64;
        let centre = |idx: usize| low + (idx as f64 + 0.5) * width;
        let mut hist = vec![0.0; self.bins];
        for c in &self.window {
            if c.volume == 0.0 {
                continue;
            }
            let lo_idx = (((c.low - low) / width).floor() as usize).min(self.bins - 1);
            let hi_idx = (((c.high - low) / width).floor() as usize).min(self.bins - 1);
            let share = c.volume / (hi_idx - lo_idx + 1) as f64;
            for bin in hist.iter_mut().take(hi_idx + 1).skip(lo_idx) {
                *bin += share;
            }
        }
        let total: f64 = hist.iter().sum();
        let mut poc = 0;
        let mut poc_vol = f64::NEG_INFINITY;
        for (idx, &vol) in hist.iter().enumerate() {
            if vol > poc_vol {
                poc_vol = vol;
                poc = idx;
            }
        }
        let target = total * self.value_area_pct;
        let mut acc = hist[poc];
        let mut top = poc;
        let mut bottom = poc;
        while acc < target && (top < self.bins - 1 || bottom > 0) {
            let above = if top < self.bins - 1 {
                hist[top + 1]
            } else {
                f64::NEG_INFINITY
            };
            let below = if bottom > 0 {
                hist[bottom - 1]
            } else {
                f64::NEG_INFINITY
            };
            if above >= below {
                top += 1;
                acc += hist[top];
            } else {
                bottom -= 1;
                acc += hist[bottom];
            }
        }
        CompositeProfileOutput {
            poc: centre(poc),
            vah: centre(top),
            val: centre(bottom),
        }
    }
}

impl Indicator for CompositeProfile {
    type Input = Candle;
    type Output = CompositeProfileOutput;

    fn update(&mut self, candle: Candle) -> Option<CompositeProfileOutput> {
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(candle);
        if self.window.len() < self.period {
            return None;
        }
        let out = self.compute();
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
        "CompositeProfile"
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
    fn rejects_invalid_params() {
        assert!(matches!(
            CompositeProfile::new(0, 50, 0.7),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            CompositeProfile::new(100, 0, 0.7),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            CompositeProfile::new(100, 50, 0.0),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            CompositeProfile::new(100, 50, 1.5),
            Err(Error::InvalidParameter { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let p = CompositeProfile::new(100, 50, 0.7).unwrap();
        assert_eq!(p.params(), (100, 50, 0.7));
        assert_eq!(p.warmup_period(), 100);
        assert_eq!(p.name(), "CompositeProfile");
        assert!(!p.is_ready());
        assert_eq!(p.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut p = CompositeProfile::new(4, 8, 0.7).unwrap();
        let candles: Vec<Candle> = (0..6).map(|_| c(110.0, 90.0, 1_000.0)).collect();
        let out = p.batch(&candles);
        for v in out.iter().take(3) {
            assert!(v.is_none());
        }
        assert!(out[3].is_some());
    }

    #[test]
    fn value_area_brackets_poc() {
        let mut p = CompositeProfile::new(20, 30, 0.7).unwrap();
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                c(
                    110.0 + (f64::from(i) * 0.3).sin() * 8.0,
                    90.0 + (f64::from(i) * 0.3).cos() * 8.0,
                    1_000.0,
                )
            })
            .collect();
        for o in p.batch(&candles).into_iter().flatten() {
            assert!(o.val <= o.poc && o.poc <= o.vah);
        }
    }

    #[test]
    fn poc_at_heavy_cluster() {
        // Volume clustered at ~100; thin pokes elsewhere -> POC near 100.
        let mut p = CompositeProfile::new(6, 30, 0.7).unwrap();
        let mut candles: Vec<Candle> = (0..5).map(|_| c(101.0, 99.0, 5_000.0)).collect();
        candles.push(c(140.0, 60.0, 50.0));
        let out = p.batch(&candles).into_iter().flatten().last().unwrap();
        assert!(
            (out.poc - 100.0).abs() < 5.0,
            "POC should sit at the cluster, got {}",
            out.poc
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut p = CompositeProfile::new(4, 8, 0.7).unwrap();
        p.batch(&[c(110.0, 90.0, 1_000.0); 6]);
        assert!(p.is_ready());
        p.reset();
        assert!(!p.is_ready());
        assert_eq!(p.value(), None);
        assert_eq!(p.update(c(110.0, 90.0, 1_000.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..120)
            .map(|i| {
                c(
                    110.0 + (f64::from(i) * 0.25).sin() * 9.0,
                    90.0,
                    1_000.0 + f64::from(i),
                )
            })
            .collect();
        let batch = CompositeProfile::new(50, 50, 0.7).unwrap().batch(&candles);
        let mut b = CompositeProfile::new(50, 50, 0.7).unwrap();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn flat_window_collapses_to_price() {
        // Zero high-low span returns the price for POC, VAH and VAL.
        let mut cp = CompositeProfile::new(2, 4, 0.7).unwrap();
        cp.update(c(50.0, 50.0, 10.0));
        let out = cp.update(c(50.0, 50.0, 10.0)).unwrap();
        assert_eq!(out.poc, out.vah);
        assert_eq!(out.poc, out.val);
    }

    #[test]
    fn zero_volume_window_is_handled() {
        // Non-flat window of zero-volume candles hits the skip path.
        let mut cp = CompositeProfile::new(2, 4, 0.7).unwrap();
        cp.update(c(60.0, 40.0, 0.0));
        assert!(cp.update(c(60.0, 40.0, 0.0)).is_some());
    }

    #[test]
    fn value_area_expands_down_from_top_poc() {
        // POC sits in the top bin; with a wide value-area target the area runs
        // out of bins above (the ceiling branch) and keeps expanding downward.
        let mut cp = CompositeProfile::new(2, 3, 0.9).unwrap();
        cp.update(c(100.0, 0.0, 30.0)); // thin spread across all three bins
        let out = cp.update(c(100.0, 67.0, 60.0)).unwrap(); // heavy in the top bin
        assert!(out.val <= out.poc && out.poc <= out.vah);
    }
}
