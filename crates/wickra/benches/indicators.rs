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
    AdaptiveCycle, Atr, BatchExt, BollingerBands, Candle, CenterOfGravity, CyberneticCycle,
    Decycler, DecyclerOscillator, EhlersStochastic, Ema, EmpiricalModeDecomposition, Fama,
    FisherTransform, HilbertDominantCycle, Indicator, InstantaneousTrendline,
    InverseFisherTransform, MacdIndicator, Mama, Obv, RoofingFilter, Rsi, SineWave, Sma,
    Stochastic, SuperSmoother, Wma,
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
    bench_macd(c, &closes);
    bench_bollinger(c, &closes);
    bench_candle_input(c, "atr", &candles, || Atr::new(14).unwrap());
    bench_candle_input(c, "stochastic", &candles, Stochastic::classic);
    bench_candle_input(c, "obv", &candles, Obv::new);

    // Family 10 — Ehlers / Cycle scalar benchmarks.
    bench_scalar(c, "super_smoother", &closes, || {
        SuperSmoother::new(10).unwrap()
    });
    bench_scalar(c, "fisher_transform", &closes, || {
        FisherTransform::new(10).unwrap()
    });
    bench_scalar(c, "inverse_fisher_transform", &closes, || {
        InverseFisherTransform::new(1.0).unwrap()
    });
    bench_scalar(c, "decycler", &closes, || Decycler::new(20).unwrap());
    bench_scalar(c, "decycler_oscillator", &closes, || {
        DecyclerOscillator::new(10, 30).unwrap()
    });
    bench_scalar(c, "roofing_filter", &closes, || {
        RoofingFilter::new(10, 48).unwrap()
    });
    bench_scalar(c, "center_of_gravity", &closes, || {
        CenterOfGravity::new(10).unwrap()
    });
    bench_scalar(c, "cybernetic_cycle", &closes, || {
        CyberneticCycle::new(10).unwrap()
    });
    bench_scalar(c, "instantaneous_trendline", &closes, || {
        InstantaneousTrendline::new(20).unwrap()
    });
    bench_scalar(c, "ehlers_stochastic", &closes, || {
        EhlersStochastic::new(20).unwrap()
    });
    bench_scalar(c, "empirical_mode_decomposition", &closes, || {
        EmpiricalModeDecomposition::new(20, 0.5).unwrap()
    });
    bench_scalar(
        c,
        "hilbert_dominant_cycle",
        &closes,
        HilbertDominantCycle::new,
    );
    bench_scalar(c, "adaptive_cycle", &closes, AdaptiveCycle::new);
    bench_scalar(c, "sine_wave", &closes, SineWave::new);
    bench_scalar(c, "fama", &closes, || Fama::new(0.5, 0.05).unwrap());

    // MAMA: multi-output, mirrored on macd's streaming-only bench style.
    {
        let mut group = c.benchmark_group("mama");
        for &n in SIZES {
            let n = n.min(closes.len());
            let series = &closes[..n];
            group.throughput(Throughput::Elements(n as u64));
            group.bench_with_input(BenchmarkId::new("streaming", n), series, |b, prices| {
                b.iter(|| {
                    let mut ind = Mama::classic();
                    for p in prices {
                        black_box(ind.update(*p));
                    }
                });
            });
        }
        group.finish();
    }
}

criterion_group!(name = wickra_benches; config = Criterion::default(); targets = benches);
criterion_main!(wickra_benches);
