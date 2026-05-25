//! Microbenchmarks for every built-in indicator.
//!
//! Run with:
//! ```text
//! cargo bench -p wickra
//! ```
//!
//! Each benchmark feeds real BTCUSDT 1-minute candles — read from the
//! checked-in dataset at the workspace `examples/data/btcusdt-1m.csv` —
//! through both the streaming (`update` loop) and batch APIs of an
//! indicator. Sizes cover small (1 000), medium (10 000), and large
//! (50 000) workloads, taken as prefixes of that dataset.
//!
//! Regenerate the dataset with:
//! ```text
//! cargo run -p wickra-examples --bin fetch_btcusdt
//! ```

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use wickra::{
    AccelerationBands, AdOscillator, Adxr, Alma, AnchoredVwap, Atr, AtrBands, BatchExt,
    BollingerBands, Candle, DemandIndex, DoubleBollinger, Ema, FractalChaosBands, Frama,
    GarmanKlassVolatility, HurstChannel, Indicator, Jma, Kst, Kvo, LinRegChannel, MaEnvelope,
    MacdIndicator, MarketFacilitationIndex, McGinleyDynamic, Nvi, Obv, ParkinsonVolatility, Pgo,
    Pvi, RogersSatchellVolatility, Rsi, Rvi, RviVolatility, Rwi, Sma, StandardErrorBands,
    StarcBands, Stochastic, Tii, Tsv, TtmSqueeze, Vidya, VolumeOscillator, VwapStdDevBands, Vzo,
    WaveTrend, Wma, YangZhangVolatility,
};
use wickra_data::csv::CandleReader;

/// Workload sizes, in candles. Each is taken as a prefix of the dataset.
const SIZES: &[usize] = &[1_000, 10_000, 50_000];

/// Load the checked-in BTCUSDT 1-minute candle dataset from the workspace
/// `examples/data/` directory.
fn load_candles() -> Vec<Candle> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/data/btcusdt-1m.csv"
    );
    let mut reader = CandleReader::open(path).unwrap_or_else(|e| {
        panic!(
            "could not open the benchmark dataset {path}: {e}\n\
             generate it with `cargo run -p wickra-examples --bin fetch_btcusdt`"
        )
    });
    reader
        .read_all()
        .expect("the benchmark dataset is valid OHLCV")
}

fn bench_scalar<I, F>(c: &mut Criterion, name: &str, prices: &[f64], make: F)
where
    F: Fn() -> I,
    I: Indicator<Input = f64, Output = f64> + BatchExt,
{
    let mut group = c.benchmark_group(name);
    for &n in SIZES {
        let n = n.min(prices.len());
        let series = &prices[..n];
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("streaming", n), series, |b, prices| {
            b.iter(|| {
                let mut ind = make();
                for p in prices {
                    black_box(ind.update(*p));
                }
            });
        });
        group.bench_with_input(BenchmarkId::new("batch", n), series, |b, prices| {
            b.iter(|| {
                let mut ind = make();
                black_box(ind.batch(prices));
            });
        });
    }
    group.finish();
}

fn bench_kst(c: &mut Criterion, prices: &[f64]) {
    let mut group = c.benchmark_group("kst");
    for &n in SIZES {
        let n = n.min(prices.len());
        let series = &prices[..n];
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("streaming", n), series, |b, prices| {
            b.iter(|| {
                let mut ind = Kst::classic();
                for p in prices {
                    black_box(ind.update(*p));
                }
            });
        });
    }
    group.finish();
}

fn bench_macd(c: &mut Criterion, prices: &[f64]) {
    let mut group = c.benchmark_group("macd");
    for &n in SIZES {
        let n = n.min(prices.len());
        let series = &prices[..n];
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("streaming", n), series, |b, prices| {
            b.iter(|| {
                let mut ind = MacdIndicator::classic();
                for p in prices {
                    black_box(ind.update(*p));
                }
            });
        });
    }
    group.finish();
}

fn bench_bollinger(c: &mut Criterion, prices: &[f64]) {
    let mut group = c.benchmark_group("bollinger");
    for &n in SIZES {
        let n = n.min(prices.len());
        let series = &prices[..n];
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("streaming", n), series, |b, prices| {
            b.iter(|| {
                let mut ind = BollingerBands::classic();
                for p in prices {
                    black_box(ind.update(*p));
                }
            });
        });
    }
    group.finish();
}

fn bench_candle_input<I, F, O>(c: &mut Criterion, name: &str, candles: &[Candle], make: F)
where
    F: Fn() -> I,
    I: Indicator<Input = Candle, Output = O>,
{
    let mut group = c.benchmark_group(name);
    for &n in SIZES {
        let n = n.min(candles.len());
        let series = &candles[..n];
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("streaming", n), series, |b, candles| {
            b.iter(|| {
                let mut ind = make();
                for c in candles {
                    black_box(ind.update(*c));
                }
            });
        });
    }
    group.finish();
}

fn benches(c: &mut Criterion) {
    let candles = load_candles();
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    bench_scalar(c, "sma", &closes, || Sma::new(14).unwrap());
    bench_scalar(c, "ema", &closes, || Ema::new(14).unwrap());
    bench_scalar(c, "wma", &closes, || Wma::new(14).unwrap());
    bench_scalar(c, "rsi", &closes, || Rsi::new(14).unwrap());
    bench_scalar(c, "tii", &closes, || Tii::new(60, 30).unwrap());
    bench_scalar(c, "alma", &closes, || Alma::new(9, 0.85, 6.0).unwrap());
    bench_scalar(c, "mcginley_dynamic", &closes, || {
        McGinleyDynamic::new(10).unwrap()
    });
    bench_scalar(c, "frama", &closes, || Frama::new(16).unwrap());
    bench_scalar(c, "vidya", &closes, || Vidya::new(14, 9).unwrap());
    bench_scalar(c, "jma", &closes, || Jma::new(14, 0.0, 2).unwrap());
    bench_macd(c, &closes);
    bench_kst(c, &closes);
    bench_bollinger(c, &closes);
    bench_candle_input(c, "atr", &candles, || Atr::new(14).unwrap());
    bench_candle_input(c, "adxr", &candles, || Adxr::new(14).unwrap());
    bench_candle_input(c, "rwi", &candles, || Rwi::new(14).unwrap());
    bench_candle_input(c, "wave_trend", &candles, || WaveTrend::classic().unwrap());
    bench_candle_input(c, "stochastic", &candles, Stochastic::classic);
    bench_candle_input(c, "obv", &candles, Obv::new);

    // --- Family 07: Volume ---
    bench_candle_input(c, "kvo", &candles, Kvo::classic);
    bench_candle_input(c, "volume_oscillator", &candles, || {
        VolumeOscillator::new(14, 28).unwrap()
    });
    bench_candle_input(c, "nvi", &candles, Nvi::new);
    bench_candle_input(c, "pvi", &candles, Pvi::new);
    bench_candle_input(c, "williams_ad", &candles, AdOscillator::new);
    bench_candle_input(c, "anchored_vwap", &candles, AnchoredVwap::new);
    bench_candle_input(c, "demand_index", &candles, || {
        DemandIndex::new(10).unwrap()
    });
    bench_candle_input(c, "tsv", &candles, || Tsv::new(18).unwrap());
    bench_candle_input(c, "vzo", &candles, || Vzo::new(14).unwrap());
    bench_candle_input(
        c,
        "market_facilitation_index",
        &candles,
        MarketFacilitationIndex::new,
    );

    // --- Family 04: Volatility ---
    bench_scalar(c, "rvi_volatility", &closes, || {
        RviVolatility::new(10).unwrap()
    });
    bench_candle_input(c, "parkinson", &candles, || {
        ParkinsonVolatility::new(20, 252).unwrap()
    });
    bench_candle_input(c, "garman_klass", &candles, || {
        GarmanKlassVolatility::new(20, 252).unwrap()
    });
    bench_candle_input(c, "rogers_satchell", &candles, || {
        RogersSatchellVolatility::new(20, 252).unwrap()
    });
    bench_candle_input(c, "yang_zhang", &candles, || {
        YangZhangVolatility::new(20, 252).unwrap()
    });
    bench_candle_input(c, "rvi", &candles, || Rvi::new(10).unwrap());
    bench_candle_input(c, "pgo", &candles, || Pgo::new(14).unwrap());

    // --- Family 05: Bands & Channels ---
    bench_candle_input(c, "acceleration_bands", &candles, || {
        AccelerationBands::new(20, 0.001).unwrap()
    });
    bench_candle_input(c, "starc_bands", &candles, || {
        StarcBands::new(6, 15, 2.0).unwrap()
    });
    bench_candle_input(c, "atr_bands", &candles, || AtrBands::new(14, 3.0).unwrap());
    bench_candle_input(c, "hurst_channel", &candles, || {
        HurstChannel::new(10, 0.5).unwrap()
    });
    bench_candle_input(c, "ttm_squeeze", &candles, || {
        TtmSqueeze::new(20, 2.0, 1.5).unwrap()
    });
    bench_candle_input(c, "fractal_chaos_bands", &candles, || {
        FractalChaosBands::new(2).unwrap()
    });
    bench_candle_input(c, "vwap_stddev_bands", &candles, || {
        VwapStdDevBands::new(2.0).unwrap()
    });
    bench_scalar_multi(c, "ma_envelope", &closes, || {
        MaEnvelope::new(20, 0.025).unwrap()
    });
    bench_scalar_multi(c, "linreg_channel", &closes, || {
        LinRegChannel::new(20, 2.0).unwrap()
    });
    bench_scalar_multi(c, "standard_error_bands", &closes, || {
        StandardErrorBands::new(21, 2.0).unwrap()
    });
    bench_scalar_multi(c, "double_bollinger", &closes, || {
        DoubleBollinger::new(20, 1.0, 2.0).unwrap()
    });
}

/// Variant of `bench_scalar` for scalar-input indicators whose output is *not*
/// `f64` (band/channel structs). Streaming-only path keeps the benchmark
/// expression flat across all multi-output indicators.
fn bench_scalar_multi<I, F, O>(c: &mut Criterion, name: &str, prices: &[f64], make: F)
where
    F: Fn() -> I,
    I: Indicator<Input = f64, Output = O>,
{
    let mut group = c.benchmark_group(name);
    for &n in SIZES {
        let n = n.min(prices.len());
        let series = &prices[..n];
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("streaming", n), series, |b, prices| {
            b.iter(|| {
                let mut ind = make();
                for p in prices {
                    black_box(ind.update(*p));
                }
            });
        });
    }
    group.finish();
}

criterion_group!(name = wickra_benches; config = Criterion::default(); targets = benches);
criterion_main!(wickra_benches);
