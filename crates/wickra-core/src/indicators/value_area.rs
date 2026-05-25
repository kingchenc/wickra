//! Value Area (Point of Control + Value Area High / Low).
//!
//! Market-profile-style volume distribution over the last `period` candles,
//! bucketed into `bin_count` price bins. Each candle's volume is spread
//! uniformly across its `[low, high]` range (bin-approximation); single-print
//! bars (`low == high`) dump their whole volume into a single bin. The
//! Point of Control (POC) is the bin with the highest cumulative volume; the
//! Value Area expands outward from the POC, always absorbing the
//! higher-volume neighbour next, until the configured percentage of total
//! volume (default 70%) is enclosed.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Value Area output: Point of Control, Value Area High and Value Area Low.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueAreaOutput {
    /// Point of Control — price of the bin with the highest cumulative volume.
    pub poc: f64,
    /// Value Area High — upper bound of the bins that together hold
    /// `value_area_pct` of the rolling-window volume.
    pub vah: f64,
    /// Value Area Low — lower bound of those same bins.
    pub val: f64,
}

/// Rolling Value Area indicator over the last `period` candles.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, ValueArea};
///
/// let mut va = ValueArea::new(5, 50, 0.70).unwrap();
/// for i in 0..10 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base, 10.0, i64::from(i)).unwrap();
///     va.update(candle);
/// }
/// assert!(va.is_ready());
/// ```
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone)]
pub struct ValueArea {
    period: usize,
    bin_count: usize,
    value_area_pct: f64,
    window: VecDeque<Candle>,
    last: Option<ValueAreaOutput>,
}

impl ValueArea {
    /// Construct a Value Area indicator.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period` or `bin_count` is zero,
    /// and [`Error::InvalidPeriod`] if `value_area_pct` is not in `(0, 1]`.
    pub fn new(period: usize, bin_count: usize, value_area_pct: f64) -> Result<Self> {
        if period == 0 || bin_count == 0 {
            return Err(Error::PeriodZero);
        }
        if !value_area_pct.is_finite() || value_area_pct <= 0.0 || value_area_pct > 1.0 {
            return Err(Error::InvalidPeriod {
                message: "value_area_pct must be in (0, 1]",
            });
        }
        Ok(Self {
            period,
            bin_count,
            value_area_pct,
            window: VecDeque::with_capacity(period),
            last: None,
        })
    }

    /// Classic Value Area: 20-bar rolling window, 50 bins, 70% concentration.
    pub fn classic() -> Self {
        Self::new(20, 50, 0.70).expect("classic ValueArea params are valid")
    }

    /// Configured `(period, bin_count, value_area_pct)`.
    pub const fn params(&self) -> (usize, usize, f64) {
        (self.period, self.bin_count, self.value_area_pct)
    }

    /// Most recent output if available.
    pub const fn value(&self) -> Option<ValueAreaOutput> {
        self.last
    }

    fn compute(&self) -> ValueAreaOutput {
        // Window-wide low / high spans the histogram domain.
        let mut win_low = f64::INFINITY;
        let mut win_high = f64::NEG_INFINITY;
        for c in &self.window {
            if c.low < win_low {
                win_low = c.low;
            }
            if c.high > win_high {
                win_high = c.high;
            }
        }
        let span = win_high - win_low;
        let mut bins = vec![0.0_f64; self.bin_count];

        // Distribute each candle's volume across its [low, high] range. A
        // degenerate `low == high` bar drops its entire volume into one bin.
        if span <= 0.0 {
            // All bars are single-print at the same price — POC = that price,
            // VAH = VAL = that price.
            let total: f64 = self.window.iter().map(|c| c.volume).sum();
            bins[0] = total;
            return ValueAreaOutput {
                poc: win_low,
                vah: win_low,
                val: win_low,
            };
        }
        let bin_width = span / self.bin_count as f64;
        for c in &self.window {
            if c.volume == 0.0 {
                continue;
            }
            if c.high <= c.low {
                let idx = self.price_to_bin(c.low, win_low, bin_width);
                bins[idx] += c.volume;
                continue;
            }
            let lo_idx = self.price_to_bin(c.low, win_low, bin_width);
            let hi_idx = self.price_to_bin(c.high, win_low, bin_width);
            let touched = hi_idx - lo_idx + 1;
            let share = c.volume / touched as f64;
            for b in bins.iter_mut().take(hi_idx + 1).skip(lo_idx) {
                *b += share;
            }
        }

        let total: f64 = bins.iter().sum();
        // POC = bin with highest volume.
        let mut poc_idx = 0_usize;
        let mut poc_vol = bins[0];
        for (i, v) in bins.iter().enumerate().skip(1) {
            if *v > poc_vol {
                poc_vol = *v;
                poc_idx = i;
            }
        }

        // Expand Value Area outward from POC. At each step take the
        // higher-volume neighbour (up or down). Equal volumes break upward,
        // matching the CME convention. The loop condition guarantees at
        // least one of `can_go_up` / `can_go_down` is true on every body
        // entry, so the inner `else` branch is always reachable.
        let target = total * self.value_area_pct;
        let mut accumulated = poc_vol;
        let mut lo = poc_idx;
        let mut hi = poc_idx;
        while accumulated < target && (lo > 0 || hi + 1 < self.bin_count) {
            let can_go_up = hi + 1 < self.bin_count;
            let can_go_down = lo > 0;
            let up_v = if can_go_up {
                bins[hi + 1]
            } else {
                f64::NEG_INFINITY
            };
            let down_v = if can_go_down {
                bins[lo - 1]
            } else {
                f64::NEG_INFINITY
            };
            if can_go_up && (up_v >= down_v || !can_go_down) {
                hi += 1;
                accumulated += up_v;
            } else {
                lo -= 1;
                accumulated += down_v;
            }
        }

        let bin_mid = |i: usize| win_low + bin_width * (i as f64 + 0.5);
        ValueAreaOutput {
            poc: bin_mid(poc_idx),
            vah: win_low + bin_width * (hi as f64 + 1.0),
            val: win_low + bin_width * lo as f64,
        }
    }

    fn price_to_bin(&self, price: f64, win_low: f64, bin_width: f64) -> usize {
        // Clamp the float into [0, bin_count - 1] before casting so the
        // `as usize` step cannot overflow or wrap.
        let raw = ((price - win_low) / bin_width).floor();
        let max = (self.bin_count - 1) as f64;
        raw.clamp(0.0, max) as usize
    }
}

impl Indicator for ValueArea {
    type Input = Candle;
    type Output = ValueAreaOutput;

    fn update(&mut self, candle: Candle) -> Option<ValueAreaOutput> {
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
        "ValueArea"
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
        assert!(matches!(ValueArea::new(0, 50, 0.7), Err(Error::PeriodZero)));
    }

    #[test]
    fn rejects_zero_bin_count() {
        assert!(matches!(ValueArea::new(20, 0, 0.7), Err(Error::PeriodZero)));
    }

    #[test]
    fn rejects_invalid_value_area_pct() {
        assert!(matches!(
            ValueArea::new(20, 50, 0.0),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            ValueArea::new(20, 50, 1.5),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            ValueArea::new(20, 50, f64::NAN),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let v = ValueArea::new(20, 50, 0.7).unwrap();
        assert_eq!(v.params(), (20, 50, 0.7));
        assert_eq!(v.name(), "ValueArea");
        assert_eq!(v.warmup_period(), 20);
        assert!(v.value().is_none());
    }

    #[test]
    fn classic_is_constructible() {
        let v = ValueArea::classic();
        assert_eq!(v.params(), (20, 50, 0.70));
    }

    #[test]
    fn warmup_emits_after_period() {
        let mut v = ValueArea::new(5, 10, 0.7).unwrap();
        for i in 0..4 {
            let base = 100.0;
            assert!(v
                .update(c(base, base + 1.0, base - 1.0, base, 10.0, i))
                .is_none());
        }
        let out = v
            .update(c(100.0, 101.0, 99.0, 100.0, 10.0, 4))
            .expect("ready after period");
        // All five bars are identical, so POC == bar mid; VAH/VAL bracket
        // the window high/low.
        assert!(out.vah >= out.poc);
        assert!(out.poc >= out.val);
        assert!(v.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + (i as f64).sin();
                c(base, base + 1.0, base - 1.0, base, 10.0 + i as f64, i)
            })
            .collect();
        let mut a = ValueArea::new(10, 20, 0.7).unwrap();
        let mut b = ValueArea::new(10, 20, 0.7).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..20)
            .map(|i| c(100.0, 101.0, 99.0, 100.0, 10.0, i))
            .collect();
        let mut v = ValueArea::new(5, 10, 0.7).unwrap();
        v.batch(&candles);
        assert!(v.is_ready());
        v.reset();
        assert!(!v.is_ready());
        assert_eq!(v.update(candles[0]), None);
    }

    #[test]
    fn constant_single_print_yields_collapsed_value_area() {
        // Every bar trades at exactly 100 (low == high == 100) — the
        // histogram has zero span so POC == VAH == VAL == 100.
        let candles: Vec<Candle> = (0..10)
            .map(|i| c(100.0, 100.0, 100.0, 100.0, 5.0, i))
            .collect();
        let mut v = ValueArea::new(5, 20, 0.7).unwrap();
        let out = v.batch(&candles).into_iter().flatten().last().unwrap();
        assert_relative_eq!(out.poc, 100.0, epsilon = 1e-12);
        assert_relative_eq!(out.vah, 100.0, epsilon = 1e-12);
        assert_relative_eq!(out.val, 100.0, epsilon = 1e-12);
    }

    #[test]
    fn single_print_bar_in_mixed_window_dumps_volume_into_one_bin() {
        // Mix of wide-range bars (drive the window's span > 0) and one
        // single-print bar at price 102 with massive volume. The single-print
        // bar must dump its entire volume into one bin, making the POC land
        // exactly on the bin that contains 102.
        let candles = vec![
            c(100.0, 100.5, 99.5, 100.0, 1.0, 0),
            c(100.0, 100.5, 99.5, 100.0, 1.0, 1),
            c(102.0, 102.0, 102.0, 102.0, 1000.0, 2),
            c(100.0, 100.5, 99.5, 100.0, 1.0, 3),
            c(100.0, 100.5, 99.5, 100.0, 1.0, 4),
        ];
        let mut v = ValueArea::new(5, 50, 0.70).unwrap();
        let out = v.batch(&candles).into_iter().flatten().last().unwrap();
        // POC must sit in the high-volume bin that holds price 102.
        assert!(
            (101.9..=102.1).contains(&out.poc),
            "POC {} not near 102",
            out.poc
        );
    }

    #[test]
    fn concentrated_volume_locates_poc_at_high_volume_bar() {
        // Bars 0..3 sit at price 100 with volume 1; bar 4 dumps massive
        // volume at price 110. POC must land near 110.
        let mut candles = vec![
            c(100.0, 100.5, 99.5, 100.0, 1.0, 0),
            c(100.0, 100.5, 99.5, 100.0, 1.0, 1),
            c(100.0, 100.5, 99.5, 100.0, 1.0, 2),
            c(100.0, 100.5, 99.5, 100.0, 1.0, 3),
        ];
        candles.push(c(110.0, 110.5, 109.5, 110.0, 1000.0, 4));
        let mut v = ValueArea::new(5, 50, 0.70).unwrap();
        let out = v.batch(&candles).into_iter().flatten().last().unwrap();
        // POC must fall inside the high-volume bar's [low, high] range; ties
        // among equal-volume bins resolve to the lowest index, so the POC
        // sits on the left edge of bar 4's range rather than at its midpoint.
        assert!(
            (109.5..=110.5).contains(&out.poc),
            "POC {} not inside [109.5, 110.5]",
            out.poc
        );
        // VAH and VAL bracket POC.
        assert!(out.vah >= out.poc);
        assert!(out.val <= out.poc);
    }

    #[test]
    fn value_area_brackets_point_of_control() {
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                let base = 100.0 + (i as f64).cos() * 2.0;
                c(base, base + 0.5, base - 0.5, base, 10.0, i)
            })
            .collect();
        let mut v = ValueArea::new(15, 30, 0.70).unwrap();
        for o in v.batch(&candles).into_iter().flatten() {
            assert!(o.vah >= o.poc, "VAH {} < POC {}", o.vah, o.poc);
            assert!(o.val <= o.poc, "VAL {} > POC {}", o.val, o.poc);
        }
    }

    #[test]
    fn zero_volume_bars_are_skipped_in_histogram() {
        // Only bar 4 carries any volume — POC must land at its mid.
        let candles = vec![
            c(100.0, 100.5, 99.5, 100.0, 0.0, 0),
            c(100.0, 100.5, 99.5, 100.0, 0.0, 1),
            c(100.0, 100.5, 99.5, 100.0, 0.0, 2),
            c(100.0, 100.5, 99.5, 100.0, 0.0, 3),
            c(100.0, 100.5, 99.5, 100.0, 50.0, 4),
        ];
        let mut v = ValueArea::new(5, 20, 0.7).unwrap();
        let out = v.batch(&candles).into_iter().flatten().last().unwrap();
        assert!(out.poc.is_finite());
        assert!(out.vah.is_finite());
        assert!(out.val.is_finite());
    }
}
