// Throughput benchmark for the Wickra WebAssembly bindings.
//
// Measures how many indicator updates per second the wasm binding sustains,
// both per-tick (streaming `update`) and bulk (`batch`), over a synthetic
// OHLCV series. It is the wasm counterpart of the Node `throughput.js` and the
// Rust criterion benches: it benchmarks Wickra's own O(1) streaming engine
// across the JS<->wasm boundary (there is no install-free TA library with a
// comparable surface to compare against), so the headline number is raw
// per-binding throughput / FFI overhead, not a cross-library ratio.
//
// Three indicators are timed, chosen by FFI call-signature archetype rather
// than algorithm (the algorithm is identical to the Rust core; only the
// boundary cost differs): SMA (1-in -> 1-out), ATR (multi-in -> 1-out), and
// MACD (1-in -> multi-out). Streaming is timed for all three; batch only for
// the single-output SMA and ATR (multi-output batch is not exposed uniformly).
//
// Build the nodejs-target package first (needs the wasm32-unknown-unknown
// target, i.e. a rustup toolchain), then run:
//
//   cd bindings/wasm && wasm-pack build --target nodejs --out-dir pkg-node --release
//   node benchmarks/throughput.mjs               # 200k bars (default)
//   node benchmarks/throughput.mjs --bars 1000000

import { createRequire } from 'node:module';
import { hrtime } from 'node:process';

const require = createRequire(import.meta.url);
// wasm-pack --target nodejs emits a CommonJS module named after the crate.
const wasm = require('../pkg-node/wickra_wasm.js');
const { SMA, ATR, MACD } = wasm;

function parseBars() {
  const idx = process.argv.indexOf('--bars');
  if (idx !== -1 && process.argv[idx + 1]) {
    const n = Number(process.argv[idx + 1]);
    if (Number.isFinite(n) && n >= 1000) return Math.floor(n);
    console.error('--bars must be a number >= 1000');
    process.exit(1);
  }
  return 200_000;
}

const BARS = parseBars();

// Deterministic synthetic OHLCV (no RNG, so runs are comparable).
const close = new Float64Array(BARS);
const high = new Float64Array(BARS);
const low = new Float64Array(BARS);
for (let i = 0; i < BARS; i++) {
  const mid = 100 + Math.sin(i * 0.001) * 20 + i * 1e-4;
  close[i] = mid + Math.sin(i * 0.05) * 2;
  high[i] = Math.max(close[i], mid) + 1.5;
  low[i] = Math.min(close[i], mid) - 1.5;
}

// Median elapsed-ns over a few repetitions, after one warmup pass.
function timeNs(fn, reps = 3) {
  fn(); // warmup (JIT + cache)
  const samples = [];
  for (let r = 0; r < reps; r++) {
    const t0 = hrtime.bigint();
    fn();
    samples.push(Number(hrtime.bigint() - t0));
  }
  samples.sort((a, b) => a - b);
  return samples[Math.floor(samples.length / 2)];
}

function mupsFromNs(ns) {
  return BARS / (ns / 1e9) / 1e6; // million updates per second
}

// SMA (scalar 1-in/1-out), ATR (multi-in/1-out), MACD (1-in/multi-out).
const indicators = [
  {
    name: 'SMA(20)',
    stream: () => {
      const ind = new SMA(20);
      for (let i = 0; i < BARS; i++) ind.update(close[i]);
      ind.free();
    },
    batch: () => {
      const ind = new SMA(20);
      ind.batch(close);
      ind.free();
    },
  },
  {
    name: 'ATR(14)',
    stream: () => {
      const ind = new ATR(14);
      for (let i = 0; i < BARS; i++) ind.update(high[i], low[i], close[i]);
      ind.free();
    },
    batch: () => {
      const ind = new ATR(14);
      ind.batch(high, low, close);
      ind.free();
    },
  },
  {
    name: 'MACD(12,26,9)',
    stream: () => {
      const ind = new MACD(12, 26, 9);
      for (let i = 0; i < BARS; i++) ind.update(close[i]);
      ind.free();
    },
    batch: null, // multi-output: streaming only
  },
];

console.log(`Wickra WASM throughput — ${BARS.toLocaleString('en-US')} bars (median of 3 runs)\n`);
console.log(`${'Indicator'.padEnd(22)}${'streaming (Mupd/s)'.padStart(20)}${'batch (Mupd/s)'.padStart(18)}`);
console.log('-'.repeat(60));

for (const ind of indicators) {
  const streamMups = mupsFromNs(timeNs(ind.stream)).toFixed(1);
  const batchMups = ind.batch ? mupsFromNs(timeNs(ind.batch)).toFixed(1) : '—';
  console.log(`${ind.name.padEnd(22)}${streamMups.padStart(20)}${batchMups.padStart(18)}`);
}

console.log(
  '\nMupd/s = million indicator updates per second. Streaming is the per-tick\n' +
    '`update` path crossing the JS<->wasm boundary once per value; batch is the\n' +
    'bulk array path (one boundary crossing). Higher is better. Numbers are\n' +
    'machine-dependent — use them for relative comparison, not as a speed claim.',
);
