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

## Python — `examples/python/`

| Example | What it does | Run |
| --- | --- | --- |
| `streaming.py` | Feed a synthetic price series through SMA / EMA / RSI / MACD tick by tick. | `python -m examples.python.streaming` |
| `backtest.py` | Basket of indicators over an OHLCV CSV. | `python -m examples.python.backtest <ohlcv.csv>` |
| `live_trading.py` | Live Binance feed → RSI / MACD / Bollinger → signals. | `python -m examples.python.live_trading --symbol BTCUSDT --interval 1m` |
| `multi_timeframe.py` | Resample a 1-minute CSV to coarser timeframes and compare. | `python -m examples.python.multi_timeframe <1m.csv>` |
| `parallel_assets.py` | Process many symbols in parallel — the Rust extension releases the GIL during batch computation. | `python -m examples.python.parallel_assets --assets 200 --bars 5000` |

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

## WebAssembly — `examples/wasm/`

| Example | What it does | Run |
| --- | --- | --- |
| `index.html` | Browser demo: streams a price series through six indicators and draws a live `<canvas>` chart. | `wasm-pack build bindings/wasm --target web --release --features panic-hook`, then serve the repository root and open `examples/wasm/index.html` |

## Example datasets

`examples/data/` holds seven real BTCUSDT OHLCV datasets, one
per timeframe (1m, 5m, 15m, 1h, 12h, 1d, 1month), in the standard
`timestamp,open,high,low,close,volume` layout. The Rust and Node backtest
examples and the indicator benchmarks run against them. Regenerate them with
the latest market history via `cargo run -p wickra-examples --bin fetch_btcusdt`.
