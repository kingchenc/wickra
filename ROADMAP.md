# Roadmap

What Wickra is heading toward, what is explicitly out of scope, and what
contributors can expect across the next 0.x versions. Roadmap items are
**aspirations, not commitments** — order may shift based on real
user-feedback and bug-priority.

For "what shipped already" see [`CHANGELOG.md`](CHANGELOG.md). For the
internal structure that the roadmap items will plug into, see
[`ARCHITECTURE.md`](ARCHITECTURE.md).

## North star

> A streaming-first technical-analysis core that is the obvious default
> for anyone writing a Rust, Python, Node or browser-based trading
> system — drop-in fast, drop-in correct, drop-in tested.

Three measurable proxies for "obvious default":

1. **Coverage.** Every textbook indicator from TA-Lib, pandas-ta and
   talipp is in Wickra and produces matching reference values.
2. **Performance.** Streaming `update` is the fastest published number
   across Rust / Python / Node / WASM for any technical indicator.
3. **Trust.** Releases are reproducible, signed, SBOM-attached, with a
   public 100% test/branch coverage badge.

## 0.3.0 — target window: Q3 2026

The first release after the org migration to `wickra-lib` lands and the
project portal at `wickra.org` is live.

### Headline goals

- **WASM has automated tests.** `wasm-bindgen-test` job in CI exercising
  a representative subset (~30 indicators across all families). Today
  WASM is only smoke-validated through manual examples.
- **Release-pipeline trust.** SBOM (CycloneDX) generated per release,
  Sigstore cosign signatures attached to every published artifact,
  npm `--provenance` flag enabled, PyPI Trusted Publishers configured.
- **Hosted documentation portal.** `wickra.org` (Cloudflare Pages,
  VitePress) replaces the GitHub Wiki as the canonical doc surface.
  Per-indicator deep-dives, quickstarts, FAQ, search.
- **End-to-end strategy examples.** 3 runnable examples that wire
  Wickra indicators into a full mean-reversion / trend-following /
  breakout strategy with PnL and equity-curve output.
- **`ARCHITECTURE.md` + `ROADMAP.md` + `CITATION.cff`** governance
  baseline (this is it).

### Stretch goals (might slip to 0.4.0)

- **Per-binding hosted API reference.** TypeDoc for Node/WASM,
  Sphinx for Python, both hosted on `wickra.org/api/*`. Rust stays on
  `docs.rs`.
- **Property-tests (`proptest`) for mathematical invariants.** Bound
  checks on RSI ∈ [0, 100], Bollinger ordering, batch-streaming
  equivalence on random inputs.
- **Nightly long-fuzz workflow.** Each fuzz target gets ~1h overnight,
  findings auto-converted to issues.

## 0.4.0 — target window: Q4 2026

### Indicator catalogue expansion

The current 214 indicators cover the textbook canon. The next wave is
the "stuff people actually ask for but skip because it's painful in
other libs":

- **Anchored indicators.** AnchoredVwap is already in, but anchored
  variants of common indicators (AnchoredATR, AnchoredVolatility) are
  on the wishlist for explicit session-based analysis.
- **Multi-timeframe (MTF) chaining helpers.** A `Mtf<Indicator>` wrapper
  that runs an indicator on a different timeframe of the same input
  stream — solves the most common reason people drop down to ad-hoc
  buffering code.
- **Order-flow primitives** (gated on tick-data ingestion landing in
  `wickra-data`): CumulativeDelta, BidAskImbalance, VolumeAtPrice.
- **Pivot-confirmation patterns.** WilliamsFractals is already in;
  ZigZag is in. Next: PivotHigh/PivotLow with configurable
  left/right-bar confirmation, used as a feature for higher-level
  pattern detectors.
- **Harmonic-chart patterns** (Gartley, Bat, Butterfly, Crab, Shark).
  These need the pivot-detector + ratio-matcher framework first; the
  candlestick-pattern family from 0.2.8 is the precedent.

### Live-data layer

- **Tick-data ingestion.** Extend `wickra-data` from OHLCV-only to
  also accept raw ticks; the existing `Aggregator` already aggregates
  ticks into bars but isn't exposed in the public live-feed API yet.
- **Generic exchange trait.** `LiveFeed { fn subscribe(...) -> impl
  Stream<Item = Candle> }` so the Binance adapter is one implementor
  among many. **Note**: Wickra will not aggregate exchanges itself
  (use `ccxt` if you need that) — the trait exists so user code can
  swap feeds without touching indicator code.

### Performance & reliability

- **Performance-regression tracking.** `bench.yml` outputs deployed to
  a `gh-pages` branch, `github-action-benchmark` plot over time,
  threshold-based alerts on regression.
- **Indicator parity test suite.** Every indicator that has a TA-Lib
  / pandas-ta / talipp equivalent must pass a golden-value test
  against that reference. Currently most do but the test names are
  scattered — consolidate into one `parity_tests` module.

## 0.5.0 — target window: 2027 H1

### Possible new bindings

Open questions, prioritised by demand signals (≈ GitHub stars + issue
requests + community polls):

- **Java / Kotlin** binding via UniFFI — interest from Android-side
  trading-app developers and the JVM-quant community.
- **Go** binding — interest from algorithmic-trading shops running on
  Linux + Go infra.
- **C / C++** header export (`cbindgen`) — drops Wickra into existing
  C/C++ trading stacks (e.g. older Bloomberg / FIX-protocol shops).
- **Swift** — niche but real, iOS / macOS native trading apps.
- **.NET** — closes the Windows-native gap that NAPI-Node doesn't fill
  (some shops are still on .NET Framework).

None of these are committed — each is a 1-2-month project on its own,
and bindings without active maintenance are a liability. Priority will
be driven by which language community shows up with PRs.

### `wickra-data` widening

- **Historical fetch from > 1 source.** Today only Binance REST/WS.
  Add Coinbase and Kraken as reference implementations — explicitly
  not exchange-aggregation, just "here are two more adapters using
  the trait".

## Beyond — long-term aspirations

- **`wickra-backtest` sub-crate** (decision pending). A minimal
  event-driven backtester wrapping signal generation + position
  sizing + fees + slippage. Decision factor: do users keep building
  these ad-hoc from `examples/`? If yes, codifying it saves the
  ecosystem time. If most users plug Wickra into existing backtesters
  (vectorbt, backtrader, Lean, Hummingbot), keep Wickra focused.
- **`wickra-plot` companion** (low priority). `plotters`-backed Rust
  rendering of indicator outputs. Mainly useful for static report
  generation; live charting belongs in the JS/web layer.
- **Academic adoption.** Citable `CITATION.cff`; targeting at least
  one peer-reviewed paper using Wickra as the reference indicator
  engine.

## What is explicitly **not** on the roadmap

These are recurring requests that Wickra will decline so contributors
don't waste time on them:

| Feature | Why declined |
|---|---|
| Exchange aggregation across N venues | That's `ccxt`'s job. Wickra ships *one* feed implementor (Binance) for tests and demo; users plug their preferred exchange. |
| Full backtesting framework (à la Lean, vectorbt) | Scope creep. May land as `wickra-backtest` *if* a clear minimal API surfaces from user demand, but Wickra core stays an indicator library. |
| Strategy auto-tuning / hyperparameter search | Wrong abstraction layer — belongs in user-side ML/backtester. |
| Order-management / broker integration | Even further out of scope. |
| GUI / web-based studio | Marketing-site demos are fine; full IDE is not. |
| Indicator implementations that require optional Python deps (e.g. scipy KDE) | Bindings must work on a clean install. Pure-Rust math only. |
| Proprietary indicator implementations (e.g. paywalled vendor formulas) | License conflicts + audit burden. |
| GPU / CUDA acceleration | O(1) per update means the bottleneck is API overhead, not compute. SIMD-batch is a maybe; GPU adds no value for streaming. |

## How indicator wishlist requests are handled

1. **Open an issue** using the `feature_request` template, naming the
   indicator, citing one of:
   - TA-Lib reference
   - pandas-ta reference
   - peer-reviewed paper
   - widely-published trading book
2. Maintainer triages within ~1 week. Acceptance criteria:
   - Formula has at least one written reference (no random
     YouTuber-only indicators)
   - Implementation can be O(1) streaming
   - Test vectors are obtainable (from reference lib, hand-computed,
     or paper-provided)
3. Once accepted, the indicator gets added to the next family-batch
   PR. Family-batches typically ship 5-20 indicators at a time.

## Versioning

Wickra follows [SemVer](https://semver.org/). The promise:

- **0.x.0 (minor):** new indicators, new bindings, new optional features,
  new optional config knobs. Adding methods to `Indicator` with default
  impls is minor.
- **0.x.y (patch):** bug fixes, performance improvements, doc fixes.
- **Major (1.0.0 and later):** changes to the `Indicator` trait
  signature, removal of indicators, breaking changes to `Candle` /
  `OHLCV` field order.

The 1.0 milestone is reserved for when the indicator catalogue is
considered "stable enough" and the bindings API has stabilised — likely
~2027 once 2-3 more bindings have shipped and stress-tested the trait.

## Discussion

For roadmap discussion, open a [Discussions](https://github.com/wickra-lib/wickra/discussions)
thread tagged `roadmap`. Specific indicator wishlist items go in
[Issues](https://github.com/wickra-lib/wickra/issues) via the
`feature_request` template.
