# Changelog

All notable changes to Wickra are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- 46 new technical indicators, taking the library from 25 to 71 and
  reorganising the catalogue into **eight families**, each with at least five
  members. Every indicator is implemented once in the Rust core and wired
  through the Python, Node and WASM bindings, with reference-value tests and a
  dedicated wiki page:
  - Moving Averages: `Smma`, `Trima`, `Zlema`, `T3`, `Vwma`.
  - Momentum Oscillators: `Mom`, `Cmo`, `Tsi`, `Pmo`, `StochRsi`,
    `UltimateOscillator`.
  - Trend & Directional: `AroonOscillator`, `Vortex`, `MassIndex`,
    `ChoppinessIndex`, `VerticalHorizontalFilter`.
  - Price Oscillators: `Ppo`, `Dpo`, `Coppock`, `AcceleratorOscillator`,
    `BalanceOfPower`.
  - Volatility & Bands: `Natr`, `StdDev`, `UlcerIndex`,
    `HistoricalVolatility`, `BollingerBandwidth`, `PercentB`, `TrueRange`,
    `ChaikinVolatility`.
  - Trailing Stops: `SuperTrend`, `ChandelierExit`, `ChandeKrollStop`,
    `AtrTrailingStop`.
  - Volume: `Adl`, `VolumePriceTrend`, `ChaikinMoneyFlow`,
    `ChaikinOscillator`, `ForceIndex`, `EaseOfMovement`.
  - Price Statistics: `TypicalPrice`, `MedianPrice`, `WeightedClose`,
    `LinearRegression`, `LinRegSlope`, `ZScore`, `LinRegAngle`.
- `TickAggregator::with_gap_fill` — opt-in mode that emits a flat placeholder
  candle for every empty bucket between two ticks, keeping the candle series
  evenly spaced for downstream indicators.
- CSV reader: a leading UTF-8 byte-order mark is stripped, fields are trimmed,
  and the header is validated against the required OHLCV columns.
- CI: an `msrv` job that builds and tests the workspace on Rust 1.75 and the
  node binding on Rust 1.77.
- Community health files: `CONTRIBUTING.md`, `SECURITY.md`,
  `CODE_OF_CONDUCT.md`, issue / pull-request templates, `CODEOWNERS`, and a
  Dependabot configuration.

### Changed
- The indicator wiki is reorganised into eight family folders under
  `docs/wiki/indicators/` (`moving-averages/`, `momentum-oscillators/`,
  `trend-directional/`, `price-oscillators/`, `volatility-bands/`,
  `trailing-stops/`, `volume/`, `price-statistics/`); `Indicators-Overview.md`,
  `Home.md` and the README indicator table follow the same eight families.
- `TickAggregator::push` returns `Result<Vec<Candle>>` (was
  `Result<Option<Candle>>`) so a single tick can yield a closed bar plus gap
  fillers.
- `Resampler::push` returns `Result<Option<Candle>>`: a candle in a bucket
  earlier than the open bar is now rejected as out of order.
- Aggregated candles are finalised through the validating `Candle::new`, so a
  volume that overflows to a non-finite value is surfaced as an error instead
  of producing a poisoned candle.
- All GitHub Actions are pinned to commit SHAs; the four publish jobs run in a
  protected `release` environment.

### Fixed
- `Timeframe::floor` no longer overflows for timestamps near `i64::MIN`.
- The aggregator rejects same-bucket ticks that arrive out of order instead of
  silently overwriting the bar's close with a stale price.
- The Binance live stream reconnects with exponential backoff, skips non-kline
  frames, applies a read timeout and message-size limits, and tracks a closed
  flag.
- Example scripts: `live_trading.py` skips non-kline frames and validates the
  symbol/interval; `backtest.py` and `multi_timeframe.py` report clear errors
  for malformed CSV input.

## [0.1.4] - 2026-05-21

### Added
- GitHub Release runs now attach every built artefact (wheels, sdist, native
  Node binaries, npm-pack tarballs, cargo `.crate` files) to the tag's
  release page.

## [0.1.3] - 2026-05-21

### Fixed
- npm package ships the napi-generated loader and is built with `--platform`
  so the per-platform binary is resolved correctly.

## [0.1.2] - 2026-05-21

### Fixed
- Release pipeline: per-platform idempotent npm publishing with a spam-filter
  retry, and committed `npm/<platform>/` package templates.

## [0.1.1] - 2026-05-21

### Fixed
- Node publish step and coordinated version bump across all bindings.

## [0.1.0] - 2026-05-21

### Added
- Initial release: a streaming-first technical-analysis library with 25
  indicators (SMA, EMA, WMA, DEMA, TEMA, HMA, KAMA, RSI, MACD, ROC, Stochastic,
  CCI, Williams %R, ADX, MFI, TRIX, Aroon, Awesome Oscillator, Bollinger Bands,
  ATR, Keltner Channels, Donchian Channels, Parabolic SAR, OBV, VWAP).
- Rust core (`wickra-core`), umbrella crate (`wickra`), and a data layer
  (`wickra-data`) with a CSV reader, tick aggregator, resampler, and an
  optional Binance live feed.
- Bindings for Python, Node.js, and WebAssembly.

[Unreleased]: https://github.com/kingchenc/wickra/compare/v0.1.4...HEAD
[0.1.4]: https://github.com/kingchenc/wickra/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/kingchenc/wickra/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/kingchenc/wickra/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/kingchenc/wickra/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/kingchenc/wickra/releases/tag/v0.1.0
