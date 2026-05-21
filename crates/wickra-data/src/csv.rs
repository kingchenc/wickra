//! Stream OHLCV candles out of a CSV file.
//!
//! The reader is generic over the column layout, but ships with a sensible
//! default ("timestamp,open,high,low,close,volume") that matches the standard
//! Binance / Yahoo Finance / kaggle dataset format.

use std::path::Path;

use serde::Deserialize;

use crate::error::{Error, Result};
use wickra_core::Candle;

/// Default OHLCV CSV row layout.
///
/// The timestamp is parsed as an `i64`; if your file ships an RFC3339 / ISO8601
/// string instead, use [`CandleReader::with_timestamp_parser`].
#[derive(Debug, Clone, Deserialize)]
pub struct DefaultRow {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl DefaultRow {
    fn into_candle(self) -> Result<Candle> {
        Candle::new(
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume,
            self.timestamp,
        )
        .map_err(Error::from)
    }
}

/// Streaming OHLCV CSV reader.
#[derive(Debug)]
pub struct CandleReader<R: std::io::Read> {
    reader: csv::Reader<R>,
}

impl CandleReader<std::fs::File> {
    /// Open a CSV file at `path`. The first line is treated as a header by default.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)?;
        Ok(Self { reader })
    }
}

impl<R: std::io::Read> CandleReader<R> {
    /// Build a reader from any [`std::io::Read`] source.
    pub fn from_reader(inner: R) -> Self {
        Self {
            reader: csv::ReaderBuilder::new()
                .has_headers(true)
                .from_reader(inner),
        }
    }

    /// Replace the underlying reader; useful for testing.
    pub fn from_csv_reader(reader: csv::Reader<R>) -> Self {
        Self { reader }
    }

    /// Iterator over decoded candles.
    pub fn candles(&mut self) -> impl Iterator<Item = Result<Candle>> + '_ {
        self.reader.deserialize::<DefaultRow>().map(|row_res| {
            let row = row_res?;
            row.into_candle()
        })
    }

    /// Read the entire stream into a `Vec<Candle>`. Convenient for backtests.
    pub fn read_all(&mut self) -> Result<Vec<Candle>> {
        self.candles().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn reads_well_formed_csv() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "timestamp,open,high,low,close,volume").unwrap();
        writeln!(tmp, "1,10.0,11.0,9.0,10.5,100").unwrap();
        writeln!(tmp, "2,10.5,11.5,10.0,11.0,150").unwrap();
        writeln!(tmp, "3,11.0,12.0,10.5,11.5,200").unwrap();
        tmp.flush().unwrap();

        let mut r = CandleReader::open(tmp.path()).unwrap();
        let candles = r.read_all().unwrap();
        assert_eq!(candles.len(), 3);
        assert_eq!(candles[0].open, 10.0);
        assert_eq!(candles[2].close, 11.5);
        assert_eq!(candles[1].timestamp, 2);
    }

    #[test]
    fn rejects_invalid_ohlc() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "timestamp,open,high,low,close,volume").unwrap();
        // high < low → core validation rejects it.
        writeln!(tmp, "1,10.0,8.0,9.0,9.5,100").unwrap();
        tmp.flush().unwrap();

        let mut r = CandleReader::open(tmp.path()).unwrap();
        let candles: Result<Vec<Candle>> = r.candles().collect();
        assert!(candles.is_err());
    }

    #[test]
    fn from_reader_works_on_in_memory_data() {
        let data = "timestamp,open,high,low,close,volume\n1,1,2,0,1,10\n2,1,2,0,1,10\n";
        let mut r = CandleReader::from_reader(data.as_bytes());
        let v = r.read_all().unwrap();
        assert_eq!(v.len(), 2);
    }
}
