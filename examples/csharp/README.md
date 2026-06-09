# Wickra examples — C# / .NET

Runnable .NET examples for the [Wickra .NET binding](../../bindings/csharp).
Each example is a small console project that references the `Wickra` project and
resolves the native library automatically (from `target/release` during local
development, or the NuGet `runtimes/` layout when packaged).

Build the native library first, then run any example:

```bash
cargo build -p wickra-c --release
dotnet run --project examples/csharp/streaming
```

| Example | What it does | Run |
| --- | --- | --- |
| `streaming` | Feed a synthetic price series through SMA / EMA / RSI / MACD tick by tick. | `dotnet run --project examples/csharp/streaming` |
| `backtest` | Compute a basket of indicators over an OHLCV series and print a summary. | `dotnet run --project examples/csharp/backtest -- <ohlcv.csv>` |
| `multi_timeframe` | Resample a 1-minute series into 5m / 15m and print an indicator per timeframe. | `dotnet run --project examples/csharp/multi_timeframe` |
| `parallel_assets` | SMA(20) batch over a panel of assets, serial vs `Parallel.For`, with speedup. | `dotnet run -c Release --project examples/csharp/parallel_assets -- 200 5000` |
| `strategy_rsi_mean_reversion` | RSI(14) mean-reversion with a PnL / Sharpe / max-DD summary. | `dotnet run -c Release --project examples/csharp/strategy_rsi_mean_reversion` |
| `strategy_macd_adx` | MACD crossover entries gated by ADX(14) > 20. | `dotnet run -c Release --project examples/csharp/strategy_macd_adx` |
| `strategy_bollinger_squeeze` | Bollinger-squeeze breakout with an ATR(14) trailing stop. | `dotnet run -c Release --project examples/csharp/strategy_bollinger_squeeze` |
| `fetch_btcusdt` | Download real BTCUSDT klines from the Binance REST API into a CSV. | `dotnet run --project examples/csharp/fetch_btcusdt` |
| `live_binance` | Stream live Binance klines through EMA(20) over a WebSocket. | `dotnet run --project examples/csharp/live_binance` |

`fetch_btcusdt` and `live_binance` require network access; the rest run offline
on deterministic synthetic data. Shared helpers (synthetic data, CSV loader,
equity summary) live in [`_common/`](_common).
