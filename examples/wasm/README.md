# Wickra WASM examples

Browser demos for the `wickra-wasm` WebAssembly binding. Every demo loads
the module the same way (`init()` then construct indicators) so the
patterns transfer one-to-one to your own page.

## Build

The WASM module ships as a `wasm-pack` `--target web` bundle. Build it
once from the repository root:

```bash
wasm-pack build bindings/wasm --target web --release --features panic-hook
```

This drops `bindings/wasm/pkg/` with the `.wasm` binary, the JS loader
and TypeScript types. Every demo here imports the loader via
`../../bindings/wasm/pkg/wickra_wasm.js`.

## Serve

ES-module workers and `fetch()` over CSV both need a real HTTP origin,
not `file://`. Any static server from the repository root works:

```bash
# Python:
python -m http.server 8000

# Or Node:
npx http-server -p 8000
```

Then open the demo you want at `http://localhost:8000/examples/wasm/<file>`.

## Demos

| File | What it does |
| --- | --- |
| `index.html` | The original showcase: streams a synthetic price series through six indicators and draws a live `<canvas>` chart with `SMA`, `EMA`, `RSI`, `MACD`, `BollingerBands` and `ATR` cards. |
| `backtest.html` | Backtest: fetches an OHLCV CSV (default the bundled BTCUSDT daily dataset), streams every candle through a basket of eight indicators, prints a summary table. Mirrors `examples/python/backtest.py`. |
| `live_trading.html` | Browser-native `WebSocket` to Binance kline streams; runs RSI / MACD / Bollinger on the incoming closes and flags BUY/SELL candidates when the three agree. Mirrors `examples/python/live_trading.py`. |
| `multi_timeframe.html` | Fetches a 1-minute CSV, rolls it up in-page to 5m / 15m / 1h / 4h / 1d buckets and prints RSI / MACD-histogram / ADX per timeframe. Mirrors `examples/python/multi_timeframe.py`. |
| `parallel_assets.html` | Synthetic `(assets, bars)` panel, serial baseline on the main thread vs. a pool of module Workers each loading its own copy of the WASM module. Mirrors `examples/python/parallel_assets.py`. |
| `parallel_worker.js` | Module worker used by `parallel_assets.html` (not loaded directly). |

## See also

- [Quickstart: WASM](https://github.com/wickra-lib/wickra/wiki/Quickstart-WASM.md) — module-load
  flow, `wasm-pack` targets, and the streaming API.
- [examples/README.md](../README.md) — cross-language index, including
  the Rust, Python and Node siblings of every demo above.
