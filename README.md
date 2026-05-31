<p align="center">
  <a href="https://wickra.org"><img src="https://wickra.org/og-banner.webp" alt="Wickra — streaming-first technical indicators" width="100%"></a>
</p>

[![CI](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml/badge.svg)](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml)
[![CodeQL](https://github.com/wickra-lib/wickra/actions/workflows/codeql.yml/badge.svg)](https://github.com/wickra-lib/wickra/actions/workflows/codeql.yml)
[![codecov](https://codecov.io/gh/wickra-lib/wickra/branch/main/graph/badge.svg)](https://codecov.io/gh/wickra-lib/wickra)
[![GitHub release](https://img.shields.io/github/v/release/wickra-lib/wickra?logo=github&color=green)](https://github.com/wickra-lib/wickra/releases/latest)
[![crates.io](https://img.shields.io/crates/v/wickra.svg?logo=rust&color=orange)](https://crates.io/crates/wickra)
[![PyPI](https://img.shields.io/pypi/v/wickra.svg?logo=pypi&color=blue)](https://pypi.org/project/wickra/)
[![npm](https://img.shields.io/npm/v/wickra.svg?logo=npm&color=red)](https://www.npmjs.com/package/wickra)
[![License: PolyForm-NC](https://img.shields.io/badge/license-PolyForm--NC--1.0.0-purple)](LICENSE)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/wickra-lib/wickra/badge)](https://scorecard.dev/viewer/?uri=github.com/wickra-lib/wickra)
[![Build provenance](https://img.shields.io/badge/provenance-attested-brightgreen?logo=github)](https://github.com/wickra-lib/wickra/attestations)

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

## Why Wickra exists

The Python TA ecosystem has plenty of libraries — TA-Lib, pandas-ta, finta,
talipp, tulipy — and every one of them shares the same blind spot:

| Library                | Install pain    | Streaming | Multi-language | Active |
|------------------------|-----------------|-----------|----------------|--------|
| **★&nbsp;Wickra**      | **clean**       | **yes**   | **Python + Node + WASM + Rust** | **yes** |
| TA-Lib (Python)        | yes (C deps)    | no        | no             | barely |
| pandas-ta              | clean           | no        | no             | slow   |
| finta                  | clean           | no        | no             | stale  |
| ta-lib-python          | yes (C deps)    | no        | no             | barely |
| talipp                 | clean           | yes       | no             | yes    |
| Tulip Indicators       | yes (C deps)    | no        | partial        | stale  |
| ooples (C#)            | clean           | no        | C# only        | yes    |

Wickra is the only library that combines all of: clean install, streaming,
multi-language reach, and active maintenance.

## Benchmark: how much faster is "streaming-first"?

The numbers below were measured on a single developer workstation and are not
guaranteed to reproduce identically on different hardware — absolute µs values
depend on CPU, memory clock and OS scheduler. Read them as **relative
speedups** between libraries on identical input, not as a universal
performance contract.

- **Reproduced on:** Windows 11 Pro 26200, AMD Ryzen 9 9950X, 64 GB DDR5,
  Rust 1.92 (release profile, `lto = "fat"`, `codegen-units = 1`),
  Python 3.12, Node 20.
- **Reproduce yourself:** `pip install -e bindings/python[bench]` then
  `python -m benchmarks.compare_libraries`. The script auto-detects every
  installed peer library and runs them on the same generated inputs as
  Wickra. The CI job `cross-library-bench` runs the same script on every
  push and uploads the raw report as a build artefact.

Lower µs/op = faster. Wickra wins every batch category outright, and the
streaming gap widens linearly with how much history a batch-only library has
to recompute on every tick.

### Batch — single full pass over a 20 000-bar series

Reading the table: each cell shows that library's runtime, plus how many times
slower it is than Wickra in parentheses. **★** marks the winner per row.

| Indicator           | **★&nbsp;Wickra**   | finta                       | talipp                        |
|---------------------|---------------------|-----------------------------|-------------------------------|
| SMA(20)             | **95.6 µs ★**       | 343.5 µs (3.6× slower)      | 7 640.6 µs (79.9× slower)     |
| EMA(20)             | **64.6 µs ★**       | 223.1 µs (3.5× slower)      | 12 160.9 µs (188.2× slower)   |
| RSI(14)             | **126.2 µs ★**      | 1 107.1 µs (8.8× slower)    | 15 792.2 µs (125.1× slower)   |
| MACD(12, 26, 9)     | **119.0 µs ★**      | 531.8 µs (4.5× slower)      | 49 788.1 µs (418.2× slower)   |
| Bollinger(20, 2.0)  | **105.3 µs ★**      | 812.0 µs (7.7× slower)      | 130 938.3 µs (1 243.7× slower)|
| ATR(14)             | **123.5 µs ★**      | 5 144.8 µs (41.7× slower)   | 28 816.0 µs (233.4× slower)   |

### Streaming — per-tick latency after seeding with 5 000 historical bars

A batch-only library has to re-run its full indicator over the entire history on
every new tick; Wickra updates state in O(1).

| Indicator | **★&nbsp;Wickra (per tick)** | talipp (per tick)         |
|-----------|---------------------|---------------------------|
| RSI(14)   | **0.119 µs ★**      | 1.644 µs (13.8× slower)   |

> TA-Lib and pandas-ta are not included here because both fail to install
> cleanly on Windows without C build tooling — which is precisely the install
> pain Wickra was built to remove. The benchmark script auto-detects every
> peer library it can find and runs them on the same inputs as Wickra; install
> them in your environment to see those rows light up too.

Run the suite yourself:

```bash
pip install -e bindings/python[bench]
python -m benchmarks.compare_libraries
```

## Indicators

214 streaming-first indicators across sixteen families. Every one passes the
`batch == streaming` equivalence test, reference-value tests, and reset
semantics tests.

| Family | Indicators |
|--------|-----------|
| Moving Averages      | SMA, EMA, WMA, DEMA, TEMA, HMA, KAMA, SMMA, TRIMA, ZLEMA, T3, VWMA, ALMA, McGinley Dynamic, FRAMA, VIDYA, JMA, Alligator, EVWMA |
| Momentum Oscillators | RSI (Wilder), Stochastic, CCI, ROC, Williams %R, MFI, Awesome Oscillator, MOM, CMO, TSI, PMO, StochRSI, Ultimate Oscillator, RVI, PGO, KST, SMI, Laguerre RSI, Connors RSI, Inertia |
| Trend & Directional  | MACD, ADX (+DI/-DI), ADXR, Aroon, TRIX, Aroon Oscillator, Vortex, Random Walk Index, Trend Intensity Index, Wave Trend Oscillator, Mass Index, Choppiness Index, Vertical Horizontal Filter |
| Price Oscillators    | PPO, DPO, Coppock, Accelerator Oscillator, Balance of Power, APO, AO Histogram, CFO, Zero-Lag MACD, Elder Impulse, STC |
| Volatility & Bands   | ATR, Bollinger Bands, Keltner Channels, Donchian Channels, NATR, StdDev, Ulcer Index, Historical Volatility, Bollinger Bandwidth, %B, True Range, Chaikin Volatility, RVI (Relative Volatility Index), Parkinson Volatility, Garman-Klass Volatility, Rogers-Satchell Volatility, Yang-Zhang Volatility |
| Bands & Channels     | MA Envelope, Acceleration Bands, STARC Bands, ATR Bands, Hurst Channel, LinReg Channel, Standard Error Bands, Double Bollinger Bands, TTM Squeeze, Fractal Chaos Bands, VWAP StdDev Bands |
| Trailing Stops       | Parabolic SAR, SuperTrend, Chandelier Exit, Chande Kroll Stop, ATR Trailing Stop, HiLo Activator, Volty Stop, Yo-Yo Exit, Donchian Channel Stop, Percentage Trailing Stop, Step Trailing Stop, Renko Trailing Stop |
| Volume               | OBV, VWAP (cumulative + rolling), ADL, Volume-Price Trend, Chaikin Money Flow, Chaikin Oscillator, Force Index, Ease of Movement, Klinger Volume Oscillator, Volume Oscillator, NVI, PVI, Williams A/D, Anchored VWAP, Demand Index, TSV, VZO, Market Facilitation Index |
| Price Statistics     | Typical Price, Median Price, Weighted Close, Linear Regression, Linear Regression Slope, Z-Score, Linear Regression Angle, Variance, Coefficient of Variation, Skewness, Kurtosis, Standard Error, Detrended StdDev, R², Median Absolute Deviation, Autocorrelation, Hurst Exponent, Pearson Correlation, Beta, Spearman Correlation |
| Ehlers / Cycle (DSP) | MAMA, FAMA, Fisher Transform, Inverse Fisher Transform, SuperSmoother, Hilbert Dominant Cycle, Sine Wave, Decycler, Decycler Oscillator, Roofing Filter, Center of Gravity, Cybernetic Cycle, Adaptive Cycle, Empirical Mode Decomposition, Ehlers Stochastic, Instantaneous Trendline |
| Pivots & S/R         | Classic Pivots, Fibonacci Pivots, Camarilla, Woodie Pivots, DeMark Pivots, Williams Fractals, ZigZag |
| DeMark               | TD Setup, TD Sequential, TD DeMarker, TD REI, TD Pressure, TD Combo, TD Countdown, TD Lines, TD Range Projection, TD Differential, TD Open, TD Risk Level |
| Ichimoku & Charts    | Ichimoku Kinko Hyo (Tenkan, Kijun, Senkou A/B, Chikou), Heikin-Ashi |
| Candlestick Patterns | Doji, Hammer, Inverted Hammer, Hanging Man, Shooting Star, Engulfing, Harami, Morning/Evening Star, Three White Soldiers/Black Crows, Piercing Line/Dark Cloud Cover, Marubozu, Tweezer, Spinning Top, Three Inside Up/Down, Three Outside Up/Down |
| Market Profile       | Value Area (POC / VAH / VAL), Initial Balance, Opening Range |
| Risk / Performance   | Sharpe Ratio, Sortino Ratio, Calmar Ratio, Omega Ratio, Max Drawdown, Average Drawdown, Drawdown Duration, Pain Index, Value at Risk, Conditional Value at Risk (CVaR), Profit Factor, Gain/Loss Ratio, Recovery Factor, Kelly Criterion, Treynor Ratio, Information Ratio, Alpha (Jensen) |

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
│   ├── wickra-core/         core engine + all 214 indicators
│   ├── wickra/              top-level facade crate (publishes on crates.io) + benches/
│   └── wickra-data/         CSV reader, tick aggregator, live exchange feeds
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

Rust benchmarks live in `crates/wickra/benches/`; runnable Rust examples live
in the workspace member crate at `examples/rust/`. There is no top-level
`benches/` directory.

## Building everything from source

```bash
# Rust core + tests
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p wickra

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

Licensed under the **PolyForm Noncommercial License 1.0.0**. See [LICENSE](LICENSE).

In plain English: use it, fork it, modify it, redistribute it, file issues, send
pull requests — all welcome. Personal projects, research, education, non-profits,
government, hobby trading bots: all fine. The one thing that's not allowed is
commercial sale of the software or of services built around it. If you want to
use Wickra commercially, get in touch about a license.

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
