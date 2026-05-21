//! Roll trade ticks up into candles of an arbitrary timeframe.

use crate::error::{Error, Result};
use wickra_core::{Candle, Tick};

/// A candle bucket size measured in the same unit as the tick timestamps.
///
/// Wickra is unit-agnostic about timestamps: choose whichever makes sense for
/// your source (milliseconds for Binance trade events, microseconds for IB,
/// seconds for daily bars).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeframe {
    bucket: i64,
}

impl Timeframe {
    /// Construct a timeframe with the given bucket size in the chosen unit.
    ///
    /// # Errors
    /// Returns [`Error::InvalidTimeframe`] if `bucket <= 0`.
    pub fn new(bucket: i64) -> Result<Self> {
        if bucket <= 0 {
            return Err(Error::InvalidTimeframe(format!(
                "bucket size must be positive, got {bucket}"
            )));
        }
        Ok(Self { bucket })
    }

    /// Convenience: build a millisecond timeframe.
    pub fn millis(ms: i64) -> Result<Self> {
        Self::new(ms)
    }

    /// Convenience: build a seconds-resolution timeframe.
    pub fn seconds(s: i64) -> Result<Self> {
        Self::new(s)
    }

    /// One-minute timeframe in milliseconds (`60_000`).
    pub fn one_minute_ms() -> Self {
        Self::new(60_000).expect("60_000 > 0")
    }

    /// Bucket size.
    pub const fn bucket(self) -> i64 {
        self.bucket
    }

    /// Floor a raw timestamp to this timeframe's bucket boundary.
    pub fn floor(self, ts: i64) -> i64 {
        ts - ts.rem_euclid(self.bucket)
    }
}

/// Incrementally builds candles out of arriving ticks.
///
/// Each call to [`TickAggregator::push`] returns `Some(Candle)` if a previously
/// open bar just closed (i.e. the new tick belongs to a new bucket). Use
/// [`TickAggregator::flush`] at the end of a stream to capture the final open
/// bar.
#[derive(Debug, Clone)]
pub struct TickAggregator {
    timeframe: Timeframe,
    open_bar: Option<OpenBar>,
}

#[derive(Debug, Clone, Copy)]
struct OpenBar {
    bucket_start: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl OpenBar {
    fn from_tick(t: Tick, bucket_start: i64) -> Self {
        Self {
            bucket_start,
            open: t.price,
            high: t.price,
            low: t.price,
            close: t.price,
            volume: t.volume,
        }
    }

    fn absorb(&mut self, t: Tick) {
        if t.price > self.high {
            self.high = t.price;
        }
        if t.price < self.low {
            self.low = t.price;
        }
        self.close = t.price;
        self.volume += t.volume;
    }

    fn into_candle(self) -> Candle {
        Candle::new_unchecked(
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume,
            self.bucket_start,
        )
    }
}

impl TickAggregator {
    /// Construct a new aggregator for the given timeframe.
    pub fn new(timeframe: Timeframe) -> Self {
        Self {
            timeframe,
            open_bar: None,
        }
    }

    /// Push a tick. Returns `Some(Candle)` if a bar boundary was crossed and a
    /// previously open bar just closed.
    ///
    /// # Errors
    /// Returns an error if `tick.timestamp` is strictly less than the start of
    /// the currently open bar (out-of-order ticks are not supported).
    pub fn push(&mut self, tick: Tick) -> Result<Option<Candle>> {
        let bucket = self.timeframe.floor(tick.timestamp);
        if let Some(mut bar) = self.open_bar {
            if bucket < bar.bucket_start {
                return Err(Error::Malformed(format!(
                    "tick timestamp {} is older than the open bar start {}",
                    tick.timestamp, bar.bucket_start
                )));
            }
            if bucket > bar.bucket_start {
                // Close the previous bar and start a new one with this tick.
                self.open_bar = Some(OpenBar::from_tick(tick, bucket));
                return Ok(Some(bar.into_candle()));
            }
            bar.absorb(tick);
            self.open_bar = Some(bar);
            return Ok(None);
        }
        self.open_bar = Some(OpenBar::from_tick(tick, bucket));
        Ok(None)
    }

    /// Drain the currently open bar (if any) and return it. Useful at the end of
    /// a backtest or when shutting down a live aggregator.
    pub fn flush(&mut self) -> Option<Candle> {
        self.open_bar.take().map(OpenBar::into_candle)
    }

    /// Configured timeframe.
    pub const fn timeframe(&self) -> Timeframe {
        self.timeframe
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(price: f64, ts: i64) -> Tick {
        Tick::new(price, 1.0, ts).unwrap()
    }

    #[test]
    fn timeframe_rejects_non_positive() {
        assert!(Timeframe::new(0).is_err());
        assert!(Timeframe::new(-1).is_err());
    }

    #[test]
    fn floors_to_bucket_boundary() {
        let tf = Timeframe::new(100).unwrap();
        assert_eq!(tf.floor(0), 0);
        assert_eq!(tf.floor(99), 0);
        assert_eq!(tf.floor(100), 100);
        assert_eq!(tf.floor(150), 100);
        assert_eq!(tf.floor(250), 200);
    }

    #[test]
    fn aggregates_ticks_into_one_candle_within_bucket() {
        let mut agg = TickAggregator::new(Timeframe::new(60).unwrap());
        assert_eq!(agg.push(t(10.0, 0)).unwrap(), None);
        assert_eq!(agg.push(t(12.0, 15)).unwrap(), None);
        assert_eq!(agg.push(t(8.0, 30)).unwrap(), None);
        assert_eq!(agg.push(t(11.0, 50)).unwrap(), None);
        let bar = agg.flush().expect("open bar");
        assert_eq!(bar.open, 10.0);
        assert_eq!(bar.high, 12.0);
        assert_eq!(bar.low, 8.0);
        assert_eq!(bar.close, 11.0);
        assert!((bar.volume - 4.0).abs() < 1e-12);
        assert_eq!(bar.timestamp, 0);
    }

    #[test]
    fn emits_candle_on_bucket_crossing() {
        let mut agg = TickAggregator::new(Timeframe::new(60).unwrap());
        agg.push(t(10.0, 0)).unwrap();
        agg.push(t(12.0, 30)).unwrap();
        let closed = agg.push(t(15.0, 60)).unwrap().expect("emits");
        assert_eq!(closed.open, 10.0);
        assert_eq!(closed.high, 12.0);
        assert_eq!(closed.low, 10.0);
        assert_eq!(closed.close, 12.0);

        // The new tick at ts=60 opens the next bar.
        let still_open = agg.flush().unwrap();
        assert_eq!(still_open.open, 15.0);
        assert_eq!(still_open.timestamp, 60);
    }

    #[test]
    fn rejects_out_of_order_ticks() {
        let mut agg = TickAggregator::new(Timeframe::new(60).unwrap());
        agg.push(t(10.0, 100)).unwrap();
        let err = agg.push(t(11.0, 30)).unwrap_err();
        assert!(matches!(err, Error::Malformed(_)));
    }
}
