//! Session VWAP — the volume-weighted average price accumulated since the start
//! of the current calendar-day session, re-anchored automatically each day.

use crate::calendar::civil_from_timestamp;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Volume-weighted average price reset at each local day boundary.
///
/// Each bar contributes its typical price `(high + low + close) / 3` weighted by
/// volume. The running VWAP is `Σ(typical · volume) / Σ volume` over the current
/// session; if the session's volume is still zero the indicator falls back to the
/// latest typical price so the output is always finite. The session boundary is
/// the wall-clock day of [`Candle::timestamp`](crate::Candle) shifted by
/// `utc_offset_minutes`.
///
/// Where [`crate::RollingVwap`] averages over a fixed bar window and
/// [`crate::AnchoredVwap`] anchors at a caller-chosen bar, Session VWAP anchors
/// at the automatically detected day open.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, SessionVwap};
///
/// let hour = 3_600_000;
/// let mut vwap = SessionVwap::new(0);
/// // typical = 100, volume 10.
/// vwap.update(Candle::new(100.0, 100.0, 100.0, 100.0, 10.0, 0).unwrap());
/// // typical = 110, volume 30 -> VWAP = (100*10 + 110*30) / 40 = 107.5.
/// let v = vwap.update(Candle::new(110.0, 110.0, 110.0, 110.0, 30.0, hour).unwrap()).unwrap();
/// assert!((v - 107.5).abs() < 1e-9);
/// ```
#[derive(Debug, Clone)]
pub struct SessionVwap {
    utc_offset_minutes: i32,
    day_key: Option<(i64, u32, u32)>,
    cum_pv: f64,
    cum_volume: f64,
    last: Option<f64>,
}

impl SessionVwap {
    /// Construct a Session VWAP indicator with the given UTC offset (minutes).
    pub const fn new(utc_offset_minutes: i32) -> Self {
        Self {
            utc_offset_minutes,
            day_key: None,
            cum_pv: 0.0,
            cum_volume: 0.0,
            last: None,
        }
    }

    /// Configured UTC offset in minutes.
    pub const fn utc_offset_minutes(&self) -> i32 {
        self.utc_offset_minutes
    }

    /// Most recent VWAP if at least one bar has been seen.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for SessionVwap {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let civil = civil_from_timestamp(candle.timestamp, self.utc_offset_minutes);
        let key = (civil.year, civil.month, civil.day);
        if self.day_key != Some(key) {
            self.day_key = Some(key);
            self.cum_pv = 0.0;
            self.cum_volume = 0.0;
        }
        let typical = (candle.high + candle.low + candle.close) / 3.0;
        self.cum_pv += typical * candle.volume;
        self.cum_volume += candle.volume;
        let vwap = if self.cum_volume > 0.0 {
            self.cum_pv / self.cum_volume
        } else {
            typical
        };
        self.last = Some(vwap);
        Some(vwap)
    }

    fn reset(&mut self) {
        self.day_key = None;
        self.cum_pv = 0.0;
        self.cum_volume = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "SessionVwap"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    const HOUR: i64 = 3_600_000;

    fn c(price: f64, volume: f64, ts: i64) -> Candle {
        Candle::new(price, price, price, price, volume, ts).unwrap()
    }

    #[test]
    fn metadata_and_accessors() {
        let vwap = SessionVwap::new(-480);
        assert_eq!(vwap.utc_offset_minutes(), -480);
        assert_eq!(vwap.name(), "SessionVwap");
        assert_eq!(vwap.warmup_period(), 1);
        assert!(!vwap.is_ready());
        assert!(vwap.value().is_none());
    }

    #[test]
    fn volume_weights_the_average() {
        let mut vwap = SessionVwap::new(0);
        let first = vwap.update(c(100.0, 10.0, 0)).unwrap();
        assert_relative_eq!(first, 100.0);
        assert!(vwap.is_ready());
        let second = vwap.update(c(110.0, 30.0, HOUR)).unwrap();
        assert_relative_eq!(second, 107.5);
    }

    #[test]
    fn zero_volume_session_falls_back_to_typical() {
        let mut vwap = SessionVwap::new(0);
        let v = vwap.update(c(100.0, 0.0, 0)).unwrap();
        assert_relative_eq!(v, 100.0);
        let v2 = vwap.update(c(120.0, 0.0, HOUR)).unwrap();
        assert_relative_eq!(v2, 120.0);
    }

    #[test]
    fn re_anchors_on_new_day() {
        let mut vwap = SessionVwap::new(0);
        vwap.update(c(100.0, 10.0, 0));
        vwap.update(c(110.0, 30.0, HOUR));
        // New day: VWAP restarts from the first bar of day 2.
        let next = vwap.update(c(200.0, 5.0, 24 * HOUR)).unwrap();
        assert_relative_eq!(next, 200.0);
    }

    #[test]
    fn typical_price_uses_high_low_close() {
        let mut vwap = SessionVwap::new(0);
        // typical = (120 + 90 + 102) / 3 = 104.
        let candle = Candle::new(100.0, 120.0, 90.0, 102.0, 10.0, 0).unwrap();
        let v = vwap.update(candle).unwrap();
        assert_relative_eq!(v, 104.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut vwap = SessionVwap::new(0);
        vwap.update(c(100.0, 10.0, 0));
        vwap.reset();
        assert!(!vwap.is_ready());
        assert!(vwap.value().is_none());
        let after = vwap.update(c(50.0, 1.0, HOUR)).unwrap();
        assert_relative_eq!(after, 50.0);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                c(
                    100.0 + f64::from(i),
                    1.0 + f64::from(i % 4),
                    i64::from(i) * HOUR,
                )
            })
            .collect();
        let mut a = SessionVwap::new(0);
        let mut b = SessionVwap::new(0);
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
