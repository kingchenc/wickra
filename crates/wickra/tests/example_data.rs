//! Integration test: the checked-in BTCUSDT example datasets parse cleanly,
//! hold enough rows, and carry strictly increasing — and, for the fixed
//! timeframes, evenly spaced — timestamps.
//!
//! The datasets live in the workspace `examples/data/` directory and are
//! produced by the `fetch_btcusdt` example. Regenerate them with:
//!
//! ```text
//! cargo run -p wickra-examples --bin fetch_btcusdt
//! ```

use wickra_data::csv::CandleReader;

/// `(file name, minimum row count, expected step in ms)`. The step is `None`
/// for the monthly file, whose buckets are 28–31 days and thus uneven.
const DATASETS: &[(&str, usize, Option<i64>)] = &[
    ("btcusdt-1m.csv", 50_000, Some(60_000)),
    ("btcusdt-5m.csv", 10_000, Some(300_000)),
    ("btcusdt-15m.csv", 10_000, Some(900_000)),
    ("btcusdt-1h.csv", 10_000, Some(3_600_000)),
    ("btcusdt-12h.csv", 5_000, Some(43_200_000)),
    // 1d and 1month collect all the history Binance offers, which grows over
    // time — assert a lower bound rather than an exact count.
    ("btcusdt-1d.csv", 3_000, Some(86_400_000)),
    ("btcusdt-1month.csv", 100, None),
];

fn dataset_path(file: &str) -> String {
    format!("{}/../../examples/data/{file}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn every_dataset_parses_and_is_well_formed() {
    for &(file, min_rows, step) in DATASETS {
        let path = dataset_path(file);
        let mut reader =
            CandleReader::open(&path).unwrap_or_else(|e| panic!("{file}: cannot open {path}: {e}"));
        // `read_all` validates every row through `Candle::new`, so a successful
        // read already proves each OHLC tuple is finite and internally
        // consistent (high >= low, etc.).
        let candles = reader
            .read_all()
            .unwrap_or_else(|e| panic!("{file}: invalid OHLCV row: {e}"));

        assert!(
            candles.len() >= min_rows,
            "{file}: expected at least {min_rows} rows, got {}",
            candles.len()
        );

        for pair in candles.windows(2) {
            let (prev, next) = (pair[0], pair[1]);
            assert!(
                next.timestamp > prev.timestamp,
                "{file}: timestamps must strictly increase, saw {} then {}",
                prev.timestamp,
                next.timestamp
            );
            if let Some(step) = step {
                assert_eq!(
                    next.timestamp - prev.timestamp,
                    step,
                    "{file}: a fixed timeframe must be evenly spaced by {step} ms"
                );
            }
        }
    }
}
