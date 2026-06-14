# Wickra WASM examples

Browser demos for the `wickra-wasm` WASM binding. Every demo loads
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
| `live_binance.html` | Browser-native `WebSocket` to Binance kline streams; runs RSI / MACD / Bollinger on the incoming closes and flags BUY/SELL candidates when the three agree. Mirrors `examples/python/live_binance.py`. |
| `multi_timeframe.html` | Fetches a 1-minute CSV, rolls it up in-page to 5m / 15m / 1h / 4h / 1d buckets and prints RSI / MACD-histogram / ADX per timeframe. Mirrors `examples/python/multi_timeframe.py`. |
| `parallel_assets.html` | Synthetic `(assets, bars)` panel, serial baseline on the main thread vs. a pool of module Workers each loading its own copy of the WASM module. Mirrors `examples/python/parallel_assets.py`. |
| `parallel_worker.js` | Module worker used by `parallel_assets.html` (not loaded directly). |
| `strategy_rsi_mean_reversion.html` | RSI(14) mean-reversion (long &lt; 30, exit &gt; 70), 0.1% fees, summary table. Mirrors `examples/python/strategy_rsi_mean_reversion.py`. |
| `strategy_macd_adx.html` | MACD(12,26,9) crossover gated by ADX(14) &gt; 20, summary table. Mirrors `examples/python/strategy_macd_adx.py`. |
| `strategy_bollinger_squeeze.html` | Bollinger-squeeze breakout with a 2&times;ATR(14) stop, summary table. Mirrors `examples/python/strategy_bollinger_squeeze.py`. |

## Performance

The in-browser benchmark is `parallel_assets.html`: it times a serial main-thread
baseline against a pool of module Workers and reports the speedup. For raw
single-thread throughput numbers see the sibling benchmarks — Rust criterion
(`crates/wickra/benches/`), Python (`bindings/python/benchmarks/compare_libraries.py`)
and Node (`bindings/node/benchmarks/throughput.js`, `npm run bench`). The WASM
engine is the same Rust core compiled to `wasm32`, so its relative ordering of
indicators tracks those.

## See also

- [Quickstart: WASM](https://docs.wickra.org/Quickstart-WASM) — module-load
  flow, `wasm-pack` targets, and the streaming API.
- [examples/README.md](../README.md) — cross-language index, including
  the Rust, Python, Node.js, C and C# siblings of every demo above.
