#![no_main]
//! Fuzz the OHLCV CSV reader with arbitrary byte input.
//!
//! The reader must never panic: malformed headers, non-numeric cells,
//! truncated rows and arbitrary binary data all have to surface as an
//! `Err`, never a crash.

use libfuzzer_sys::fuzz_target;
use wickra_data::csv::CandleReader;

fuzz_target!(|data: &[u8]| {
    if let Ok(mut reader) = CandleReader::from_reader(data) {
        for candle in reader.candles() {
            let _ = candle;
        }
    }
});
