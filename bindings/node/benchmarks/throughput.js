// Throughput benchmark for the Wickra Node bindings.
//
// Measures how many indicator updates per second the native binding sustains,
// both per-tick (streaming `update`) and bulk (`batch`), over a synthetic
// OHLCV series. It is the Node counterpart of the Rust criterion benches and
// the Python `benchmarks/compare_libraries.py`; it benchmarks Wickra's own
// O(1) streaming engine (there is no install-free TA library on npm with a
// comparable surface to compare against), so the headline number is raw
// throughput, not a cross-library ratio.
//
// Run after building the binding:
//
//   cd bindings/node && npm install && npx napi build --platform --release
//   node benchmarks/throughput.js               # 200k bars (default)
//   node benchmarks/throughput.js --bars 1000000

const wickra = require('..');

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
const close = new Array(BARS);
const high = new Array(BARS);
const low = new Array(BARS);
const volume = new Array(BARS);
for (let i = 0; i < BARS; i++) {
  const mid = 100 + Math.sin(i * 0.001) * 20 + i * 1e-4;
  close[i] = mid + Math.sin(i * 0.05) * 2;
  high[i] = Math.max(close[i], mid) + 1.5;
  low[i] = Math.min(close[i], mid) - 1.5;
  volume[i] = 1000 + (i % 97) * 13;
}

// Median elapsed-ns over a few repetitions, after one warmup pass.
function timeNs(fn, reps = 3) {
  fn(); // warmup (JIT + cache)
  const samples = [];
  for (let r = 0; r < reps; r++) {
    const t0 = process.hrtime.bigint();
    fn();
    samples.push(Number(process.hrtime.bigint() - t0));
  }
  samples.sort((a, b) => a - b);
  return samples[Math.floor(samples.length / 2)];
}

function mupsFromNs(ns) {
  return (BARS / (ns / 1e9)) / 1e6; // million updates per second
}

// Each indicator: a streaming step and a batch call over the full series.
const indicators = [
  { name: 'SMA(20)', make: () => new wickra.SMA(20), step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  { name: 'EMA(20)', make: () => new wickra.EMA(20), step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  { name: 'RSI(14)', make: () => new wickra.RSI(14), step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  { name: 'StdDev(20)', make: () => new wickra.StdDev(20), step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  { name: 'MACD(12,26,9)', make: () => new wickra.MACD(12, 26, 9), step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  { name: 'BollingerBands(20,2)', make: () => new wickra.BollingerBands(20, 2), step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  { name: 'KAMA(10,2,30)', make: () => new wickra.KAMA(10, 2, 30), step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  { name: 'ATR(14)', make: () => new wickra.ATR(14), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  { name: 'ADX(14)', make: () => new wickra.ADX(14), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  { name: 'Stochastic(14,3)', make: () => new wickra.Stochastic(14, 3), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  { name: 'SuperTrend(10,3)', make: () => new wickra.SuperTrend(10, 3), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  { name: 'OBV', make: () => new wickra.OBV(), step: (ind, i) => ind.update(close[i], volume[i]), batch: (ind) => ind.batch(close, volume) },
];

console.log(`Wickra Node throughput — ${BARS.toLocaleString('en-US')} bars (median of 3 runs)\n`);
console.log(`${'Indicator'.padEnd(22)}${'streaming (Mupd/s)'.padStart(20)}${'batch (Mupd/s)'.padStart(18)}`);
console.log('-'.repeat(60));

for (const ind of indicators) {
  const streamNs = timeNs(() => {
    const inst = ind.make();
    for (let i = 0; i < BARS; i++) ind.step(inst, i);
  });
  const batchNs = timeNs(() => {
    ind.batch(ind.make());
  });
  console.log(
    `${ind.name.padEnd(22)}${mupsFromNs(streamNs).toFixed(1).padStart(20)}${mupsFromNs(batchNs).toFixed(1).padStart(18)}`,
  );
}

console.log(
  '\nMupd/s = million indicator updates per second. Streaming is the per-tick\n' +
    '`update` path (one value at a time); batch is the bulk array path. Higher is\n' +
    'better. Numbers are machine-dependent — use them for relative comparison.',
);
