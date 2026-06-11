# Wickra — R <img src="man/figures/logo.png" align="right" height="120" alt="Wickra logo" />

[![CI](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml/badge.svg)](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/wickra-lib/wickra/branch/main/graph/badge.svg)](https://codecov.io/gh/wickra-lib/wickra)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue)](https://github.com/wickra-lib/wickra#license)

**Streaming-first technical indicators for R, over the Wickra C ABI hub via `.Call`.**

Wickra is a multi-language technical-analysis library with a Rust core and
bindings for Python, Node.js and WebAssembly, plus a C ABI for C/C++, C#, Go, Java, R
and any other C-capable language. Every indicator is an O(1) streaming state
machine, so live trading and historical backtests share the exact same
implementation. This package is the R binding; it reaches the C ABI hub through
R's native `.Call` interface and exposes all 514 indicators as constructors that
return a lightweight `wickra_indicator` object.

## Install

The package compiles a thin C glue layer (`.Call`) against the prebuilt Wickra
C ABI library, so a C toolchain (Rtools on Windows) is required, plus the C ABI
header and library. Build the library from the workspace, then install the
package pointing at it:

```bash
cargo build -p wickra-c --release
WICKRA_INCLUDE_DIR="$PWD/bindings/c/include" \
WICKRA_LIB_DIR="$PWD/target/release" \
R CMD INSTALL bindings/r
```

On Windows the C ABI DLL is bundled into the package and put on the load path
automatically; on Linux and macOS the library path is baked in via rpath.

## Quick start

```r
library(wickra)

# Batch: run an indicator over a whole series (NaN at warmup positions).
prices <- 100 + (0:999) * 0.1
sma <- Sma(20)
values <- batch(sma, prices)

# Streaming: the same indicator, fed one observation at a time in O(1).
rsi <- Rsi(14)
for (price in prices) {
  v <- update(rsi, price) # NaN during warmup
  if (!is.na(v) && v > 70) message("overbought")
}

# Multi-output indicators return a named vector (NA while warming up).
macd <- MacdIndicator(12, 26, 9)
update(macd, 42) # c(macd = NA, signal = NA, histogram = NA)
```

`batch(ind, prices)` and feeding the same prices through `update()` produce
identical values — the equivalence is enforced by the test suite. Candle-input
indicators take the OHLCV fields plus a timestamp, e.g.
`update(atr, open, high, low, close, volume, timestamp)`. The native handle is
freed automatically when the object is garbage-collected.

## Benchmark

`benchmarks/throughput.R` reports streaming and batch updates-per-second for
`SMA`, `ATR` and `MACD`. It measures this binding's FFI overhead, not a
cross-library ratio (the same Rust core runs under every binding) — see the
repository [BENCHMARKS.md](https://github.com/wickra-lib/wickra/blob/main/BENCHMARKS.md) §3.

```bash
Rscript benchmarks/throughput.R
```

## Documentation

The full indicator catalogue, guides, quickstarts, and API reference live in the
main repository and documentation site:

- **Repository & full indicator list:** <https://github.com/wickra-lib/wickra>
- **Docs** (quickstarts, cookbook, TA-Lib migration): <https://docs.wickra.org>
- **Runnable examples:** [`examples/r/`](https://github.com/wickra-lib/wickra/tree/main/examples/r)

Wickra ships native bindings for Python, Node.js, WebAssembly and Rust, plus a
C ABI hub that any C-capable language (C, C++, C#, Go, Java, R) links against —
all exposing the same indicators from the shared, `unsafe`-forbidden Rust core.

## Disclaimer

Wickra is an indicator toolkit, not a trading system. The values it computes are
deterministic transforms of the input data — they are not financial advice and
do not predict the market. Any use in a live trading context is at your own risk.
The library is provided **as is**, without warranty of any kind.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra/blob/main/LICENSE-MIT) at your option.
