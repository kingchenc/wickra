# Wickra — Go

[![CI](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml/badge.svg)](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/wickra-lib/wickra/branch/main/graph/badge.svg)](https://codecov.io/gh/wickra-lib/wickra)
[![Go Reference](https://pkg.go.dev/badge/github.com/wickra-lib/wickra/bindings/go.svg)](https://pkg.go.dev/github.com/wickra-lib/wickra/bindings/go)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue)](https://github.com/wickra-lib/wickra#license)

**Streaming-first technical indicators for Go, over the Wickra C ABI hub via cgo.**

Wickra is a multi-language technical-analysis library with a Rust core and
bindings for Python, Node.js and WebAssembly, plus a C ABI for C/C++, C#, Go and
any other C-capable language. Every indicator is an O(1) streaming state machine,
so live trading bots and historical backtests share the exact same
implementation. This package is the Go binding; it consumes the C ABI hub through
cgo and exposes all 514 streaming-first indicators as idiomatic types.

## Install

```bash
go get github.com/wickra-lib/wickra/bindings/go
```

The binding uses cgo, so a C compiler is required, and it links against the
prebuilt Wickra C ABI library. Build that library from the workspace and stage
it under this package's `lib/` directory:

```bash
cargo build -p wickra-c --release
cp target/release/libwickra.so    bindings/go/lib/   # Linux
cp target/release/libwickra.dylib bindings/go/lib/   # macOS
cp target/release/wickra.dll      bindings/go/lib/   # Windows (also on PATH at run time)
```

On Linux and macOS the library path is baked in via rpath; on Windows the DLL
must be discoverable at run time (next to the executable or on `PATH`).

## Quick start

```go
package main

import (
	"fmt"

	wickra "github.com/wickra-lib/wickra/bindings/go"
)

func main() {
	// Batch: run an indicator over a whole series (NaN at warmup positions).
	prices := make([]float64, 1000)
	for i := range prices {
		prices[i] = 100.0 + float64(i)*0.1
	}
	sma, _ := wickra.NewSma(20)
	defer sma.Close()
	values := sma.Batch(prices)

	// Streaming: the same indicator, fed tick by tick in O(1).
	rsi, _ := wickra.NewRsi(14)
	defer rsi.Close()
	for _, price := range prices {
		value := rsi.Update(price) // NaN during warmup, no recomputation
		if value > 70 {
			fmt.Println("overbought")
		}
	}
	_ = values
}
```

`Batch(prices)` and feeding the same prices through `Update()` produce identical
values — the equivalence is enforced by the test suite. Multi-output indicators
(MACD, Bollinger, ADX, …) return `(Output, bool)`, with `false` while warming up.
Every indicator owns a native handle freed by `Close()`; a finalizer is wired as
a backstop, but call `Close()` (e.g. with `defer`) to release memory promptly.

## Documentation

The full indicator catalogue, guides, quickstarts, and API reference live in the
main repository and documentation site:

- **Repository & full indicator list:** <https://github.com/wickra-lib/wickra>
- **Docs** (quickstarts, cookbook, TA-Lib migration): <https://docs.wickra.org>
- **Runnable examples:** [`examples/go/`](https://github.com/wickra-lib/wickra/tree/main/examples/go)

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
