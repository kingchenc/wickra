// Cross-language data-layer parity for the WASM binding: replay the shared
// golden tick stream through TickAggregator and check the candles against the
// Rust-generated fixtures, with and without gap filling.

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const W = require('../pkg/wickra_wasm.js');

const GOLDEN = path.resolve(__dirname, '..', '..', '..', 'testdata', 'golden');

function readCsv(name) {
  const lines = fs.readFileSync(path.join(GOLDEN, `${name}.csv`), 'utf8').split(/\r?\n/);
  lines.shift();
  return lines.filter((l) => l.length > 0).map((l) => l.split(',').map(Number));
}

const TICKS = readCsv('data_ticks');

function run(gapFill) {
  const agg = new W.TickAggregator(1000, gapFill);
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

test('wasm tick aggregator matches the golden candles', () => {
  assertCandles(run(false), readCsv('data_candles'), 'no-gap');
});

test('wasm tick aggregator gap-fill matches the golden candles', () => {
  assertCandles(run(true), readCsv('data_candles_gap'), 'gap');
});

test('wasm candle reader matches the golden candles', () => {
  const csv = fs.readFileSync(path.join(GOLDEN, 'data_csv.csv'), 'utf8');
  const reader = new W.CandleReader(csv);
  const got = reader.read().map((c) => [c.open, c.high, c.low, c.close, c.volume, c.timestamp]);
  assertCandles(got, readCsv('data_csv_candles'), 'candle-reader');
});

const INPUT = readCsv('input'); // open,high,low,close,volume (timestamp = row index)

// `data_resampled_gap` drops input rows 20..44 (see `RESAMPLE_GAP` in
// gen_golden.rs), opening a five-bucket hole that gap filling covers with flat
// placeholder candles.
const RESAMPLE_GAP_START = 20;
const RESAMPLE_GAP_END = 44;

function runResample(gapFill) {
  const r = new W.Resampler(5, gapFill);
  const out = [];
  INPUT.forEach(([o, h, l, c, v], i) => {
    if (gapFill && i >= RESAMPLE_GAP_START && i <= RESAMPLE_GAP_END) {
      return;
    }
    // `update` returns every candle the push completed, which is more than one
    // when gap filling covers skipped buckets.
    for (const candle of r.update(o, h, l, c, v, i)) {
      out.push([candle.open, candle.high, candle.low, candle.close, candle.volume, candle.timestamp]);
    }
  });
  const f = r.flush();
  if (f) {
    out.push([f.open, f.high, f.low, f.close, f.volume, f.timestamp]);
  }
  return out;
}

test('wasm resampler matches the golden candles', () => {
  assert.equal(new W.Resampler(5).fillsGaps(), false);
  assertCandles(runResample(false), readCsv('data_resampled'), 'resample');
});

test('wasm resampler gap-fill matches the golden candles', () => {
  assert.equal(new W.Resampler(5, true).fillsGaps(), true);
  assertCandles(runResample(true), readCsv('data_resampled_gap'), 'resample-gap');
});
