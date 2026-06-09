# Wickra examples — Java

Runnable examples for the [Wickra Java binding](../../bindings/java). Each
example is a small `main` class that uses the `org.wickra:wickra` artifact and
resolves the native library automatically (from `target/release` during local
development, or the bundled per-platform library when packaged).

Build the native library and install the binding to your local Maven repo
first, then run any example with the `exec` plugin:

```bash
cargo build -p wickra-c --release
mvn -f bindings/java install -DskipTests
mvn -f examples/java compile
mvn -f examples/java exec:exec -Dexec.mainClass=org.wickra.examples.Streaming
```

(The `exec:exec` goal forks a JVM with `--enable-native-access=ALL-UNNAMED`, the
flag the FFM API needs.)

| Example | What it does | Main class |
| --- | --- | --- |
| `streaming` | Feed a synthetic price series through SMA / EMA / RSI / MACD tick by tick. | `org.wickra.examples.Streaming` |
| `backtest` | Compute a basket of indicators over an OHLCV series and print a summary. | `org.wickra.examples.Backtest` |
| `multi_timeframe` | Resample a 1-minute series into 5m / 15m and print an indicator per timeframe. | `org.wickra.examples.MultiTimeframe` |
| `parallel_assets` | SMA(20) batch over a panel of assets, serial vs parallel streams, with speedup. | `org.wickra.examples.ParallelAssets` |
| `strategy_rsi_mean_reversion` | RSI(14) mean-reversion with a PnL / Sharpe / max-DD summary. | `org.wickra.examples.StrategyRsiMeanReversion` |
| `strategy_macd_adx` | MACD crossover entries gated by ADX(14) > 20. | `org.wickra.examples.StrategyMacdAdx` |
| `strategy_bollinger_squeeze` | Bollinger-squeeze breakout with an ATR(14) trailing stop. | `org.wickra.examples.StrategyBollingerSqueeze` |
| `fetch_btcusdt` | Download real BTCUSDT klines from the Binance REST API into a CSV. | `org.wickra.examples.FetchBtcusdt` |
| `live_binance` | Stream live Binance klines through EMA(20) over a WebSocket. | `org.wickra.examples.LiveBinance` |

`fetch_btcusdt` and `live_binance` require network access; the rest run offline
on deterministic synthetic data. Shared helpers (synthetic data, CSV loader,
equity summary) live in `MarketData` and `Equity`.
