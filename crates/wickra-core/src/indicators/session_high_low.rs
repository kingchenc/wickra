//! Session High/Low — the running high and low of the current calendar-day
//! session, re-anchored automatically at each day boundary.

use crate::calendar::civil_from_timestamp;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Session High/Low output: the high and low established so far in the current
/// session.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionHighLowOutput {
    /// Highest high seen since the current session opened.
    pub high: f64,
    /// Lowest low seen since the current session opened.
    pub low: f64,
}

/// Running high / low of the current session, keyed off the wall-clock day of
/// [`Candle::timestamp`](crate::Candle).
///
/// Unlike [`crate::OpeningRange`] or [`crate::InitialBalance`], which require the
/// caller to invoke `reset()` at every session boundary, this indicator detects
/// the boundary itself: whenever a candle falls on a different local calendar
/// day (after shifting by `utc_offset_minutes`) the high / low are re-anchored to
/// that candle. `utc_offset_minutes` lets callers align the day boundary to an
/// exchange session — `0` for UTC, `-300` for U.S. Eastern standard time.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, SessionHighLow};
///
/// // One bar per hour; the day rolls over after 24 bars at UTC.
/// let mut shl = SessionHighLow::new(0);
/// let hour = 3_600_000;
/// shl.update(Candle::new(100.0, 105.0, 99.0, 101.0, 1.0, 0).unwrap());
/// let v = shl.update(Candle::new(101.0, 108.0, 100.0, 107.0, 1.0, hour).unwrap()).unwrap();
/// assert_eq!(v.high, 108.0);
/// assert_eq!(v.low, 99.0);
/// // A bar on the next day re-anchors to that bar alone.
/// let v = shl.update(Candle::new(50.0, 51.0, 49.0, 50.0, 1.0, 24 * hour).unwrap()).unwrap();
/// assert_eq!(v.high, 51.0);
/// assert_eq!(v.low, 49.0);
/// ```
#[derive(Debug, Clone)]
pub struct SessionHighLow {
    utc_offset_minutes: i32,
    day_key: Option<(i64, u32, u32)>,
    high: f64,
    low: f64,
    last: Option<SessionHighLowOutput>,
}

impl SessionHighLow {
    /// Construct a Session High/Low indicator with the given UTC offset (minutes).
    pub const fn new(utc_offset_minutes: i32) -> Self {
        Self {
            utc_offset_minutes,
            day_key: None,
            high: f64::NEG_INFINITY,
            low: f64::INFINITY,
            last: None,
        }
    }

    /// Configured UTC offset in minutes.
    pub const fn utc_offset_minutes(&self) -> i32 {
        self.utc_offset_minutes
    }

    /// Most recent output if at least one bar has been seen.
    pub const fn value(&self) -> Option<SessionHighLowOutput> {
        self.last
    }
}

impl Indicator for SessionHighLow {
    type Input = Candle;
    type Output = SessionHighLowOutput;

    fn update(&mut self, candle: Candle) -> Option<SessionHighLowOutput> {
        let civil = civil_from_timestamp(candle.timestamp, self.utc_offset_minutes);
        let key = (civil.year, civil.month, civil.day);
        if self.day_key == Some(key) {
            if candle.high > self.high {
                self.high = candle.high;
            }
            if candle.low < self.low {
                self.low = candle.low;
            }
        } else {
            self.day_key = Some(key);
            self.high = candle.high;
            self.low = candle.low;
        }
        let out = SessionHighLowOutput {
            high: self.high,
            low: self.low,
        };
        self.last = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.day_key = None;
        self.high = f64::NEG_INFINITY;
        self.low = f64::INFINITY;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "SessionHighLow"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    const HOUR: i64 = 3_600_000;

    fn c(high: f64, low: f64, ts: i64) -> Candle {
        let mid = f64::midpoint(high, low);
        Candle::new(mid, high, low, mid, 1.0, ts).unwrap()
    }

    #[test]
    fn metadata_and_accessors() {
        let shl = SessionHighLow::new(-300);
        assert_eq!(shl.utc_offset_minutes(), -300);
        assert_eq!(shl.name(), "SessionHighLow");
        assert_eq!(shl.warmup_period(), 1);
        assert!(!shl.is_ready());
        assert!(shl.value().is_none());
    }

    #[test]
    fn tracks_high_low_within_day() {
        let mut shl = SessionHighLow::new(0);
        let first = shl.update(c(105.0, 99.0, 0)).unwrap();
        assert_relative_eq!(first.high, 105.0);
        assert_relative_eq!(first.low, 99.0);
        assert!(shl.is_ready());
        let second = shl.update(c(108.0, 100.0, HOUR)).unwrap();
        assert_relative_eq!(second.high, 108.0);
        assert_relative_eq!(second.low, 99.0);
        // A narrower bar does not shrink the range.
        let third = shl.update(c(106.0, 101.0, 2 * HOUR)).unwrap();
        assert_relative_eq!(third.high, 108.0);
        assert_relative_eq!(third.low, 99.0);
        // A bar with a lower low extends the range downward (same day).
        let fourth = shl.update(c(107.0, 95.0, 3 * HOUR)).unwrap();
        assert_relative_eq!(fourth.high, 108.0);
        assert_relative_eq!(fourth.low, 95.0);
    }

    #[test]
    fn re_anchors_on_new_day() {
        let mut shl = SessionHighLow::new(0);
        shl.update(c(105.0, 99.0, 0));
        shl.update(c(108.0, 100.0, HOUR));
        let next = shl.update(c(51.0, 49.0, 24 * HOUR)).unwrap();
        assert_relative_eq!(next.high, 51.0);
        assert_relative_eq!(next.low, 49.0);
    }

    #[test]
    fn utc_offset_shifts_day_boundary() {
        // Two bars 1h apart straddling UTC midnight. At UTC they are different
        // days; at +120 min they fall on the same local day.
        let pre = 23 * HOUR; // 1970-01-01 23:00 UTC
        let post = 24 * HOUR; // 1970-01-02 00:00 UTC
        let mut utc = SessionHighLow::new(0);
        utc.update(c(105.0, 99.0, pre));
        let rolled = utc.update(c(108.0, 100.0, post)).unwrap();
        assert_relative_eq!(rolled.high, 108.0);
        assert_relative_eq!(rolled.low, 100.0); // re-anchored

        let mut shifted = SessionHighLow::new(120);
        shifted.update(c(105.0, 99.0, pre));
        let same = shifted.update(c(108.0, 100.0, post)).unwrap();
        assert_relative_eq!(same.high, 108.0);
        assert_relative_eq!(same.low, 99.0); // same local day, range kept
    }

    #[test]
    fn reset_clears_state() {
        let mut shl = SessionHighLow::new(0);
        shl.update(c(105.0, 99.0, 0));
        shl.reset();
        assert!(!shl.is_ready());
        assert!(shl.value().is_none());
        let after = shl.update(c(60.0, 50.0, HOUR)).unwrap();
        assert_relative_eq!(after.high, 60.0);
        assert_relative_eq!(after.low, 50.0);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                c(
                    100.0 + f64::from(i),
                    90.0 + f64::from(i) * 0.5,
                    i64::from(i) * HOUR,
                )
            })
            .collect();
        let mut a = SessionHighLow::new(0);
        let mut b = SessionHighLow::new(0);
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
