//! Session Range — the high-minus-low range accumulated within each of the
//! three canonical trading sessions (Asia / EU / US) of the current day.

use crate::calendar::civil_from_timestamp;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Session Range output: the current day's range within each session.
///
/// A session with no bars yet reports `0.0`. All three reset at the local day
/// boundary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionRangeOutput {
    /// High − low within the Asia session (local hours `00:00..08:00`).
    pub asia: f64,
    /// High − low within the EU session (local hours `08:00..16:00`).
    pub eu: f64,
    /// High − low within the US session (local hours `16:00..24:00`).
    pub us: f64,
}

#[derive(Debug, Clone, Copy)]
struct Extent {
    high: f64,
    low: f64,
}

impl Extent {
    const EMPTY: Self = Self {
        high: f64::NEG_INFINITY,
        low: f64::INFINITY,
    };

    fn add(&mut self, candle: Candle) {
        if candle.high > self.high {
            self.high = candle.high;
        }
        if candle.low < self.low {
            self.low = candle.low;
        }
    }

    fn range(self) -> f64 {
        if self.high >= self.low {
            self.high - self.low
        } else {
            0.0
        }
    }
}

/// Per-session high-low range, keyed off the wall-clock hour of
/// [`Candle::timestamp`](crate::Candle).
///
/// The local day (after shifting by `utc_offset_minutes`) is split into three
/// eight-hour sessions: **Asia** `00:00..08:00`, **EU** `08:00..16:00`, **US**
/// `16:00..24:00`. Each session accumulates its own high / low; the reported
/// range is `high - low`, or `0.0` before that session has seen a bar. All three
/// re-anchor automatically at the day boundary.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, SessionRange};
///
/// let hour = 3_600_000;
/// let mut sr = SessionRange::new(0);
/// // 02:00 UTC — Asia session.
/// sr.update(Candle::new(100.0, 104.0, 98.0, 101.0, 1.0, 2 * hour).unwrap());
/// // 10:00 UTC — EU session.
/// let v = sr.update(Candle::new(101.0, 110.0, 100.0, 109.0, 1.0, 10 * hour).unwrap()).unwrap();
/// assert_eq!(v.asia, 6.0);
/// assert_eq!(v.eu, 10.0);
/// assert_eq!(v.us, 0.0);
/// ```
#[derive(Debug, Clone)]
pub struct SessionRange {
    utc_offset_minutes: i32,
    day_key: Option<(i64, u32, u32)>,
    sessions: [Extent; 3],
    last: Option<SessionRangeOutput>,
}

impl SessionRange {
    /// Construct a Session Range indicator with the given UTC offset (minutes).
    pub const fn new(utc_offset_minutes: i32) -> Self {
        Self {
            utc_offset_minutes,
            day_key: None,
            sessions: [Extent::EMPTY; 3],
            last: None,
        }
    }

    /// Configured UTC offset in minutes.
    pub const fn utc_offset_minutes(&self) -> i32 {
        self.utc_offset_minutes
    }

    /// Most recent output if at least one bar has been seen.
    pub const fn value(&self) -> Option<SessionRangeOutput> {
        self.last
    }

    fn snapshot(&self) -> SessionRangeOutput {
        SessionRangeOutput {
            asia: self.sessions[0].range(),
            eu: self.sessions[1].range(),
            us: self.sessions[2].range(),
        }
    }
}

impl Indicator for SessionRange {
    type Input = Candle;
    type Output = SessionRangeOutput;

    fn update(&mut self, candle: Candle) -> Option<SessionRangeOutput> {
        let civil = civil_from_timestamp(candle.timestamp, self.utc_offset_minutes);
        let key = (civil.year, civil.month, civil.day);
        if self.day_key != Some(key) {
            self.day_key = Some(key);
            self.sessions = [Extent::EMPTY; 3];
        }
        let session = (civil.hour / 8) as usize; // 0 Asia, 1 EU, 2 US
        self.sessions[session].add(candle);
        let out = self.snapshot();
        self.last = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.day_key = None;
        self.sessions = [Extent::EMPTY; 3];
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "SessionRange"
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
        let sr = SessionRange::new(60);
        assert_eq!(sr.utc_offset_minutes(), 60);
        assert_eq!(sr.name(), "SessionRange");
        assert_eq!(sr.warmup_period(), 1);
        assert!(!sr.is_ready());
        assert!(sr.value().is_none());
    }

    #[test]
    fn assigns_bars_to_sessions() {
        let mut sr = SessionRange::new(0);
        let asia = sr.update(c(104.0, 98.0, 2 * HOUR)).unwrap();
        assert_relative_eq!(asia.asia, 6.0);
        assert_relative_eq!(asia.eu, 0.0);
        assert_relative_eq!(asia.us, 0.0);
        assert!(sr.is_ready());
        let eu = sr.update(c(110.0, 100.0, 10 * HOUR)).unwrap();
        assert_relative_eq!(eu.eu, 10.0);
        let us = sr.update(c(120.0, 118.0, 20 * HOUR)).unwrap();
        assert_relative_eq!(us.us, 2.0);
        assert_relative_eq!(us.asia, 6.0);
    }

    #[test]
    fn widens_within_one_session() {
        let mut sr = SessionRange::new(0);
        sr.update(c(104.0, 98.0, HOUR));
        let wider = sr.update(c(106.0, 95.0, 3 * HOUR)).unwrap();
        assert_relative_eq!(wider.asia, 11.0);
    }

    #[test]
    fn resets_sessions_on_new_day() {
        let mut sr = SessionRange::new(0);
        sr.update(c(104.0, 98.0, 2 * HOUR));
        sr.update(c(110.0, 100.0, 10 * HOUR));
        let next = sr.update(c(101.0, 99.0, (24 + 2) * HOUR)).unwrap();
        assert_relative_eq!(next.asia, 2.0);
        assert_relative_eq!(next.eu, 0.0);
    }

    #[test]
    fn utc_offset_moves_bar_between_sessions() {
        // 07:00 UTC is Asia; shifted +120 min it becomes 09:00 -> EU.
        let mut utc = SessionRange::new(0);
        let a = utc.update(c(104.0, 98.0, 7 * HOUR)).unwrap();
        assert_relative_eq!(a.asia, 6.0);
        assert_relative_eq!(a.eu, 0.0);

        let mut shifted = SessionRange::new(120);
        let e = shifted.update(c(104.0, 98.0, 7 * HOUR)).unwrap();
        assert_relative_eq!(e.asia, 0.0);
        assert_relative_eq!(e.eu, 6.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut sr = SessionRange::new(0);
        sr.update(c(104.0, 98.0, 2 * HOUR));
        sr.reset();
        assert!(!sr.is_ready());
        assert!(sr.value().is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                c(
                    100.0 + f64::from(i % 5),
                    95.0 - f64::from(i % 3),
                    i64::from(i) * HOUR,
                )
            })
            .collect();
        let mut a = SessionRange::new(0);
        let mut b = SessionRange::new(0);
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
