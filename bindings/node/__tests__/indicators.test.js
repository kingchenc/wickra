// Comprehensive tests for the Wickra Node bindings: streaming-vs-batch
// equivalence, reference values, and lifecycle methods across all 71
// indicators. Ported from the Python test_streaming_vs_batch / test_known_values
// suites.

const test = require('node:test');
const assert = require('node:assert/strict');
const wickra = require('..');

// Synthetic OHLCV series long enough to warm up every indicator.
const N = 120;
const close = Array.from({ length: N }, (_, i) => 100 + Math.sin(i * 0.2) * 10 + i * 0.1);
const high = close.map((c) => c + 1.5);
const low = close.map((c) => c - 1.5);
const volume = Array.from({ length: N }, (_, i) => 1000 + (i % 7) * 50);
const open = close.map((c) => c - 0.5);

function eq(a, b) {
  if (Number.isNaN(a)) return Number.isNaN(b);
  return Math.abs(a - b) < 1e-9;
}

function num(v) {
  return v === null || v === undefined ? NaN : v;
}

// --- Scalar indicators: update(value) vs batch(prices) ---

const scalarFactories = {
  SMA: () => new wickra.SMA(14),
  EMA: () => new wickra.EMA(14),
  WMA: () => new wickra.WMA(14),
  RSI: () => new wickra.RSI(14),
  DEMA: () => new wickra.DEMA(10),
  TEMA: () => new wickra.TEMA(10),
  HMA: () => new wickra.HMA(9),
  ROC: () => new wickra.ROC(12),
  TRIX: () => new wickra.TRIX(9),
  KAMA: () => new wickra.KAMA(10, 2, 30),
  SMMA: () => new wickra.SMMA(14),
  TRIMA: () => new wickra.TRIMA(20),
  ZLEMA: () => new wickra.ZLEMA(14),
  T3: () => new wickra.T3(5, 0.7),
  MOM: () => new wickra.MOM(10),
  CMO: () => new wickra.CMO(14),
  TSI: () => new wickra.TSI(25, 13),
  PMO: () => new wickra.PMO(35, 20),
  StochRSI: () => new wickra.StochRSI(14, 14),
  PPO: () => new wickra.PPO(12, 26),
  DPO: () => new wickra.DPO(20),
  Coppock: () => new wickra.Coppock(14, 11, 10),
  StdDev: () => new wickra.StdDev(20),
  UlcerIndex: () => new wickra.UlcerIndex(14),
  HistoricalVolatility: () => new wickra.HistoricalVolatility(20, 252),
  BollingerBandwidth: () => new wickra.BollingerBandwidth(20, 2),
  PercentB: () => new wickra.PercentB(20, 2),
  LinearRegression: () => new wickra.LinearRegression(14),
  LinRegSlope: () => new wickra.LinRegSlope(14),
  VerticalHorizontalFilter: () => new wickra.VerticalHorizontalFilter(28),
  ZScore: () => new wickra.ZScore(20),
  LinRegAngle: () => new wickra.LinRegAngle(14),
};

for (const [name, make] of Object.entries(scalarFactories)) {
  test(`${name}: streaming update matches batch`, () => {
    const batch = make().batch(close);
    const streaming = make();
    assert.equal(batch.length, N);
    for (let i = 0; i < N; i++) {
      const s = num(streaming.update(close[i]));
      assert.ok(eq(s, batch[i]), `${name} mismatch at ${i}: ${s} vs ${batch[i]}`);
    }
  });
}

// --- Scalar-output candle indicators: update(...) vs batch(...) ---

const candleScalar = {
  ATR: { make: () => new wickra.ATR(14), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  CCI: { make: () => new wickra.CCI(20), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  WilliamsR: { make: () => new wickra.WilliamsR(14), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  PSAR: { make: () => new wickra.PSAR(0.02, 0.02, 0.2), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  MFI: { make: () => new wickra.MFI(14), step: (ind, i) => ind.update(high[i], low[i], close[i], volume[i]), batch: (ind) => ind.batch(high, low, close, volume) },
  VWAP: { make: () => new wickra.VWAP(), step: (ind, i) => ind.update(high[i], low[i], close[i], volume[i]), batch: (ind) => ind.batch(high, low, close, volume) },
  RollingVWAP: { make: () => new wickra.RollingVWAP(20), step: (ind, i) => ind.update(high[i], low[i], close[i], volume[i]), batch: (ind) => ind.batch(high, low, close, volume) },
  AwesomeOscillator: { make: () => new wickra.AwesomeOscillator(5, 34), step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  OBV: { make: () => new wickra.OBV(), step: (ind, i) => ind.update(close[i], volume[i]), batch: (ind) => ind.batch(close, volume) },
  VWMA: { make: () => new wickra.VWMA(20), step: (ind, i) => ind.update(close[i], volume[i]), batch: (ind) => ind.batch(close, volume) },
  UltimateOscillator: { make: () => new wickra.UltimateOscillator(7, 14, 28), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  AroonOscillator: { make: () => new wickra.AroonOscillator(14), step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  NATR: { make: () => new wickra.NATR(14), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  MassIndex: { make: () => new wickra.MassIndex(9, 25), step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  ADL: { make: () => new wickra.ADL(), step: (ind, i) => ind.update(high[i], low[i], close[i], volume[i]), batch: (ind) => ind.batch(high, low, close, volume) },
  VolumePriceTrend: { make: () => new wickra.VolumePriceTrend(), step: (ind, i) => ind.update(close[i], volume[i]), batch: (ind) => ind.batch(close, volume) },
  ChaikinMoneyFlow: { make: () => new wickra.ChaikinMoneyFlow(20), step: (ind, i) => ind.update(high[i], low[i], close[i], volume[i]), batch: (ind) => ind.batch(high, low, close, volume) },
  ChaikinOscillator: { make: () => new wickra.ChaikinOscillator(3, 10), step: (ind, i) => ind.update(high[i], low[i], close[i], volume[i]), batch: (ind) => ind.batch(high, low, close, volume) },
  ForceIndex: { make: () => new wickra.ForceIndex(13), step: (ind, i) => ind.update(close[i], volume[i]), batch: (ind) => ind.batch(close, volume) },
  EaseOfMovement: { make: () => new wickra.EaseOfMovement(14, 1e8), step: (ind, i) => ind.update(high[i], low[i], volume[i]), batch: (ind) => ind.batch(high, low, volume) },
  KVO: { make: () => new wickra.KVO(34, 55), step: (ind, i) => ind.update(high[i], low[i], close[i], volume[i]), batch: (ind) => ind.batch(high, low, close, volume) },
  VolumeOscillator: { make: () => new wickra.VolumeOscillator(14, 28), step: (ind, i) => ind.update(volume[i]), batch: (ind) => ind.batch(volume) },
  NVI: { make: () => new wickra.NVI(), step: (ind, i) => ind.update(close[i], volume[i]), batch: (ind) => ind.batch(close, volume) },
  PVI: { make: () => new wickra.PVI(), step: (ind, i) => ind.update(close[i], volume[i]), batch: (ind) => ind.batch(close, volume) },
  AtrTrailingStop: { make: () => new wickra.AtrTrailingStop(14, 3), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  TypicalPrice: { make: () => new wickra.TypicalPrice(), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  MedianPrice: { make: () => new wickra.MedianPrice(), step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  WeightedClose: { make: () => new wickra.WeightedClose(), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  AcceleratorOscillator: { make: () => new wickra.AcceleratorOscillator(5, 34, 5), step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  BalanceOfPower: { make: () => new wickra.BalanceOfPower(), step: (ind, i) => ind.update(open[i], high[i], low[i], close[i]), batch: (ind) => ind.batch(open, high, low, close) },
  ChoppinessIndex: { make: () => new wickra.ChoppinessIndex(14), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  TrueRange: { make: () => new wickra.TrueRange(), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  ChaikinVolatility: { make: () => new wickra.ChaikinVolatility(10, 10), step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
};

for (const [name, d] of Object.entries(candleScalar)) {
  test(`${name}: streaming update matches batch`, () => {
    const batch = d.batch(d.make());
    const streaming = d.make();
    assert.equal(batch.length, N);
    for (let i = 0; i < N; i++) {
      const s = num(d.step(streaming, i));
      assert.ok(eq(s, batch[i]), `${name} mismatch at ${i}: ${s} vs ${batch[i]}`);
    }
  });
}

// --- Multi-output indicators: object update vs interleaved batch ---

const multi = {
  MACD: { make: () => new wickra.MACD(12, 26, 9), fields: ['macd', 'signal', 'histogram'], step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  BollingerBands: { make: () => new wickra.BollingerBands(20, 2), fields: ['upper', 'middle', 'lower', 'stddev'], step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  Stochastic: { make: () => new wickra.Stochastic(14, 3), fields: ['k', 'd'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  ADX: { make: () => new wickra.ADX(14), fields: ['plusDi', 'minusDi', 'adx'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  Keltner: { make: () => new wickra.Keltner(20, 10, 2), fields: ['upper', 'middle', 'lower'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  Donchian: { make: () => new wickra.Donchian(20), fields: ['upper', 'middle', 'lower'], step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  Aroon: { make: () => new wickra.Aroon(14), fields: ['up', 'down'], step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  Vortex: { make: () => new wickra.Vortex(14), fields: ['plus', 'minus'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  SuperTrend: { make: () => new wickra.SuperTrend(10, 3), fields: ['value', 'direction'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  ChandelierExit: { make: () => new wickra.ChandelierExit(22, 3), fields: ['longStop', 'shortStop'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  ChandeKrollStop: { make: () => new wickra.ChandeKrollStop(10, 1, 9), fields: ['stopLong', 'stopShort'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
};

for (const [name, d] of Object.entries(multi)) {
  test(`${name}: streaming update matches interleaved batch`, () => {
    const k = d.fields.length;
    const batch = d.batch(d.make());
    const streaming = d.make();
    assert.equal(batch.length, N * k);
    for (let i = 0; i < N; i++) {
      const o = d.step(streaming, i);
      d.fields.forEach((field, j) => {
        const s = o === null || o === undefined ? NaN : o[field];
        assert.ok(eq(s, batch[i * k + j]), `${name}.${field} mismatch at ${i}`);
      });
    }
  });
}

// --- Lifecycle: every indicator exposes reset / isReady / warmupPeriod ---

test('every indicator exposes reset, isReady and warmupPeriod', () => {
  const all = [
    ...Object.values(scalarFactories).map((f) => f()),
    ...Object.values(candleScalar).map((d) => d.make()),
    ...Object.values(multi).map((d) => d.make()),
  ];
  for (const ind of all) {
    assert.equal(typeof ind.reset, 'function');
    assert.equal(typeof ind.isReady, 'function');
    assert.equal(typeof ind.warmupPeriod, 'function');
    assert.equal(ind.isReady(), false);
    assert.ok(ind.warmupPeriod() >= 1);
  }
});

test('reset returns an indicator to its un-warmed state', () => {
  const sma = new wickra.SMA(5);
  sma.batch([1, 2, 3, 4, 5]);
  assert.equal(sma.isReady(), true);
  sma.reset();
  assert.equal(sma.isReady(), false);
  assert.equal(sma.update(10), null);
});

// --- Reference values ---

test('SMA(3) reference values', () => {
  const out = new wickra.SMA(3).batch([2, 4, 6, 8, 10]);
  assert.ok(Number.isNaN(out[0]) && Number.isNaN(out[1]));
  assert.equal(out[2], 4);
  assert.equal(out[3], 6);
  assert.equal(out[4], 8);
});

test('MFI(2) reference value equals 1200/23', () => {
  // Candle 1 seeds; candle 2 (tp 12 > 10) +mf 1200; candle 3 (tp 11 < 12) -mf 1100.
  const mfi = new wickra.MFI(2);
  assert.equal(mfi.update(10, 10, 10, 100), null);
  assert.equal(mfi.update(12, 12, 12, 100), null);
  const v = mfi.update(11, 11, 11, 100);
  assert.ok(Math.abs(v - 1200 / 23) < 1e-9);
});

test('RSI pure uptrend yields 100', () => {
  const prices = Array.from({ length: 20 }, (_, i) => i + 1);
  const out = new wickra.RSI(14).batch(prices);
  for (let i = 14; i < out.length; i++) {
    assert.equal(out[i], 100);
  }
});

test('MACD histogram equals macd minus signal', () => {
  const macd = new wickra.MACD(12, 26, 9);
  let v = null;
  for (let i = 1; i <= 60; i++) v = macd.update(i);
  assert.ok(v);
  assert.ok(Math.abs(v.histogram - (v.macd - v.signal)) < 1e-9);
});

test('TypicalPrice reference value', () => {
  // (high + low + close) / 3 = (12 + 6 + 9) / 3 = 9.
  assert.equal(new wickra.TypicalPrice().update(12, 6, 9), 9);
});

test('ChaikinMoneyFlow(2) reference value equals 0.5', () => {
  // Bar 1 closes at the high (MFV +100); bar 2 closes mid-range (MFV 0).
  const cmf = new wickra.ChaikinMoneyFlow(2);
  assert.equal(cmf.update(10, 8, 10, 100), null);
  assert.ok(Math.abs(cmf.update(12, 8, 10, 100) - 0.5) < 1e-9);
});

test('LinearRegression(3) reference values', () => {
  // Least-squares line through [1, 2, 9] is y = 4x; endpoint 4·2 = 8.
  const out = new wickra.LinearRegression(3).batch([1, 2, 9]);
  assert.ok(Number.isNaN(out[0]) && Number.isNaN(out[1]));
  assert.ok(Math.abs(out[2] - 8) < 1e-9);
});

test('SuperTrend flat market holds the lower band and an uptrend', () => {
  // Flat candles: ATR 2, hl2 10, lower band 10 - 3·2 = 4.
  const n = 20;
  const out = new wickra.SuperTrend(5, 3).batch(
    Array(n).fill(11),
    Array(n).fill(9),
    Array(n).fill(10),
  );
  assert.ok(Math.abs(out[2 * n - 2] - 4) < 1e-9); // value
  assert.equal(out[2 * n - 1], 1); // direction
});

test('BalanceOfPower reference value', () => {
  // (close - open) / (high - low) = (12 - 10) / (14 - 10) = 0.5.
  assert.ok(Math.abs(new wickra.BalanceOfPower().update(10, 14, 10, 12) - 0.5) < 1e-9);
});

test('TrueRange reference values', () => {
  const tr = new wickra.TrueRange();
  assert.equal(tr.update(12, 8, 11), 4); // no prev close -> high - low
  assert.equal(tr.update(10, 9, 9.5), 2); // prev close 11 -> max(1, 1, 2)
});

test('LinRegAngle of a unit-slope series is 45 degrees', () => {
  const out = new wickra.LinRegAngle(5).batch([1, 2, 3, 4, 5, 6]);
  assert.ok(Math.abs(out[4] - 45) < 1e-9);
});
