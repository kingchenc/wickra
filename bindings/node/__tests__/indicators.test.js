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
  ALMA: () => new wickra.ALMA(9, 0.85, 6.0),
  McGinleyDynamic: () => new wickra.McGinleyDynamic(10),
  FRAMA: () => new wickra.FRAMA(16),
  VIDYA: () => new wickra.VIDYA(14, 9),
  JMA: () => new wickra.JMA(14, 0, 2),
  SMMA: () => new wickra.SMMA(14),
  TRIMA: () => new wickra.TRIMA(20),
  ZLEMA: () => new wickra.ZLEMA(14),
  T3: () => new wickra.T3(5, 0.7),
  MOM: () => new wickra.MOM(10),
  CMO: () => new wickra.CMO(14),
  TSI: () => new wickra.TSI(25, 13),
  PMO: () => new wickra.PMO(35, 20),
  TII: () => new wickra.TII(20, 10),
  StochRSI: () => new wickra.StochRSI(14, 14),
  PPO: () => new wickra.PPO(12, 26),
  APO: () => new wickra.APO(12, 26),
  CFO: () => new wickra.CFO(14),
  ElderImpulse: () => new wickra.ElderImpulse(13, 12, 26, 9),
  STC: () => new wickra.STC(23, 50, 10, 0.5),
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
  PercentageTrailingStop: () => new wickra.PercentageTrailingStop(5),
  StepTrailingStop: () => new wickra.StepTrailingStop(1),
  RenkoTrailingStop: () => new wickra.RenkoTrailingStop(1),
  LaguerreRSI: () => new wickra.LaguerreRSI(0.5),
  ConnorsRSI: () => new wickra.ConnorsRSI(3, 2, 100),
  RVIVolatility: () => new wickra.RVIVolatility(10),
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
  RVI: { make: () => new wickra.RVI(10), step: (ind, i) => ind.update(open[i], high[i], low[i], close[i]), batch: (ind) => ind.batch(open, high, low, close) },
  Inertia: { make: () => new wickra.Inertia(14, 20), step: (ind, i) => ind.update(open[i], high[i], low[i], close[i]), batch: (ind) => ind.batch(open, high, low, close) },
  PGO: { make: () => new wickra.PGO(14), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  SMI: { make: () => new wickra.SMI(5, 3, 3), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  EVWMA: { make: () => new wickra.EVWMA(20), step: (ind, i) => ind.update(close[i], volume[i]), batch: (ind) => ind.batch(close, volume) },
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
  WilliamsAD: { make: () => new wickra.WilliamsAD(), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  AnchoredVWAP: { make: () => new wickra.AnchoredVWAP(), step: (ind, i) => ind.update(high[i], low[i], close[i], volume[i]), batch: (ind) => ind.batch(high, low, close, volume) },
  DemandIndex: { make: () => new wickra.DemandIndex(10), step: (ind, i) => ind.update(high[i], low[i], close[i], volume[i]), batch: (ind) => ind.batch(high, low, close, volume) },
  TSV: { make: () => new wickra.TSV(18), step: (ind, i) => ind.update(close[i], volume[i]), batch: (ind) => ind.batch(close, volume) },
  VZO: { make: () => new wickra.VZO(14), step: (ind, i) => ind.update(close[i], volume[i]), batch: (ind) => ind.batch(close, volume) },
  MarketFacilitationIndex: { make: () => new wickra.MarketFacilitationIndex(), step: (ind, i) => ind.update(high[i], low[i], volume[i]), batch: (ind) => ind.batch(high, low, volume) },
  AtrTrailingStop: { make: () => new wickra.AtrTrailingStop(14, 3), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  HiLoActivator: { make: () => new wickra.HiLoActivator(3), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  VoltyStop: { make: () => new wickra.VoltyStop(14, 2), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  YoyoExit: { make: () => new wickra.YoyoExit(14, 2), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  TypicalPrice: { make: () => new wickra.TypicalPrice(), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  MedianPrice: { make: () => new wickra.MedianPrice(), step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  WeightedClose: { make: () => new wickra.WeightedClose(), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  AcceleratorOscillator: { make: () => new wickra.AcceleratorOscillator(5, 34, 5), step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  AwesomeOscillatorHistogram: { make: () => new wickra.AwesomeOscillatorHistogram(5, 34, 5), step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  BalanceOfPower: { make: () => new wickra.BalanceOfPower(), step: (ind, i) => ind.update(open[i], high[i], low[i], close[i]), batch: (ind) => ind.batch(open, high, low, close) },
  ChoppinessIndex: { make: () => new wickra.ChoppinessIndex(14), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  TrueRange: { make: () => new wickra.TrueRange(), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  ChaikinVolatility: { make: () => new wickra.ChaikinVolatility(10, 10), step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  ADXR: { make: () => new wickra.ADXR(7), step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  ParkinsonVolatility: { make: () => new wickra.ParkinsonVolatility(20, 252), step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  GarmanKlassVolatility: { make: () => new wickra.GarmanKlassVolatility(20, 252), step: (ind, i) => ind.update(open[i], high[i], low[i], close[i]), batch: (ind) => ind.batch(open, high, low, close) },
  RogersSatchellVolatility: { make: () => new wickra.RogersSatchellVolatility(20, 252), step: (ind, i) => ind.update(open[i], high[i], low[i], close[i]), batch: (ind) => ind.batch(open, high, low, close) },
  YangZhangVolatility: { make: () => new wickra.YangZhangVolatility(20, 252), step: (ind, i) => ind.update(open[i], high[i], low[i], close[i]), batch: (ind) => ind.batch(open, high, low, close) },
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
  KST: { make: () => new wickra.KST(10, 15, 20, 30, 10, 10, 10, 15, 9), fields: ['kst', 'signal'], step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  Alligator: { make: () => new wickra.Alligator(13, 8, 5), fields: ['jaw', 'teeth', 'lips'], step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  ZeroLagMACD: { make: () => new wickra.ZeroLagMACD(12, 26, 9), fields: ['macd', 'signal', 'histogram'], step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  MACD: { make: () => new wickra.MACD(12, 26, 9), fields: ['macd', 'signal', 'histogram'], step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  KST: { make: () => wickra.KST.classic(), fields: ['kst', 'signal'], step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  BollingerBands: { make: () => new wickra.BollingerBands(20, 2), fields: ['upper', 'middle', 'lower', 'stddev'], step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  Stochastic: { make: () => new wickra.Stochastic(14, 3), fields: ['k', 'd'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  ADX: { make: () => new wickra.ADX(14), fields: ['plusDi', 'minusDi', 'adx'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  Keltner: { make: () => new wickra.Keltner(20, 10, 2), fields: ['upper', 'middle', 'lower'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  Donchian: { make: () => new wickra.Donchian(20), fields: ['upper', 'middle', 'lower'], step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  Aroon: { make: () => new wickra.Aroon(14), fields: ['up', 'down'], step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  Vortex: { make: () => new wickra.Vortex(14), fields: ['plus', 'minus'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  RWI: { make: () => new wickra.RWI(14), fields: ['high', 'low'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  WaveTrend: { make: () => wickra.WaveTrend.classic(), fields: ['wt1', 'wt2'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  SuperTrend: { make: () => new wickra.SuperTrend(10, 3), fields: ['value', 'direction'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  ChandelierExit: { make: () => new wickra.ChandelierExit(22, 3), fields: ['longStop', 'shortStop'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  ChandeKrollStop: { make: () => new wickra.ChandeKrollStop(10, 1, 9), fields: ['stopLong', 'stopShort'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  DonchianStop: { make: () => new wickra.DonchianStop(10), fields: ['stopLong', 'stopShort'], step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  // Family 05: bands & channels
  MaEnvelope: { make: () => new wickra.MaEnvelope(20, 0.025), fields: ['upper', 'middle', 'lower'], step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  AccelerationBands: { make: () => new wickra.AccelerationBands(20, 0.001), fields: ['upper', 'middle', 'lower'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  StarcBands: { make: () => new wickra.StarcBands(6, 15, 2), fields: ['upper', 'middle', 'lower'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  AtrBands: { make: () => new wickra.AtrBands(14, 3), fields: ['upper', 'middle', 'lower'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  HurstChannel: { make: () => new wickra.HurstChannel(10, 0.5), fields: ['upper', 'middle', 'lower'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  LinRegChannel: { make: () => new wickra.LinRegChannel(20, 2), fields: ['upper', 'middle', 'lower'], step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  StandardErrorBands: { make: () => new wickra.StandardErrorBands(21, 2), fields: ['upper', 'middle', 'lower'], step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  DoubleBollinger: { make: () => new wickra.DoubleBollinger(20, 1, 2), fields: ['upperOuter', 'upperInner', 'middle', 'lowerInner', 'lowerOuter'], step: (ind, i) => ind.update(close[i]), batch: (ind) => ind.batch(close) },
  TtmSqueeze: { make: () => new wickra.TtmSqueeze(20, 2, 1.5), fields: ['squeeze', 'momentum'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  FractalChaosBands: { make: () => new wickra.FractalChaosBands(2), fields: ['upper', 'lower'], step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  VwapStdDevBands: { make: () => new wickra.VwapStdDevBands(2), fields: ['upper', 'middle', 'lower', 'stddev'], step: (ind, i) => ind.update(high[i], low[i], close[i], volume[i]), batch: (ind) => ind.batch(high, low, close, volume) },
  // Family 08: Pivots & Support/Resistance
  ClassicPivots: { make: () => new wickra.ClassicPivots(), fields: ['pp', 'r1', 'r2', 'r3', 's1', 's2', 's3'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  FibonacciPivots: { make: () => new wickra.FibonacciPivots(), fields: ['pp', 'r1', 'r2', 'r3', 's1', 's2', 's3'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  Camarilla: { make: () => new wickra.Camarilla(), fields: ['pp', 'r1', 'r2', 'r3', 'r4', 's1', 's2', 's3', 's4'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  WoodiePivots: { make: () => new wickra.WoodiePivots(), fields: ['pp', 'r1', 'r2', 's1', 's2'], step: (ind, i) => ind.update(high[i], low[i], close[i]), batch: (ind) => ind.batch(high, low, close) },
  DemarkPivots: { make: () => new wickra.DemarkPivots(), fields: ['pp', 'r1', 's1'], step: (ind, i) => ind.update(open[i], high[i], low[i], close[i]), batch: (ind) => ind.batch(open, high, low, close) },
  WilliamsFractals: { make: () => new wickra.WilliamsFractals(), fields: ['up', 'down'], step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
  ZigZag: { make: () => new wickra.ZigZag(0.02), fields: ['swing', 'direction'], step: (ind, i) => ind.update(high[i], low[i]), batch: (ind) => ind.batch(high, low) },
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

test('PercentageTrailingStop seeds and ratchets', () => {
  const s = new wickra.PercentageTrailingStop(10);
  assert.ok(Math.abs(s.update(100) - 90) < 1e-9);
  assert.ok(Math.abs(s.update(110) - 99) < 1e-9);
});

test('RenkoTrailingStop only advances after a full block', () => {
  const s = new wickra.RenkoTrailingStop(1);
  assert.ok(Math.abs(s.update(100) - 99) < 1e-9);
  assert.ok(Math.abs(s.update(100.5) - 99) < 1e-9);
  assert.ok(Math.abs(s.update(101) - 100) < 1e-9);
});

test('DonchianStop window extremes', () => {
  const out = new wickra.DonchianStop(5).batch([1, 2, 3, 4, 5], [0, 1, 2, 3, 4]);
  // [long0, short0, long1, short1, ...]: idx 8 (=4*2) = long_5th, idx 9 = short_5th.
  assert.ok(Math.abs(out[8] - 0) < 1e-9);
  assert.ok(Math.abs(out[9] - 5) < 1e-9);
});

test('MaEnvelope reference values', () => {
  // SMA([10, 20, 30]) = 20; with percent 0.10: upper=22, lower=18.
  const out = new wickra.MaEnvelope(3, 0.10).batch([10, 20, 30]);
  assert.ok(Number.isNaN(out[0]) && Number.isNaN(out[3]));
  assert.ok(Math.abs(out[2 * 3 + 0] - 22) < 1e-9); // upper
  assert.ok(Math.abs(out[2 * 3 + 1] - 20) < 1e-9); // middle
  assert.ok(Math.abs(out[2 * 3 + 2] - 18) < 1e-9); // lower
});

test('AccelerationBands single-bar reference', () => {
  // high=12, low=8, close=10, factor=0.5, period=1.
  // ratio=0.2, raw_up=13.2, raw_lo=7.2.
  const v = new wickra.AccelerationBands(1, 0.5).update(12, 8, 10);
  assert.ok(Math.abs(v.upper - 13.2) < 1e-9);
  assert.ok(Math.abs(v.middle - 10) < 1e-9);
  assert.ok(Math.abs(v.lower - 7.2) < 1e-9);
});

test('LinRegChannel reference values for [1, 2, 9]', () => {
  // Line y=4x, endpoint=8, residuals=[1,-2,1], sigma=sqrt(2).
  const out = new wickra.LinRegChannel(3, 2).batch([1, 2, 9]);
  const s = Math.sqrt(2);
  const i = 2;
  assert.ok(Math.abs(out[i * 3 + 0] - (8 + 2 * s)) < 1e-9);
  assert.ok(Math.abs(out[i * 3 + 1] - 8) < 1e-9);
  assert.ok(Math.abs(out[i * 3 + 2] - (8 - 2 * s)) < 1e-9);
});

test('VwapStdDevBands two-bar reference', () => {
  const v = new wickra.VwapStdDevBands(1.5);
  v.update(8, 8, 8, 1);
  const o = v.update(12, 12, 12, 1);
  assert.ok(Math.abs(o.upper - 13) < 1e-9);
  assert.ok(Math.abs(o.middle - 10) < 1e-9);
  assert.ok(Math.abs(o.lower - 7) < 1e-9);
  assert.ok(Math.abs(o.stddev - 2) < 1e-9);
});

test('RVIVolatility pure uptrend saturates at 100', () => {
  const prices = Array.from({ length: 40 }, (_, i) => i + 1);
  const out = new wickra.RVIVolatility(5).batch(prices);
  for (let i = 9; i < out.length; i++) {
    assert.ok(Math.abs(out[i] - 100) < 1e-9, `RVIVolatility[${i}] = ${out[i]}`);
  }
});

test('ParkinsonVolatility zero-range bars yield zero', () => {
  const n = 30;
  const h = Array(n).fill(10);
  const l = Array(n).fill(10);
  const out = new wickra.ParkinsonVolatility(14, 252).batch(h, l);
  for (let i = 13; i < n; i++) {
    assert.ok(Math.abs(out[i]) < 1e-12, `Parkinson[${i}] = ${out[i]}`);
  }
});

test('GarmanKlassVolatility zero-movement bars yield zero', () => {
  const n = 30;
  const flat = Array(n).fill(10);
  const out = new wickra.GarmanKlassVolatility(14, 252).batch(flat, flat, flat, flat);
  for (let i = 13; i < n; i++) {
    assert.ok(Math.abs(out[i]) < 1e-12, `GK[${i}] = ${out[i]}`);
  }
});

test('RogersSatchellVolatility zero-movement bars yield zero', () => {
  const n = 30;
  const flat = Array(n).fill(10);
  const out = new wickra.RogersSatchellVolatility(14, 252).batch(flat, flat, flat, flat);
  for (let i = 13; i < n; i++) {
    assert.ok(Math.abs(out[i]) < 1e-12, `RS[${i}] = ${out[i]}`);
  }
});

test('YangZhangVolatility zero-movement bars yield zero', () => {
  const n = 30;
  const flat = Array(n).fill(10);
  const out = new wickra.YangZhangVolatility(14, 252).batch(flat, flat, flat, flat);
  for (let i = 14; i < n; i++) {
    assert.ok(Math.abs(out[i]) < 1e-12, `YZ[${i}] = ${out[i]}`);
  }
});

test('ZeroLagMACD on a flat series converges to zero', () => {
  const out = new wickra.ZeroLagMACD(3, 5, 3).batch(Array(60).fill(42));
  // Last interleaved row: macd, signal, histogram all 0.
  const n = 60;
  assert.ok(Math.abs(out[(n - 1) * 3]) < 1e-12);
  assert.ok(Math.abs(out[(n - 1) * 3 + 1]) < 1e-12);
  assert.ok(Math.abs(out[(n - 1) * 3 + 2]) < 1e-12);
});

test('AwesomeOscillatorHistogram on a flat median converges to zero', () => {
  const n = 50;
  const out = new wickra.AwesomeOscillatorHistogram(3, 5, 3).batch(
    Array(n).fill(11),
    Array(n).fill(9),
  );
  // warmup = 5 + 3 - 1 = 7.
  for (let i = 6; i < n; i++) assert.ok(Math.abs(out[i]) < 1e-12);
});

test('STC on a flat series stays at zero', () => {
  const out = new wickra.STC(3, 5, 4, 0.5).batch(Array(60).fill(42));
  // Latest values must be exactly zero.
  for (let i = out.length - 5; i < out.length; i++) {
    if (Number.isNaN(out[i])) continue;
    assert.equal(out[i], 0);
  }
});

test('ElderImpulse on a flat series stays neutral (0)', () => {
  const out = new wickra.ElderImpulse(13, 12, 26, 9).batch(Array(120).fill(42));
  for (let i = 0; i < out.length; i++) {
    if (Number.isNaN(out[i])) continue;
    assert.equal(out[i], 0);
  }
});

test('CFO(5) on a perfectly linear series yields zero', () => {
  const prices = Array.from({ length: 20 }, (_, i) => (i + 1) * 2);
  const out = new wickra.CFO(5).batch(prices);
  for (let i = 4; i < 20; i++) assert.ok(Math.abs(out[i]) < 1e-9);
});

test('APO(3, 5) on a flat series converges to zero', () => {
  const out = new wickra.APO(3, 5).batch(Array(30).fill(42));
  for (let i = 0; i < 4; i++) assert.ok(Number.isNaN(out[i]));
  for (let i = 4; i < 30; i++) assert.ok(Math.abs(out[i]) < 1e-12);
});

test('Inertia(3, 4) on a constant RVI series equals that RVI', () => {
  const n = 60;
  // Every bar (open, high, low, close) = (10, 11, 9, 10.5) -> RVI = 0.25.
  const out = new wickra.Inertia(3, 4).batch(
    Array(n).fill(10),
    Array(n).fill(11),
    Array(n).fill(9),
    Array(n).fill(10.5),
  );
  for (let i = 5; i < n; i++) assert.ok(Math.abs(out[i] - 0.25) < 1e-12);
});

test('ConnorsRSI stays bounded in [0, 100]', () => {
  const prices = Array.from({ length: 250 }, (_, i) => 100 + 20 * Math.sin(i * 0.12));
  const out = new wickra.ConnorsRSI(3, 2, 100).batch(prices);
  for (let i = 0; i < out.length; i++) {
    if (Number.isNaN(out[i])) continue;
    assert.ok(out[i] >= 0 && out[i] <= 100, `out[${i}] = ${out[i]}`);
  }
});

test('LaguerreRSI on a flat series stays at the neutral 50', () => {
  const out = new wickra.LaguerreRSI(0.5).batch(Array(40).fill(42));
  for (let i = 0; i < out.length; i++) assert.ok(Math.abs(out[i] - 50) < 1e-12);
});

test('SMI with close at range centre emits zero after warmup', () => {
  const n = 60;
  const out = new wickra.SMI(5, 3, 3).batch(Array(n).fill(11), Array(n).fill(9), Array(n).fill(10));
  // warmup_period = 5 + 3 + 3 - 2 = 9.
  for (let i = 8; i < n; i++) assert.ok(Math.abs(out[i]) < 1e-12);
});

test('KST on a flat series emits zero after warmup', () => {
  const kst = new wickra.KST(10, 15, 20, 30, 10, 10, 10, 15, 9);
  const n = 80;
  const out = kst.batch(Array(n).fill(42));
  const warmup = kst.warmupPeriod();
  for (let i = warmup - 1; i < n; i++) {
    assert.ok(Math.abs(out[i * 2]) < 1e-12, `kst[${i}] = ${out[i * 2]}`);
    assert.ok(Math.abs(out[i * 2 + 1]) < 1e-12, `signal[${i}] = ${out[i * 2 + 1]}`);
  }
});

test('PGO(5) on a flat close emits zero after warmup', () => {
  const n = 20;
  const out = new wickra.PGO(5).batch(Array(n).fill(11), Array(n).fill(9), Array(n).fill(10));
  for (let i = 0; i < 4; i++) assert.ok(Number.isNaN(out[i]));
  for (let i = 4; i < n; i++) assert.ok(Math.abs(out[i]) < 1e-12, `out[${i}] = ${out[i]}`);
});

test('RVI(2) reference value on two bars', () => {
  // Bars (open, high, low, close): (10, 11, 9, 10.5), (10.5, 11.5, 10, 11).
  const out = new wickra.RVI(2).batch([10, 10.5], [11, 11.5], [9, 10], [10.5, 11]);
  assert.ok(Number.isNaN(out[0]));
  assert.ok(Math.abs(out[1] - 1 / 3.5) < 1e-12);
});

test('EVWMA(2) reference values on [10, 20, 30] with volumes [1, 3, 1]', () => {
  const out = new wickra.EVWMA(2).batch([10, 20, 30], [1, 3, 1]);
  assert.ok(Number.isNaN(out[0]));
  assert.ok(Math.abs(out[1] - 20) < 1e-12);
  assert.ok(Math.abs(out[2] - 22.5) < 1e-12);
});

test('Alligator on a flat median price seeds to that median', () => {
  const n = 30;
  const out = new wickra.Alligator(13, 8, 5).batch(Array(n).fill(11), Array(n).fill(9));
  // All three SMMAs see median (11 + 9) / 2 = 10 every bar.
  for (let i = 12; i < n; i++) {
    assert.ok(Math.abs(out[i * 3] - 10) < 1e-12, `jaw at ${i}: ${out[i * 3]}`);
    assert.ok(Math.abs(out[i * 3 + 1] - 10) < 1e-12);
    assert.ok(Math.abs(out[i * 3 + 2] - 10) < 1e-12);
  }
});

test('JMA on a flat series reproduces the constant', () => {
  const out = new wickra.JMA(14, 0, 2).batch(Array(30).fill(42));
  for (let i = 0; i < 30; i++) assert.ok(Math.abs(out[i] - 42) < 1e-12);
});

test('VIDYA on a flat series holds the seed', () => {
  const out = new wickra.VIDYA(14, 4).batch(Array(20).fill(42));
  for (let i = 0; i < 4; i++) assert.ok(Number.isNaN(out[i]));
  for (let i = 4; i < 20; i++) assert.ok(Math.abs(out[i] - 42) < 1e-12);
});

test('FRAMA pure uptrend hugs the latest close', () => {
  const out = new wickra.FRAMA(4).batch([1, 2, 3, 4, 5, 6, 7, 8]);
  assert.ok(Math.abs(out[out.length - 1] - 8) < 0.05);
});

test('McGinleyDynamic(3) seeds with SMA and recurses on the next price', () => {
  // Seed = SMA([10, 20, 30]) = 20. On 40: ratio = 2, divisor = 0.6*3*16 = 28.8.
  const out = new wickra.McGinleyDynamic(3).batch([10, 20, 30, 40]);
  assert.ok(Number.isNaN(out[0]) && Number.isNaN(out[1]));
  assert.ok(Math.abs(out[2] - 20) < 1e-12);
  const expected = 20 + 20 / (0.6 * 3 * 16);
  assert.ok(Math.abs(out[3] - expected) < 1e-12);
});

test('ALMA(3, 0.85, 6) reference value on [10, 20, 30]', () => {
  // m = 0.85 * 2 = 1.7; s = 3 / 6 = 0.5; 2*s^2 = 0.5.
  const out = new wickra.ALMA(3, 0.85, 6).batch([10, 20, 30]);
  assert.ok(Number.isNaN(out[0]) && Number.isNaN(out[1]));
  const w = [0, 1, 2].map((i) => Math.exp(-Math.pow(i - 1.7, 2) / 0.5));
  const s = w[0] + w[1] + w[2];
  const expected = (10 * w[0] + 20 * w[1] + 30 * w[2]) / s;
  assert.ok(Math.abs(out[2] - expected) < 1e-12);
  // The heavy offset toward the newest sample lifts the average above the
  // simple mean of 20.
  assert.ok(out[2] > 20);
});
