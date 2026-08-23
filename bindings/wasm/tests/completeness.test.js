// Completeness contract for the WASM bindings, mirroring the Node one.
//
// The macro-generated wrappers always carried the full interface; the classes
// written out by hand drifted, and 73 of them ended up without `isReady` and
// `warmupPeriod` and 63 without `batch`. Nothing noticed, because the golden
// suite only ever calls `update`.
//
//   wasm-pack build --target nodejs --out-dir pkg
//   node --test tests/

const test = require('node:test');
const assert = require('node:assert/strict');
const W = require('../pkg/wickra_wasm.js');

// Bar builders implement `BarBuilder`, not `Indicator`: one candle can complete
// any number of bars, so they have no fixed warmup or ready state. They expose
// update/batch/reset/name and deliberately not isReady/warmupPeriod, which is
// the same exclusion the Node suite makes.
const BAR_BUILDERS = new Set([
  'RenkoBars',
  'KagiBars',
  'PointAndFigureBars',
  'RangeBars',
  'TickBars',
  'VolumeBars',
  'DollarBars',
  'ImbalanceBars',
  'RunBars',
  'ThreeLineBreakBars',
]);

// Data-layer types transform raw market data into candles and have their own
// push/flush/read shape.
const DATA_LAYER = new Set(['TickAggregator', 'Resampler', 'CandleReader']);

// The catalogue is a deliberate number, so assert it exactly: a `>=` bound
// passes just as happily when half the classes fail to build.
const INDICATOR_CLASS_COUNT = 504;

function exportedClasses() {
  return Object.keys(W).filter((name) => {
    const value = W[name];
    return (
      typeof value === 'function' &&
      value.prototype &&
      Object.getOwnPropertyNames(value.prototype).length > 1
    );
  });
}

function indicatorClasses() {
  return exportedClasses().filter(
    (name) =>
      typeof W[name].prototype.update === 'function' &&
      !BAR_BUILDERS.has(name) &&
      !DATA_LAYER.has(name),
  );
}

test('the binding exports the full indicator catalogue', () => {
  const names = indicatorClasses();
  assert.equal(
    names.length,
    INDICATOR_CLASS_COUNT,
    `expected ${INDICATOR_CLASS_COUNT} indicator classes, got ${names.length}` +
      ' — if an indicator was added or removed, update INDICATOR_CLASS_COUNT',
  );
});

test('no export resolves to undefined', () => {
  const dead = Object.keys(W).filter((name) => W[name] === undefined);
  assert.deepEqual(dead, [], `exports resolving to undefined: ${dead.join(', ')}`);
});

test('every exported indicator exposes update / batch / reset / name / isReady / warmupPeriod', () => {
  const required = ['update', 'batch', 'reset', 'name', 'isReady', 'warmupPeriod'];
  const missing = [];
  for (const name of indicatorClasses()) {
    for (const method of required) {
      if (typeof W[name].prototype[method] !== 'function') {
        missing.push(`${name}.${method}`);
      }
    }
  }
  assert.deepEqual(missing, [], `missing methods: ${missing.join(', ')}`);
});

test('every bar builder exposes update / batch / reset / name', () => {
  const missing = [];
  for (const name of BAR_BUILDERS) {
    assert.ok(W[name], `missing bar builder ${name}`);
    for (const method of ['update', 'batch', 'reset', 'name']) {
      if (typeof W[name].prototype[method] !== 'function') {
        missing.push(`${name}.${method}`);
      }
    }
  }
  assert.deepEqual(missing, [], `missing methods: ${missing.join(', ')}`);
});

test('a freshly constructed indicator reports not-ready with a positive warmup', () => {
  // Indicators that need constructor arguments are covered by the golden and
  // batch suites, which build every one of them from the manifest.
  let checked = 0;
  for (const name of indicatorClasses()) {
    let instance;
    try {
      instance = new W[name]();
    } catch {
      continue;
    }
    assert.equal(instance.isReady(), false, `${name} should start un-ready`);
    assert.ok(instance.warmupPeriod() >= 1, `${name} warmup must be >= 1`);
    checked += 1;
  }
  assert.ok(checked > 0, 'expected at least one zero-arg indicator to check');
});
