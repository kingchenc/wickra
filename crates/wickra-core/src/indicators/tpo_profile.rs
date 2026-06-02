//! TPO Profile — the Time-Price-Opportunity (market-profile letter) distribution.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// TPO Profile output: the price domain plus the per-bin time-period counts.
///
/// `counts[i]` is the number of periods in the rolling window whose `[low, high]`
/// range touched the price bucket `[price_low + i * w, price_low + (i + 1) * w)`
/// where `w = (price_high - price_low) / counts.len()`. This is the classic
/// market-profile "letter" count: one Time-Price-Opportunity per period per
/// price level it traded at, independent of volume.
#[derive(Debug, Clone, PartialEq)]
pub struct TpoProfileOutput {
    /// Lowest price in the window — the lower edge of bin 0.
    pub price_low: f64,
    /// Highest price in the window — the upper edge of the last bin.
    pub price_high: f64,
    /// Per-bin TPO count, lowest price bucket first. Length equals `bin_count`.
    pub counts: Vec<f64>,
}

/// Rolling TPO (Time Price Opportunity) Profile over the last `period` candles.
///
/// Where [`crate::VolumeProfile`] distributes each bar's *volume* across the
/// bins it touches, the TPO profile counts *time*: every period that trades at a
/// price level contributes exactly one TPO mark there, regardless of how much
/// volume it carried. The result highlights the prices the market spent the most
/// time at — the market-profile bell curve. Each touched bin receives a full
/// `+1` per period (no sharing), so a wide-range bar marks every level it spans.
///
/// A window whose bars are all single-print at one price (`price_high == price_low`)
/// is degenerate: every period's mark lands in bin 0 and both edges collapse to
/// that price.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, TpoProfile};
///
/// let mut tpo = TpoProfile::new(5, 10).unwrap();
/// let mut last = None;
/// for i in 0..10 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base, 10.0, i64::from(i)).unwrap();
///     last = tpo.update(candle);
/// }
/// let profile = last.unwrap();
/// assert_eq!(profile.counts.len(), 10);
/// ```
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone)]
pub struct TpoProfile {
    period: usize,
    bin_count: usize,
    window: VecDeque<Candle>,
    last: Option<TpoProfileOutput>,
}

impl TpoProfile {
    /// Construct a TPO Profile indicator.
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

    /// Classic TPO Profile: 30-bar rolling window, 50 bins.
    pub fn classic() -> Self {
        Self::new(30, 50).expect("classic TpoProfile params are valid")
    }

    /// Configured `(period, bin_count)`.
    pub const fn params(&self) -> (usize, usize) {
        (self.period, self.bin_count)
    }

    /// Most recent profile if available.
    pub fn value(&self) -> Option<&TpoProfileOutput> {
        self.last.as_ref()
    }

    fn price_to_bin(&self, price: f64, win_low: f64, bin_width: f64) -> usize {
        let raw = ((price - win_low) / bin_width).floor();
        let max = (self.bin_count - 1) as f64;
        raw.clamp(0.0, max) as usize
    }

    fn compute(&self) -> TpoProfileOutput {
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
        let mut counts = vec![0.0_f64; self.bin_count];

        if span <= 0.0 {
            // All bars are single-print at the same price: every period marks bin 0.
            counts[0] = self.window.len() as f64;
            return TpoProfileOutput {
                price_low: win_low,
                price_high: win_low,
                counts,
            };
        }

        let bin_width = span / self.bin_count as f64;
        for candle in &self.window {
            if candle.high <= candle.low {
                let idx = self.price_to_bin(candle.low, win_low, bin_width);
                counts[idx] += 1.0;
                continue;
            }
            let lo_idx = self.price_to_bin(candle.low, win_low, bin_width);
            let hi_idx = self.price_to_bin(candle.high, win_low, bin_width);
            for count in counts.iter_mut().take(hi_idx + 1).skip(lo_idx) {
                *count += 1.0;
            }
        }

        TpoProfileOutput {
            price_low: win_low,
            price_high: win_high,
            counts,
        }
    }
}

impl Indicator for TpoProfile {
    type Input = Candle;
    type Output = TpoProfileOutput;

    fn update(&mut self, candle: Candle) -> Option<TpoProfileOutput> {
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
        "TpoProfile"
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
        assert!(matches!(TpoProfile::new(0, 50), Err(Error::PeriodZero)));
    }

    #[test]
    fn rejects_zero_bin_count() {
        assert!(matches!(TpoProfile::new(20, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let tpo = TpoProfile::new(30, 50).unwrap();
        assert_eq!(tpo.name(), "TpoProfile");
        assert_eq!(tpo.warmup_period(), 30);
        assert_eq!(tpo.params(), (30, 50));
        assert!(tpo.value().is_none());
        assert!(!tpo.is_ready());
    }

    #[test]
    fn classic_params() {
        let tpo = TpoProfile::classic();
        assert_eq!(tpo.params(), (30, 50));
    }

    #[test]
    fn warms_up_over_period() {
        let mut tpo = TpoProfile::new(3, 4).unwrap();
        assert!(tpo.update(c(10.0, 11.0, 9.0, 10.0, 5.0, 0)).is_none());
        assert!(tpo.update(c(10.0, 11.0, 9.0, 10.0, 5.0, 1)).is_none());
        assert!(tpo.update(c(10.0, 11.0, 9.0, 10.0, 5.0, 2)).is_some());
        assert!(tpo.is_ready());
    }

    #[test]
    fn reference_counts() {
        // Window of 2 candles, 4 bins, domain 10..14, width 1.
        // bar0: 10..14 touches bins 0,1,2,3 -> +1 each.
        // bar1: 11..12 touches bins 1,2 -> +1 each.
        // counts = [1, 2, 2, 1]. TPO is volume-agnostic.
        let mut tpo = TpoProfile::new(2, 4).unwrap();
        assert!(tpo.update(c(10.0, 14.0, 10.0, 12.0, 5.0, 0)).is_none());
        let out = tpo.update(c(11.0, 12.0, 11.0, 11.5, 999.0, 1)).unwrap();
        assert_relative_eq!(out.price_low, 10.0, epsilon = 1e-12);
        assert_relative_eq!(out.price_high, 14.0, epsilon = 1e-12);
        assert_eq!(out.counts.len(), 4);
        assert_relative_eq!(out.counts[0], 1.0, epsilon = 1e-12);
        assert_relative_eq!(out.counts[1], 2.0, epsilon = 1e-12);
        assert_relative_eq!(out.counts[2], 2.0, epsilon = 1e-12);
        assert_relative_eq!(out.counts[3], 1.0, epsilon = 1e-12);
    }

    #[test]
    fn volume_independent() {
        // Identical ranges with wildly different volumes give identical TPO counts.
        let mut a = TpoProfile::new(2, 4).unwrap();
        let mut b = TpoProfile::new(2, 4).unwrap();
        a.update(c(10.0, 14.0, 10.0, 12.0, 1.0, 0));
        let out_a = a.update(c(10.0, 14.0, 10.0, 12.0, 1.0, 1)).unwrap();
        b.update(c(10.0, 14.0, 10.0, 12.0, 9_999.0, 0));
        let out_b = b.update(c(10.0, 14.0, 10.0, 12.0, 9_999.0, 1)).unwrap();
        assert_eq!(out_a.counts, out_b.counts);
    }

    #[test]
    fn degenerate_single_price_window() {
        let mut tpo = TpoProfile::new(3, 4).unwrap();
        tpo.update(c(50.0, 50.0, 50.0, 50.0, 10.0, 0));
        tpo.update(c(50.0, 50.0, 50.0, 50.0, 20.0, 1));
        let out = tpo.update(c(50.0, 50.0, 50.0, 50.0, 30.0, 2)).unwrap();
        assert_relative_eq!(out.price_low, 50.0, epsilon = 1e-12);
        assert_relative_eq!(out.price_high, 50.0, epsilon = 1e-12);
        assert_relative_eq!(out.counts[0], 3.0, epsilon = 1e-12);
        assert_relative_eq!(out.counts[1], 0.0, epsilon = 1e-12);
    }

    #[test]
    fn single_print_bar_marks_one_bin() {
        // A single-print bar inside a wider domain marks exactly its own bin.
        let mut tpo = TpoProfile::new(2, 4).unwrap();
        tpo.update(c(10.0, 14.0, 10.0, 12.0, 5.0, 0)); // domain setter
        let out = tpo.update(c(13.0, 13.0, 13.0, 13.0, 5.0, 1)).unwrap();
        // domain 10..14, width 1; price 13 -> bin 3.
        assert_relative_eq!(out.counts[3], 2.0, epsilon = 1e-12);
    }

    #[test]
    fn reset_clears_state() {
        let mut tpo = TpoProfile::new(2, 4).unwrap();
        tpo.update(c(10.0, 11.0, 9.0, 10.0, 5.0, 0));
        tpo.update(c(10.0, 11.0, 9.0, 10.0, 5.0, 1));
        assert!(tpo.is_ready());
        tpo.reset();
        assert!(!tpo.is_ready());
        assert!(tpo.value().is_none());
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
        let mut a = TpoProfile::new(10, 16).unwrap();
        let mut b = TpoProfile::new(10, 16).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
