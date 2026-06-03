//! Turn-of-Month Effect — the mean daily return of sessions that fall inside the
//! turn-of-month window (the last `n_last` and first `n_first` days of a month).

use crate::calendar::{civil_from_timestamp, days_in_month};
use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Whether a day-of-month lies in the turn-of-month window.
///
/// The window is the first `n_first` calendar days plus the last `n_last` days of
/// the month (`days_in_month - n_last < dom`).
fn in_turn_window(dom: u32, dim: u32, n_first: u32, n_last: u32) -> bool {
    dom <= n_first || dom > dim.saturating_sub(n_last)
}

/// Turn-of-Month effect: the running mean of daily close-to-close returns for the
/// sessions that fall in the turn-of-month window.
///
/// Each completed session (the wall-clock day of
/// [`Candle::timestamp`](crate::Candle) shifted by `utc_offset_minutes`)
/// contributes its return `close / previous_close - 1`. Only sessions whose
/// day-of-month is within the first `n_first` or last `n_last` days of their month
/// are averaged; the rest are ignored. The classic effect uses `n_first = 3`,
/// `n_last = 1`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, TurnOfMonth};
///
/// let day = 24 * 3_600_000;
/// // 2021-01-29 .. 02-02 — all turn-of-month days with n_first=3, n_last=1.
/// let mut tom = TurnOfMonth::new(3, 1, 0).unwrap();
/// let start = 1_611_878_400_000; // 2021-01-29 00:00 UTC
/// let mut last = None;
/// for (i, close) in [100.0, 101.0, 102.0, 103.0].iter().enumerate() {
///     let ts = start + i as i64 * day;
///     last = tom.update(Candle::new(*close, *close, *close, *close, 1.0, ts).unwrap());
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct TurnOfMonth {
    n_first: u32,
    n_last: u32,
    utc_offset_minutes: i32,
    day: Option<(i64, u32, u32)>,
    cur_close: f64,
    prev_day_close: Option<f64>,
    sum: f64,
    count: u64,
}

impl TurnOfMonth {
    /// Construct a Turn-of-Month indicator.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if both `n_first` and `n_last` are zero (the
    /// window would never include a day).
    pub fn new(n_first: u32, n_last: u32, utc_offset_minutes: i32) -> Result<Self> {
        if n_first == 0 && n_last == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            n_first,
            n_last,
            utc_offset_minutes,
            day: None,
            cur_close: 0.0,
            prev_day_close: None,
            sum: 0.0,
            count: 0,
        })
    }

    /// Classic turn-of-month window: first 3 and last 1 day of the month.
    pub fn classic() -> Self {
        Self::new(3, 1, 0).expect("classic turn-of-month window is valid")
    }

    /// Configured `(n_first, n_last, utc_offset_minutes)`.
    pub const fn params(&self) -> (u32, u32, i32) {
        (self.n_first, self.n_last, self.utc_offset_minutes)
    }

    /// Most recent mean turn-of-month return if any in-window day has completed.
    pub fn value(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }

    /// Settle the just-finished day `(year, month, dom)` whose last close is
    /// `self.cur_close`, then start `next_key`.
    fn roll_into(
        &mut self,
        year: i64,
        month: u32,
        dom: u32,
        next_key: (i64, u32, u32),
        close: f64,
    ) {
        if let Some(prev) = self.prev_day_close {
            let ret = if prev == 0.0 {
                0.0
            } else {
                self.cur_close / prev - 1.0
            };
            if in_turn_window(dom, days_in_month(year, month), self.n_first, self.n_last) {
                self.sum += ret;
                self.count += 1;
            }
        }
        self.prev_day_close = Some(self.cur_close);
        self.day = Some(next_key);
        self.cur_close = close;
    }
}

impl Indicator for TurnOfMonth {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let civil = civil_from_timestamp(candle.timestamp, self.utc_offset_minutes);
        let key = (civil.year, civil.month, civil.day);
        match self.day {
            Some(prev) if prev == key => {
                self.cur_close = candle.close;
            }
            Some((year, month, dom)) => {
                self.roll_into(year, month, dom, key, candle.close);
            }
            None => {
                self.day = Some(key);
                self.cur_close = candle.close;
            }
        }
        self.value()
    }

    fn reset(&mut self) {
        self.day = None;
        self.cur_close = 0.0;
        self.prev_day_close = None;
        self.sum = 0.0;
        self.count = 0;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.count > 0
    }

    fn name(&self) -> &'static str {
        "TurnOfMonth"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    const DAY: i64 = 24 * 3_600_000;
    // 2021-01-28 00:00 UTC.
    const JAN28_2021: i64 = 1_611_792_000_000;

    fn c(close: f64, ts: i64) -> Candle {
        Candle::new(close, close, close, close, 1.0, ts).unwrap()
    }

    #[test]
    fn window_predicate_branches() {
        // First-days branch.
        assert!(in_turn_window(1, 31, 3, 1));
        assert!(in_turn_window(3, 31, 3, 1));
        assert!(!in_turn_window(4, 31, 3, 1));
        // Last-days branch.
        assert!(in_turn_window(31, 31, 3, 1));
        assert!(!in_turn_window(30, 31, 3, 1));
        // Saturating subtraction when n_last exceeds the month length.
        assert!(in_turn_window(1, 28, 0, 40));
    }

    #[test]
    fn rejects_empty_window() {
        assert!(matches!(TurnOfMonth::new(0, 0, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn metadata_and_accessors() {
        let tom = TurnOfMonth::classic();
        assert_eq!(tom.params(), (3, 1, 0));
        assert_eq!(tom.name(), "TurnOfMonth");
        assert_eq!(tom.warmup_period(), 2);
        assert!(!tom.is_ready());
        assert!(tom.value().is_none());
    }

    #[test]
    fn averages_in_window_returns_only() {
        let mut tom = TurnOfMonth::new(3, 1, 0).unwrap();
        // 2021-01-28 (out of window, no prior close): close 100.
        assert!(tom.update(c(100.0, JAN28_2021)).is_none());
        // 2021-01-29 (out of window: dom 29, dim 31 -> 29 <= 30): return ignored.
        assert!(tom.update(c(110.0, JAN28_2021 + DAY)).is_none());
        // 2021-01-30 (out of window): completes 01-29; still none.
        assert!(tom.update(c(120.0, JAN28_2021 + 2 * DAY)).is_none());
        // 2021-01-31 (last day, in window): completes 01-30 (out). Still none.
        assert!(tom.update(c(121.0, JAN28_2021 + 3 * DAY)).is_none());
        // 2021-02-01 (first day, in window): completes 01-31 (in window).
        // return = 121 / 120 - 1.
        let v = tom.update(c(130.0, JAN28_2021 + 4 * DAY)).unwrap();
        assert_relative_eq!(v, 121.0 / 120.0 - 1.0);
        assert!(tom.is_ready());
    }

    #[test]
    fn zero_prev_close_contributes_zero() {
        let mut tom = TurnOfMonth::new(3, 1, 0).unwrap();
        // 2021-01-30 closes at 0 — becomes the prior close for 01-31.
        tom.update(c(0.0, JAN28_2021 + 2 * DAY));
        // 2021-01-31 (last day, in window): finalizes 01-30 with no prior -> no
        // contribution, but records prev_day_close = 0.
        tom.update(c(5.0, JAN28_2021 + 3 * DAY));
        // 2021-02-01 (in window): finalizes 01-31 with prev_close 0 -> ret 0.
        let v = tom.update(c(50.0, JAN28_2021 + 4 * DAY)).unwrap();
        assert_relative_eq!(v, 0.0);
    }

    #[test]
    fn same_day_bars_use_latest_close() {
        let mut tom = TurnOfMonth::new(3, 1, 0).unwrap();
        // 2021-01-30 closes at 100 (prior day, sets prev_day_close).
        tom.update(c(100.0, JAN28_2021 + 2 * DAY));
        // 2021-01-31 two bars on the same day; the later close (120) wins.
        tom.update(c(110.0, JAN28_2021 + 3 * DAY));
        tom.update(c(120.0, JAN28_2021 + 3 * DAY + 3_600_000));
        // 2021-02-01 (in window) finalizes 01-31: return = 120 / 100 - 1 = 0.20.
        let v = tom.update(c(130.0, JAN28_2021 + 4 * DAY)).unwrap();
        assert_relative_eq!(v, 0.20);
    }

    #[test]
    fn reset_clears_state() {
        let mut tom = TurnOfMonth::new(3, 1, 0).unwrap();
        tom.update(c(121.0, JAN28_2021 + 3 * DAY));
        tom.update(c(130.0, JAN28_2021 + 4 * DAY));
        tom.reset();
        assert!(!tom.is_ready());
        assert!(tom.value().is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| c(100.0 + f64::from(i), JAN28_2021 + i64::from(i) * DAY))
            .collect();
        let mut a = TurnOfMonth::new(3, 2, 0).unwrap();
        let mut b = TurnOfMonth::new(3, 2, 0).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
