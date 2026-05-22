# Wickra examples

Runnable examples for every Wickra binding. Rust and Node examples live next
to the code they exercise so the language tooling (`cargo run --example`,
`node`) can find them; the Python examples have no crate of their own and
live here under [`python/`](python/).

## Rust — `crates/*/examples/`

| Example | What it does | Run |
| --- | --- | --- |
| `backtest.rs` | Compute a basket of indicators over an OHLCV CSV and print a summary. | `cargo run -p wickra --example backtest -- <ohlcv.csv>` |
| `fetch_btcusdt.rs` | Download real BTCUSDT klines from the Binance REST API into `crates/wickra/examples/data/`. | `cargo run -p wickra --example fetch_btcusdt` |
| `live_binance.rs` | Stream live Binance klines through an indicator over a resilient WebSocket. | `cargo run -p wickra-data --example live_binance --features live-binance` |

## Python — `examples/python/`

| Example | What it does | Run |
| --- | --- | --- |
| `backtest.py` | Basket of indicators over an OHLCV CSV. | `python -m examples.python.backtest <ohlcv.csv>` |
| `live_trading.py` | Live Binance feed → RSI / MACD / Bollinger → signals. | `python -m examples.python.live_trading --symbol BTCUSDT --interval 1m` |
| `multi_timeframe.py` | Resample a 1-minute CSV to coarser timeframes and compare. | `python -m examples.python.multi_timeframe <1m.csv>` |
| `parallel_assets.py` | Process many symbols in parallel — the Rust extension releases the GIL during batch computation. | `python -m examples.python.parallel_assets --assets 200 --bars 5000` |

`live_trading.py` additionally needs `pip install websockets`.

## Node.js — `bindings/node/examples/`

Build the native module first: `cd bindings/node && npm install && npx napi build --platform --release`.

| Example | What it does | Run |
| --- | --- | --- |
| `streaming.js` | Feed a synthetic price series through several indicators tick by tick. | `node examples/streaming.js` |
| `backtest.js` | Basket of indicators over an OHLCV CSV; defaults to the bundled BTCUSDT daily dataset. | `node examples/backtest.js [ohlcv.csv]` |
| `live_trading.js` | Live Binance feed → RSI / MACD / Bollinger → signals. | `node examples/live_trading.js --symbol BTCUSDT --interval 1m` |

## WebAssembly — `bindings/wasm/examples/`

| Example | What it does | Run |
| --- | --- | --- |
| `index.html` | Browser demo: streams a price series through six indicators and draws a live `<canvas>` chart. | `wasm-pack build bindings/wasm --target web --release --features panic-hook`, then serve `bindings/wasm/` and open `examples/index.html` |

## Example datasets

`crates/wickra/examples/data/` holds seven real BTCUSDT OHLCV datasets, one
per timeframe (1m, 5m, 15m, 1h, 12h, 1d, 1month), in the standard
`timestamp,open,high,low,close,volume` layout. The Rust and Node backtest
examples and the indicator benchmarks run against them. Regenerate them with
the latest market history via `cargo run -p wickra --example fetch_btcusdt`.
