// Completeness contract for the Wickra Node bindings: every exported indicator
// class must expose the full streaming + batch + lifecycle interface. This
// catches a new indicator being wired into the binding without the standard
// methods (or an export silently disappearing) without needing a hand-written
// test per indicator.

const test = require('node:test');
const assert = require('node:assert/strict');
const wickra = require('..');

// Bar builders (Renko / Kagi / Point & Figure) implement the `BarBuilder`
// contract, not `Indicator`: they emit a variable number of completed bars per
// candle and have no fixed warmup or ready state. They expose update/batch/reset
// but intentionally not isReady/warmupPeriod, so they are excluded from the
// Indicator completeness contract below (their interface is covered by the
// dedicated bar-builder tests).
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

// Data-layer types (tick aggregator, resampler) are not `Indicator`s: they
// transform raw market data into candles and have their own update/flush shape,
// so they are excluded from the streaming-indicator completeness contract.
const DATA_LAYER = new Set(['TickAggregator', 'Resampler']);

// An "indicator class" is an exported constructor whose prototype carries the
// streaming `update` method. This excludes `version` (a plain function), the bar
// builders, the data-layer types, and any non-indicator export.
function indicatorClasses() {
  return Object.keys(wickra).filter((name) => {
    const value = wickra[name];
    return (
      typeof value === 'function' &&
      value.prototype &&
      typeof value.prototype.update === 'function' &&
      !BAR_BUILDERS.has(name) &&
      !DATA_LAYER.has(name)
    );
  });
}

test('the binding exports the full indicator catalogue', () => {
  const names = indicatorClasses();
  // The published catalogue is 214 indicators. Guard against a regression that
  // silently drops exported classes (e.g. a stale or partial native build).
  assert.ok(
    names.length >= 200,
    `expected at least 200 indicator classes, got ${names.length}`,
  );
});

test('every exported indicator exposes update / batch / reset / isReady / warmupPeriod', () => {
  const required = ['update', 'batch', 'reset', 'isReady', 'warmupPeriod'];
  const missing = [];
  for (const name of indicatorClasses()) {
    const proto = wickra[name].prototype;
    for (const method of required) {
      if (typeof proto[method] !== 'function') {
        missing.push(`${name}.${method}`);
      }
    }
  }
  assert.deepEqual(
    missing,
    [],
    `indicator classes missing required methods: ${missing.join(', ')}`,
  );
});

test('a freshly constructed indicator reports not-ready with a positive warmup', () => {
  // Every indicator that takes no constructor arguments must still satisfy the
  // pre-warmup contract. (Indicators with required parameters are exercised by
  // the dedicated suites; here we cover the zero-arg ones generically.)
  let checked = 0;
  for (const name of indicatorClasses()) {
    let instance;
    try {
      instance = new wickra[name]();
    } catch {
      continue; // needs constructor arguments — covered elsewhere
    }
    assert.equal(instance.isReady(), false, `${name} should start un-ready`);
    assert.ok(instance.warmupPeriod() >= 1, `${name} warmup must be >= 1`);
    checked += 1;
  }
  assert.ok(checked > 0, 'expected at least one zero-arg indicator to check');
});
