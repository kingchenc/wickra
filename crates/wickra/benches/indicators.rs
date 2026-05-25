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
    AccelerationBands, AdOscillator, AdaptiveCycle, Adxr, Alma, AnchoredVwap, Atr, AtrBands,
    Autocorrelation, BatchExt, BollingerBands, CalmarRatio, Camarilla, Candle, CenterOfGravity,
    ClassicPivots, CoefficientOfVariation, CyberneticCycle, Decycler, DecyclerOscillator,
    DemandIndex, DemarkPivots, DetrendedStdDev, Doji, DonchianStop, DoubleBollinger,
    EhlersStochastic, Ema, EmpiricalModeDecomposition, Engulfing, Fama, FibonacciPivots,
    FisherTransform, FractalChaosBands, Frama, GarmanKlassVolatility, Hammer, HeikinAshi,
    HiLoActivator, HilbertDominantCycle, HurstChannel, HurstExponent, Ichimoku, Indicator,
    InitialBalance, InstantaneousTrendline, InverseFisherTransform, Jma, Kst, Kurtosis, Kvo,
    LinRegChannel, MaEnvelope, MacdIndicator, Mama, MarketFacilitationIndex, MaxDrawdown,
    McGinleyDynamic, MedianAbsoluteDeviation, MorningEveningStar, Nvi, Obv, OpeningRange,
    ParkinsonVolatility, PercentageTrailingStop, Pgo, ProfitFactor, Pvi, RSquared,
    RenkoTrailingStop, RogersSatchellVolatility, RoofingFilter, Rsi, Rvi, RviVolatility, Rwi,
    SharpeRatio, SineWave, Skewness, Sma, StandardError, StandardErrorBands, StarcBands,
    StepTrailingStop, Stochastic, SuperSmoother, TdCombo, TdCountdown, TdDeMarker, TdDifferential,
    TdLines, TdOpen, TdPressure, TdRangeProjection, TdRei, TdRiskLevel, TdSequential, TdSetup,
    ThreeInside, Tii, Tsv, TtmSqueeze, ValueArea, ValueAtRisk, Variance, Vidya, VoltyStop,
    VolumeOscillator, VwapStdDevBands, Vzo, WaveTrend, WilliamsFractals, Wma, WoodiePivots,
    YangZhangVolatility, YoyoExit, ZigZag,
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

#[allow(clippy::too_many_lines)]
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
    bench_candle_input(c, "ichimoku", &candles, Ichimoku::classic);
    bench_candle_input(c, "heikin_ashi", &candles, HeikinAshi::new);

    // Family 14 — Candlestick patterns.
    // 1-bar, 2-bar and 3-bar representatives. The shape check itself is
    // stateless arithmetic, so this also serves as a cost-floor reference.
    bench_candle_input(c, "doji", &candles, Doji::new);
    bench_candle_input(c, "hammer", &candles, Hammer::new);
    bench_candle_input(c, "engulfing", &candles, Engulfing::new);
    bench_candle_input(c, "morning_evening_star", &candles, MorningEveningStar::new);
    bench_candle_input(c, "three_inside", &candles, ThreeInside::new);

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

    // --- Family 11: DeMark ---
    bench_candle_input(c, "td_setup", &candles, TdSetup::classic);
    bench_candle_input(c, "td_sequential", &candles, TdSequential::classic);
    bench_candle_input(c, "td_demarker", &candles, || TdDeMarker::new(14).unwrap());
    bench_candle_input(c, "td_rei", &candles, TdRei::classic);
    bench_candle_input(c, "td_pressure", &candles, || TdPressure::new(5).unwrap());
    bench_candle_input(c, "td_combo", &candles, TdCombo::classic);
    bench_candle_input(c, "td_countdown", &candles, TdCountdown::classic);
    bench_candle_input(c, "td_lines", &candles, TdLines::classic);
    bench_candle_input(c, "td_risk_level", &candles, TdRiskLevel::classic);
    bench_candle_input(c, "td_range_projection", &candles, TdRangeProjection::new);
    bench_candle_input(c, "td_differential", &candles, TdDifferential::new);
    bench_candle_input(c, "td_open", &candles, TdOpen::new);

    // --- Family 08: Pivots & Support/Resistance ---
    bench_candle_input(c, "classic_pivots", &candles, ClassicPivots::new);
    bench_candle_input(c, "fibonacci_pivots", &candles, FibonacciPivots::new);
    bench_candle_input(c, "camarilla", &candles, Camarilla::new);
    bench_candle_input(c, "woodie_pivots", &candles, WoodiePivots::new);
    bench_candle_input(c, "demark_pivots", &candles, DemarkPivots::new);
    bench_candle_input(c, "williams_fractals", &candles, WilliamsFractals::new);
    bench_candle_input(c, "zig_zag", &candles, || ZigZag::new(0.05).unwrap());

    // --- Family 09: Trailing Stops ---
    bench_candle_input(c, "hilo_activator", &candles, HiLoActivator::classic);
    bench_candle_input(c, "volty_stop", &candles, VoltyStop::classic);
    bench_candle_input(c, "yoyo_exit", &candles, YoyoExit::classic);
    bench_candle_input(c, "donchian_stop", &candles, DonchianStop::classic);
    bench_scalar(c, "percentage_trailing_stop", &closes, || {
        PercentageTrailingStop::new(5.0).unwrap()
    });
    bench_scalar(c, "step_trailing_stop", &closes, || {
        StepTrailingStop::new(1.0).unwrap()
    });
    bench_scalar(c, "renko_trailing_stop", &closes, || {
        RenkoTrailingStop::new(1.0).unwrap()
    });

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

    // --- Family 12: Statistik / Regression ---
    bench_scalar(c, "variance", &closes, || Variance::new(20).unwrap());
    bench_scalar(c, "coefficient_of_variation", &closes, || {
        CoefficientOfVariation::new(20).unwrap()
    });
    bench_scalar(c, "skewness", &closes, || Skewness::new(20).unwrap());
    bench_scalar(c, "kurtosis", &closes, || Kurtosis::new(20).unwrap());
    bench_scalar(c, "standard_error", &closes, || {
        StandardError::new(14).unwrap()
    });
    bench_scalar(c, "detrended_std_dev", &closes, || {
        DetrendedStdDev::new(14).unwrap()
    });
    bench_scalar(c, "r_squared", &closes, || RSquared::new(14).unwrap());
    bench_scalar(c, "median_absolute_deviation", &closes, || {
        MedianAbsoluteDeviation::new(20).unwrap()
    });
    bench_scalar(c, "autocorrelation", &closes, || {
        Autocorrelation::new(20, 1).unwrap()
    });
    bench_scalar(c, "hurst_exponent", &closes, || {
        HurstExponent::new(100, 4).unwrap()
    });

    // --- Family 16: Market Profile ---
    bench_candle_input(c, "value_area", &candles, || {
        ValueArea::new(20, 50, 0.70).unwrap()
    });
    bench_candle_input(c, "initial_balance", &candles, || {
        InitialBalance::new(12).unwrap()
    });
    bench_candle_input(c, "opening_range", &candles, || {
        OpeningRange::new(6).unwrap()
    });

    // --- Family 15: Risk / Performance Metrics ---
    // Close-prices stand in for the equity curve / return stream; absolute
    // numbers aren't meaningful here — what matters is the per-update cost.
    bench_scalar(c, "sharpe_ratio", &closes, || {
        SharpeRatio::new(20, 0.0).unwrap()
    });
    bench_scalar(c, "max_drawdown", &closes, || MaxDrawdown::new(20).unwrap());
    bench_scalar(c, "profit_factor", &closes, || {
        ProfitFactor::new(20).unwrap()
    });
    bench_scalar(c, "calmar_ratio", &closes, || CalmarRatio::new(20).unwrap());
    bench_scalar(c, "value_at_risk", &closes, || {
        ValueAtRisk::new(50, 0.95).unwrap()
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
