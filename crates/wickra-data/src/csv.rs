//! Stream OHLCV candles out of a CSV file.
//!
//! The reader is generic over the column layout, but ships with a sensible
//! default ("timestamp,open,high,low,close,volume") that matches the standard
//! Binance / Yahoo Finance / kaggle dataset format.
//!
//! The reader is defensive about real-world files: a leading UTF-8 byte-order
//! mark is stripped, surrounding whitespace is trimmed from every field, and a
//! file whose header does not name the required columns is rejected with a
//! clear [`Error::Malformed`] instead of silently consuming its first data row.

use std::path::Path;

use serde::Deserialize;

use crate::error::{Error, Result};
use wickra_core::Candle;

/// Column names the default OHLCV layout requires. The CSV header must contain
/// every one of these (extra columns are ignored); matching is exact and
/// case-sensitive because the underlying `serde` deserialization maps header
/// names to [`DefaultRow`]'s fields by name.
const REQUIRED_COLUMNS: [&str; 6] = ["timestamp", "open", "high", "low", "close", "volume"];

/// Default OHLCV CSV row layout.
///
/// The timestamp is parsed as an `i64` (for example a Unix epoch). RFC3339 /
/// ISO8601 string timestamps are not handled by this layout; convert them to
/// integers before reading.
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

/// A [`std::io::Read`] adapter that transparently skips a leading UTF-8
/// byte-order mark.
///
/// Spreadsheet exporters — Excel in particular — prefix CSV files with the
/// three-byte UTF-8 BOM `EF BB BF`. Left in place it becomes part of the first
/// header name (`\u{feff}timestamp`), which then fails to match the
/// `timestamp` column. This adapter drops the BOM before the CSV parser ever
/// sees it; files without a BOM pass through untouched.
#[derive(Debug)]
pub struct BomStripReader<R> {
    inner: R,
    /// Whether the leading bytes have been inspected for a BOM yet.
    checked: bool,
    /// Bytes read during BOM detection that turned out *not* to be a BOM and
    /// must still be handed to the consumer.
    leftover: Vec<u8>,
    leftover_pos: usize,
}

impl<R: std::io::Read> BomStripReader<R> {
    /// Wrap `inner`, stripping a leading UTF-8 BOM on the first read.
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            checked: false,
            leftover: Vec::new(),
            leftover_pos: 0,
        }
    }

    /// On the first read, consume up to three bytes and decide whether they
    /// form a BOM. A BOM is discarded; anything else is buffered for replay.
    fn check_bom(&mut self) -> std::io::Result<()> {
        if self.checked {
            return Ok(());
        }
        self.checked = true;

        let mut probe = [0u8; 3];
        let mut filled = 0;
        while filled < probe.len() {
            let n = self.inner.read(&mut probe[filled..])?;
            if n == 0 {
                break; // short source — fewer than 3 bytes total
            }
            filled += n;
        }

        if probe[..filled] != [0xEF, 0xBB, 0xBF] {
            // Not a BOM (or a short file): replay every probed byte verbatim.
            self.leftover.extend_from_slice(&probe[..filled]);
        }
        Ok(())
    }
}

impl<R: std::io::Read> std::io::Read for BomStripReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.check_bom()?;
        if self.leftover_pos < self.leftover.len() {
            let n = (self.leftover.len() - self.leftover_pos).min(buf.len());
            buf[..n].copy_from_slice(&self.leftover[self.leftover_pos..self.leftover_pos + n]);
            self.leftover_pos += n;
            return Ok(n);
        }
        self.inner.read(buf)
    }
}

/// Validate that a CSV reader's header row names every required OHLCV column.
fn validate_headers<R: std::io::Read>(reader: &mut csv::Reader<R>) -> Result<()> {
    let headers = reader.headers()?;
    let present: Vec<String> = headers.iter().map(|h| h.trim().to_string()).collect();
    let missing: Vec<&str> = REQUIRED_COLUMNS
        .iter()
        .copied()
        .filter(|col| !present.iter().any(|h| h == col))
        .collect();
    if !missing.is_empty() {
        return Err(Error::Malformed(format!(
            "CSV header is missing required column(s) [{}]; found [{}] — \
             the first line must be a header naming {}",
            missing.join(", "),
            present.join(", "),
            REQUIRED_COLUMNS.join(",")
        )));
    }
    Ok(())
}

/// Streaming OHLCV CSV reader.
#[derive(Debug)]
pub struct CandleReader<R: std::io::Read> {
    reader: csv::Reader<R>,
}

impl<R: std::io::Read> CandleReader<R> {
    /// Build a trimming CSV reader around `inner` and validate its header.
    fn build(inner: R) -> Result<Self> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(inner);
        validate_headers(&mut reader)?;
        Ok(Self { reader })
    }
}

impl CandleReader<BomStripReader<std::fs::File>> {
    /// Open a CSV file at `path`.
    ///
    /// The first line must be a header row naming the OHLCV columns; a leading
    /// UTF-8 BOM and whitespace around values are tolerated.
    ///
    /// # Errors
    /// Returns [`Error::Io`] if the file cannot be opened and
    /// [`Error::Malformed`] if the header does not contain every required
    /// column (`timestamp,open,high,low,close,volume`).
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        Self::from_reader(file)
    }
}

impl<R: std::io::Read> CandleReader<BomStripReader<R>> {
    /// Build a reader from any [`std::io::Read`] source.
    ///
    /// A leading UTF-8 BOM is stripped and the header is validated.
    ///
    /// # Errors
    /// Returns [`Error::Malformed`] if the header does not contain every
    /// required column.
    pub fn from_reader(inner: R) -> Result<Self> {
        Self::build(BomStripReader::new(inner))
    }
}

impl<R: std::io::Read> CandleReader<R> {
    /// Wrap a pre-built [`csv::Reader`]; useful for testing or for non-default
    /// reader configuration.
    ///
    /// Unlike [`from_reader`](Self::from_reader) this does *not* strip a BOM —
    /// the caller owns the reader's configuration — but the header is still
    /// validated.
    ///
    /// # Errors
    /// Returns [`Error::Malformed`] if the header does not contain every
    /// required column.
    pub fn from_csv_reader(mut reader: csv::Reader<R>) -> Result<Self> {
        validate_headers(&mut reader)?;
        Ok(Self { reader })
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
        let mut r = CandleReader::from_reader(data.as_bytes()).unwrap();
        let v = r.read_all().unwrap();
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn rejects_file_without_header() {
        // No header row — the first line is data. Without validation the
        // reader would silently swallow it as the header.
        let data = "1,10.0,11.0,9.0,10.5,100\n2,10.5,11.5,10.0,11.0,150\n";
        let err = CandleReader::from_reader(data.as_bytes()).unwrap_err();
        assert!(matches!(err, Error::Malformed(_)));
    }

    #[test]
    fn rejects_header_missing_a_column() {
        // "volume" is absent.
        let data = "timestamp,open,high,low,close\n1,10.0,11.0,9.0,10.5\n";
        let err = CandleReader::from_reader(data.as_bytes()).unwrap_err();
        // The error variant must be Malformed and the message must mention
        // the missing column. Asserting directly (rather than match-and-
        // panic-on-other) keeps the assertion's cold path branch-free for
        // coverage and still pins the diagnostic.
        assert!(
            matches!(&err, Error::Malformed(msg) if msg.contains("volume")),
            "expected Malformed mentioning 'volume', got {err:?}"
        );
    }

    /// Cover `from_csv_reader` (lines 201-204): existing tests use
    /// `from_reader` / `open`, which both construct the inner `csv::Reader`
    /// internally. Callers that want non-default csv configuration must
    /// build the reader themselves and pass it through `from_csv_reader`.
    #[test]
    fn from_csv_reader_accepts_a_prebuilt_reader() {
        let data = "timestamp;open;high;low;close;volume\n1;10.0;11.0;9.0;10.5;100\n";
        let inner = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(data.as_bytes());
        let mut r = CandleReader::from_csv_reader(inner).unwrap();
        let candles = r.read_all().unwrap();
        assert_eq!(candles.len(), 1);
        assert_eq!(candles[0].close, 10.5);
    }

    #[test]
    fn strips_leading_utf8_bom() {
        // A BOM (\u{feff}) prefixes the header — Excel exports look like this.
        let data = "\u{feff}timestamp,open,high,low,close,volume\n1,10.0,11.0,9.0,10.5,100\n";
        let mut r = CandleReader::from_reader(data.as_bytes()).unwrap();
        let v = r.read_all().unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].timestamp, 1);
        assert_eq!(v[0].open, 10.0);
    }

    #[test]
    fn tolerates_whitespace_around_fields() {
        let data = " timestamp , open , high , low , close , volume \n\
                     1 , 10.0 , 11.0 , 9.0 , 10.5 , 100 \n";
        let mut r = CandleReader::from_reader(data.as_bytes()).unwrap();
        let v = r.read_all().unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].close, 10.5);
        assert_eq!(v[0].volume, 100.0);
    }

    #[test]
    fn bom_stripper_passes_through_non_bom_input() {
        use std::io::Read;
        let mut out = String::new();
        BomStripReader::new("hello".as_bytes())
            .read_to_string(&mut out)
            .unwrap();
        assert_eq!(out, "hello");
    }

    #[test]
    fn bom_stripper_handles_short_input() {
        use std::io::Read;
        let mut out = Vec::new();
        // Two bytes — shorter than a 3-byte BOM.
        BomStripReader::new([0x41u8, 0x42u8].as_slice())
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, vec![0x41, 0x42]);
    }
}
