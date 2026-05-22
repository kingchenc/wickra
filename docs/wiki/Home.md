# Wickra

Wickra is a streaming-first technical-indicators library. Every indicator is
implemented in Rust as an O(1) state machine that consumes one input at a
time, and the same engine is exposed through ergonomic bindings for Python,
Node.js, WebAssembly, and Rust itself. The same `update` call you write inside
a live trading loop also drives the historical backtest of that same
strategy — there is no second code path that drifts behind the streaming one.

The project ships 25 indicators across the four classical families (trend,
momentum, volatility, volume) and a small set of supporting types (`Candle`,
`Tick`, `Chain`). The Rust core forbids `unsafe`, so every binding inherits a
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
| crates.io | `wickra`       | 0.1.4   |
| crates.io | `wickra-core`  | 0.1.4   |
| crates.io | `wickra-data`  | 0.1.4   |
| PyPI      | `wickra`       | 0.1.4   |
| npm       | `wickra`       | 0.1.4   |
| npm       | `wickra-wasm`  | 0.1.4   |

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
  `SMA` and `MACD` calls, and the current Windows install caveat
  (`wickra-win32-x64-msvc@0.1.4` is held by the npm spam filter).
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

### Indicator reference

Start with [Indicators-Overview.md](Indicators-Overview.md) for the
cross-cutting taxonomy (trend / momentum / volatility / volume) and the
shared `Indicator` trait surface. The per-indicator pages below cover
formulas, parameters, warmup behaviour, edge cases, and verified
Rust / Python / Node examples. They are grouped by family, mirroring the
`indicators/<family>/` directory layout.

**Trend** — smooth the price series to surface direction.

- [Indicator-Sma.md](indicators/trend/Indicator-Sma.md)
- [Indicator-Ema.md](indicators/trend/Indicator-Ema.md)
- [Indicator-Wma.md](indicators/trend/Indicator-Wma.md)
- [Indicator-Dema.md](indicators/trend/Indicator-Dema.md)
- [Indicator-Tema.md](indicators/trend/Indicator-Tema.md)
- [Indicator-Hma.md](indicators/trend/Indicator-Hma.md)
- [Indicator-Kama.md](indicators/trend/Indicator-Kama.md)
- [Indicator-Smma.md](indicators/trend/Indicator-Smma.md)
- [Indicator-Trima.md](indicators/trend/Indicator-Trima.md)
- [Indicator-Zlema.md](indicators/trend/Indicator-Zlema.md)
- [Indicator-T3.md](indicators/trend/Indicator-T3.md)
- [Indicator-Vwma.md](indicators/trend/Indicator-Vwma.md)

**Momentum** — measure the rate of price change rather than the level.

- [Indicator-Rsi.md](indicators/momentum/Indicator-Rsi.md)
- [Indicator-MacdIndicator.md](indicators/momentum/Indicator-MacdIndicator.md)
- [Indicator-Stochastic.md](indicators/momentum/Indicator-Stochastic.md)
- [Indicator-Cci.md](indicators/momentum/Indicator-Cci.md)
- [Indicator-Roc.md](indicators/momentum/Indicator-Roc.md)
- [Indicator-WilliamsR.md](indicators/momentum/Indicator-WilliamsR.md)
- [Indicator-Adx.md](indicators/momentum/Indicator-Adx.md)
- [Indicator-Mfi.md](indicators/momentum/Indicator-Mfi.md)
- [Indicator-Trix.md](indicators/momentum/Indicator-Trix.md)
- [Indicator-AwesomeOscillator.md](indicators/momentum/Indicator-AwesomeOscillator.md)
- [Indicator-Aroon.md](indicators/momentum/Indicator-Aroon.md)
- [Indicator-Mom.md](indicators/momentum/Indicator-Mom.md)
- [Indicator-Cmo.md](indicators/momentum/Indicator-Cmo.md)
- [Indicator-Tsi.md](indicators/momentum/Indicator-Tsi.md)
- [Indicator-Pmo.md](indicators/momentum/Indicator-Pmo.md)
- [Indicator-StochRsi.md](indicators/momentum/Indicator-StochRsi.md)
- [Indicator-UltimateOscillator.md](indicators/momentum/Indicator-UltimateOscillator.md)
- [Indicator-Ppo.md](indicators/momentum/Indicator-Ppo.md)
- [Indicator-Dpo.md](indicators/momentum/Indicator-Dpo.md)
- [Indicator-Coppock.md](indicators/momentum/Indicator-Coppock.md)
- [Indicator-AroonOscillator.md](indicators/momentum/Indicator-AroonOscillator.md)
- [Indicator-Vortex.md](indicators/momentum/Indicator-Vortex.md)
- [Indicator-MassIndex.md](indicators/momentum/Indicator-MassIndex.md)

**Volatility** — envelope width and per-bar dispersion measures.

- [Indicator-BollingerBands.md](indicators/volatility/Indicator-BollingerBands.md)
- [Indicator-Atr.md](indicators/volatility/Indicator-Atr.md)
- [Indicator-Keltner.md](indicators/volatility/Indicator-Keltner.md)
- [Indicator-Donchian.md](indicators/volatility/Indicator-Donchian.md)
- [Indicator-Psar.md](indicators/volatility/Indicator-Psar.md)
- [Indicator-Natr.md](indicators/volatility/Indicator-Natr.md)
- [Indicator-StdDev.md](indicators/volatility/Indicator-StdDev.md)
- [Indicator-UlcerIndex.md](indicators/volatility/Indicator-UlcerIndex.md)
- [Indicator-HistoricalVolatility.md](indicators/volatility/Indicator-HistoricalVolatility.md)
- [Indicator-BollingerBandwidth.md](indicators/volatility/Indicator-BollingerBandwidth.md)
- [Indicator-PercentB.md](indicators/volatility/Indicator-PercentB.md)

**Volume** — price moves weighted or confirmed by traded volume.

- [Indicator-Obv.md](indicators/volume/Indicator-Obv.md)
- [Indicator-Vwap.md](indicators/volume/Indicator-Vwap.md)

## See also

- Source code: <https://github.com/kingchenc/wickra>
- Releases: <https://github.com/kingchenc/wickra/releases>
- Issue tracker: <https://github.com/kingchenc/wickra/issues>
