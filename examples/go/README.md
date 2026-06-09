# Wickra examples — Go

Runnable Go examples for the [Wickra Go binding](../../bindings/go). Each example
is a small `main` program in its own directory; they share the deterministic
synthetic data, CSV loader, and equity summary in
[`internal/market`](internal/market).

The binding links against the prebuilt Wickra C ABI library, so build and stage
it once before running anything:

```bash
cargo build -p wickra-c --release
cp target/release/libwickra.so   bindings/go/lib/    # Linux
cp target/release/libwickra.dylib bindings/go/lib/   # macOS
cp target/release/wickra.dll     bindings/go/lib/    # Windows (also put it on PATH)
```

Then run any example from the `examples/go` module:

```bash
cd examples/go
go run ./streaming
```

| Example | What it does | Run |
| --- | --- | --- |
| `streaming` | Feed a synthetic price series through SMA / EMA / RSI / MACD tick by tick. | `go run ./streaming` |
| `backtest` | Compute a basket of indicators over an OHLCV series and print a summary. | `go run ./backtest <ohlcv.csv>` |
| `multi_timeframe` | Resample a 1-minute series into 5m / 15m and print an indicator per timeframe. | `go run ./multi_timeframe` |
| `parallel_assets` | SMA(20) batch over a panel of assets, serial vs goroutine fan-out, with speedup. | `go run ./parallel_assets 200 5000` |
| `strategy_rsi_mean_reversion` | RSI(14) mean-reversion with a PnL / Sharpe / max-DD summary. | `go run ./strategy_rsi_mean_reversion` |
| `strategy_macd_adx` | MACD crossover entries gated by ADX(14) > 20. | `go run ./strategy_macd_adx` |
| `strategy_bollinger_squeeze` | Bollinger-squeeze breakout with an ATR(14) trailing stop. | `go run ./strategy_bollinger_squeeze` |
| `fetch_btcusdt` | Download real BTCUSDT klines from the Binance REST API into a CSV. | `go run ./fetch_btcusdt` |
| `live_binance` | Stream live Binance klines through EMA(20) over a WebSocket. | `go run ./live_binance` |

`fetch_btcusdt` and `live_binance` require network access; the rest run offline
on deterministic synthetic data.
