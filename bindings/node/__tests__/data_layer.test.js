// Cross-language data-layer parity for the Node binding: replay the shared
// golden tick stream through the TickAggregator and check the candles bit-for-bit
// (within fp tolerance) against the Rust-generated fixtures, with and without
// gap filling. Fixtures are produced by `cargo run -p wickra-examples --bin
// gen_golden`.

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { TickAggregator, Resampler, CandleReader } = require('..');

const GOLDEN = path.resolve(__dirname, '..', '..', '..', 'testdata', 'golden');

function readCsv(name) {
  const lines = fs.readFileSync(path.join(GOLDEN, `${name}.csv`), 'utf8').split(/\r?\n/);
  lines.shift();
  return lines.filter((l) => l.length > 0).map((l) => l.split(',').map(Number));
}

const TICKS = readCsv('data_ticks');

function run(gapFill) {
  const agg = new TickAggregator(1000, gapFill);
  const out = [];
  for (const [price, size, ts] of TICKS) {
    for (const c of agg.push(price, size, ts)) {
      out.push([c.open, c.high, c.low, c.close, c.volume, c.timestamp]);
    }
  }
  return out;
}

function assertCandles(got, want, label) {
  assert.equal(got.length, want.length, `${label}: candle count ${got.length} vs ${want.length}`);
  for (let i = 0; i < got.length; i++) {
    for (let j = 0; j < 6; j++) {
      const tol = 1e-9 * Math.max(1, Math.abs(want[i][j]));
      assert.ok(
        Math.abs(got[i][j] - want[i][j]) <= tol,
        `${label} row ${i} col ${j}: ${got[i][j]} vs ${want[i][j]}`,
      );
    }
  }
}

test('tick aggregator matches the golden candles', () => {
  assertCandles(run(false), readCsv('data_candles'), 'no-gap');
});

test('tick aggregator gap-fill matches the golden candles', () => {
  assertCandles(run(true), readCsv('data_candles_gap'), 'gap');
});

test('candle reader matches the golden candles', () => {
  const csv = fs.readFileSync(path.join(GOLDEN, 'data_csv.csv'), 'utf8');
  const reader = new CandleReader(csv);
  const got = reader.read().map((c) => [c.open, c.high, c.low, c.close, c.volume, c.timestamp]);
  assertCandles(got, readCsv('data_csv_candles'), 'candle-reader');
});

const INPUT = readCsv('input'); // open,high,low,close,volume (timestamp = row index)

function runResample() {
  const r = new Resampler(5);
  const out = [];
  INPUT.forEach(([o, h, l, c, v], i) => {
    const candle = r.update(o, h, l, c, v, i);
    if (candle) {
      out.push([candle.open, candle.high, candle.low, candle.close, candle.volume, candle.timestamp]);
    }
  });
  const f = r.flush();
  if (f) {
    out.push([f.open, f.high, f.low, f.close, f.volume, f.timestamp]);
  }
  return out;
}

test('resampler matches the golden candles', () => {
  assertCandles(runResample(), readCsv('data_resampled'), 'resample');
});
