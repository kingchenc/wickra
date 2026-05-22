# wickra

Node.js bindings for the Wickra streaming-first technical indicators library.

## Install

Once published, install per platform via the precompiled native package:

```bash
npm install wickra
```

## Build from source

```bash
cd bindings/node
npm install
npm run build
npm test
```

The native module is built via [napi-rs](https://napi.rs/). The build script
produces a `wickra.<platform>-<arch>.node` binary in the package root that
`index.js` loads at runtime.

## Usage

```js
import { SMA, RSI, MACD, version } from 'wickra';

console.log('wickra', version());

// Batch:
const prices = Array.from({ length: 1000 }, (_, i) => 100 + Math.sin(i * 0.1) * 5);
const rsi = new RSI(14).batch(prices);

// Streaming:
const macd = new MACD(12, 26, 9);
for (const p of livePriceStream) {
  const v = macd.update(p);
  if (v && v.histogram > 0) console.log('bullish crossover candidate');
}
```

See `index.d.ts` for the full TypeScript surface.
