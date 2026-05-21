//! Microbenchmarks for every built-in indicator.
//!
//! Run with:
//! ```text
//! cargo bench -p wickra
//! ```
//!
//! Each benchmark feeds a deterministic synthetic price series through both the
//! streaming (`update` loop) and batch APIs of an indicator. Sizes cover small
//! (1 000), medium (10 000), and large (100 000) workloads.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use wickra::{
    Atr, BatchExt, BollingerBands, Candle, Ema, Indicator, MacdIndicator, Obv, Rsi, Sma,
    Stochastic, Wma,
};

/// Deterministic synthetic price series of length `n`.
fn price_series(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            let t = i as f64;
            100.0 + (t * 0.013).sin() * 12.0 + (t * 0.071).cos() * 4.0 + (t * 0.003).sin() * 30.0
        })
        .collect()
}

/// Synthetic OHLC candle series.
fn candle_series(n: usize) -> Vec<Candle> {
    let closes = price_series(n);
    closes
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let t = i as f64;
            let spread = 0.5 + (t * 0.05).sin().abs();
            // Benchmark synthetic data: i originates from a usize counter capped at 100_000,
            // well within i64::MAX. The wrap-around lint does not apply here.
            #[allow(clippy::cast_possible_wrap)]
            let ts = i as i64;
            Candle::new_unchecked(*c, c + spread, c - spread, *c, 1_000.0, ts)
        })
        .collect()
}

fn bench_scalar<I, F>(c: &mut Criterion, name: &str, sizes: &[usize], make: F)
where
    F: Fn() -> I,
    I: Indicator<Input = f64, Output = f64> + BatchExt,
{
    let mut group = c.benchmark_group(name);
    for &n in sizes {
        let series = price_series(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("streaming", n), &series, |b, prices| {
            b.iter(|| {
                let mut ind = make();
                for p in prices {
                    black_box(ind.update(*p));
                }
            });
        });
        group.bench_with_input(BenchmarkId::new("batch", n), &series, |b, prices| {
            b.iter(|| {
                let mut ind = make();
                black_box(ind.batch(prices));
            });
        });
    }
    group.finish();
}

fn bench_macd(c: &mut Criterion, sizes: &[usize]) {
    let mut group = c.benchmark_group("macd");
    for &n in sizes {
        let series = price_series(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("streaming", n), &series, |b, prices| {
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

fn bench_bollinger(c: &mut Criterion, sizes: &[usize]) {
    let mut group = c.benchmark_group("bollinger");
    for &n in sizes {
        let series = price_series(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("streaming", n), &series, |b, prices| {
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

fn bench_candle_input<I, F, O>(c: &mut Criterion, name: &str, sizes: &[usize], make: F)
where
    F: Fn() -> I,
    I: Indicator<Input = Candle, Output = O>,
{
    let mut group = c.benchmark_group(name);
    for &n in sizes {
        let candles = candle_series(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("streaming", n), &candles, |b, candles| {
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
    let sizes = [1_000_usize, 10_000, 100_000];
    bench_scalar(c, "sma", &sizes, || Sma::new(14).unwrap());
    bench_scalar(c, "ema", &sizes, || Ema::new(14).unwrap());
    bench_scalar(c, "wma", &sizes, || Wma::new(14).unwrap());
    bench_scalar(c, "rsi", &sizes, || Rsi::new(14).unwrap());
    bench_macd(c, &sizes);
    bench_bollinger(c, &sizes);
    bench_candle_input(c, "atr", &sizes, || Atr::new(14).unwrap());
    bench_candle_input(c, "stochastic", &sizes, Stochastic::classic);
    bench_candle_input(c, "obv", &sizes, Obv::new);
}

criterion_group!(name = wickra_benches; config = Criterion::default(); targets = benches);
criterion_main!(wickra_benches);
