# Quickstart: WebAssembly

A five-minute tour of the Wickra WebAssembly binding. The same Rust core that
powers the Python, Node, and Rust APIs is compiled to WebAssembly with
[wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/), so indicators run
entirely client-side — in a browser tab, a bundler build, or Node — with no
server round-trips.

## Install

The published package is `wickra-wasm` on npm:

```bash
npm install wickra-wasm
```

The npm package is built for the bundler target, so it works directly with
Webpack, Vite, Rollup, and similar toolchains.

## Build from source

To build the binding yourself you need [`wasm-pack`](https://rustwasm.github.io/wasm-pack/)
and the `wasm32-unknown-unknown` target:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

# Browser ES-module build (import directly from a <script type="module">):
wasm-pack build bindings/wasm --target web --release --features panic-hook

# Bundler build (Webpack / Vite / Rollup):
wasm-pack build bindings/wasm --target bundler --release --features panic-hook
```

`wasm-pack` writes the generated module, the `.wasm` binary, and TypeScript
definitions into `bindings/wasm/pkg/`. The `panic-hook` feature routes Rust
panics to `console.error` for readable stack traces during development.

## A first run (browser)

With a `--target web` build, import the generated module directly:

```html
<script type="module">
  import init, { version, SMA } from "./pkg/wickra_wasm.js";

  await init();                       // load and instantiate the .wasm binary
  console.log("wickra-wasm", version());

  const sma = new SMA(3);
  console.log(sma.batch([2, 4, 6, 8, 10]));
  // -> Float64Array [ NaN, NaN, 4, 6, 8 ]
</script>
```

`init()` must be awaited once before any indicator is constructed — it fetches
and instantiates the WebAssembly binary. After that the API mirrors the other
bindings.

## Streaming

```javascript
import init, { RSI } from "wickra-wasm";

await init();

const rsi = new RSI(14);
for (const price of liveFeed) {
  const value = rsi.update(price);    // number, or null/undefined during warmup
  if (value != null && value > 70) {
    console.log("overbought");
  }
}
```

Every indicator is an O(1)-per-update state machine: `update` advances the
indicator by exactly one input, so a browser charting app pays no cost for
recomputing history on each tick.

## Multi-output indicators

`MACD` and `BollingerBands` return a structured object from `update`:

```javascript
import init, { MACD } from "wickra-wasm";

await init();

const macd = new MACD(12, 26, 9);
let last = null;
for (let i = 0; i < 40; i++) {
  last = macd.update(100 + i * 0.5);  // null during warmup, else { macd, signal, histogram }
}
console.log(last);
```

`batch` returns a flat `Float64Array`; multi-output indicators interleave
their fields per row (`[macd0, signal0, hist0, macd1, ...]`). The exact
layout is documented in the generated `pkg/wickra_wasm.d.ts`.

## Errors

Unlike the Node binding (whose constructors clamp pathological values), the
WASM binding's constructors throw a JavaScript error for invalid parameters:

```javascript
try {
  new MACD(0, 0, 0);
} catch (e) {
  console.error("invalid MACD parameters:", e);
}
```

## A complete example

`examples/wasm/index.html` is a self-contained browser demo: it streams a
synthetic price series through six indicators and draws a live chart on a
`<canvas>`. Open it after a `--target web` build:

```bash
wasm-pack build bindings/wasm --target web --release --features panic-hook
# then serve the repository root and open examples/wasm/index.html
```

## See also

- [Quickstart: Node](Quickstart-Node.md) — the native (non-WASM) Node binding.
- [Streaming vs Batch](Streaming-vs-Batch.md) — why `update` is the primary
  entry point.
- [Indicators Overview](Indicators-Overview.md) — every indicator and its
  parameters.
- Source: <https://github.com/kingchenc/wickra>
