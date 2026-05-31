# Wickra — Node.js

[![CI](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml/badge.svg)](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/wickra-lib/wickra/branch/main/graph/badge.svg)](https://codecov.io/gh/wickra-lib/wickra)
[![npm](https://img.shields.io/npm/v/wickra.svg?logo=npm&color=red)](https://www.npmjs.com/package/wickra)
[![License: PolyForm-NC](https://img.shields.io/badge/license-PolyForm--NC--1.0.0-purple)](https://github.com/wickra-lib/wickra/blob/main/LICENSE)

**Streaming-first technical indicators for Node.js. `npm install wickra` —
prebuilt native binary, no system dependencies.**

Wickra is a multi-language technical-analysis library with a Rust core and
bindings for Python, Node.js, and WebAssembly. Every indicator is an O(1)
streaming state machine, so live trading bots and historical backtests share
the exact same implementation. This package is the Node.js binding (napi-rs);
it exposes 200+ streaming-first indicators across sixteen families.

## Install

```bash
npm install wickra
```

The native addon ships as a prebuilt binary per platform (Linux, macOS,
Windows — x64 and arm64), selected automatically through optional
dependencies. There is nothing to compile.

## Quick start

```js
const wickra = require('wickra');

// Batch: run an indicator over a whole array.
const prices = Array.from({ length: 1000 }, (_, i) => 100 + i * 0.1);
const values = new wickra.RSI(14).batch(prices); // null during warmup

// Streaming: the same indicator, fed tick by tick in O(1).
const rsi = new wickra.RSI(14);
for (const price of liveFeed) {
  const value = rsi.update(price); // no recomputation over history
  if (value !== null && value > 70) {
    console.log('overbought');
  }
}
```

`batch(prices)` and feeding the same prices through `update()` produce
identical values — the equivalence is enforced by the test suite.

## Documentation

The full indicator catalogue, guides, quickstarts, and API reference live in
the main repository and wiki:

- **Repository & full indicator list:** <https://github.com/wickra-lib/wickra>
- **Wiki** (quickstarts, cookbook, TA-Lib migration): <https://github.com/wickra-lib/wickra/wiki>
- **Runnable examples:** [`examples/node/`](https://github.com/wickra-lib/wickra/tree/main/examples/node)

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
