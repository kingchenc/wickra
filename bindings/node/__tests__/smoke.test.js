// Smoke tests for the Wickra Node bindings.
//
// Run with:
//   cd bindings/node && npm install && npm run build && npm test

const test = require('node:test');
const assert = require('node:assert/strict');
const wickra = require('..');

test('version is non-empty', () => {
  assert.ok(typeof wickra.version() === 'string');
  assert.ok(wickra.version().length > 0);
});

test('SMA batch matches reference values', () => {
  const sma = new wickra.SMA(3);
  const out = sma.batch([2, 4, 6, 8, 10]);
  assert.ok(Number.isNaN(out[0]));
  assert.ok(Number.isNaN(out[1]));
  assert.equal(out[2], 4);
  assert.equal(out[3], 6);
  assert.equal(out[4], 8);
});

test('RSI pure uptrend yields 100', () => {
  const rsi = new wickra.RSI(14);
  const prices = Array.from({ length: 20 }, (_, i) => i + 1);
  const out = rsi.batch(prices);
  for (let i = 14; i < out.length; i++) {
    assert.equal(out[i], 100);
  }
});

test('streaming and batch agree on EMA', () => {
  const prices = Array.from({ length: 60 }, (_, i) => 100 + Math.sin(i * 0.3) * 5);
  const batch = new wickra.EMA(14).batch(prices);
  const ema = new wickra.EMA(14);
  const streamed = prices.map((p) => {
    const v = ema.update(p);
    return v === null || v === undefined ? NaN : v;
  });
  for (let i = 0; i < prices.length; i++) {
    if (Number.isNaN(batch[i])) {
      assert.ok(Number.isNaN(streamed[i]));
    } else {
      assert.ok(Math.abs(batch[i] - streamed[i]) < 1e-9);
    }
  }
});

test('MACD returns macd/signal/histogram object', () => {
  const macd = new wickra.MACD(12, 26, 9);
  let value = null;
  for (let i = 1; i <= 60; i++) {
    value = macd.update(i);
  }
  assert.ok(value);
  assert.equal(typeof value.macd, 'number');
  assert.equal(typeof value.signal, 'number');
  assert.equal(typeof value.histogram, 'number');
  assert.ok(Math.abs(value.histogram - (value.macd - value.signal)) < 1e-9);
});

test('ATR batch shape', () => {
  const high = Array.from({ length: 30 }, () => 11);
  const low = Array.from({ length: 30 }, () => 9);
  const close = Array.from({ length: 30 }, () => 10);
  const out = new wickra.ATR(14).batch(high, low, close);
  assert.equal(out.length, 30);
  // Once seeded, ATR is the constant TR of 2.
  for (let i = 13; i < 30; i++) {
    assert.ok(Math.abs(out[i] - 2) < 1e-9);
  }
});

test('zero period is clamped to a valid window', () => {
  // Constructors cannot throw from JS (napi-rs 2.16 limitation), so they
  // clamp pathological values like period=0 to the smallest valid window.
  const sma = new wickra.SMA(0);
  assert.equal(sma.warmupPeriod(), 1);
  assert.equal(sma.update(42), 42);
});
