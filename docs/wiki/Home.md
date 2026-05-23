# Wickra

Wickra is a streaming-first technical-indicators library. Every indicator is
implemented in Rust as an O(1) state machine that consumes one input at a
time, and the same engine is exposed through ergonomic bindings for Python,
Node.js, WebAssembly, and Rust itself. The same `update` call you write inside
a live trading loop also drives the historical backtest of that same
strategy — there is no second code path that drifts behind the streaming one.

The project ships 71 indicators across eight families — moving averages,
momentum oscillators, trend & directional, price oscillators, volatility &
bands, trailing stops, volume, and price statistics — plus a small set of
supporting types (`Candle`, `Tick`, `Chain`). The Rust core forbids `unsafe`,
so every binding inherits a
memory-safe implementation. Install is one command on every supported
platform: `pip install wickra`, `cargo add wickra`, `npm install wickra` — no
system compilers, no C dependencies, no headers.

Wickra is licensed under the **PolyForm Noncommercial 1.0.0** license.
Personal projects, research, hobby trading bots, education, non-profits, and
government use are all permitted; commercial sale of the software or of
services built around it is not. If you want to use Wickra commercially,
open an issue on GitHub to discuss a separate license.

## Published versions

| Registry  | Package        | Version |
|-----------|----------------|---------|
| crates.io | `wickra`       | 0.1.5   |
| crates.io | `wickra-core`  | 0.1.5   |
| crates.io | `wickra-data`  | 0.1.5   |
| PyPI      | `wickra`       | 0.1.5   |
| npm       | `wickra`       | 0.1.5   |
| npm       | `wickra-wasm`  | 0.1.5   |

Release notes and tagged builds:
<https://github.com/kingchenc/wickra/releases>.

## Wiki contents

- [Quickstart: Python](Quickstart-Python.md) — `pip install wickra`, a batch
  RSI on a NumPy array, a streaming RSI loop, and the multi-column NaN
  pattern that MACD and friends share.
- [Quickstart: Rust](Quickstart-Rust.md) — `cargo add wickra`, batch and
  streaming via the `Indicator` and `BatchExt` traits, and the `Chain`
  combinator.
- [Quickstart: Node](Quickstart-Node.md) — `npm install wickra`, basic
  `SMA` and `MACD` calls, and the install surface. Windows x64 was
  previously blocked by an npm spam filter on `wickra-win32-x64-msvc`;
  that was resolved with npm Support, and 0.1.5 is the first release in
  which `npm install wickra` works end-to-end on Windows.
- [Quickstart: WASM](Quickstart-WASM.md) — `npm install wickra-wasm`,
  building with `wasm-pack`, and running indicators client-side in a
  browser or bundler.
- [Data Layer](Data-Layer.md) — the `wickra-data` crate: the CSV reader,
  the tick-to-candle aggregator, the multi-timeframe resampler, and the
  Binance live feed.
- [Streaming vs Batch](Streaming-vs-Batch.md) — the conceptual difference
  between Wickra's O(1) `update` and the recompute-everything loops in
  batch-only libraries, with the benchmark numbers from the project README.
- [Warmup Periods](Warmup-Periods.md) — a verified table of every
  indicator's `warmup_period()`, plus the reasoning behind the off-by-one
  cases (RSI(14) needs 15 inputs because it needs 14 diffs).
- [Indicator Chaining](Indicator-Chaining.md) — `Chain::new(first, second)`
  and `.then(third)`, with a worked EMA(14) → RSI(7) example and the rule
  for stacked warmups.
- [Cookbook](Cookbook.md) — copy-paste strategy recipes built on streaming
  indicators (RSI mean reversion, MACD crossover, Bollinger breakout,
  ADX-gated trend, multi-timeframe, SuperTrend trailing stop).
- [TA-Lib Migration](TA-Lib-Migration.md) — function-by-function mapping
  table from TA-Lib's `talib.X(...)` calls to the equivalent Wickra
  expressions.
- [FAQ](FAQ.md) — quick answers to the most common questions about
  warmup, NaN handling, thread safety, and the streaming-vs-batch contract.

### Indicator reference

Start with [Indicators-Overview.md](Indicators-Overview.md) for the full
eight-family taxonomy and the shared `Indicator` trait surface. The
per-indicator pages below cover formulas, parameters, warmup behaviour, edge
cases, and verified Rust / Python / Node examples. They are grouped by family,
mirroring the `indicators/<family>/` directory layout.

**Moving Averages** — smooth the price series to surface direction.

- [Indicator-Sma.md](indicators/moving-averages/Indicator-Sma.md)
- [Indicator-Ema.md](indicators/moving-averages/Indicator-Ema.md)
- [Indicator-Wma.md](indicators/moving-averages/Indicator-Wma.md)
- [Indicator-Dema.md](indicators/moving-averages/Indicator-Dema.md)
- [Indicator-Tema.md](indicators/moving-averages/Indicator-Tema.md)
- [Indicator-Hma.md](indicators/moving-averages/Indicator-Hma.md)
- [Indicator-Kama.md](indicators/moving-averages/Indicator-Kama.md)
- [Indicator-Smma.md](indicators/moving-averages/Indicator-Smma.md)
- [Indicator-Trima.md](indicators/moving-averages/Indicator-Trima.md)
- [Indicator-Zlema.md](indicators/moving-averages/Indicator-Zlema.md)
- [Indicator-T3.md](indicators/moving-averages/Indicator-T3.md)
- [Indicator-Vwma.md](indicators/moving-averages/Indicator-Vwma.md)

**Momentum Oscillators** — measure the rate of price change.

- [Indicator-Rsi.md](indicators/momentum-oscillators/Indicator-Rsi.md)
- [Indicator-Stochastic.md](indicators/momentum-oscillators/Indicator-Stochastic.md)
- [Indicator-Cci.md](indicators/momentum-oscillators/Indicator-Cci.md)
- [Indicator-Roc.md](indicators/momentum-oscillators/Indicator-Roc.md)
- [Indicator-WilliamsR.md](indicators/momentum-oscillators/Indicator-WilliamsR.md)
- [Indicator-Mfi.md](indicators/momentum-oscillators/Indicator-Mfi.md)
- [Indicator-AwesomeOscillator.md](indicators/momentum-oscillators/Indicator-AwesomeOscillator.md)
- [Indicator-Mom.md](indicators/momentum-oscillators/Indicator-Mom.md)
- [Indicator-Cmo.md](indicators/momentum-oscillators/Indicator-Cmo.md)
- [Indicator-Tsi.md](indicators/momentum-oscillators/Indicator-Tsi.md)
- [Indicator-Pmo.md](indicators/momentum-oscillators/Indicator-Pmo.md)
- [Indicator-StochRsi.md](indicators/momentum-oscillators/Indicator-StochRsi.md)
- [Indicator-UltimateOscillator.md](indicators/momentum-oscillators/Indicator-UltimateOscillator.md)

**Trend & Directional** — is there a trend, and which way?

- [Indicator-MacdIndicator.md](indicators/trend-directional/Indicator-MacdIndicator.md)
- [Indicator-Adx.md](indicators/trend-directional/Indicator-Adx.md)
- [Indicator-Aroon.md](indicators/trend-directional/Indicator-Aroon.md)
- [Indicator-Trix.md](indicators/trend-directional/Indicator-Trix.md)
- [Indicator-AroonOscillator.md](indicators/trend-directional/Indicator-AroonOscillator.md)
- [Indicator-Vortex.md](indicators/trend-directional/Indicator-Vortex.md)
- [Indicator-MassIndex.md](indicators/trend-directional/Indicator-MassIndex.md)
- [Indicator-ChoppinessIndex.md](indicators/trend-directional/Indicator-ChoppinessIndex.md)
- [Indicator-VerticalHorizontalFilter.md](indicators/trend-directional/Indicator-VerticalHorizontalFilter.md)

**Price Oscillators** — difference-of-averages momentum around zero.

- [Indicator-Ppo.md](indicators/price-oscillators/Indicator-Ppo.md)
- [Indicator-Dpo.md](indicators/price-oscillators/Indicator-Dpo.md)
- [Indicator-Coppock.md](indicators/price-oscillators/Indicator-Coppock.md)
- [Indicator-AcceleratorOscillator.md](indicators/price-oscillators/Indicator-AcceleratorOscillator.md)
- [Indicator-BalanceOfPower.md](indicators/price-oscillators/Indicator-BalanceOfPower.md)

**Volatility & Bands** — dispersion measures and price envelopes.

- [Indicator-Atr.md](indicators/volatility-bands/Indicator-Atr.md)
- [Indicator-BollingerBands.md](indicators/volatility-bands/Indicator-BollingerBands.md)
- [Indicator-Keltner.md](indicators/volatility-bands/Indicator-Keltner.md)
- [Indicator-Donchian.md](indicators/volatility-bands/Indicator-Donchian.md)
- [Indicator-Natr.md](indicators/volatility-bands/Indicator-Natr.md)
- [Indicator-StdDev.md](indicators/volatility-bands/Indicator-StdDev.md)
- [Indicator-UlcerIndex.md](indicators/volatility-bands/Indicator-UlcerIndex.md)
- [Indicator-HistoricalVolatility.md](indicators/volatility-bands/Indicator-HistoricalVolatility.md)
- [Indicator-BollingerBandwidth.md](indicators/volatility-bands/Indicator-BollingerBandwidth.md)
- [Indicator-PercentB.md](indicators/volatility-bands/Indicator-PercentB.md)
- [Indicator-TrueRange.md](indicators/volatility-bands/Indicator-TrueRange.md)
- [Indicator-ChaikinVolatility.md](indicators/volatility-bands/Indicator-ChaikinVolatility.md)

**Trailing Stops** — ATR-driven stop-loss trackers.

- [Indicator-Psar.md](indicators/trailing-stops/Indicator-Psar.md)
- [Indicator-SuperTrend.md](indicators/trailing-stops/Indicator-SuperTrend.md)
- [Indicator-ChandelierExit.md](indicators/trailing-stops/Indicator-ChandelierExit.md)
- [Indicator-ChandeKrollStop.md](indicators/trailing-stops/Indicator-ChandeKrollStop.md)
- [Indicator-AtrTrailingStop.md](indicators/trailing-stops/Indicator-AtrTrailingStop.md)

**Volume** — price moves weighted or confirmed by traded volume.

- [Indicator-Obv.md](indicators/volume/Indicator-Obv.md)
- [Indicator-Vwap.md](indicators/volume/Indicator-Vwap.md)
- [Indicator-Adl.md](indicators/volume/Indicator-Adl.md)
- [Indicator-VolumePriceTrend.md](indicators/volume/Indicator-VolumePriceTrend.md)
- [Indicator-ChaikinMoneyFlow.md](indicators/volume/Indicator-ChaikinMoneyFlow.md)
- [Indicator-ChaikinOscillator.md](indicators/volume/Indicator-ChaikinOscillator.md)
- [Indicator-ForceIndex.md](indicators/volume/Indicator-ForceIndex.md)
- [Indicator-EaseOfMovement.md](indicators/volume/Indicator-EaseOfMovement.md)

**Price Statistics** — per-bar transforms and rolling regressions.

- [Indicator-TypicalPrice.md](indicators/price-statistics/Indicator-TypicalPrice.md)
- [Indicator-MedianPrice.md](indicators/price-statistics/Indicator-MedianPrice.md)
- [Indicator-WeightedClose.md](indicators/price-statistics/Indicator-WeightedClose.md)
- [Indicator-LinearRegression.md](indicators/price-statistics/Indicator-LinearRegression.md)
- [Indicator-LinRegSlope.md](indicators/price-statistics/Indicator-LinRegSlope.md)
- [Indicator-ZScore.md](indicators/price-statistics/Indicator-ZScore.md)
- [Indicator-LinRegAngle.md](indicators/price-statistics/Indicator-LinRegAngle.md)

## See also

- Source code: <https://github.com/kingchenc/wickra>
- Releases: <https://github.com/kingchenc/wickra/releases>
- Issue tracker: <https://github.com/kingchenc/wickra/issues>
