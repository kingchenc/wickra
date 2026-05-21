# Wickra

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

| Library            | Install pain    | Streaming | Multi-language | Active |
|--------------------|-----------------|-----------|----------------|--------|
| TA-Lib (Python)    | yes (C deps)    | no        | no             | barely |
| pandas-ta          | clean           | no        | no             | slow   |
| finta              | clean           | no        | no             | stale  |
| ta-lib-python      | yes (C deps)    | no        | no             | barely |
| talipp             | clean           | yes       | no             | yes    |
| Tulip Indicators   | yes (C deps)    | no        | partial        | stale  |
| ooples (C#)        | clean           | no        | C# only        | yes    |
| **Wickra**         | **clean**       | **yes**   | **Python+Node+WASM+Rust** | **yes** |

Wickra is the only library that combines all of: clean install, streaming,
multi-language reach, and active maintenance.

## Benchmark: how much faster is "streaming-first"?

Reproduced on this machine with `python -m benchmarks.compare_libraries`.
Lower µs/op = faster. Wickra wins every batch category outright, and the
streaming gap widens linearly with how much history a batch-only library has
to recompute on every tick.

### Batch — single full pass over a 5 000-bar series

Reading the table: each cell shows that library's runtime, plus how many times
slower it is than Wickra in parentheses. **★** marks the winner per row.

| Indicator           | Wickra              | finta                  | talipp                       |
|---------------------|---------------------|------------------------|------------------------------|
| SMA(20)             | **26.0 µs ★**       | 295.3 µs (11.4× slower) | 1 812.8 µs (69.7× slower)   |
| EMA(20)             | **16.8 µs ★**       | 205.5 µs (12.2× slower) | 2 534.4 µs (150.9× slower)  |
| RSI(14)             | **31.2 µs ★**       | 714.1 µs (22.9× slower) | 3 751.7 µs (120.2× slower)  |
| MACD(12, 26, 9)     | **30.8 µs ★**       | 359.5 µs (11.7× slower) | 11 642.2 µs (378.0× slower) |
| Bollinger(20, 2.0)  | **26.7 µs ★**       | 690.6 µs (25.9× slower) | 27 482.4 µs (1 030.1× slower) |
| ATR(14)             | **40.6 µs ★**       | 1 120.3 µs (27.6× slower) | 3 760.2 µs (92.7× slower) |

### Streaming — per-tick latency after seeding with 2 000 historical bars

A batch-only library has to re-run its full indicator over the entire history on
every new tick; Wickra updates state in O(1).

| Indicator | Wickra (per tick)   | talipp (per tick)         |
|-----------|---------------------|---------------------------|
| RSI(14)   | **0.07 µs ★**       | 1.16 µs (17.5× slower)    |

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

## Indicators in 0.1.0

25 streaming-first indicators across four families. Every one passes the
`batch == streaming` equivalence test, reference-value tests, and reset
semantics tests.

| Family      | Indicators |
|-------------|-----------|
| Trend       | SMA, EMA, WMA, DEMA, TEMA, HMA, KAMA |
| Momentum    | RSI (Wilder), MACD, Stochastic, CCI, ROC, Williams %R, ADX (+DI/-DI), MFI, TRIX, Awesome Oscillator, Aroon |
| Volatility  | Bollinger Bands, ATR, Keltner Channels, Donchian Channels, Parabolic SAR |
| Volume      | OBV, VWAP (cumulative + rolling) |

Adding a new indicator means implementing one trait in Rust; all four bindings
inherit it automatically.

## Languages

| Binding           | Install                                       | Example |
|-------------------|-----------------------------------------------|---------|
| Python (PyO3)     | `pip install wickra`                          | `examples/python/backtest.py` |
| Node.js (napi-rs) | `npm install @wickra/wickra`                  | `bindings/node/__tests__/smoke.test.js` |
| Browser / WASM    | `wasm-pack build bindings/wasm --target web`  | `bindings/wasm/examples/index.html` |
| Rust              | `cargo add wickra`                            | `examples/rust/backtest.rs` |

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
│   ├── wickra-core/         core engine + all 25 indicators
│   ├── wickra/              top-level facade crate (publishes on crates.io)
│   └── wickra-data/         CSV reader, tick aggregator, live exchange feeds
├── bindings/
│   ├── python/              PyO3 + maturin (publishes on PyPI)
│   ├── node/                napi-rs (publishes on npm)
│   └── wasm/                wasm-bindgen (browsers, bundlers, Node)
├── examples/
│   ├── python/              backtest, live trading, parallel assets, multi-tf
│   └── rust/                backtest, live Binance
├── benches/                 cargo bench targets
└── .github/workflows/       CI and release pipelines
```

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

## Test counts

- `wickra-core`: 171 unit tests + 2 doctests, including textbook-value tests
  for Wilder RSI, Bollinger Bands, MACD, ATR, and Stochastic.
- `wickra-data`: 11 unit tests + 1 doctest, covers CSV decoding, the tick
  aggregator, the resampler, and the Binance payload parser.
- `bindings/python`: 56 pytest tests covering smoke checks, streaming==batch
  equivalence, reference values, lifecycle, and dict/tuple candle inputs.
- `bindings/node`: 7 Node test-runner cases via `node --test`.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
