// Streaming-vs-batch equivalence and reference values for the Seasonality &
// Session family. These indicators consume the full candle (open, high, low,
// close, volume, timestamp), so they have a dedicated suite.

const test = require('node:test');
const assert = require('node:assert/strict');
const wickra = require('..');

const HOUR = 3_600_000;
const N = 240;
const close = Array.from({ length: N }, (_, i) => 100 + Math.sin(i * 0.3) * 5 + Math.cos(i * 0.1) * 3);
const open = close.map((c, i) => c + Math.sin(i * 0.5) * 0.5);
const high = close.map((c, i) => Math.max(open[i], c) + 1);
const low = close.map((c, i) => Math.min(open[i], c) - 1);
const volume = Array.from({ length: N }, (_, i) => 1000 + (i % 24) * 50);
const ts = Array.from({ length: N }, (_, i) => i * HOUR);

function eq(a, b) {
  if (Number.isNaN(a)) return Number.isNaN(b);
  return Math.abs(a - b) < 1e-9;
}

function streamScalar(ind, i) {
  const v = ind.update(open[i], high[i], low[i], close[i], volume[i], ts[i]);
  return v === null || v === undefined ? NaN : v;
}

function checkScalar(name, make) {
  test(`${name} streaming equals batch`, () => {
    const a = make();
    const b = make();
    const batch = b.batch(open, high, low, close, volume, ts);
    for (let i = 0; i < N; i += 1) {
      assert.ok(eq(streamScalar(a, i), batch[i]), `${name} row ${i}`);
    }
  });
}

function checkMatrix(name, make, k, pick) {
  test(`${name} streaming equals batch`, () => {
    const a = make();
    const b = make();
    const batch = b.batch(open, high, low, close, volume, ts);
    for (let i = 0; i < N; i += 1) {
      const out = a.update(open[i], high[i], low[i], close[i], volume[i], ts[i]);
      for (let j = 0; j < k; j += 1) {
        const s = out === null || out === undefined ? NaN : pick(out, j);
        assert.ok(eq(s, batch[i * k + j]), `${name} row ${i} col ${j}`);
      }
    }
  });
}

checkScalar('SessionVwap', () => new wickra.SessionVwap(0));
checkScalar('OvernightGap', () => new wickra.OvernightGap(0));
checkScalar('SeasonalZScore', () => new wickra.SeasonalZScore(0));
checkScalar('AverageDailyRange', () => new wickra.AverageDailyRange(3, 0));
checkScalar('TurnOfMonth', () => new wickra.TurnOfMonth(3, 1, 0));

checkMatrix('SessionHighLow', () => new wickra.SessionHighLow(0), 2, (o, j) => (j === 0 ? o.high : o.low));
checkMatrix('SessionRange', () => new wickra.SessionRange(0), 3, (o, j) => [o.asia, o.eu, o.us][j]);
checkMatrix(
  'OvernightIntradayReturn',
  () => new wickra.OvernightIntradayReturn(0),
  2,
  (o, j) => (j === 0 ? o.overnight : o.intraday),
);
checkMatrix('TimeOfDayReturnProfile', () => new wickra.TimeOfDayReturnProfile(24, 0), 24, (o, j) => o[j]);
checkMatrix('IntradayVolatilityProfile', () => new wickra.IntradayVolatilityProfile(12, 0), 12, (o, j) => o[j]);
checkMatrix('VolumeByTimeProfile', () => new wickra.VolumeByTimeProfile(24, 0), 24, (o, j) => o[j]);
checkMatrix('DayOfWeekProfile', () => new wickra.DayOfWeekProfile(0), 7, (o, j) => o[j]);

test('SessionVwap reference value', () => {
  const vwap = new wickra.SessionVwap(0);
  assert.ok(eq(vwap.update(100, 100, 100, 100, 10, 0), 100));
  assert.ok(eq(vwap.update(110, 110, 110, 110, 30, HOUR), 107.5));
  assert.ok(eq(vwap.update(200, 200, 200, 200, 5, 24 * HOUR), 200));
});

test('OvernightGap reference value', () => {
  const gap = new wickra.OvernightGap(0);
  assert.equal(gap.update(99, 101, 98, 100, 1, 0), null);
  assert.ok(eq(gap.update(105, 106, 104, 105.5, 1, 24 * HOUR), 0.05));
});

test('SessionHighLow reference object', () => {
  const shl = new wickra.SessionHighLow(0);
  shl.update(100, 105, 99, 101, 1, 0);
  const out = shl.update(101, 108, 100, 107, 1, HOUR);
  assert.ok(eq(out.high, 108));
  assert.ok(eq(out.low, 99));
});

test('AverageDailyRange rejects zero period', () => {
  assert.throws(() => new wickra.AverageDailyRange(0, 0));
});
