// Offline backtest example for the Wickra Node binding.
//
// Reads an OHLCV CSV, streams every candle through a basket of indicators
// (SMA, EMA, RSI, MACD, Bollinger Bands, ATR, ADX, OBV) with the O(1)
// `update` call, and prints a summary of each resulting series. It is the
// Node counterpart of `examples/python/backtest.py` and the Rust
// `crates/wickra/examples/backtest.rs`.
//
// Run it from the repository after building the native module:
//
//   cd bindings/node
//   npm install
//   npx napi build --platform --release
//   node examples/backtest.js                  # uses the bundled BTCUSDT 1d data
//   node examples/backtest.js path/to/ohlcv.csv # or any OHLCV CSV
//
// In your own project, install the package and use `require('wickra')`
// instead of the relative `require('..')` below.

const fs = require('node:fs');
const path = require('node:path');

const wickra = require('..');

// The OHLCV columns the default layout requires; the CSV header must name
// every one of them (any extra columns are ignored).
const REQUIRED_COLUMNS = ['timestamp', 'open', 'high', 'low', 'close', 'volume'];

// Default dataset: the checked-in BTCUSDT daily candles under the workspace
// `examples/data/` directory, resolved relative to this file.
const DEFAULT_CSV = path.join(
  __dirname,
  '..',
  '..',
  '..',
  'examples',
  'data',
  'btcusdt-1d.csv',
);

// Parse an OHLCV CSV into column arrays of numbers.
//
// The Wickra CSV layout is plain numeric — no quoted fields, no embedded
// commas — so splitting on `,` is a complete and correct parse for it.
function readHistory(csvPath) {
  const text = fs.readFileSync(csvPath, 'utf8');
  const lines = text.split(/\r?\n/).filter((line) => line.length > 0);
  if (lines.length === 0) {
    throw new Error(`${csvPath}: file is empty`);
  }

  const header = lines[0].split(',').map((cell) => cell.trim());
  const missing = REQUIRED_COLUMNS.filter((col) => !header.includes(col));
  if (missing.length > 0) {
    throw new Error(
      `${csvPath}: CSV header is missing required column(s): ${missing.join(', ')}; ` +
        `found: ${header.join(', ')}`,
    );
  }
  if (lines.length === 1) {
    throw new Error(`${csvPath}: CSV has a header but no data rows`);
  }

  const columnIndex = {};
  for (const col of REQUIRED_COLUMNS) {
    columnIndex[col] = header.indexOf(col);
  }

  const cols = { timestamp: [], open: [], high: [], low: [], close: [], volume: [] };
  for (let row = 1; row < lines.length; row++) {
    const cells = lines[row].split(',');
    for (const col of REQUIRED_COLUMNS) {
      const raw = cells[columnIndex[col]];
      const value = raw === undefined ? NaN : Number(raw.trim());
      if (raw === undefined || raw.trim() === '' || !Number.isFinite(value)) {
        throw new Error(
          `${csvPath}: row ${row + 1} column '${col}' is not numeric: ${JSON.stringify(raw)}`,
        );
      }
      cols[col].push(value);
    }
  }
  return cols;
}

// Running min / max / mean / last of one indicator output series. Null and
// non-finite values (the warmup phase) are skipped.
class Series {
  constructor(name) {
    this.name = name;
    this.count = 0;
    this.sum = 0;
    this.min = Infinity;
    this.max = -Infinity;
    this.last = NaN;
  }

  add(value) {
    if (value === null || value === undefined || !Number.isFinite(value)) {
      return;
    }
    this.count += 1;
    this.sum += value;
    if (value < this.min) this.min = value;
    if (value > this.max) this.max = value;
    this.last = value;
  }

  print() {
    if (this.count === 0) {
      console.log(`  ${this.name.padEnd(12)} (no valid samples — series too short)`);
      return;
    }
    const cell = (v) => v.toFixed(4).padStart(14);
    console.log(
      `  ${this.name.padEnd(12)} mean=${cell(this.sum / this.count)}  ` +
        `min=${cell(this.min)}  max=${cell(this.max)}  last=${cell(this.last)}`,
    );
  }
}

function main() {
  const csvPath = process.argv[2] || DEFAULT_CSV;

  let history;
  try {
    history = readHistory(csvPath);
  } catch (err) {
    console.error(`error: ${err.message}`);
    process.exit(1);
  }
  const bars = history.close.length;

  const sma = new wickra.SMA(20);
  const ema = new wickra.EMA(20);
  const rsi = new wickra.RSI(14);
  const macd = new wickra.MACD(12, 26, 9);
  const bb = new wickra.BollingerBands(20, 2.0);
  const atr = new wickra.ATR(14);
  const adx = new wickra.ADX(14);
  const obv = new wickra.OBV();

  const series = {
    sma: new Series('SMA(20)'),
    ema: new Series('EMA(20)'),
    rsi: new Series('RSI(14)'),
    macdLine: new Series('MACD line'),
    macdHist: new Series('MACD hist'),
    bbUpper: new Series('BB upper'),
    bbLower: new Series('BB lower'),
    atr: new Series('ATR(14)'),
    adx: new Series('ADX(14)'),
    obv: new Series('OBV'),
  };

  for (let i = 0; i < bars; i++) {
    const close = history.close[i];
    series.sma.add(sma.update(close));
    series.ema.add(ema.update(close));
    series.rsi.add(rsi.update(close));

    const m = macd.update(close); // { macd, signal, histogram } or null
    if (m) {
      series.macdLine.add(m.macd);
      series.macdHist.add(m.histogram);
    }

    const b = bb.update(close); // { upper, middle, lower, stddev } or null
    if (b) {
      series.bbUpper.add(b.upper);
      series.bbLower.add(b.lower);
    }

    series.atr.add(atr.update(history.high[i], history.low[i], close));

    const a = adx.update(history.high[i], history.low[i], close); // { plusDi, minusDi, adx } or null
    if (a) {
      series.adx.add(a.adx);
    }

    series.obv.add(obv.update(close, history.volume[i]));
  }

  console.log(`Backtest summary for ${csvPath} (${bars} bars)`);
  for (const s of Object.values(series)) {
    s.print();
  }
}

main();
