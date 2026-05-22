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

const REQUIRED_COLUMNS = ['timestamp', 'open', 'high', 'low', 'close', 'volume'];
const DEFAULT_CSV = path.join(__dirname, '..', 'data', 'btcusdt-1m.csv');
const ONE_MINUTE_MS = 60_000;

function readCsv(csvPath) {
  const text = fs.readFileSync(csvPath, 'utf8');
  const lines = text.split(/\r?\n/).filter((line) => line.length > 0);
  if (lines.length === 0) {
    throw new Error(`${csvPath}: file is empty`);
  }
  const header = lines[0].split(',').map((cell) => cell.trim());
  const missing = REQUIRED_COLUMNS.filter((col) => !header.includes(col));
  if (missing.length > 0) {
    throw new Error(
      `${csvPath}: missing required column(s): ${missing.join(', ')}; found: ${header.join(', ')}`,
    );
  }
  if (lines.length === 1) {
    throw new Error(`${csvPath}: CSV has a header but no data rows`);
  }
  const idx = {};
  for (const col of REQUIRED_COLUMNS) {
    idx[col] = header.indexOf(col);
  }
  const cols = { timestamp: [], open: [], high: [], low: [], close: [], volume: [] };
  for (let i = 1; i < lines.length; i++) {
    const cells = lines[i].split(',');
    for (const col of REQUIRED_COLUMNS) {
      const value = Number(cells[idx[col]]);
      if (!Number.isFinite(value)) {
        throw new Error(
          `${csvPath}: row ${i + 1} column '${col}' is not numeric: ${JSON.stringify(cells[idx[col]])}`,
        );
      }
      cols[col].push(value);
    }
  }
  return cols;
}

// Roll an OHLCV series up to `bucketMs`-sized buckets keyed on each bar's
// `floor(timestamp / bucketMs)`. Input timestamps must be monotonic
// non-decreasing (the bundled BTCUSDT-1m dataset is contiguous, so this
// holds by construction).
function resample(cols, bucketMs) {
  if (cols.timestamp.length === 0) {
    throw new Error('resample: empty input series');
  }
  const out = { timestamp: [], open: [], high: [], low: [], close: [], volume: [] };
  let bucketStart = Math.floor(cols.timestamp[0] / bucketMs) * bucketMs;
  let [o, h, l, c, v] = [cols.open[0], cols.high[0], cols.low[0], cols.close[0], cols.volume[0]];

  for (let i = 1; i < cols.timestamp.length; i++) {
    const start = Math.floor(cols.timestamp[i] / bucketMs) * bucketMs;
    if (start === bucketStart) {
      if (cols.high[i] > h) h = cols.high[i];
      if (cols.low[i] < l) l = cols.low[i];
      c = cols.close[i];
      v += cols.volume[i];
    } else {
      out.timestamp.push(bucketStart);
      out.open.push(o);
      out.high.push(h);
      out.low.push(l);
      out.close.push(c);
      out.volume.push(v);
      bucketStart = start;
      o = cols.open[i];
      h = cols.high[i];
      l = cols.low[i];
      c = cols.close[i];
      v = cols.volume[i];
    }
  }
  // Flush the final open bucket.
  out.timestamp.push(bucketStart);
  out.open.push(o);
  out.high.push(h);
  out.low.push(l);
  out.close.push(c);
  out.volume.push(v);
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
