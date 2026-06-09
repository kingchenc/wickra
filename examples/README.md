# Wickra examples

Runnable examples for every Wickra binding. Rust and Node examples live next
to the code they exercise so the language tooling (`cargo run --example`,
`node`) can find them; the Python examples have no crate of their own and
live here under [`python/`](python/).

## Rust — `examples/rust/`

The Rust examples live in the `wickra-examples` workspace member crate.

| Example | What it does | Run |
| --- | --- | --- |
| `streaming.rs` | Feed a synthetic price series through SMA / EMA / RSI / MACD tick by tick. | `cargo run -p wickra-examples --bin streaming` |
| `backtest.rs` | Compute a basket of indicators over an OHLCV CSV and print a summary. | `cargo run -p wickra-examples --bin backtest -- <ohlcv.csv>` |
| `multi_timeframe.rs` | Resample a 1-minute CSV via wickra-data and print indicators per timeframe. | `cargo run -p wickra-examples --bin multi_timeframe` |
| `parallel_assets.rs` | Serial vs `BatchExt::batch_parallel` (rayon) over a synthetic panel, with speedup. | `cargo run --release -p wickra-examples --bin parallel_assets -- --assets 200 --bars 5000` |
| `fetch_btcusdt.rs` | Download real BTCUSDT klines from the Binance REST API into `examples/data/`. | `cargo run -p wickra-examples --bin fetch_btcusdt` |
| `live_binance.rs` | Stream live Binance klines through an indicator over a resilient WebSocket. | `cargo run -p wickra-examples --bin live_binance` |
| `strategy_rsi_mean_reversion.rs` | Hourly BTCUSDT mean-reversion using RSI(14) thresholds, with PnL / Sharpe / max-DD summary. | `cargo run --release -p wickra-examples --bin strategy_rsi_mean_reversion` |
| `strategy_macd_adx.rs` | Hourly BTCUSDT trend-follower: MACD crossover entries gated by ADX(14) > 20. | `cargo run --release -p wickra-examples --bin strategy_macd_adx` |
| `strategy_bollinger_squeeze.rs` | Daily BTCUSDT Bollinger-squeeze breakout with ATR(14) trailing stop. | `cargo run --release -p wickra-examples --bin strategy_bollinger_squeeze` |

## C / C++ — `examples/c/`

Build the library first (`cargo build -p wickra-c --release`), then build and run
the examples via CMake:
`cmake -S examples/c -B examples/c/build -DWICKRA_LIB_DIR="$PWD/target/release"` →
`cmake --build examples/c/build` → `ctest --test-dir examples/c/build`.

| Example | What it does | CMake target |
| --- | --- | --- |
| `smoke.c` | Links the generated header + library and asserts SMA streaming / batch values across the boundary. | `smoke` |
| `streaming.c` | Feed a tick stream through an EMA, printing each value (NaN during warmup). | `streaming` |
| `smoke.cpp` | C++ RAII via `wickra::Handle` from [`wickra.hpp`](../bindings/c/include/wickra.hpp): construct, move, auto-free. | `cpp_smoke` |

## Python — `examples/python/`

| Example | What it does | Run |
| --- | --- | --- |
| `streaming.py` | Feed a synthetic price series through SMA / EMA / RSI / MACD tick by tick. | `python -m examples.python.streaming` |
| `backtest.py` | Basket of indicators over an OHLCV CSV. | `python -m examples.python.backtest <ohlcv.csv>` |
| `live_trading.py` | Live Binance feed → RSI / MACD / Bollinger → signals. | `python -m examples.python.live_trading --symbol BTCUSDT --interval 1m` |
| `multi_timeframe.py` | Resample a 1-minute CSV to coarser timeframes and compare. | `python -m examples.python.multi_timeframe <1m.csv>` |
| `parallel_assets.py` | Process many symbols in parallel — the Rust extension releases the GIL during batch computation. | `python -m examples.python.parallel_assets --assets 200 --bars 5000` |
| `fetch_btcusdt.py` | Download real BTCUSDT klines from the Binance REST API into `examples/data/` (urllib + stdlib only). | `python -m examples.python.fetch_btcusdt` |
| `strategy_rsi_mean_reversion.py` | Hourly BTCUSDT mean-reversion using RSI(14) thresholds, with PnL / Sharpe / max-DD summary. | `python -m examples.python.strategy_rsi_mean_reversion` |
| `strategy_macd_adx.py` | Hourly BTCUSDT trend-follower: MACD crossover entries gated by ADX(14) > 20. | `python -m examples.python.strategy_macd_adx` |
| `strategy_bollinger_squeeze.py` | Daily BTCUSDT Bollinger-squeeze breakout with ATR(14) trailing stop. | `python -m examples.python.strategy_bollinger_squeeze` |

`live_trading.py` additionally needs `pip install websockets`.

## Node.js — `examples/node/`

Build the native binding once, then link it into the examples directory:

```bash
cd bindings/node && npm install && npx napi build --platform --release
cd ../../examples/node && npm install        # links wickra + installs `ws`
```

| Example | What it does | Run |
| --- | --- | --- |
| `streaming.js` | Feed a synthetic price series through several indicators tick by tick. | `node streaming.js` |
| `backtest.js` | Basket of indicators over an OHLCV CSV; defaults to the bundled BTCUSDT daily dataset. | `node backtest.js [ohlcv.csv]` |
| `multi_timeframe.js` | Roll a 1-minute CSV up to 5m / 15m / 1h / 4h / 1d and print indicators per timeframe. | `node multi_timeframe.js [path/to/1m.csv]` |
| `parallel_assets.js` | Serial vs `worker_threads` pool over a synthetic panel, with speedup. | `node parallel_assets.js --assets 200 --bars 5000` |
| `live_trading.js` | Live Binance feed → RSI / MACD / Bollinger → signals. | `node live_trading.js --symbol BTCUSDT --interval 1m` |
| `fetch_btcusdt.js` | Download real BTCUSDT klines from the Binance REST API into `examples/data/` (built-in `fetch`, Node 18+). | `node fetch_btcusdt.js` |
| `strategy_rsi_mean_reversion.js` | Hourly BTCUSDT mean-reversion using RSI(14) thresholds, with PnL / Sharpe / max-DD summary. | `node strategy_rsi_mean_reversion.js` |
| `strategy_macd_adx.js` | Hourly BTCUSDT trend-follower: MACD crossover entries gated by ADX(14) > 20. | `node strategy_macd_adx.js` |
| `strategy_bollinger_squeeze.js` | Daily BTCUSDT Bollinger-squeeze breakout with ATR(14) trailing stop. | `node strategy_bollinger_squeeze.js` |

## WebAssembly — `examples/wasm/`

Build the WASM module first (one-time):

```bash
wasm-pack build bindings/wasm --target web --release --features panic-hook
```

Then serve the repository root (`python -m http.server`, `npx http-server`,
…) and open the demo you want in a browser.

| Example | What it does |
| --- | --- |
| `index.html` | Streams a synthetic price series through six indicators and draws a live `<canvas>` chart. |
| `backtest.html` | Streams a fetched OHLCV CSV through a basket of indicators (SMA, EMA, RSI, MACD, Bollinger, ATR, ADX, OBV) and prints a per-series summary table. |
| `live_trading.html` | Opens a browser-native `WebSocket` to Binance, runs RSI / MACD / Bollinger and flags BUY/SELL candidates. |
| `multi_timeframe.html` | Fetches a 1-minute CSV, rolls it up to 5m / 15m / 1h / 4h / 1d in-page, prints RSI / MACD hist / ADX per timeframe. |
| `parallel_assets.html` | Spawns a pool of module Workers (each loading its own copy of the WASM module) and reports the speedup over a serial baseline. |
| `strategy_rsi_mean_reversion.html` | Hourly BTCUSDT RSI(14) mean-reversion (long &lt; 30, exit &gt; 70); prints a PnL / Sharpe / max-DD summary table. |
| `strategy_macd_adx.html` | Hourly BTCUSDT MACD crossover gated by ADX(14) &gt; 20, with the same summary table. |
| `strategy_bollinger_squeeze.html` | Daily BTCUSDT Bollinger-squeeze breakout with a 2&times;ATR(14) stop and summary table. |

## Example datasets

`examples/data/` holds seven real BTCUSDT OHLCV datasets, one
per timeframe (1m, 5m, 15m, 1h, 12h, 1d, 1month), in the standard
`timestamp,open,high,low,close,volume` layout. The Rust and Node backtest
examples and the indicator benchmarks run against them. Regenerate them with
the latest market history via `cargo run -p wickra-examples --bin fetch_btcusdt`.
