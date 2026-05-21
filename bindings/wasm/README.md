# wickra-wasm

WebAssembly bindings for the Wickra streaming-first technical indicators library.

## Build

You need [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) and the
`wasm32-unknown-unknown` Rust target:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

Then from the repository root:

```bash
wasm-pack build bindings/wasm --target web --release --features panic-hook
```

The compiled package lands in `bindings/wasm/pkg/`. Targets:

- `--target web` for native ES modules in browsers
- `--target bundler` for webpack/Vite/Rollup
- `--target nodejs` for Node.js

## Example

```js
import init, { SMA, RSI, MACD, version } from "./pkg/wickra_wasm.js";

await init();
console.log("wickra:", version());

// Streaming
const rsi = new RSI(14);
for (const price of livePrices) {
  const v = rsi.update(price);
  if (v !== undefined && v > 70) console.log("overbought");
}

// Batch (returns a Float64Array; NaN for warmup positions)
const sma = new SMA(20).batch(new Float64Array(historicalPrices));
```

An interactive demo lives in `bindings/wasm/examples/index.html`. After building
the package serve the `bindings/wasm/` directory and open `examples/index.html`.
