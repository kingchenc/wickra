// Download real BTCUSDT spot candles from the Binance REST API and write
// them as CSV datasets under `examples/data/`.
//
// The Node counterpart of `examples/rust/src/bin/fetch_btcusdt.rs` and
// `examples/python/fetch_btcusdt.py` — same pagination, same validity
// check, same output layout. Uses Node's built-in global `fetch` and
// `node:fs` only; zero npm dependencies beyond the bundled `wickra`
// loader (which this script does not even import — it just talks HTTP
// and writes CSV).
//
// Requires Node 18 or newer (`fetch` is built-in there).
//
// Run with:
//   node fetch_btcusdt.js

const fs = require('node:fs');
const path = require('node:path');

// Binance Spot REST endpoint for historical klines.
const KLINES_URL = 'https://api.binance.com/api/v3/klines';
const SYMBOL = 'BTCUSDT';
// Binance caps a single klines response at 1000 rows.
const PAGE_LIMIT = 1000;
const REQUEST_PAUSE_MS = 200;

/** @type {{interval: string, file: string, target: number}[]} */
const DATASETS = [
  { interval: '1m', file: 'btcusdt-1m.csv', target: 50_000 },
  { interval: '5m', file: 'btcusdt-5m.csv', target: 10_000 },
  { interval: '15m', file: 'btcusdt-15m.csv', target: 10_000 },
  { interval: '1h', file: 'btcusdt-1h.csv', target: 10_000 },
  { interval: '12h', file: 'btcusdt-12h.csv', target: 5_000 },
  { interval: '1d', file: 'btcusdt-1d.csv', target: 5_000 },
  // The monthly file is `btcusdt-1month.csv`, not `btcusdt-1M.csv`, so it
  // does not collide with `btcusdt-1m.csv` on case-insensitive filesystems.
  { interval: '1M', file: 'btcusdt-1month.csv', target: 5_000 },
];

// Parse one Binance kline array. Returns null for any row that is malformed
// or fails the same OHLC validity check `Candle::new` applies in the Rust
// core.
function parseKline(raw) {
  if (!Array.isArray(raw) || raw.length < 7) return null;
  const openTime = Number(raw[0]);
  const closeTime = Number(raw[6]);
  const o = Number(raw[1]);
  const h = Number(raw[2]);
  const l = Number(raw[3]);
  const c = Number(raw[4]);
  const v = Number(raw[5]);
  if (![openTime, closeTime, o, h, l, c, v].every(Number.isFinite)) return null;
  if (h < l || h < o || h < c || l > o || l > c || v < 0) return null;
  return { openTime, closeTime, o, h, l, c, v };
}

async function fetchPage(interval, endTime) {
  const params = new URLSearchParams({
    symbol: SYMBOL,
    interval,
    limit: String(PAGE_LIMIT),
  });
  if (endTime !== null) params.set('endTime', String(endTime));
  const url = `${KLINES_URL}?${params}`;

  const resp = await fetch(url, { signal: AbortSignal.timeout(30000) });
  if (!resp.ok) {
    throw new Error(`Binance returned HTTP ${resp.status} for ${url}`);
  }
  const data = await resp.json();
  if (!Array.isArray(data)) {
    const sample = JSON.stringify(data).slice(0, 160);
    throw new Error(`expected a JSON array from Binance, got: ${sample}`);
  }
  return data;
}

async function collect(interval, target, nowMs) {
  // Keyed by open time so the map sorts chronologically and dedupes the
  // small overlap that can occur between adjacent pages.
  const byOpen = new Map();
  let endTime = null;
  let pages = 0;

  while (true) {
    const page = await fetchPage(interval, endTime);
    pages += 1;
    if (page.length === 0) break;

    let oldest = Number.MAX_SAFE_INTEGER;
    for (const raw of page) {
      const k = parseKline(raw);
      if (!k) continue;
      if (k.openTime < oldest) oldest = k.openTime;
      // Keep only fully closed candles — drop the in-progress bucket.
      if (k.closeTime < nowMs) byOpen.set(k.openTime, k);
    }
    process.stderr.write(
      `\r  ${interval}: collected ${byOpen.size} candles over ${pages} page(s)…`,
    );

    if (byOpen.size >= target || page.length < PAGE_LIMIT) break;
    if (oldest === Number.MAX_SAFE_INTEGER) {
      throw new Error(`Binance page for ${interval} held no parseable klines`);
    }
    endTime = oldest - 1;
    await new Promise((r) => setTimeout(r, REQUEST_PAUSE_MS));
  }
  process.stderr.write('\n');

  const sorted = [...byOpen.values()].sort((a, b) => a.openTime - b.openTime);
  return sorted.length > target ? sorted.slice(sorted.length - target) : sorted;
}

// JavaScript's `String(v)` already produces the shortest round-trip
// representation for finite numbers, and (unlike Python's repr) it already
// strips the `.0` suffix for whole-number floats — `String(77552.0)` is
// `'77552'`. The CSVs the Rust, Python and Node fetchers write are
// byte-for-byte identical on the same Binance snapshot.
function fmtNumber(v) {
  return String(v);
}

function writeCsv(filePath, candles) {
  const out = ['timestamp,open,high,low,close,volume'];
  for (const k of candles) {
    out.push(
      `${k.openTime},${fmtNumber(k.o)},${fmtNumber(k.h)},${fmtNumber(k.l)},${fmtNumber(k.c)},${fmtNumber(k.v)}`,
    );
  }
  fs.writeFileSync(filePath, out.join('\n') + '\n');
}

async function main() {
  if (typeof fetch !== 'function') {
    console.error(
      'This example needs Node 18+ for the global fetch() function.',
    );
    process.exit(1);
  }

  const nowMs = Date.now();
  const dataDir = path.join(__dirname, '..', 'data');
  fs.mkdirSync(dataDir, { recursive: true });
  console.log(`Fetching ${SYMBOL} klines from Binance into ${dataDir}`);

  for (const ds of DATASETS) {
    const candles = await collect(ds.interval, ds.target, nowMs);
    if (candles.length === 0) {
      console.error(`error: Binance returned no closed candles for ${ds.interval}`);
      process.exit(1);
    }
    writeCsv(path.join(dataDir, ds.file), candles);
    console.log(
      `  ${ds.interval.padStart(3)}  ${String(candles.length).padStart(6)} candles  ->  examples/data/${ds.file}`,
    );
  }
  console.log(`Done — ${DATASETS.length} datasets written.`);
}

main().catch((err) => {
  console.error(`error: ${err.message || err}`);
  process.exit(1);
});
