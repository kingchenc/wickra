<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=447" alt="Wickra — streaming-first technical indicators" width="100%"></a>
</p>

[![CI](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml/badge.svg)](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml)
[![CodeQL](https://github.com/wickra-lib/wickra/actions/workflows/codeql.yml/badge.svg)](https://github.com/wickra-lib/wickra/actions/workflows/codeql.yml)
[![codecov](https://codecov.io/gh/wickra-lib/wickra/branch/main/graph/badge.svg)](https://codecov.io/gh/wickra-lib/wickra)
[![GitHub release](https://img.shields.io/github/v/release/wickra-lib/wickra?logo=github&color=green)](https://github.com/wickra-lib/wickra/releases/latest)
[![crates.io](https://img.shields.io/crates/v/wickra.svg?logo=rust&color=orange)](https://crates.io/crates/wickra)
[![PyPI](https://img.shields.io/pypi/v/wickra.svg?logo=pypi&color=blue)](https://pypi.org/project/wickra/)
[![npm](https://img.shields.io/npm/v/wickra.svg?logo=npm&color=red)](https://www.npmjs.com/package/wickra)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue)](#license)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/wickra-lib/wickra/badge)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra)
[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/13094/badge)](https://www.bestpractices.dev/projects/13094)
[![Build provenance](https://img.shields.io/badge/provenance-attested-brightgreen?logo=github)](https://github.com/wickra-lib/wickra/attestations)
[![Docs](https://img.shields.io/badge/docs-docs.wickra.org-0ea5e9?logo=readthedocs&logoColor=white)](https://docs.wickra.org)

**Streaming-first technical indicators. Install with `pip install wickra` — no system dependencies.**

Wickra is a multi-language technical-analysis library with a Rust core and
bindings for Python, Node.js, and WebAssembly. Every indicator is a state
machine that updates in O(1) per new data point, so live trading bots and
historical backtests share the exact same implementation.

```python
import numpy as np
import wickra as ta

# Batch: classic TA-Lib-style usage
prices = np.linspace(100, 200, 1000)
rsi = ta.RSI(14)
values = rsi.batch(prices)              # numpy array, NaN during warmup

# Streaming: same indicator, fed tick by tick
rsi = ta.RSI(14)
for price in live_feed:
    value = rsi.update(price)           # O(1) — no recomputation over history
    if value is not None and value > 70:
        print("overbought")
```

## Documentation

Full documentation lives at **[docs.wickra.org](https://docs.wickra.org)**:

- **Quickstarts** — [Rust](https://docs.wickra.org/Quickstart-Rust),
  [Python](https://docs.wickra.org/Quickstart-Python),
  [Node](https://docs.wickra.org/Quickstart-Node),
  [WASM](https://docs.wickra.org/Quickstart-WASM).
- **Indicators** — a per-indicator deep dive (formula, parameters, warmup) for
  every one of the 447 indicators; start at the
  [indicators overview](https://docs.wickra.org/Indicators-Overview).
- **Reference** — [warmup periods](https://docs.wickra.org/Warmup-Periods),
  [streaming vs batch](https://docs.wickra.org/Streaming-vs-Batch),
  [indicator chaining](https://docs.wickra.org/Indicator-Chaining), the
  [data layer](https://docs.wickra.org/Data-Layer).
- **Guides** — [Cookbook](https://docs.wickra.org/Cookbook),
  [TA-Lib migration](https://docs.wickra.org/TA-Lib-Migration),
  [FAQ](https://docs.wickra.org/FAQ).

## Why Wickra exists

Wickra started as a personal itch. The existing TA libraries never quite fit the
projects I was building, so I decided to build one from the ground up — partly to
learn, partly because I genuinely enjoy taking something that already exists and
trying to do it differently (and, ideally, better). It's open source because the
useful version of that itch is the one other people can build on too.

Plenty of TA libraries are fast. Each one forces a trade-off Wickra does not:

| Library          | Install     | Streaming   | Languages                   | Indicators | Active |
|------------------|-------------|-------------|-----------------------------|-----------:|--------|
| **★&nbsp;Wickra**| **clean**   | **yes, O(1)** | **Python · Node · WASM · Rust** | **423** | **yes** |
| kand             | clean       | yes         | Python · WASM · Rust        |       ~60  | yes    |
| ta-rs            | clean       | yes         | Rust only                   |       ~30  | stale  |
| yata             | clean       | partial     | Rust only                   |       ~35  | yes    |
| TA-Lib           | yes (C deps)| no          | many bindings               |      ~150  | barely |
| pandas-ta        | clean       | no          | Python                      |      ~130  | slow   |
| finta            | clean       | no          | Python                      |       ~80  | stale  |
| talipp           | clean       | yes         | Python                      |       ~40  | yes    |

Wickra's edge is **breadth with reach**: 447 indicators that all update in O(1)
per tick and ship natively to Python, Node.js, WebAssembly and Rust from a
single engine.

**On speed — and why Wickra isn't the fastest.** It deliberately isn't. The
leaner Rust crates (kand, ta-rs) win several of the micro-benchmarks below, and
those losses are shown rather than hidden. The gap is a *choice*, not a ceiling:
every `update` validates its input, runs a real warmup before it emits a value,
and returns an `Option` so a single bad tick can't silently poison the state.
ta-rs, by contrast, hands back a bare `f64` from the first tick with no
validation. If Wickra threw all of that away — raw `f64` out, no checks, no
warmup contract — it would match or beat the leanest crate on every row. It
keeps the guarantees instead, and still wins RSI, Bollinger and ATR against kand.
What no other library matches is the *combination*: catalogue size, native O(1)
streaming, NaN-safety, and four first-class language targets at once.

## Benchmarks

Three comparisons, split by layer and mode. Read them as **relative** speedups
on identical input — absolute µs depend on CPU, memory clock and OS scheduler,
not a universal contract.

- **Reproduced on:** Windows 11 Pro 26200, AMD Ryzen 9 9950X, 64 GB DDR5,
  Rust 1.92 (release: `lto = "fat"`, `codegen-units = 1`), Python 3.12.
- **Reproduce yourself:**
  - Rust core vs Rust crates: `cargo bench -p wickra-bench`
  - Python vs Python libs: `pip install -e bindings/python[bench]` then
    `python -m benchmarks.compare_libraries` (auto-detects installed peers).

### 1. Rust core vs the other Rust TA crates

Like-for-like, no language-binding overhead, over a 50 000-bar series (µs for
the whole series, lower = faster). This is the honest engine comparison —
Wickra wins some and loses some, and both are shown.

**Streaming** (one value fed per `update`):

| Indicator        | **★&nbsp;Wickra** | kand | ta-rs | yata |
|------------------|------------------:|-----:|------:|-----:|
| SMA(20)          | 50                | 38   | 47    | 38   |
| EMA(20)          | 154               | 69   | 56    | 69   |
| RSI(14)          | 164               | 216  | 74    | —    |
| MACD(12, 26, 9)  | 275               | 143  | 66    | —    |
| Bollinger(20, 2) | **128 ★**         | 248  | 168   | —    |
| ATR(14)          | 152               | 166  | 61    | —    |

**Batch** (whole series at once). Only Wickra and kand expose a batch API;
ta-rs and yata are streaming-only.

| Indicator        | **★&nbsp;Wickra** | kand |
|------------------|------------------:|-----:|
| SMA(20)          | 82                | 42   |
| EMA(20)          | 159               | 74   |
| RSI(14)          | **253 ★**         | 274  |
| MACD(12, 26, 9)  | 681               | 283  |
| Bollinger(20, 2) | **445 ★**         | 462  |
| ATR(14)          | 175               | 173  |

ta-rs is the per-indicator speed champion on almost every row — it returns a
bare `f64` with no warmup state and no input validation, trading away the
`None`-warmup and NaN-safety semantics Wickra keeps. Against kand, Wickra wins
streaming RSI, Bollinger and ATR (and batch RSI + Bollinger); Bollinger is the
one row where Wickra is the outright fastest of all four. The leaner crates
still win the pure recurrences (EMA, MACD) and SMA. yata exposes only SMA/EMA as
raw-value methods, so its other rows are omitted rather than faked.

### 2. Python vs the Python TA ecosystem — batch

Full pass over a 20 000-bar series, µs/op (lower = faster). **★** per row.

| Indicator        | **★&nbsp;Wickra** | finta               | TA-Lib | tulipy |
|------------------|------------------:|---------------------|--------|--------|
| SMA(20)          | **59.6 ★**        | 354.2 (5.9× slower) | ⧗      | ⧗      |
| EMA(20)          | **88.4 ★**        | 309.3 (3.5× slower) | ⧗      | ⧗      |
| RSI(14)          | **77.3 ★**        | 1 283 (16.6× slower)| ⧗      | ⧗      |
| MACD(12, 26, 9)  | **116.4 ★**       | 529.5 (4.6× slower) | ⧗      | ⧗      |
| Bollinger(20, 2) | **146.0 ★**       | 1 246 (8.5× slower) | ⧗      | ⧗      |
| ATR(14)          | **135.8 ★**       | 3 812 (28× slower)  | ⧗      | ⧗      |

> ⧗ = published by the CI Linux job. TA-Lib and tulipy ship C extensions that
> don't build cleanly on every desktop, so their canonical numbers come from the
> `cross-library-bench` workflow rather than this local table. pandas-ta needs
> Python ≥ 3.12 and isn't in the 3.11 CI matrix. The script auto-detects
> whichever peers are installed in your environment.

### 3. Python — streaming (per-tick latency)

Seed 5 000 bars, then feed ticks one at a time. talipp is the only Python peer
with a true incremental API; batch-only libraries like TA-Lib must recompute the
entire history on every tick — Wickra updates in O(1).

| Indicator        | **★&nbsp;Wickra (per tick)** | talipp (per tick)       |
|------------------|------------------------------:|-------------------------|
| SMA(20)          | **0.067 µs ★**                | 0.63 µs (9.4× slower)   |
| EMA(20)          | **0.051 µs ★**                | 0.63 µs (12.2× slower)  |
| RSI(14)          | **0.053 µs ★**                | 1.00 µs (19.1× slower)  |
| MACD(12, 26, 9)  | **0.071 µs ★**                | 3.64 µs (51.5× slower)  |
| Bollinger(20, 2) | **0.085 µs ★**                | 4.87 µs (57.2× slower)  |

Run the suite yourself:

```bash
cargo bench -p wickra-bench            # Rust core vs kand / ta-rs / yata
pip install -e bindings/python[bench]  # Python peers
python -m benchmarks.compare_libraries
```

## Indicators

447 streaming-first indicators across twenty-four families. Every one passes the
`batch == streaming` equivalence test, reference-value tests, and reset
semantics tests. Each has a per-indicator deep dive (formula, parameters,
warmup) at [docs.wickra.org](https://docs.wickra.org/Indicators-Overview).

| Family | Indicators |
|--------|-----------|
| Moving Averages      | SMA, EMA, WMA, DEMA, TEMA, HMA, KAMA, SMMA, TRIMA, ZLEMA, T3, VWMA, ALMA, McGinley Dynamic, FRAMA, VIDYA, JMA, Alligator, EVWMA, SWMA, GMA, EHMA, Median MA, Adaptive Laguerre, GD, Holt-Winters |
| Momentum Oscillators | RSI (Wilder), Anchored RSI, Stochastic, CCI, ROC, Williams %R, MFI, Awesome Oscillator, MOM, CMO, TSI, PMO, StochRSI, Ultimate Oscillator, RVI, PGO, KST, SMI, Laguerre RSI, Connors RSI, Inertia, ROC Percentage (ROCP), ROC Ratio (ROCR), ROC Ratio 100 (ROCR100), Disparity Index, Fisher RSI, RSX, Dynamic Momentum Index, Stochastic CCI, RMI, Derivative Oscillator, Elder Ray, Intraday Momentum Index, QQE |
| Trend & Directional  | MACD, MACD Fixed (MACDFIX), MACD Extended (MACDEXT), ADX (+DI/-DI), ADXR, Aroon, TRIX, Aroon Oscillator, Vortex, Random Walk Index, Trend Intensity Index, Wave Trend Oscillator, Mass Index, Choppiness Index, Vertical Horizontal Filter, Plus DM, Minus DM, Plus DI, Minus DI, DX, TTM Trend, Trend Strength Index, Qstick, Polarized Fractal Efficiency, Wave PM, Gator Oscillator, Kase Permission Stochastic |
| Price Oscillators    | PPO, DPO, Coppock, Accelerator Oscillator, Balance of Power, APO, AO Histogram, CFO, Zero-Lag MACD, Elder Impulse, STC, TSF Oscillator, MACD Histogram, PPO Histogram |
| Volatility & Bands   | ATR, Bollinger Bands, Keltner Channels, Donchian Channels, NATR, StdDev, Ulcer Index, Historical Volatility, Bollinger Bandwidth, %B, True Range, Chaikin Volatility, RVI (Relative Volatility Index), Parkinson Volatility, Garman-Klass Volatility, Rogers-Satchell Volatility, Yang-Zhang Volatility, Volatility Cone |
| Bands & Channels     | MA Envelope, Acceleration Bands, STARC Bands, ATR Bands, Hurst Channel, LinReg Channel, Standard Error Bands, Double Bollinger Bands, TTM Squeeze, Fractal Chaos Bands, VWAP StdDev Bands, Quartile Bands, Bomar Bands, Median Channel, Projection Bands, Projection Oscillator |
| Trailing Stops       | Parabolic SAR, Parabolic SAR Extended (SAREXT), SuperTrend, Chandelier Exit, Chande Kroll Stop, ATR Trailing Stop, HiLo Activator, Volty Stop, Yo-Yo Exit, Donchian Channel Stop, Percentage Trailing Stop, Step Trailing Stop, Renko Trailing Stop, Kase DevStop, Elder SafeZone, ATR Ratchet, NRTR, Time-Based Stop, Modified MA Stop |
| Volume               | OBV, VWAP (cumulative + rolling), ADL, Volume-Price Trend, Chaikin Money Flow, Chaikin Oscillator, Force Index, Ease of Movement, Klinger Volume Oscillator, Volume Oscillator, NVI, PVI, Williams A/D, Anchored VWAP, Demand Index, TSV, VZO, Market Facilitation Index, Volume RSI, Williams Accumulation/Distribution, Twiggs Money Flow, Trade Volume Index, Intraday Intensity Index, Better Volume, Volume-Weighted MACD |
| Price Statistics     | Typical Price, Median Price, Weighted Close, Linear Regression, Linear Regression Slope, Z-Score, Linear Regression Angle, Variance, Coefficient of Variation, Skewness, Kurtosis, Standard Error, Detrended StdDev, R², Median Absolute Deviation, Autocorrelation, Hurst Exponent, Pearson Correlation, Beta, Pairwise Beta, Pair Spread Z-Score, Lead-Lag Cross-Correlation, Cointegration, Relative Strength A-vs-B, Spearman Correlation, Mid Price, Mid Point, Average Price, Linear Regression Intercept, Time Series Forecast, Rolling Correlation, Rolling Covariance, OU Half-Life, Spread Hurst, Distance SSD, Beta-Neutral Spread, Variance Ratio, Granger Causality, Kalman Hedge Ratio, Spread Bollinger Bands, Spread AR(1) Coefficient |
| Ehlers / Cycle (DSP) | MAMA, FAMA, Fisher Transform, Inverse Fisher Transform, SuperSmoother, Hilbert Dominant Cycle, Hilbert Phasor, Hilbert DC Phase, Hilbert Trend Mode, Sine Wave, Decycler, Decycler Oscillator, Roofing Filter, Center of Gravity, Cybernetic Cycle, Adaptive Cycle, Empirical Mode Decomposition, Ehlers Stochastic, Instantaneous Trendline |
| Pivots & S/R         | Classic Pivots, Fibonacci Pivots, Camarilla, Woodie Pivots, DeMark Pivots, Williams Fractals, ZigZag |
| DeMark               | TD Setup, TD Sequential, TD DeMarker, TD REI, TD Pressure, TD Combo, TD Countdown, TD Lines, TD Range Projection, TD Differential, TD Open, TD Risk Level |
| Ichimoku & Charts    | Ichimoku Kinko Hyo (Tenkan, Kijun, Senkou A/B, Chikou), Heikin-Ashi |
| Alt-Chart Bars       | Renko (box-size bricks), Kagi (reversal-amount lines), Point & Figure (X/O columns) |
| Candlestick Patterns | Doji, Hammer, Inverted Hammer, Hanging Man, Shooting Star, Engulfing, Harami, Morning/Evening Star, Three White Soldiers/Black Crows, Piercing Line/Dark Cloud Cover, Marubozu, Tweezer, Spinning Top, Three Inside Up/Down, Three Outside Up/Down, Two Crows, Upside Gap Two Crows, Identical Three Crows, Three Line Strike, Three Stars in the South, Abandoned Baby, Advance Block, Belt-hold, Breakaway, Counterattack, Doji Star, Dragonfly Doji, Gravestone Doji, Long-Legged Doji, Rickshaw Man, Evening Doji Star, Morning Doji Star, Gap Side-by-Side White, High-Wave, Hikkake, Modified Hikkake, Homing Pigeon, On-Neck, In-Neck, Thrusting, Separating Lines, Kicking, Kicking by Length, Ladder Bottom, Mat Hold, Matching Low, Long Line, Short Line, Rising Three Methods, Falling Three Methods, Upside Gap Three Methods, Downside Gap Three Methods, Stalled Pattern, Stick Sandwich, Takuri, Closing Marubozu, Opening Marubozu, Tasuki Gap, Unique Three River, Concealing Baby Swallow |
| Chart Patterns       | Double Top / Bottom, Triple Top / Bottom, Head and Shoulders, Triangle (asc/desc/sym), Wedge (rising/falling), Flag / Pennant, Rectangle / Range, Cup and Handle |
| Harmonic Patterns    | AB=CD, Gartley, Butterfly, Bat, Crab, Shark, Cypher, Three Drives |
| Fibonacci            | Fibonacci Retracement, Fibonacci Extension, Fibonacci Projection, Auto-Fibonacci, Golden Pocket, Fibonacci Confluence, Fibonacci Fan, Fibonacci Arcs, Fibonacci Channel, Fibonacci Time Zones |
| Microstructure       | Order-Book Imbalance (Top-1 / Top-N / Full), Microprice, Quoted Spread, Depth Slope, Signed Volume, Cumulative Volume Delta, Trade Imbalance, Effective Spread, Realized Spread, Kyle's Lambda, Footprint, Order Flow Imbalance, VPIN, Amihud Illiquidity, Roll Measure |
| Derivatives          | Funding Rate, Funding Rate Mean, Funding Rate Z-Score, Funding Basis, Open-Interest Delta, OI / Price Divergence, OI-Weighted Price, Long/Short Ratio, Taker Buy/Sell Ratio, Liquidation Features, Term-Structure Basis, Calendar Spread |
| Market Profile       | Value Area (POC / VAH / VAL), Volume Profile (histogram), TPO Profile, Initial Balance, Opening Range |
| Market Breadth       | Advance/Decline Line, Advance/Decline Ratio, Advance/Decline Volume Line, McClellan Oscillator, McClellan Summation Index, TRIN / Arms Index, Breadth Thrust, New Highs - New Lows, High-Low Index, Percent Above Moving Average, Up/Down Volume Ratio, Bullish Percent Index, Cumulative Volume Index, Absolute Breadth Index, TICK Index |
| Risk / Performance   | Sharpe Ratio, Sortino Ratio, Calmar Ratio, Omega Ratio, Max Drawdown, Average Drawdown, Drawdown Duration, Pain Index, Value at Risk, Conditional Value at Risk (CVaR), Profit Factor, Gain/Loss Ratio, Recovery Factor, Kelly Criterion, Treynor Ratio, Information Ratio, Alpha (Jensen) |
| Seasonality & Session | Session VWAP, Session High/Low, Session Range, Average Daily Range, Overnight Gap, Overnight/Intraday Return, Turn-of-Month, Seasonal Z-Score, Time-of-Day Return Profile, Day-of-Week Profile, Intraday Volatility Profile, Volume-by-Time Profile |

Every candlestick pattern emits a signed per-bar value — `+1.0` bullish,
`−1.0` bearish, `0.0` none — so the family drops straight into a feature matrix
as one column each. `Doji` is direction-less by default (`+1.0` / `0.0`);
construct it in signed mode (`Doji::new().signed()`, `Doji(signed=True)`,
`new Doji(true)`) for a dragonfly / gravestone `±1` reading.

Adding a new indicator means implementing one trait in Rust; all four bindings
inherit it automatically.

## Languages

| Binding           | Install                                       | Example |
|-------------------|-----------------------------------------------|---------|
| Python (PyO3)     | `pip install wickra`                          | `examples/python/backtest.py` |
| Node.js (napi-rs) | `npm install wickra`                          | `examples/node/backtest.js` |
| Browser / WASM    | `npm install wickra-wasm`                     | `examples/wasm/index.html` |
| Rust              | `cargo add wickra`                            | `examples/rust/src/bin/backtest.rs` |

Each binding ships several runnable examples (streaming, backtest, live feed);
[`examples/README.md`](examples/README.md) is the full cross-language index.

The wickra-core crate is `unsafe`-forbidden, so every binding inherits a
memory-safe implementation.

## Rust API

```rust
use wickra::{Indicator, BatchExt, Chain, Ema, Rsi, Sma};

// Streaming or batch — same trait, same code.
let mut sma = Sma::new(14)?;
let out: Vec<Option<f64>> = sma.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);

let mut rsi = Rsi::new(14)?;
for price in live_feed {
    if let Some(v) = rsi.update(price) {
        println!("RSI = {v}");
    }
}

// Compose indicators: RSI(7) on top of EMA(14).
let mut chain = Chain::new(Ema::new(14)?, Rsi::new(7)?);
chain.update(price);
```

## Live data sources

`wickra-data` (separate crate, opt-in) ships:

- A streaming OHLCV **CSV reader**.
- A **tick-to-candle aggregator** with arbitrary timeframes.
- A **candle resampler** for multi-timeframe analysis (1m → 5m → 1h on the fly).
- A **Binance Spot WebSocket** kline adapter (feature `live-binance`).

```rust
use wickra::{Indicator, Rsi};
use wickra_data::live::binance::{BinanceKlineStream, Interval};

let mut stream = BinanceKlineStream::connect(&["BTCUSDT".into()], Interval::OneMinute).await?;
let mut rsi = Rsi::new(14)?;
while let Some(event) = stream.next_event().await? {
    if event.is_closed {
        if let Some(v) = rsi.update(event.candle.close) {
            println!("RSI = {v:.2}");
        }
    }
}
```

A Python live-trading example using the public `websockets` package lives at
`examples/python/live_trading.py`.

## Project layout

```
wickra/
├── crates/
│   ├── wickra-core/         core engine + all 447 indicators
│   ├── wickra/              top-level facade crate (publishes on crates.io) + benches/
│   ├── wickra-data/         CSV reader, tick aggregator, live exchange feeds
│   └── wickra-bench/        internal cross-library benchmark harness (not published)
├── bindings/
│   ├── python/              PyO3 + maturin (publishes on PyPI)
│   ├── node/                napi-rs (publishes on npm)
│   └── wasm/                wasm-bindgen (browsers, bundlers, Node)
├── examples/                examples/README.md indexes every language
│   ├── data/                real BTCUSDT OHLCV datasets, one per timeframe
│   ├── rust/                Rust workspace member (`wickra-examples`)
│   ├── python/              backtest, live trading, parallel assets, multi-tf
│   ├── node/                streaming, backtest, live trading (load `wickra`)
│   └── wasm/                browser demo for `wickra-wasm`
└── .github/workflows/       CI and release pipelines
```

Wickra's own regression benchmarks live in `crates/wickra/benches/`; the
cross-library comparison against kand, ta-rs and yata lives in the internal
`crates/wickra-bench/` crate. Runnable Rust examples live in the workspace member
crate at `examples/rust/`. There is no top-level `benches/` directory.

## Building everything from source

```bash
# Rust core + tests
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p wickra           # Wickra's own regression benchmarks
cargo bench -p wickra-bench     # cross-library comparison (kand, ta-rs, yata)

# Python binding (requires Rust toolchain + maturin)
cd bindings/python
maturin develop --release
pytest

# WASM binding (requires wasm-pack + wasm32-unknown-unknown target)
wasm-pack build bindings/wasm --target web --release --features panic-hook

# Node binding (requires @napi-rs/cli)
cd bindings/node && npm install && npm run build && npm test
```

## Testing

Every layer is covered; run the suites with the commands in
[Building everything from source](#building-everything-from-source).

- `wickra-core`: unit tests per indicator — textbook reference values
  (Wilder RSI, Bollinger Bands, MACD, ATR, Stochastic), `batch == streaming`
  equivalence, `reset` semantics, NaN/Inf handling, and property tests.
- `wickra-data`: unit tests for CSV decoding, the tick aggregator, the
  resampler, and the Binance payload parser.
- `bindings/python`: pytest covering smoke checks, streaming/batch
  equivalence, reference values, lifecycle, input validation, and
  dict/tuple candle inputs.
- `bindings/node`: `node --test` cases for batch, streaming, and reference
  values across all indicators.
- `bindings/wasm`: `wasm-bindgen-test` cases for constructors, equivalence,
  and reference values.

## Contributing

Contributions are very welcome — issues, bug reports, ideas, and pull requests
all land in the same place: <https://github.com/wickra-lib/wickra>.

A short orientation for first-time contributors:

- **Adding an indicator.** Implement the `Indicator` trait in
  `crates/wickra-core/src/indicators/<name>.rs`, wire it into
  `indicators/mod.rs` and the crate root, and add reference-value tests,
  a `batch == streaming` equivalence test, and (where it makes sense) a
  proptest. The four bindings inherit your indicator automatically once
  you expose it in the language wrappers.
- **Fixing a numeric bug.** Add a failing test that pins the textbook value
  first, then fix the math. Property tests in `crates/wickra-core` catch
  most regressions; please don't disable them.
- **Improving a binding.** Each binding lives under `bindings/<lang>` with
  its own tests; please keep the `batch == streaming` invariant.
- **Style.** `cargo fmt --all` + `cargo clippy --workspace --all-targets -- -D warnings`
  are CI gates; running them locally before pushing keeps reviews short.

For larger architectural changes, open an issue first so we can sketch the
shape together before you invest the time.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option. Use it, fork it, modify it, redistribute it — commercially or
not — file issues, send pull requests; all welcome.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Disclaimer

Wickra is an indicator toolkit, not a trading system. Values it computes are
deterministic transforms of the input data — they are not financial advice and
they do not predict the market. Any use of this library in a production
trading context is at your own risk.

The library is provided **as is**, without warranty of any kind; see
[LICENSE](LICENSE) for the full terms.

---

<p align="center">
  <a href="https://github.com/wickra-lib/wickra/stargazers">
    <img alt="GitHub stars" src="https://img.shields.io/github/stars/wickra-lib/wickra?style=for-the-badge&logo=github&logoColor=white&color=ffd866">
  </a>
  <a href="https://github.com/wickra-lib/wickra/network/members">
    <img alt="GitHub forks" src="https://img.shields.io/github/forks/wickra-lib/wickra?style=for-the-badge&logo=github&logoColor=white&color=78dce8">
  </a>
  <a href="https://github.com/wickra-lib/wickra/issues">
    <img alt="GitHub issues" src="https://img.shields.io/github/issues/wickra-lib/wickra?style=for-the-badge&logo=github&logoColor=white&color=ff6188">
  </a>
</p>

<p align="center">
  If Wickra saved you time, the cheapest way to say thanks is to ⭐ the repo.
</p>

<p align="center">
  <a href="https://star-history.com/#wickra-lib/wickra&Date">
    <img alt="Wickra star history" width="640"
         src="https://api.star-history.com/svg?repos=wickra-lib/wickra&type=Date&theme=dark">
  </a>
</p>
