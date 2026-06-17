//! Throughput microbenchmarks for the native data layer: CSV parsing, tick
//! aggregation, and resampling.
//!
//! Run with:
//! ```text
//! cargo bench -p wickra --bench data_layer
//! ```
//!
//! These exercise `wickra-data` — the same native code every binding rides
//! through the FFI boundary characterised in `BENCHMARKS.md` §3. It is what
//! replaces `pandas` / `csv-parse` / manual tick bucketing / `pandas.resample`
//! in the nine non-Rust languages, so the numbers here are the upper bound a
//! binding can reach for "load a CSV, roll ticks into candles, resample a
//! series" without pulling in a single third-party package.
//!
//! The dataset is the checked-in `examples/data/btcusdt-1m.csv` (50 000 real
//! BTCUSDT one-minute candles). Regenerate it with
//! `cargo run -p wickra-examples --bin fetch_btcusdt`.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use wickra::{Candle, Tick};
use wickra_data::{
    aggregator::{TickAggregator, Timeframe},
    csv::CandleReader,
    resample::Resampler,
};

const DATASET: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/data/btcusdt-1m.csv"
);
const ONE_MINUTE_MS: i64 = 60_000;

fn dataset_bytes() -> Vec<u8> {
    std::fs::read(DATASET).unwrap_or_else(|e| {
        panic!(
            "could not read the benchmark dataset {DATASET}: {e}\n\
             generate it with `cargo run -p wickra-examples --bin fetch_btcusdt`"
        )
    })
}

fn load_candles() -> Vec<Candle> {
    CandleReader::from_reader(dataset_bytes().as_slice())
        .unwrap()
        .read_all()
        .unwrap()
}

/// CSV bytes -> `Vec<Candle>`. Throughput is candles (rows) parsed per second.
fn bench_csv_read(c: &mut Criterion) {
    let bytes = dataset_bytes();
    let rows = load_candles().len() as u64;
    let mut group = c.benchmark_group("data_layer/csv_read");
    group.throughput(Throughput::Elements(rows));
    group.bench_function("btcusdt_1m", |b| {
        b.iter(|| {
            let candles = CandleReader::from_reader(black_box(bytes.as_slice()))
                .unwrap()
                .read_all()
                .unwrap();
            black_box(candles.len())
        });
    });
    group.finish();
}

/// Ticks -> one-minute candles. Throughput is ticks aggregated per second.
fn bench_tick_aggregate(c: &mut Criterion) {
    let ticks: Vec<Tick> = load_candles()
        .iter()
        .map(|candle| Tick::new(candle.close, candle.volume, candle.timestamp).unwrap())
        .collect();
    let mut group = c.benchmark_group("data_layer/tick_aggregate");
    group.throughput(Throughput::Elements(ticks.len() as u64));
    group.bench_function("1m_buckets", |b| {
        b.iter(|| {
            let mut agg = TickAggregator::new(Timeframe::millis(ONE_MINUTE_MS).unwrap());
            let mut emitted = 0usize;
            for tick in &ticks {
                emitted += agg.push(black_box(*tick)).unwrap().len();
            }
            black_box(emitted)
        });
    });
    group.finish();
}

/// One-minute candles -> five-minute candles. Throughput is input candles per second.
fn bench_resample(c: &mut Criterion) {
    let candles = load_candles();
    let mut group = c.benchmark_group("data_layer/resample");
    group.throughput(Throughput::Elements(candles.len() as u64));
    group.bench_function("1m_to_5m", |b| {
        b.iter(|| {
            let mut resampler = Resampler::new(Timeframe::millis(5 * ONE_MINUTE_MS).unwrap());
            let mut emitted = 0usize;
            for candle in &candles {
                if resampler.push(black_box(*candle)).unwrap().is_some() {
                    emitted += 1;
                }
            }
            if resampler.flush().unwrap().is_some() {
                emitted += 1;
            }
            black_box(emitted)
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_csv_read,
    bench_tick_aggregate,
    bench_resample
);
criterion_main!(benches);
