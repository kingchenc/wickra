# Wickra examples — R

Runnable R examples for the [Wickra R binding](../../bindings/r). Each example is
a small script; they share the deterministic synthetic data, CSV loader, and
equity summary in [`_common.R`](_common.R).

Install the binding first (it compiles against the C ABI library — see the
[binding README](../../bindings/r)), then run any example from this directory:

```bash
cargo build -p wickra-c --release
WICKRA_INCLUDE_DIR="$PWD/bindings/c/include" WICKRA_LIB_DIR="$PWD/target/release" \
  R CMD INSTALL bindings/r
cd examples/r
Rscript streaming.R
```

| Example | What it does | Run |
| --- | --- | --- |
| `streaming.R` | Feed a synthetic price series through SMA / EMA / RSI / MACD tick by tick. | `Rscript streaming.R` |
| `backtest.R` | Compute a basket of indicators over an OHLCV series and print a summary. | `Rscript backtest.R <ohlcv.csv>` |
| `multi_timeframe.R` | Resample a 1-minute series into 5m / 15m and print an indicator per timeframe. | `Rscript multi_timeframe.R` |
| `parallel_assets.R` | SMA(20) batch over a panel, serial vs `mclapply`, with speedup. | `Rscript parallel_assets.R 200 5000` |
| `strategy_rsi_mean_reversion.R` | RSI(14) mean-reversion with a PnL / Sharpe / max-DD summary. | `Rscript strategy_rsi_mean_reversion.R` |
| `strategy_macd_adx.R` | MACD crossover entries gated by ADX(14) > 20. | `Rscript strategy_macd_adx.R` |
| `strategy_bollinger_squeeze.R` | Bollinger-squeeze breakout with an ATR(14) trailing stop. | `Rscript strategy_bollinger_squeeze.R` |
| `fetch_btcusdt.R` | Download real BTCUSDT klines from the Binance REST API into a CSV (`jsonlite`). | `Rscript fetch_btcusdt.R` |
| `live_binance.R` | Stream live Binance klines through EMA(20) over a WebSocket (`websocket`). | `Rscript live_binance.R` |

`fetch_btcusdt.R` and `live_binance.R` require network access (and the
`jsonlite` / `websocket` packages); the rest run offline on deterministic
synthetic data. `parallel_assets.R` forks via `parallel::mclapply` on Unix and
runs serially on Windows.
