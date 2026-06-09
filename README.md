<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514" alt="Wickra — streaming-first technical indicators" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/ci.svg)](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml)
[![CodeQL](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/codeql.svg)](https://github.com/wickra-lib/wickra/actions/workflows/codeql.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra)
[![GitHub release](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/release.svg)](https://github.com/wickra-lib/wickra/releases/latest)
[![crates.io](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/crates.svg)](https://crates.io/crates/wickra)
[![PyPI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/pypi.svg)](https://pypi.org/project/wickra/)
[![npm](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/npm.svg)](https://www.npmjs.com/package/wickra)
[![NuGet](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/nuget.svg)](https://www.nuget.org/packages/Wickra)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/license.svg)](#license)
[![OpenSSF Scorecard](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/scorecard.svg)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra)
[![OpenSSF Best Practices](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/best-practices.svg)](https://www.bestpractices.dev/projects/13094)
[![Build provenance](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/provenance.svg)](https://github.com/wickra-lib/wickra/attestations)
[![Docs](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/docs.svg)](https://docs.wickra.org)

**Streaming-first technical indicators. Install with `pip install wickra` — no system dependencies.**

Wickra is a multi-language technical-analysis library with a Rust core and
native bindings for Python, Node.js and WebAssembly, plus a C ABI that C, C++,
C# / .NET, Go, Java, R and any other C-capable language links against. Every indicator is a
state machine that updates in O(1) per new data point, so live trading bots and
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
  [WASM](https://docs.wickra.org/Quickstart-WASM),
  [C](https://docs.wickra.org/Quickstart-C),
  [C#](https://docs.wickra.org/Quickstart-CSharp),
  [Go](https://docs.wickra.org/Quickstart-Go),
  [R](https://docs.wickra.org/Quickstart-R).
- **Indicators** — a per-indicator deep dive (formula, parameters, warmup) for
  every one of the 514 indicators; start at the
  [indicators overview](https://docs.wickra.org/Indicators-Overview).
- **Reference** — [warmup periods](https://docs.wickra.org/Warmup-Periods),
  [streaming vs batch](https://docs.wickra.org/Streaming-vs-Batch),
  [indicator chaining](https://docs.wickra.org/Indicator-Chaining), the
  [data layer](https://docs.wickra.org/Data-Layer).
- **Guides** — [Cookbook](https://docs.wickra.org/Cookbook),
  [TA-Lib migration](https://docs.wickra.org/TA-Lib-Migration),
  [FAQ](https://docs.wickra.org/FAQ).

## Why Wickra

Most TA libraries are fast, *or* multi-language, *or* broad. Wickra refuses to
pick. It's the streaming-first engine built for the workload the others treat as
an afterthought — **live, tick-by-tick data** — without giving up the breadth of
a full batch library, and without making you reimplement your indicators four
times to get there.

- **The biggest streaming-native catalogue, period.** 514 indicators across 24
  families — candlesticks, harmonic & chart patterns, market profile, market
  breadth, Renko/Kagi/Point&Figure bars, Ehlers DSP cycles, risk/performance
  metrics — every single one updating in **O(1) per tick**. TA-Lib ships ~150 and
  none of them stream.
- **One Rust core, five first-class targets.** Native **Python · Node.js ·
  WebAssembly · Rust** plus a **C ABI** for C / C++, C# / .NET, Go, Java, R and any other C-capable language —
  identical math, identical results, zero per-language reimplementation and zero
  GIL bottleneck.
- **Correct by construction, not by hope.** Every `update` validates its input,
  runs a real warmup, and returns an `Option` so a single bad tick can't silently
  poison state. `batch == streaming` is **bit-exact, fuzzed and 100 %-line-covered
  for all 514 indicators**.
- **Orders of magnitude faster where it counts.** In streaming Wickra is **11–56×**
  faster than the only other incremental peer and **thousands of times** faster
  than recompute-on-every-tick libraries. On batch it wins several rows outright
  and trades the simple recurrences (SMA, EMA, MACD) for its guarantees — and
  the losses are shown, not hidden.
- **Install in one line, anywhere.** `pip install wickra` / `npm install wickra` —
  precompiled wheels and binaries, **no C toolchain, none of TA-Lib's setup pain**.
  macOS · Linux · Windows.
- **Batteries included.** Indicator chaining, a streaming OHLCV CSV reader, and a
  live Binance kline feed ship in the box.
- **Truly permissive.** **MIT OR Apache-2.0** — drop it straight into commercial
  and closed-source work.

Every other library forces one of those compromises. Wickra doesn't:

| Library          | Install     | Streaming   | Languages                   | Indicators | Active |
|------------------|-------------|-------------|-----------------------------|-----------:|--------|
| **★&nbsp;Wickra**| **clean**   | **yes, O(1)** | **Rust · Python · Node · WASM** | **514** | **yes** |
|                  |             |               | **C · C# · Go · Java · R**          |            |        |
| kand             | clean       | yes         | Python · WASM · Rust        |       ~60  | yes    |
| ta-rs            | clean       | yes         | Rust only                   |       ~30  | stale  |
| yata             | clean       | partial     | Rust only                   |       ~35  | yes    |
| TA-Lib           | yes (C deps)| no          | many bindings               |      ~150  | barely |
| pandas-ta        | clean       | no          | Python                      |      ~130  | slow   |
| finta            | clean       | no          | Python                      |       ~80  | stale  |
| talipp           | clean       | yes         | Python                      |       ~40  | yes    |

Broad, multi-language, streaming-native **and** honest about its trade-offs — at
the same time. That's the combination no one else ships.

## Why Wickra exists

Wickra started as a personal itch. The existing TA libraries never quite fit the
projects I was building, so I decided to build one from the ground up — partly to
learn, partly because I genuinely enjoy taking something that already exists and
trying to do it differently (and, ideally, better). It's open source because the
useful version of that itch is the one other people can build on too.

## Benchmarks

Wickra updates every indicator in **O(1)** per tick. In **streaming** — the
workload it is built for — it is **11–56× faster** than the only other incremental
peer and **thousands of times** faster than recompute-on-every-tick libraries.
**Batch** is competitive: it wins several rows outright and trades a few µs
elsewhere for `None`-warmup, NaN-safety and bit-exact `batch == streaming`.

Full tables (Rust + Python, streaming + batch) and how to reproduce them live in
**[BENCHMARKS.md](BENCHMARKS.md)**.

## Indicators

514 streaming-first indicators across twenty-four families. Every one passes the
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
| Price Statistics     | Typical Price, Median Price, Weighted Close, Linear Regression, Linear Regression Slope, Z-Score, Linear Regression Angle, Variance, Coefficient of Variation, Skewness, Kurtosis, Standard Error, Detrended StdDev, R², Median Absolute Deviation, Autocorrelation, Hurst Exponent, Pearson Correlation, Beta, Pairwise Beta, Pair Spread Z-Score, Lead-Lag Cross-Correlation, Cointegration, Relative Strength A-vs-B, Spearman Correlation, Mid Price, Mid Point, Average Price, Linear Regression Intercept, Time Series Forecast, Rolling Correlation, Rolling Covariance, OU Half-Life, Spread Hurst, Distance SSD, Beta-Neutral Spread, Variance Ratio, Granger Causality, Kalman Hedge Ratio, Spread Bollinger Bands, Spread AR(1) Coefficient, Jarque-Bera, Rolling Min-Max Scaler, Shannon Entropy, Sample Entropy, Kendall Tau |
| Ehlers / Cycle (DSP) | MAMA, FAMA, Fisher Transform, Inverse Fisher Transform, SuperSmoother, Hilbert Dominant Cycle, Hilbert Phasor, Hilbert DC Phase, Hilbert Trend Mode, Sine Wave, Decycler, Decycler Oscillator, Roofing Filter, Center of Gravity, Cybernetic Cycle, Adaptive Cycle, Empirical Mode Decomposition, Ehlers Stochastic, Instantaneous Trendline, Highpass Filter, Reflex, Trendflex, Correlation Trend Indicator, Adaptive RSI, Universal Oscillator, Adaptive CCI, Bandpass Filter, Even Better Sinewave, Autocorrelation Periodogram |
| Pivots & S/R         | Classic Pivots, Fibonacci Pivots, Camarilla, Woodie Pivots, DeMark Pivots, Williams Fractals, ZigZag, Central Pivot Range, Murrey Math Lines, Andrews Pitchfork, Volume-Weighted Support/Resistance, Pivot Reversal |
| DeMark               | TD Setup, TD Sequential, TD DeMarker, TD REI, TD Pressure, TD Combo, TD Countdown, TD Lines, TD Range Projection, TD Differential, TD Open, TD Risk Level, TD Camouflage, TD Clop, TD Clopwin, TD Propulsion, TD Trap, TD D-Wave, TD Moving Averages |
| Ichimoku & Charts    | Ichimoku Kinko Hyo (Tenkan, Kijun, Senkou A/B, Chikou), Heikin-Ashi, Heikin-Ashi Oscillator, Three Line Break, Smoothed Heikin-Ashi, Equivolume, CandleVolume |
| Alt-Chart Bars       | Renko (box-size bricks), Kagi (reversal-amount lines), Point & Figure (X/O columns), Range, Tick, Volume, Dollar, Imbalance, Run, Three-Line Break |
| Candlestick Patterns | Doji, Hammer, Inverted Hammer, Hanging Man, Shooting Star, Engulfing, Harami, Morning/Evening Star, Three White Soldiers/Black Crows, Piercing Line/Dark Cloud Cover, Marubozu, Tweezer, Spinning Top, Three Inside Up/Down, Three Outside Up/Down, Two Crows, Upside Gap Two Crows, Identical Three Crows, Three Line Strike, Three Stars in the South, Abandoned Baby, Advance Block, Belt-hold, Breakaway, Counterattack, Doji Star, Dragonfly Doji, Gravestone Doji, Long-Legged Doji, Rickshaw Man, Evening Doji Star, Morning Doji Star, Gap Side-by-Side White, High-Wave, Hikkake, Modified Hikkake, Homing Pigeon, On-Neck, In-Neck, Thrusting, Separating Lines, Kicking, Kicking by Length, Ladder Bottom, Mat Hold, Matching Low, Long Line, Short Line, Rising Three Methods, Falling Three Methods, Upside Gap Three Methods, Downside Gap Three Methods, Stalled Pattern, Stick Sandwich, Takuri, Closing Marubozu, Opening Marubozu, Tasuki Gap, Unique Three River, Concealing Baby Swallow, Tristar, Harami Cross, Tower Top/Bottom, Dumpling Top, New Price Lines, Frying Pan Bottom |
| Chart Patterns       | Double Top / Bottom, Triple Top / Bottom, Head and Shoulders, Triangle (asc/desc/sym), Wedge (rising/falling), Flag / Pennant, Rectangle / Range, Cup and Handle |
| Harmonic Patterns    | AB=CD, Gartley, Butterfly, Bat, Crab, Shark, Cypher, Three Drives |
| Fibonacci            | Fibonacci Retracement, Fibonacci Extension, Fibonacci Projection, Auto-Fibonacci, Golden Pocket, Fibonacci Confluence, Fibonacci Fan, Fibonacci Arcs, Fibonacci Channel, Fibonacci Time Zones |
| Microstructure       | Order-Book Imbalance (Top-1 / Top-N / Full), Microprice, Quoted Spread, Depth Slope, Signed Volume, Cumulative Volume Delta, Trade Imbalance, Effective Spread, Realized Spread, Kyle's Lambda, Footprint, Order Flow Imbalance, VPIN, Amihud Illiquidity, Roll Measure, Trade-Sign Autocorrelation, Hasbrouck Information Share |
| Derivatives          | Funding Rate, Funding Rate Mean, Funding Rate Z-Score, Funding Basis, Open-Interest Delta, OI / Price Divergence, OI-Weighted Price, Long/Short Ratio, Taker Buy/Sell Ratio, Liquidation Features, Term-Structure Basis, Calendar Spread, Estimated Leverage Ratio, OI-to-Volume Ratio, Perpetual Premium Index, Funding-Implied APR, Open-Interest Momentum |
| Market Profile       | Value Area (POC / VAH / VAL), Volume Profile (histogram), TPO Profile, Initial Balance, Opening Range, Naked POC, Single Prints, Profile Shape, High/Low Volume Nodes, Composite Profile |
| Market Breadth       | Advance/Decline Line, Advance/Decline Ratio, Advance/Decline Volume Line, McClellan Oscillator, McClellan Summation Index, TRIN / Arms Index, Breadth Thrust, New Highs - New Lows, High-Low Index, Percent Above Moving Average, Up/Down Volume Ratio, Bullish Percent Index, Cumulative Volume Index, Absolute Breadth Index, TICK Index |
| Risk / Performance   | Sharpe Ratio, Sortino Ratio, Calmar Ratio, Omega Ratio, Max Drawdown, Average Drawdown, Drawdown Duration, Pain Index, Value at Risk, Conditional Value at Risk (CVaR), Profit Factor, Gain/Loss Ratio, Recovery Factor, Kelly Criterion, Treynor Ratio, Information Ratio, Alpha (Jensen) |
| Seasonality & Session | Session VWAP, Session High/Low, Session Range, Average Daily Range, Overnight Gap, Overnight/Intraday Return, Turn-of-Month, Seasonal Z-Score, Time-of-Day Return Profile, Day-of-Week Profile, Intraday Volatility Profile, Volume-by-Time Profile |

Every candlestick pattern emits a signed per-bar value — `+1.0` bullish,
`−1.0` bearish, `0.0` none — so the family drops straight into a feature matrix
as one column each. `Doji` is direction-less by default (`+1.0` / `0.0`);
construct it in signed mode (`Doji::new().signed()`, `Doji(signed=True)`,
`new Doji(true)`) for a dragonfly / gravestone `±1` reading.

Adding a new indicator means implementing one trait in Rust; every binding
inherits it automatically (the C ABI — and the C#, Go, Java and R bindings generated from
it — regenerate from the core).

## Languages

| Binding           | Install                                       | Example |
|-------------------|-----------------------------------------------|---------|
| Python (PyO3)     | `pip install wickra`                          | `examples/python/backtest.py` |
| Node.js (napi-rs) | `npm install wickra`                          | `examples/node/backtest.js` |
| Browser / WASM    | `npm install wickra-wasm`                     | `examples/wasm/index.html` |
| Rust              | `cargo add wickra`                            | `examples/rust/src/bin/backtest.rs` |
| C / C++ (C ABI)   | header + library, see [`bindings/c`](bindings/c) | `examples/c/streaming.c` |
| C# / .NET (C ABI) | `dotnet add package Wickra`, see [`bindings/csharp`](bindings/csharp) | `examples/csharp/streaming` |
| Go (cgo, C ABI)   | `go get github.com/wickra-lib/wickra/bindings/go`, see [`bindings/go`](bindings/go) | `examples/go/streaming` |
| Java (FFM, C ABI)  | Maven Central `org.wickra:wickra`, see [`bindings/java`](bindings/java) | `examples/java` (`Streaming`) |
| R (`.Call`, C ABI) | `R CMD INSTALL bindings/r`, see [`bindings/r`](bindings/r) | `examples/r/streaming.R` |

Each binding ships several runnable examples (streaming, backtest, live feed);
[`examples/README.md`](examples/README.md) is the full cross-language index.

The wickra-core crate is `unsafe`-forbidden, so the native bindings are
memory-safe end to end. The C ABI runs the same safe core; only its thin FFI
boundary uses `unsafe`, and the caller owns handle lifetimes (`_new` / `_free`).

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
│   ├── wickra-core/         core engine + all 514 indicators
│   ├── wickra/              top-level facade crate (publishes on crates.io) + benches/
│   ├── wickra-data/         CSV reader, tick aggregator, live exchange feeds
│   └── wickra-bench/        internal cross-library benchmark harness (not published)
├── bindings/
│   ├── python/              PyO3 + maturin (publishes on PyPI)
│   ├── node/                napi-rs (publishes on npm)
│   ├── wasm/                wasm-bindgen (browsers, bundlers, Node)
│   ├── c/                   C ABI (cdylib + staticlib) + generated include/wickra.h
│   ├── csharp/              .NET binding over the C ABI (publishes on NuGet)
│   ├── go/                  Go binding over the C ABI via cgo (module tag)
│   ├── r/                   R binding over the C ABI via .Call (R package)
│   └── java/                Java binding over the C ABI via the FFM API (Maven Central)
├── examples/                examples/README.md indexes every language
│   ├── data/                real BTCUSDT OHLCV datasets, one per timeframe
│   ├── rust/                Rust workspace member (`wickra-examples`)
│   ├── python/              backtest, live trading, parallel assets, multi-tf
│   ├── node/                streaming, backtest, live trading (load `wickra`)
│   ├── wasm/                browser demo for `wickra-wasm`
│   ├── c/                   C smoke + streaming, C++ RAII wrapper
│   ├── csharp/              streaming, backtest, strategies (load `Wickra`)
│   ├── go/                  streaming, backtest, strategies (cgo binding)
│   ├── r/                   streaming, backtest, strategies (.Call binding)
│   └── java/                streaming, backtest, strategies (FFM binding)
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

# C ABI (cdylib + staticlib + generated header)
cargo build -p wickra-c --release
cmake -S examples/c -B examples/c/build -DWICKRA_LIB_DIR="$PWD/target/release"
cmake --build examples/c/build && ctest --test-dir examples/c/build --output-on-failure

# C# / .NET binding (requires the .NET 8 SDK; links the C ABI above)
dotnet test bindings/csharp/Wickra.Tests/Wickra.Tests.csproj

# Go binding (requires a C compiler for cgo; links the C ABI above)
cp target/release/libwickra.so bindings/go/lib/   # .dylib on macOS, wickra.dll on Windows
cd bindings/go && go test ./...

# R binding (requires a C toolchain / Rtools; links the C ABI above)
WICKRA_INCLUDE_DIR="$PWD/bindings/c/include" WICKRA_LIB_DIR="$PWD/target/release" \
  R CMD INSTALL bindings/r

# Java binding (requires JDK 22+ and Maven; links the C ABI above)
mvn -f bindings/java test
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
- `bindings/go`: `go test` cases covering one indicator per FFI archetype
  (scalar/batch, multi-output, bars, profile, array input), reset, and lifecycle.
- `bindings/r`: `testthat` cases covering one indicator per FFI archetype
  (scalar/batch, multi-output, bars, profile, array input), reset, and validation.
- `bindings/java`: JUnit cases covering one indicator per FFI archetype
  (scalar/batch, multi-output, bars, profile, array input) plus batch equivalence.

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
