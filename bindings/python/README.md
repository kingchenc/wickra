# Wickra — Python

[![CI](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml/badge.svg)](https://github.com/wickra-lib/wickra/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/wickra-lib/wickra/branch/main/graph/badge.svg)](https://codecov.io/gh/wickra-lib/wickra)
[![PyPI](https://img.shields.io/pypi/v/wickra.svg?logo=pypi&color=blue)](https://pypi.org/project/wickra/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue)](https://github.com/wickra-lib/wickra#license)

**Streaming-first technical indicators for Python. `pip install wickra` — no
system dependencies, no C build tooling.**

Wickra is a multi-language technical-analysis library with a Rust core and
bindings for Python, Node.js and WebAssembly, plus a C ABI for C/C++, C#, Go and any
other C-capable language. Every indicator is an O(1)
streaming state machine, so live trading bots and historical backtests share
the exact same implementation. This package is the Python binding (PyO3); it
exposes 200+ streaming-first indicators across sixteen families.

## Install

```bash
pip install wickra
```

Pre-built wheels ship for Linux, macOS, and Windows — there is nothing to
compile and no C library to track down.

## Quick start

```python
import numpy as np
import wickra as ta

# Batch: classic TA-Lib-style usage over a whole array.
prices = np.linspace(100, 200, 1000)
rsi = ta.RSI(14)
values = rsi.batch(prices)              # numpy array, NaN during warmup

# Streaming: the same indicator, fed tick by tick in O(1).
rsi = ta.RSI(14)
for price in live_feed:
    value = rsi.update(price)           # no recomputation over history
    if value is not None and value > 70:
        print("overbought")
```

`batch(prices)` and feeding the same prices through `update()` produce
identical values — the equivalence is enforced by the test suite.

## Documentation

The full indicator catalogue, guides, quickstarts, and API reference live in
the main repository and documentation site:

- **Repository & full indicator list:** <https://github.com/wickra-lib/wickra>
- **Docs** (quickstarts, cookbook, TA-Lib migration): <https://docs.wickra.org>
- **Runnable examples:** [`examples/python/`](https://github.com/wickra-lib/wickra/tree/main/examples/python)

Wickra ships native bindings for Python, Node.js, WebAssembly and Rust, plus a
C ABI hub that any C-capable language (C, C++, Go, C#, Java, R) links against —
all exposing the same indicators from the shared, `unsafe`-forbidden Rust core.

## Disclaimer

Wickra is an indicator toolkit, not a trading system. The values it computes
are deterministic transforms of the input data — they are not financial advice
and do not predict the market. Any use in a live trading context is at your own
risk. The library is provided **as is**, without warranty of any kind.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra/blob/main/LICENSE-MIT) at your option.
