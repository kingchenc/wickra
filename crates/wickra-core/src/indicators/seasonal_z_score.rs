//! Seasonal Z-Score — how far the current bar's return sits from the historical
//! mean return of bars in the *same hour of day*, in standard deviations.

use crate::calendar::civil_from_timestamp;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

const HOURS: usize = 24;

/// Seasonal Z-Score keyed on hour of day.
///
/// For every bar the indicator forms the simple return `close / previous_close - 1`
/// and compares it to the running mean and standard deviation of all prior
/// returns that fell in the *same* local hour (the wall-clock hour of
/// [`Candle::timestamp`](crate::Candle) shifted by `utc_offset_minutes`). The
/// output is `(return - hour_mean) / hour_std`. A bucket needs at least two prior
/// samples before it can emit; a bucket with zero historical variance reports
/// `0.0`. The per-hour statistics use Welford's online algorithm.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, SeasonalZScore};
///
/// let day = 24 * 3_600_000;
/// let mut z = SeasonalZScore::new(0);
/// // Same hour each day so they share a bucket; close grows then jumps.
/// for (i, close) in [100.0, 101.0, 103.0].iter().enumerate() {
///     z.update(Candle::new(*close, *close, *close, *close, 1.0, i as i64 * day).unwrap());
/// }
/// // Fourth same-hour sample has two priors in the bucket -> emits a z-score.
/// let out = z.update(Candle::new(110.0, 110.0, 110.0, 110.0, 1.0, 3 * day).unwrap());
/// assert!(out.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct SeasonalZScore {
    utc_offset_minutes: i32,
    prev_close: Option<f64>,
    count: [u64; HOURS],
    mean: [f64; HOURS],
    m2: [f64; HOURS],
    last: Option<f64>,
}

impl SeasonalZScore {
    /// Construct a Seasonal Z-Score indicator with the given UTC offset (minutes).
    pub const fn new(utc_offset_minutes: i32) -> Self {
        Self {
            utc_offset_minutes,
            prev_close: None,
            count: [0; HOURS],
            mean: [0.0; HOURS],
            m2: [0.0; HOURS],
            last: None,
        }
    }

    /// Configured UTC offset in minutes.
    pub const fn utc_offset_minutes(&self) -> i32 {
        self.utc_offset_minutes
    }

    /// Most recent z-score if a populated bucket has produced one.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }

    fn z_for(&self, hour: usize, ret: f64) -> Option<f64> {
        if self.count[hour] < 2 {
            return None;
        }
        let variance = self.m2[hour] / (self.count[hour] - 1) as f64;
        if variance > 0.0 {
            Some((ret - self.mean[hour]) / variance.sqrt())
        } else {
            Some(0.0)
        }
    }

    fn accumulate(&mut self, hour: usize, ret: f64) {
        self.count[hour] += 1;
        let delta = ret - self.mean[hour];
        self.mean[hour] += delta / self.count[hour] as f64;
        let delta2 = ret - self.mean[hour];
        self.m2[hour] += delta * delta2;
    }
}

impl Indicator for SeasonalZScore {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let civil = civil_from_timestamp(candle.timestamp, self.utc_offset_minutes);
        let hour = civil.hour as usize;
        let result = if let Some(prev) = self.prev_close {
            let ret = if prev == 0.0 {
                0.0
            } else {
                candle.close / prev - 1.0
            };
            let z = self.z_for(hour, ret);
            self.accumulate(hour, ret);
            z
        } else {
            None
        };
        self.prev_close = Some(candle.close);
        if result.is_some() {
            self.last = result;
        }
        result
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.count = [0; HOURS];
        self.mean = [0.0; HOURS];
        self.m2 = [0.0; HOURS];
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "SeasonalZScore"
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
        let z = SeasonalZScore::new(120);
        assert_eq!(z.utc_offset_minutes(), 120);
        assert_eq!(z.name(), "SeasonalZScore");
        assert_eq!(z.warmup_period(), 2);
        assert!(!z.is_ready());
        assert!(z.value().is_none());
    }

    #[test]
    fn no_output_until_bucket_has_two_priors() {
        let mut z = SeasonalZScore::new(0);
        // Each bar shares the same hour bucket (same time-of-day, daily spacing).
        assert!(z.update(c(100.0, 0)).is_none()); // first: no return
        assert!(z.update(c(101.0, DAY)).is_none()); // return #1 -> bucket has 0 priors
        assert!(z.update(c(102.0, 2 * DAY)).is_none()); // return #2 -> bucket has 1 prior
                                                        // return #3 -> bucket has 2 priors -> emits.
        assert!(z.update(c(104.0, 3 * DAY)).is_some());
        assert!(z.is_ready());
    }

    #[test]
    fn z_score_matches_manual_welford() {
        let mut z = SeasonalZScore::new(0);
        // Returns into one hourly bucket: r1 = 0.01, r2 = 0.02, r3 = 0.03.
        z.update(c(100.0, 0));
        z.update(c(101.0, DAY)); // r1 = 0.01
        z.update(c(103.02, 2 * DAY)); // r2 = 0.02
                                      // Priors {0.01, 0.02}: mean 0.015, sample std = sqrt(((.005)^2*2)/1).
        let mean = 0.015;
        let std = (((0.01_f64 - mean).powi(2) + (0.02 - mean).powi(2)) / 1.0).sqrt();
        let r3 = 0.03;
        let expected = (r3 - mean) / std;
        let close = 103.02 * (1.0 + r3);
        let out = z.update(c(close, 3 * DAY)).unwrap();
        assert_relative_eq!(out, expected, epsilon = 1e-9);
    }

    #[test]
    fn zero_variance_bucket_reports_zero() {
        let mut z = SeasonalZScore::new(0);
        // Constant return into the bucket -> variance 0 -> z = 0.
        z.update(c(100.0, 0));
        z.update(c(110.0, DAY)); // r1 = 0.10
        z.update(c(121.0, 2 * DAY)); // r2 = 0.10
        let out = z.update(c(133.1, 3 * DAY)).unwrap(); // r3 = 0.10
        assert_relative_eq!(out, 0.0);
    }

    #[test]
    fn zero_prev_close_uses_zero_return() {
        let mut z = SeasonalZScore::new(0);
        z.update(c(0.0, 0)); // prev close 0
        z.update(c(0.0, DAY)); // ret = 0 (guarded), bucket sample
        z.update(c(0.0, 2 * DAY)); // ret = 0, bucket now 2 priors
        let out = z.update(c(0.0, 3 * DAY)).unwrap();
        assert_relative_eq!(out, 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut z = SeasonalZScore::new(0);
        for i in 0..4 {
            z.update(c(100.0 + f64::from(i), i64::from(i) * DAY));
        }
        z.reset();
        assert!(!z.is_ready());
        assert!(z.value().is_none());
        assert!(z.update(c(100.0, 4 * DAY)).is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..50)
            .map(|i| c(100.0 + f64::from(i % 9), i64::from(i) * 3 * 3_600_000))
            .collect();
        let mut a = SeasonalZScore::new(0);
        let mut b = SeasonalZScore::new(0);
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
