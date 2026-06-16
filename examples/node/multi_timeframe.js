// Multi-timeframe indicators with the Wickra Node binding.
//
// Reads the bundled 1-minute BTCUSDT CSV (or a path passed on the command
// line), rolls it up to coarser timeframes (5m / 15m / 1h / 4h / 1d) and
// prints the last RSI(14), MACD(12,26,9) histogram and ADX(14) on each.
// Mirrors examples/python/multi_timeframe.py — wickra-data's Resampler is
// only exposed in Rust today, so the roll-up is computed inline here.
//
// Run with:
//   node examples/node/multi_timeframe.js [path/to/1m.csv]

const fs = require('node:fs');
const path = require('node:path');

const wickra = require('wickra');

const DEFAULT_CSV = path.join(__dirname, '..', 'data', 'btcusdt-1m.csv');
const ONE_MINUTE_MS = 60_000;

function readCsv(csvPath) {
  // Native CandleReader: header validation, BOM/whitespace tolerance, throws on
  // a malformed row.
  const text = fs.readFileSync(csvPath, 'utf8');
  const candles = new wickra.CandleReader(text).read();
  if (candles.length === 0) {
    throw new Error(`${csvPath}: CSV has a header but no data rows`);
  }
  const cols = { timestamp: [], open: [], high: [], low: [], close: [], volume: [] };
  for (const k of candles) {
    cols.timestamp.push(k.timestamp);
    cols.open.push(k.open);
    cols.high.push(k.high);
    cols.low.push(k.low);
    cols.close.push(k.close);
    cols.volume.push(k.volume);
  }
  return cols;
}

function resample(cols, bucketMs) {
  if (cols.timestamp.length === 0) {
    throw new Error('resample: empty input series');
  }
  // Native Resampler — no hand-written bucketing. update() emits a closed candle
  // when a bucket boundary is crossed; flush() yields the final partial bucket.
  const r = new wickra.Resampler(bucketMs);
  const out = { timestamp: [], open: [], high: [], low: [], close: [], volume: [] };
  const push = (k) => {
    out.timestamp.push(k.timestamp);
    out.open.push(k.open);
    out.high.push(k.high);
    out.low.push(k.low);
    out.close.push(k.close);
    out.volume.push(k.volume);
  };
  for (let i = 0; i < cols.timestamp.length; i++) {
    const k = r.update(cols.open[i], cols.high[i], cols.low[i], cols.close[i], cols.volume[i], cols.timestamp[i]);
    if (k !== null) push(k);
  }
  const last = r.flush();
  if (last !== null) push(last);
  return out;
}

function summarize(label, cols) {
  if (cols.close.length === 0) {
    console.log(`  ${label.padEnd(5)} (empty)`);
    return;
  }
  const rsi = new wickra.RSI(14);
  const macd = new wickra.MACD(12, 26, 9);
  const adx = new wickra.ADX(14);

  let lastRsi = null;
  let lastHist = null;
  let lastAdx = null;
  for (let i = 0; i < cols.close.length; i++) {
    const r = rsi.update(cols.close[i]);
    if (r !== null) lastRsi = r;
    const m = macd.update(cols.close[i]);
    if (m) lastHist = m.histogram;
    const a = adx.update(cols.high[i], cols.low[i], cols.close[i]);
    if (a) lastAdx = a.adx;
  }
  const lastClose = cols.close[cols.close.length - 1];
  const fmtR = lastRsi === null ? '    --' : lastRsi.toFixed(2).padStart(6);
  const fmtH = lastHist === null ? ' --   ' : `${lastHist >= 0 ? '+' : ''}${lastHist.toFixed(2)}`.padStart(6);
  const fmtA = lastAdx === null ? '    --' : lastAdx.toFixed(2).padStart(6);
  console.log(
    `  ${label.padEnd(5)} bars=${String(cols.close.length).padStart(5)}  ` +
      `last_close=${lastClose.toFixed(2).padStart(10)}  ` +
      `rsi=${fmtR}  macd_hist=${fmtH}  adx=${fmtA}`,
  );
}

function main() {
  const csvPath = process.argv[2] || DEFAULT_CSV;
  let cols;
  try {
    cols = readCsv(csvPath);
  } catch (err) {
    console.error(`error: ${err.message}`);
    process.exit(1);
  }
  console.log(`Multi-timeframe view of ${csvPath}`);
  summarize('1m', cols);
  for (const [label, mins] of [
    ['5m', 5],
    ['15m', 15],
    ['1h', 60],
    ['4h', 240],
    ['1d', 1440],
  ]) {
    summarize(label, resample(cols, mins * ONE_MINUTE_MS));
  }
}

main();
