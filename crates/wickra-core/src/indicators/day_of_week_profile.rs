//! Day-of-Week Profile — the mean bar return for each weekday.

use crate::calendar::civil_from_timestamp;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

const DAYS: usize = 7;

/// Day-of-Week Profile output: the per-weekday mean return.
///
/// `bins[i]` is the mean simple return of all bars whose local weekday was `i`,
/// with Monday as `0` through Sunday as `6`. Weekdays with no bars read `0.0`.
#[derive(Debug, Clone, PartialEq)]
pub struct DayOfWeekProfileOutput {
    /// Per-weekday mean return, Monday first. Always length 7.
    pub bins: Vec<f64>,
}

/// Mean bar return bucketed by local weekday (Monday `0` .. Sunday `6`).
///
/// Each bar's simple return `close / previous_close - 1` is accumulated into the
/// bucket of its local weekday (the wall-clock day of
/// [`Candle::timestamp`](crate::Candle) shifted by `utc_offset_minutes`), and the
/// profile reports the running mean per weekday. The first bar produces no output.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, DayOfWeekProfile};
///
/// let day = 24 * 3_600_000;
/// let mut prof = DayOfWeekProfile::new(0);
/// // 1970-01-01 was a Thursday (weekday 3).
/// assert!(prof.update(Candle::new(100.0, 100.0, 100.0, 100.0, 1.0, 0).unwrap()).is_none());
/// let out = prof.update(Candle::new(101.0, 101.0, 101.0, 101.0, 1.0, day).unwrap()).unwrap();
/// assert_eq!(out.bins.len(), 7);
/// ```
#[derive(Debug, Clone)]
pub struct DayOfWeekProfile {
    utc_offset_minutes: i32,
    prev_close: Option<f64>,
    sum: [f64; DAYS],
    count: [u64; DAYS],
    last: Option<DayOfWeekProfileOutput>,
}

impl DayOfWeekProfile {
    /// Construct a Day-of-Week Profile with the given UTC offset (minutes).
    pub const fn new(utc_offset_minutes: i32) -> Self {
        Self {
            utc_offset_minutes,
            prev_close: None,
            sum: [0.0; DAYS],
            count: [0; DAYS],
            last: None,
        }
    }

    /// Configured UTC offset in minutes.
    pub const fn utc_offset_minutes(&self) -> i32 {
        self.utc_offset_minutes
    }

    /// Most recent profile if at least one return has been recorded.
    pub fn value(&self) -> Option<&DayOfWeekProfileOutput> {
        self.last.as_ref()
    }

    fn snapshot(&self) -> DayOfWeekProfileOutput {
        let bins = self
            .sum
            .iter()
            .zip(&self.count)
            .map(|(total, n)| if *n > 0 { total / *n as f64 } else { 0.0 })
            .collect();
        DayOfWeekProfileOutput { bins }
    }
}

impl Indicator for DayOfWeekProfile {
    type Input = Candle;
    type Output = DayOfWeekProfileOutput;

    fn update(&mut self, candle: Candle) -> Option<DayOfWeekProfileOutput> {
        let civil = civil_from_timestamp(candle.timestamp, self.utc_offset_minutes);
        let result = if let Some(prev) = self.prev_close {
            let ret = if prev == 0.0 {
                0.0
            } else {
                candle.close / prev - 1.0
            };
            let day = civil.weekday as usize;
            self.sum[day] += ret;
            self.count[day] += 1;
            let out = self.snapshot();
            self.last = Some(out.clone());
            Some(out)
        } else {
            None
        };
        self.prev_close = Some(candle.close);
        result
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.sum = [0.0; DAYS];
        self.count = [0; DAYS];
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "DayOfWeekProfile"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    const DAY: i64 = 24 * 3_600_000;

    fn c(close: f64, ts: i64) -> Candle {
        Candle::new(close, close, close, close, 1.0, ts).unwrap()
    }

    #[test]
    fn metadata_and_accessors() {
        let prof = DayOfWeekProfile::new(60);
        assert_eq!(prof.utc_offset_minutes(), 60);
        assert_eq!(prof.name(), "DayOfWeekProfile");
        assert_eq!(prof.warmup_period(), 2);
        assert!(!prof.is_ready());
        assert!(prof.value().is_none());
    }

    #[test]
    fn buckets_by_weekday() {
        let mut prof = DayOfWeekProfile::new(0);
        // 1970-01-01 Thursday (3); 01-02 Friday (4).
        assert!(prof.update(c(100.0, 0)).is_none());
        let out = prof.update(c(101.0, DAY)).unwrap(); // Friday return +0.01
        assert_eq!(out.bins.len(), 7);
        assert_relative_eq!(out.bins[4], 0.01); // Friday
        assert_relative_eq!(out.bins[3], 0.0); // Thursday had no return
        assert!(prof.is_ready());
    }

    #[test]
    fn averages_same_weekday_across_weeks() {
        let mut prof = DayOfWeekProfile::new(0);
        prof.update(c(100.0, 0)); // Thu
        prof.update(c(101.0, DAY)); // Fri +0.01
                                    // Jump to next Friday (7 days later from day 0 -> +7 days, weekday 4).
        prof.update(c(100.0, 7 * DAY)); // Thu+? actually day 7 -> weekday (7+3)%7=3 Thu
        let out = prof.update(c(103.0, 8 * DAY)).unwrap(); // day 8 -> Fri, return
                                                           // Friday now has two samples; both positive.
        assert!(out.bins[4] > 0.0);
    }

    #[test]
    fn zero_prev_close_uses_zero_return() {
        let mut prof = DayOfWeekProfile::new(0);
        prof.update(c(0.0, 0));
        let out = prof.update(c(5.0, DAY)).unwrap();
        assert_relative_eq!(out.bins[4], 0.0); // Friday, guarded return 0
    }

    #[test]
    fn reset_clears_state() {
        let mut prof = DayOfWeekProfile::new(0);
        prof.update(c(100.0, 0));
        prof.update(c(101.0, DAY));
        prof.reset();
        assert!(!prof.is_ready());
        assert!(prof.value().is_none());
        assert!(prof.update(c(100.0, 2 * DAY)).is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..30)
            .map(|i| c(100.0 + f64::from(i % 5), i64::from(i) * DAY))
            .collect();
        let mut a = DayOfWeekProfile::new(0);
        let mut b = DayOfWeekProfile::new(0);
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
