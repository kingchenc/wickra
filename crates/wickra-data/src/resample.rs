//! Resample an existing candle stream from a finer timeframe to a coarser one.

use crate::aggregator::Timeframe;
use crate::error::{Error, Result};
use wickra_core::Candle;

/// Roll a stream of candles up to a coarser timeframe.
///
/// Used to derive 5m bars from a 1m feed, or 1h bars from 5m bars, without
/// touching the original tick stream. The output timeframe's bucket must be a
/// strict multiple of the input timeframe's bucket, but this is not enforced
/// — callers are responsible for picking sensible aggregations.
#[derive(Debug, Clone)]
pub struct Resampler {
    timeframe: Timeframe,
    open: Option<RolledBar>,
}

#[derive(Debug, Clone, Copy)]
struct RolledBar {
    bucket_start: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl RolledBar {
    fn from_candle(c: Candle, bucket_start: i64) -> Self {
        Self {
            bucket_start,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        }
    }

    fn absorb(&mut self, c: Candle) {
        if c.high > self.high {
            self.high = c.high;
        }
        if c.low < self.low {
            self.low = c.low;
        }
        self.close = c.close;
        self.volume += c.volume;
    }

    /// Finalise the rolled bar into a validated [`Candle`].
    ///
    /// # Errors
    /// Returns [`Error::Core`] if the accumulated `volume` is no longer finite.
    /// `volume` is summed across every absorbed candle, so a long or large run
    /// can drift it to `inf`; emitting such a candle would silently poison
    /// every downstream indicator, so it is surfaced instead. The OHLC fields
    /// are finite and correctly ordered by construction.
    fn into_candle(self) -> Result<Candle> {
        Candle::new(
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume,
            self.bucket_start,
        )
        .map_err(Error::from)
    }
}

impl Resampler {
    /// Build a resampler targeting the given output timeframe.
    pub fn new(timeframe: Timeframe) -> Self {
        Self {
            timeframe,
            open: None,
        }
    }

    /// Push a finer-grained candle. Returns the coarser candle that just closed,
    /// if any.
    ///
    /// # Errors
    /// Returns [`Error::Malformed`] if `candle.timestamp` falls into a bucket
    /// strictly before the currently open bar — out-of-order candles are not
    /// supported, matching [`crate::aggregator::TickAggregator::push`].
    pub fn push(&mut self, candle: Candle) -> Result<Option<Candle>> {
        let bucket = self.timeframe.floor(candle.timestamp);
        match self.open {
            Some(mut bar) if bucket == bar.bucket_start => {
                bar.absorb(candle);
                self.open = Some(bar);
                Ok(None)
            }
            Some(bar) if bucket > bar.bucket_start => {
                let closed = bar.into_candle()?;
                self.open = Some(RolledBar::from_candle(candle, bucket));
                Ok(Some(closed))
            }
            Some(bar) => Err(Error::Malformed(format!(
                "candle timestamp {} is older than the open bar start {}",
                candle.timestamp, bar.bucket_start
            ))),
            None => {
                self.open = Some(RolledBar::from_candle(candle, bucket));
                Ok(None)
            }
        }
    }

    /// Flush the currently open coarser bar, if any.
    ///
    /// # Errors
    /// Returns an error if the open bar's accumulated volume is non-finite
    /// (see the internal `RolledBar::into_candle`).
    pub fn flush(&mut self) -> Result<Option<Candle>> {
        self.open.take().map(RolledBar::into_candle).transpose()
    }
}

/// Roll an entire iterator of candles into a `Vec` of coarser candles. The final
/// open bar (if any) is appended via [`Resampler::flush`].
pub fn resample_all<I>(timeframe: Timeframe, iter: I) -> Result<Vec<Candle>>
where
    I: IntoIterator<Item = Result<Candle>>,
{
    let mut r = Resampler::new(timeframe);
    let mut out = Vec::new();
    for c in iter {
        let c = c?;
        if let Some(closed) = r.push(c)? {
            out.push(closed);
        }
    }
    if let Some(last) = r.flush()? {
        out.push(last);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(ts: i64, o: f64, h: f64, l: f64, cl: f64, v: f64) -> Candle {
        Candle::new(o, h, l, cl, v, ts).unwrap()
    }

    #[test]
    fn resamples_1m_to_5m() {
        let tf = Timeframe::new(5).unwrap();
        let one_m = vec![
            c(0, 10.0, 11.0, 9.0, 10.5, 10.0),
            c(1, 10.5, 12.0, 10.0, 11.5, 12.0),
            c(2, 11.5, 13.0, 11.0, 12.5, 15.0),
            c(3, 12.5, 12.8, 11.5, 12.0, 8.0),
            c(4, 12.0, 12.2, 11.0, 11.5, 6.0),
            c(5, 11.5, 11.9, 11.0, 11.5, 4.0),
        ];
        let rolled = resample_all(tf, one_m.into_iter().map(Ok)).unwrap();
        // First 5 candles share bucket 0 -> aggregate. Last candle opens bucket 5.
        assert_eq!(rolled.len(), 2);
        let a = rolled[0];
        assert_eq!(a.open, 10.0);
        assert_eq!(a.close, 11.5);
        assert_eq!(a.high, 13.0);
        assert_eq!(a.low, 9.0);
        assert!((a.volume - 51.0).abs() < 1e-12);
        let b = rolled[1];
        assert_eq!(b.open, 11.5);
        assert_eq!(b.timestamp, 5);
    }

    #[test]
    fn rejects_out_of_order_candle() {
        let mut r = Resampler::new(Timeframe::new(5).unwrap());
        assert!(r.push(c(10, 10.0, 11.0, 9.0, 10.5, 1.0)).unwrap().is_none());
        // A candle in an earlier bucket than the open bar is rejected.
        let err = r.push(c(2, 10.0, 11.0, 9.0, 10.5, 1.0)).unwrap_err();
        assert!(matches!(err, Error::Malformed(_)));
    }

    #[test]
    fn same_bucket_candles_aggregate() {
        let mut r = Resampler::new(Timeframe::new(5).unwrap());
        assert!(r.push(c(0, 10.0, 11.0, 9.0, 10.5, 1.0)).unwrap().is_none());
        assert!(r.push(c(3, 10.5, 12.0, 10.0, 11.0, 1.0)).unwrap().is_none());
        let bar = r.flush().unwrap().unwrap();
        assert_eq!(bar.high, 12.0);
        assert_eq!(bar.low, 9.0);
    }

    #[test]
    fn absorb_lowers_low_on_dipping_candle() {
        // The first candle in the bucket sets low = 10.0; the second dips to
        // 8.0 and must overwrite. Exercises the `c.low < self.low` branch in
        // RolledBar::absorb that the other resampler tests never trigger
        // because their follow-up candles always have a higher low.
        let mut r = Resampler::new(Timeframe::new(5).unwrap());
        r.push(c(0, 10.0, 11.0, 10.0, 10.5, 1.0)).unwrap();
        r.push(c(1, 10.5, 11.5, 8.0, 9.0, 1.0)).unwrap();
        let bar = r.flush().unwrap().unwrap();
        assert_eq!(bar.low, 8.0);
        assert_eq!(bar.high, 11.5);
    }

    #[test]
    fn flushes_a_non_finite_volume_as_an_error() {
        let mut r = Resampler::new(Timeframe::new(5).unwrap());
        // Two near-max volumes in the same bucket sum to +inf.
        assert!(r
            .push(c(0, 10.0, 11.0, 9.0, 10.5, f64::MAX))
            .unwrap()
            .is_none());
        assert!(r
            .push(c(1, 10.0, 11.0, 9.0, 10.5, f64::MAX))
            .unwrap()
            .is_none());
        let err = r.flush().unwrap_err();
        assert!(matches!(err, Error::Core(_)));
    }
}
