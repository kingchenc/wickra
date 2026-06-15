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
