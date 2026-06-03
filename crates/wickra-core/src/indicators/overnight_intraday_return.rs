//! Overnight vs. Intraday Return — decomposes a session's total return into its
//! overnight (close-to-open) and intraday (open-to-close) components.

use crate::calendar::civil_from_timestamp;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// The two return components of the current session.
///
/// `overnight` is fixed at the session open (`open / previous_close - 1`);
/// `intraday` updates with every bar (`close / open - 1`). Compounding the two —
/// `(1 + overnight)(1 + intraday) - 1` — reconstructs the full previous-close to
/// latest-close return.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OvernightIntradayReturnOutput {
    /// Close-to-open return carried into the session.
    pub overnight: f64,
    /// Open-to-latest-close return accumulated within the session.
    pub intraday: f64,
}

/// Overnight / intraday return decomposition, re-anchored at each local day
/// boundary of [`Candle::timestamp`](crate::Candle) shifted by
/// `utc_offset_minutes`.
///
/// The first session yields no output (there is no prior close to anchor the
/// overnight leg); from the second session onward every bar reports both
/// components.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, OvernightIntradayReturn};
///
/// let hour = 3_600_000;
/// let mut oi = OvernightIntradayReturn::new(0);
/// // Day 1 closes at 100.
/// assert!(oi.update(Candle::new(99.0, 101.0, 98.0, 100.0, 1.0, 0).unwrap()).is_none());
/// // Day 2 opens 110 (overnight +10%), closes 121 (intraday +10%).
/// let v = oi.update(Candle::new(110.0, 122.0, 109.0, 121.0, 1.0, 24 * hour).unwrap()).unwrap();
/// assert!((v.overnight - 0.10).abs() < 1e-9);
/// assert!((v.intraday - 0.10).abs() < 1e-9);
/// ```
#[derive(Debug, Clone)]
pub struct OvernightIntradayReturn {
    utc_offset_minutes: i32,
    day_key: Option<(i64, u32, u32)>,
    last_close: Option<f64>,
    today_open: f64,
    overnight: Option<f64>,
    last: Option<OvernightIntradayReturnOutput>,
}

impl OvernightIntradayReturn {
    /// Construct the indicator with the given UTC offset (minutes).
    pub const fn new(utc_offset_minutes: i32) -> Self {
        Self {
            utc_offset_minutes,
            day_key: None,
            last_close: None,
            today_open: 0.0,
            overnight: None,
            last: None,
        }
    }

    /// Configured UTC offset in minutes.
    pub const fn utc_offset_minutes(&self) -> i32 {
        self.utc_offset_minutes
    }

    /// Most recent decomposition if at least one day boundary has been crossed.
    pub const fn value(&self) -> Option<OvernightIntradayReturnOutput> {
        self.last
    }
}

impl Indicator for OvernightIntradayReturn {
    type Input = Candle;
    type Output = OvernightIntradayReturnOutput;

    fn update(&mut self, candle: Candle) -> Option<OvernightIntradayReturnOutput> {
        let civil = civil_from_timestamp(candle.timestamp, self.utc_offset_minutes);
        let key = (civil.year, civil.month, civil.day);
        if self.day_key != Some(key) {
            if let Some(prev_close) = self.last_close {
                self.overnight = Some(if prev_close == 0.0 {
                    0.0
                } else {
                    candle.open / prev_close - 1.0
                });
            }
            self.today_open = candle.open;
            self.day_key = Some(key);
        }
        self.last_close = Some(candle.close);
        let overnight = self.overnight?;
        let intraday = if self.today_open == 0.0 {
            0.0
        } else {
            candle.close / self.today_open - 1.0
        };
        let out = OvernightIntradayReturnOutput {
            overnight,
            intraday,
        };
        self.last = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.day_key = None;
        self.last_close = None;
        self.today_open = 0.0;
        self.overnight = None;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "OvernightIntradayReturn"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    const HOUR: i64 = 3_600_000;

    fn c(open: f64, close: f64, ts: i64) -> Candle {
        let high = open.max(close) + 1.0;
        let low = open.min(close) - 1.0;
        Candle::new(open, high, low.max(0.0), close, 1.0, ts).unwrap()
    }

    #[test]
    fn metadata_and_accessors() {
        let oi = OvernightIntradayReturn::new(-300);
        assert_eq!(oi.utc_offset_minutes(), -300);
        assert_eq!(oi.name(), "OvernightIntradayReturn");
        assert_eq!(oi.warmup_period(), 2);
        assert!(!oi.is_ready());
        assert!(oi.value().is_none());
    }

    #[test]
    fn first_session_yields_none() {
        let mut oi = OvernightIntradayReturn::new(0);
        assert!(oi.update(c(99.0, 100.0, 0)).is_none());
        assert!(oi.update(c(100.0, 102.0, HOUR)).is_none());
        assert!(!oi.is_ready());
    }

    #[test]
    fn decomposes_overnight_and_intraday() {
        let mut oi = OvernightIntradayReturn::new(0);
        oi.update(c(99.0, 100.0, 0)); // day 1 close 100
        let v = oi.update(c(110.0, 121.0, 24 * HOUR)).unwrap();
        assert_relative_eq!(v.overnight, 0.10);
        assert_relative_eq!(v.intraday, 0.10);
        assert!(oi.is_ready());
    }

    #[test]
    fn intraday_updates_through_the_session() {
        let mut oi = OvernightIntradayReturn::new(0);
        oi.update(c(99.0, 100.0, 0));
        oi.update(c(110.0, 110.0, 24 * HOUR)); // open 110, close 110 -> intraday 0
        let later = oi.update(c(111.0, 132.0, 25 * HOUR)).unwrap();
        assert_relative_eq!(later.overnight, 0.10); // fixed at open
        assert_relative_eq!(later.intraday, 0.20); // 132 / 110 - 1
    }

    #[test]
    fn zero_anchors_yield_zero_components() {
        let mut oi = OvernightIntradayReturn::new(0);
        oi.update(c(1.0, 0.0, 0)); // day 1 closes at 0
                                   // Day 2 opens at 0: overnight uses zero prev_close -> 0; intraday uses
                                   // zero today_open -> 0.
        let candle = Candle::new(0.0, 5.0, 0.0, 4.0, 1.0, 24 * HOUR).unwrap();
        let v = oi.update(candle).unwrap();
        assert_relative_eq!(v.overnight, 0.0);
        assert_relative_eq!(v.intraday, 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut oi = OvernightIntradayReturn::new(0);
        oi.update(c(99.0, 100.0, 0));
        oi.update(c(110.0, 121.0, 24 * HOUR));
        oi.reset();
        assert!(!oi.is_ready());
        assert!(oi.value().is_none());
        assert!(oi.update(c(50.0, 55.0, 48 * HOUR)).is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..48)
            .map(|i| {
                c(
                    100.0 + f64::from(i % 6),
                    100.0 + f64::from(i % 4),
                    i64::from(i) * 8 * HOUR,
                )
            })
            .collect();
        let mut a = OvernightIntradayReturn::new(0);
        let mut b = OvernightIntradayReturn::new(0);
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
