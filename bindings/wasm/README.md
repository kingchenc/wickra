# Wickra — WebAssembly

[![CI](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml/badge.svg)](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/wickra-lib/wickra/branch/main/graph/badge.svg)](https://codecov.io/gh/wickra-lib/wickra)
[![npm](https://img.shields.io/npm/v/wickra-wasm.svg?logo=npm&color=red)](https://www.npmjs.com/package/wickra-wasm)
[![License: PolyForm-NC](https://img.shields.io/badge/license-PolyForm--NC--1.0.0-purple)](https://github.com/wickra-lib/wickra/blob/main/LICENSE)

**Streaming-first technical indicators in the browser. `npm install
wickra-wasm` — pure WebAssembly, runs anywhere a modern JS engine does.**

Wickra is a multi-language technical-analysis library with a Rust core and
bindings for Python, Node.js, and WebAssembly. Every indicator is an O(1)
streaming state machine, so live trading dashboards and historical backtests
share the exact same implementation. This package is the WebAssembly binding
(wasm-bindgen, built for the `web` target); it exposes 200+ streaming-first
indicators across sixteen families.

## Install

```bash
npm install wickra-wasm
```

## Quick start

The module ships a default `init` export that loads the `.wasm` payload; await
it once before constructing indicators.

```js
import init, { RSI } from 'wickra-wasm';

await init(); // load the WebAssembly module once

// Streaming: feed prices tick by tick in O(1).
const rsi = new RSI(14);
for (const price of liveFeed) {
  const value = rsi.update(price); // null during warmup
  if (value !== null && value > 70) {
    console.log('overbought');
  }
}
```

Constructors mirror the other bindings (`new SMA(20)`, `new MACD(12, 26, 9)`,
`new BollingerBands(20, 2.0)`, …); `update()` returns the latest value or
`null` while the indicator is still warming up.

## Documentation

The full indicator catalogue, guides, quickstarts, and API reference live in
the main repository and wiki:

- **Repository & full indicator list:** <https://github.com/wickra-lib/wickra>
- **Wiki** (quickstarts, cookbook, TA-Lib migration): <https://github.com/wickra-lib/wickra/wiki>
- **Runnable browser examples:** [`examples/wasm/`](https://github.com/wickra-lib/wickra/tree/main/examples/wasm)

Wickra ships four bindings — Python, Node.js, WebAssembly, and Rust — that all
expose the same indicators from the shared, `unsafe`-forbidden Rust core.

## Disclaimer

Wickra is an indicator toolkit, not a trading system. The values it computes
are deterministic transforms of the input data — they are not financial advice
and do not predict the market. Any use in a live trading context is at your own
risk. The library is provided **as is**, without warranty of any kind.

## License

Licensed under the **PolyForm Noncommercial License 1.0.0**. Personal projects,
research, education, non-profits, and hobby trading bots are all fine; the one
thing not allowed is commercial sale of the software or of services built
around it. See [LICENSE](https://github.com/wickra-lib/wickra/blob/main/LICENSE).
