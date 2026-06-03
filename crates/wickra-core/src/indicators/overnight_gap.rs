//! Overnight Gap — the return from the previous session's close to the current
//! session's open, detected automatically at each day boundary.

use crate::calendar::civil_from_timestamp;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Close-to-open overnight gap as a simple return.
///
/// At every local day boundary the indicator computes
/// `open / previous_close - 1`, where `previous_close` is the close of the last
/// bar of the prior session and `open` is the open of the first bar of the new
/// session. The value holds for the rest of the session until the next boundary.
/// The boundary is the wall-clock day of [`Candle::timestamp`](crate::Candle)
/// shifted by `utc_offset_minutes`. The first session yields no gap (there is no
/// prior close to compare against).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, OvernightGap};
///
/// let hour = 3_600_000;
/// let mut gap = OvernightGap::new(0);
/// // Day 1 closes at 100.
/// assert!(gap.update(Candle::new(99.0, 101.0, 98.0, 100.0, 1.0, 0).unwrap()).is_none());
/// // Day 2 opens at 105 -> gap = 105 / 100 - 1 = 0.05.
/// let g = gap.update(Candle::new(105.0, 106.0, 104.0, 105.5, 1.0, 24 * hour).unwrap()).unwrap();
/// assert!((g - 0.05).abs() < 1e-9);
/// ```
#[derive(Debug, Clone)]
pub struct OvernightGap {
    utc_offset_minutes: i32,
    day_key: Option<(i64, u32, u32)>,
    last_close: Option<f64>,
    gap: Option<f64>,
}

impl OvernightGap {
    /// Construct an Overnight Gap indicator with the given UTC offset (minutes).
    pub const fn new(utc_offset_minutes: i32) -> Self {
        Self {
            utc_offset_minutes,
            day_key: None,
            last_close: None,
            gap: None,
        }
    }

    /// Configured UTC offset in minutes.
    pub const fn utc_offset_minutes(&self) -> i32 {
        self.utc_offset_minutes
    }

    /// Most recent overnight gap if at least one day boundary has been crossed.
    pub const fn value(&self) -> Option<f64> {
        self.gap
    }
}

impl Indicator for OvernightGap {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let civil = civil_from_timestamp(candle.timestamp, self.utc_offset_minutes);
        let key = (civil.year, civil.month, civil.day);
        if self.day_key != Some(key) {
            if let Some(prev_close) = self.last_close {
                self.gap = Some(if prev_close == 0.0 {
                    0.0
                } else {
                    candle.open / prev_close - 1.0
                });
            }
            self.day_key = Some(key);
        }
        self.last_close = Some(candle.close);
        self.gap
    }

    fn reset(&mut self) {
        self.day_key = None;
        self.last_close = None;
        self.gap = None;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.gap.is_some()
    }

    fn name(&self) -> &'static str {
        "OvernightGap"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    const HOUR: i64 = 3_600_000;

    fn c(open: f64, close: f64, ts: i64) -> Candle {
        let high = open.max(close);
        let low = open.min(close);
        Candle::new(open, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn metadata_and_accessors() {
        let gap = OvernightGap::new(330);
        assert_eq!(gap.utc_offset_minutes(), 330);
        assert_eq!(gap.name(), "OvernightGap");
        assert_eq!(gap.warmup_period(), 2);
        assert!(!gap.is_ready());
        assert!(gap.value().is_none());
    }

    #[test]
    fn first_session_has_no_gap() {
        let mut gap = OvernightGap::new(0);
        assert!(gap.update(c(99.0, 100.0, 0)).is_none());
        // Same day, still no gap.
        assert!(gap.update(c(100.0, 101.0, HOUR)).is_none());
        assert!(!gap.is_ready());
    }

    #[test]
    fn computes_gap_at_day_boundary() {
        let mut gap = OvernightGap::new(0);
        gap.update(c(99.0, 100.0, 0)); // day 1 closes 100
        let g = gap.update(c(105.0, 105.5, 24 * HOUR)).unwrap();
        assert_relative_eq!(g, 0.05);
        assert!(gap.is_ready());
        // Holds for the rest of the session.
        let same = gap.update(c(106.0, 107.0, 25 * HOUR)).unwrap();
        assert_relative_eq!(same, 0.05);
    }

    #[test]
    fn negative_gap_down() {
        let mut gap = OvernightGap::new(0);
        gap.update(c(99.0, 100.0, 0));
        let g = gap.update(c(90.0, 91.0, 24 * HOUR)).unwrap();
        assert_relative_eq!(g, -0.1);
    }

    #[test]
    fn zero_prev_close_yields_zero_gap() {
        let mut gap = OvernightGap::new(0);
        gap.update(c(0.0, 0.0, 0)); // degenerate day 1 closing at 0
        let g = gap.update(c(5.0, 6.0, 24 * HOUR)).unwrap();
        assert_relative_eq!(g, 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut gap = OvernightGap::new(0);
        gap.update(c(99.0, 100.0, 0));
        gap.update(c(105.0, 105.5, 24 * HOUR));
        gap.reset();
        assert!(!gap.is_ready());
        assert!(gap.value().is_none());
        assert!(gap.update(c(10.0, 11.0, 48 * HOUR)).is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..50)
            .map(|i| {
                c(
                    100.0 + f64::from(i % 7),
                    100.0 + f64::from(i % 5),
                    i64::from(i) * 6 * HOUR,
                )
            })
            .collect();
        let mut a = OvernightGap::new(0);
        let mut b = OvernightGap::new(0);
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
