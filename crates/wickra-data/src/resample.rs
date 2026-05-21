//! Resample an existing candle stream from a finer timeframe to a coarser one.

use crate::aggregator::Timeframe;
use crate::error::Result;
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
    pub fn push(&mut self, candle: Candle) -> Option<Candle> {
        let bucket = self.timeframe.floor(candle.timestamp);
        match self.open {
            Some(mut bar) if bucket == bar.bucket_start => {
                bar.absorb(candle);
                self.open = Some(bar);
                None
            }
            Some(bar) => {
                let closed = bar.into_candle();
                self.open = Some(RolledBar::from_candle(candle, bucket));
                Some(closed)
            }
            None => {
                self.open = Some(RolledBar::from_candle(candle, bucket));
                None
            }
        }
    }

    /// Flush the currently open coarser bar, if any.
    pub fn flush(&mut self) -> Option<Candle> {
        self.open.take().map(RolledBar::into_candle)
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
        if let Some(closed) = r.push(c) {
            out.push(closed);
        }
    }
    if let Some(last) = r.flush() {
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
}
