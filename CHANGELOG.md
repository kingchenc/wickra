# Changelog

All notable changes to Wickra are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **MSRV bumped.** Workspace minimum supported Rust version is now **1.85**
  (was 1.75) and the Node binding (`wickra-node`) is now **1.88** (was 1.77).
  The bumps are driven by transitive-dependency floors that were lifted in
  recent updates: `clap_lex >= 1.1.0` (pulled in via the criterion dev-dep)
  requires the stabilized `edition2024` feature (stable since Rust 1.85),
  and `napi-build >= 2.3.2` requires Rust 1.88. Pinning the deps to the
  older versions would have frozen us out of future security fixes from
  those upstreams, so lifting the MSRV is the cleaner path for a young 0.x
  library. Downstream consumers on older Rust toolchains can stay on
  Wickra 0.2.0.

## [0.2.0] - 2026-05-23

### Fixed
- `HistoricalVolatility::update` no longer substitutes a `0.0` log-return on
  non-positive prices (audit finding R13). Negative or zero prices are
  semantically invalid for a log-return calculation; silently treating them as
  "no movement" underreported realised volatility. They are now skipped — the
  previous valid value is returned and the indicator's state (`prev_price`,
  window, sums) is left untouched — matching how every other indicator handles
  invalid inputs.
- `Tick::new` now returns the new `Error::InvalidTick` variant for negative
  volume instead of `Error::InvalidCandle` (audit finding R14). A tick is not
  a candle, and downstream tick-stream pipelines should be able to match on a
  semantically-correct error. The Python binding's `map_err` was extended to
  forward the new variant as a `ValueError`; the Node and WASM bindings format
  via `Error::to_string()` and pick the new variant up automatically.
- `Psar::is_ready` now matches the convention shared by every other indicator:
  `is_ready() == true` iff a real value has been produced (audit finding R6).
  The previous implementation returned `self.initialised`, which flipped to
  `true` after the seed candle even though the seed candle itself returns
  `None`. A streaming consumer that wrote
  `if ind.is_ready() { use(ind.update(c)?) }` would hit an unexpected `None`
  on the first post-seed update. The fix introduces a `has_emitted` gate set
  when the first `Some` value is returned.
- `Psar::reset` now restores the compute fields (`prev_high`, `prev_low`,
  `sar`, `ep`) to `f64::NAN` sentinels instead of `0.0` (audit Opus-Bonus 1).
  The fields are gated by `initialised` today, so the `0.0` sentinel never
  leaked into output — but a future refactor that read them pre-init would
  have silently treated `0.0` as a real price. A `debug_assert!` at the read
  site makes the invariant explicit.

### Changed
- `Sma` and `BollingerBands` now reseed their incremental `sum` (and `sum_sq`
  for Bollinger) from the live window every `16 · period` finite updates,
  capping floating-point drift on long-running streams (audit findings R7 and
  L2-Rust). Previously the incremental single-subtract `sum -= old` could
  accumulate catastrophic-cancellation error on streams with alternating
  large/small magnitudes; the misleading `sma.rs` comment that claimed the
  drift was already bounded "by recomputing the sum after each pop" is
  replaced with an accurate description of the new reseed strategy. Amortised
  cost stays at O(1) (`O(period)` work amortised over `O(period)` updates),
  values are bit-identical on inputs that did not drift to begin with, and
  two new `long_stream_drift_stays_bounded` tests stress the recompute by
  alternating `1e9` / `1.0` (SMA) and `1e6` / `1.0` (Bollinger) for several
  recompute cycles and verify the reported values track a fresh from-scratch
  computation over the live window.
- `LinearRegression`, `LinRegSlope` and `LinRegAngle` (via composition over
  `LinRegSlope`) now run their rolling ordinary-least-squares fit
  **incrementally** in O(1) per update (audit finding R2). Previously every
  tick refit the line from scratch in O(period). The OLS denominators (`Σx`
  and `Σxx`) depend only on `period`, so they were already precomputed; this
  release adds running `Σy` and `Σxy` accumulators and slides them in closed
  form via the identity
  `new_Σxy = old_Σxy − old_Σy + popped_y₀` (then `Σxy += (n − 1) · new_value`
  and `Σy += new_value`). New per-bar equivalence tests compare the O(1)
  output against a fresh O(n) refit on noisy ramps, step functions, and
  constants — values agree to within 1e-9.
- Fuzz suite expanded from 2 indicators to the full catalogue (audit finding
  R9). The existing `indicator_update` target now exercises every scalar-input
  indicator (~33 classes including MACD and Bollinger Bands); a new
  `indicator_update_candle` target exercises every candle-input indicator (~37
  classes, including ATR, ADX, Stochastic, PSAR, Keltner, SuperTrend,
  ChandelierExit, AwesomeOscillator, OBV, MFI, VWAP, RollingVWAP, and the rest
  of the volume / volatility / trailing-stop / price-statistics families). Each
  iteration sweeps every indicator through both the streaming `update` loop
  and a full `batch` call so any state-mutation bug surfaces on either path.
  CI gains a `fuzz-smoke` job that runs each of the five targets for 30 s on
  every push and pull-request.
- `UlcerIndex::update` now tracks the trailing maximum with a monotonically-
  decreasing deque of `(index, price)` pairs instead of scanning the whole
  trailing window on every tick. The indicator now honours the `Indicator`
  trait's O(1)-per-tick contract; values and warmup semantics are unchanged
  (verified by a new adversarial-input test that compares the deque output
  bar-by-bar against a naive O(n) trailing-max scan on strictly increasing,
  strictly decreasing, constant, and sawtooth inputs). The doc comment on
  `warmup_period()` is also corrected: the two windows overlap by one bar, so
  the formula is `2 * period - 1`.

### Added
- `RollingVWAP` is now exposed in Python, Node and WASM under that name
  (previously the rolling-window VWAP existed only in the Rust core, even
  though the README's volume-family table already advertised
  `VWAP (cumulative + rolling)`). All four bindings now ship the same
  cumulative `VWAP` plus the finite-window `RollingVWAP(period)`. The wiki page
  `Indicator-Vwap.md` adds Python, Node and WASM examples and drops the
  "Rust-only" caveat.
- WASM binding now exposes the streaming `update()` method on every candle-input
  indicator: `Adx`, `WilliamsR`, `Cci`, `Mfi`, `Psar`, `Keltner`, `Donchian`,
  `Vwap`, `AwesomeOscillator`, `Aroon`, `Stochastic`, and `Obv`. Multi-output
  indicators (`Adx`, `Keltner`, `Donchian`, `Aroon`, `Stochastic`) return a
  named JS object (`{ plusDi, minusDi, adx }`, `{ upper, middle, lower }`,
  `{ up, down }`, `{ k, d }`) once warm, or `null` during warmup — matching the
  existing `SuperTrend` convention. Each class also gains `reset()`, `isReady()`
  and `warmupPeriod()`, bringing the WASM surface to full parity with Python
  and Node so browser-side streaming code no longer has to replay `batch()`
  on every tick. `WasmKama` gains the previously missing `warmupPeriod()`.
- New `wasm-bindgen` integration test exercises `update == batch` plus the full
  lifecycle (`reset` / `isReady` / `warmupPeriod`) for all twelve newly wired
  classes against a deterministic 40-bar synthetic OHLCV stream.

### Security
- Upgrade `pyo3` (0.22 → 0.28) and `numpy` (0.22 → 0.28) in the Python binding.
  Fixes [RUSTSEC-2025-0020](https://rustsec.org/advisories/RUSTSEC-2025-0020) —
  a buffer overflow in `PyString::from_object` that affected the published
  Python wheels. The `cargo-deny` ignore entry that previously suppressed the
  advisory has been removed; `cargo deny check` is now clean without
  suppression. Migrated `into_pyarray_bound` to `into_pyarray`,
  `downcast::<PyDict>` to `cast::<PyDict>`, and opted every `#[pyclass]` out of
  the deprecated automatic `FromPyObject` derive via `skip_from_py_object`.

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
- Seven example OHLCV datasets under `examples/data/`, one per timeframe
  (1m / 5m / 15m / 1h / 12h / 1d / 1month), holding real BTCUSDT spot klines,
  alongside the `fetch_btcusdt` example that regenerates them from the
  Binance REST API.
- `Timeframe::minutes`, `Timeframe::hours` and `Timeframe::days` convenience
  constructors, each building on seconds with a checked-multiplication
  overflow guard.

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
- The indicator benchmarks (`crates/wickra/benches/indicators.rs`) now run
  against the checked-in real BTCUSDT 1-minute dataset instead of a synthetic
  price series.
- Every language's examples now live under a uniform `examples/<lang>/`
  tree: Rust moved into a new `examples/rust/` workspace member crate
  (`wickra-examples`, run via `cargo run -p wickra-examples --bin <name>`),
  Node into `examples/node/` with its own `package.json` linking `wickra` via
  `file:../../bindings/node`, and the WASM browser demos into
  `examples/wasm/`. The bundled BTCUSDT datasets move alongside them at
  `examples/data/`. Six new examples close the cross-language parity matrix:
  streaming demos for Python and Rust; multi-timeframe and parallel-assets
  demos for both Rust and Node.
- Cross-language data-generator parity: `examples/python/fetch_btcusdt.py`
  (stdlib only: `urllib` + `json` + `csv`) and `examples/node/fetch_btcusdt.js`
  (Node 18+ built-in `fetch`) mirror the Rust `fetch_btcusdt` binary —
  byte-for-byte identical CSV output on the same Binance snapshot.
- Four additional WebAssembly browser demos under `examples/wasm/`
  alongside the original `index.html`: `backtest.html` (fetch + basket of
  indicators), `live_trading.html` (browser-native `WebSocket` to
  Binance), `multi_timeframe.html` (in-page resample) and
  `parallel_assets.html` + `parallel_worker.js` (module-Worker pool with
  serial-vs-parallel speedup). The cross-language matrix is now closed
  for every cell where the pattern makes sense.
- Three new wiki pages: `TA-Lib-Migration.md` (full mapping table from
  `talib.X(...)` calls to Wickra), `Cookbook.md` (seven concrete
  strategy recipes — RSI mean reversion, MACD crossover, Bollinger
  breakout, ADX-gated trend, multi-timeframe confirmation, SuperTrend,
  chained indicators) and `FAQ.md`. All three linked from `Home.md`.

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

[Unreleased]: https://github.com/kingchenc/wickra/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/kingchenc/wickra/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/kingchenc/wickra/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/kingchenc/wickra/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/kingchenc/wickra/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/kingchenc/wickra/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/kingchenc/wickra/releases/tag/v0.1.0
