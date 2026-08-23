// Surface contract for the WASM binding.
//
// The macro-generated wrappers always carried the full interface; the classes
// written out by hand drifted, and 73 of them ended up without `isReady` and
// `warmupPeriod` and 63 without `batch`. Nothing noticed, because the golden
// suite only ever calls `update`.
//
// WASM is the one binding whose surface lives in a build artifact rather than in
// the repository, so `scripts/check_binding_surface.py` -- which holds the other
// seven to the shape the C ABI declares -- cannot read it statically. This is
// the same contract, checked at runtime against the same manifest, so the two
// cannot disagree about what the contract is.
//
//   wasm-pack build --target nodejs --out-dir pkg
//   node --test tests/

const test = require('node:test');
const assert = require('node:assert/strict');
const W = require('../pkg/wickra_wasm.js');
const { MANIFEST, NAMES } = require('./harness.js');

// Every indicator exposes these.
const CONTRACT = ['update', 'batch', 'reset', 'name'];
// An Indicator carries a warmup and a ready flag; a BarBuilder does not,
// because one candle can complete any number of bars, so there is nothing for it
// to be ready for. `out === 'bars'` is exactly the ten builders — Footprint is
// an Indicator despite its bar-shaped output.
const STATEFUL = ['isReady', 'warmupPeriod'];
const isBarBuilder = (spec) => spec.out === 'bars';

// Data-layer types transform raw market data into candles and have their own
// push/flush/read shape, so they are not part of the indicator catalogue.
const DATA_LAYER = new Set(['TickAggregator', 'Resampler', 'CandleReader']);

test('the manifest covers the whole indicator catalogue', () => {
  const manifested = new Set(MANIFEST.map((s) => s.canonical));
  const canonical = new Set(Object.keys(NAMES));
  const missing = [...canonical].filter((n) => !manifested.has(n));
  const extra = [...manifested].filter((n) => !canonical.has(n));
  assert.deepEqual(missing, [], `not in the WASM manifest: ${missing.join(', ')}`);
  assert.deepEqual(extra, [], `in the WASM manifest but not the catalogue: ${extra.join(', ')}`);
});

test('every catalogued indicator is exported as a class', () => {
  const absent = MANIFEST.filter((spec) => typeof W[spec.js] !== 'function').map((s) => s.js);
  assert.deepEqual(absent, [], `missing WASM classes: ${absent.join(', ')}`);
});

test('no export resolves to undefined', () => {
  const dead = Object.keys(W).filter((name) => W[name] === undefined);
  assert.deepEqual(dead, [], `exports resolving to undefined: ${dead.join(', ')}`);
});

test('every indicator exposes update / batch / reset / name', () => {
  const missing = [];
  for (const spec of MANIFEST) {
    for (const method of CONTRACT) {
      if (typeof W[spec.js].prototype[method] !== 'function') missing.push(`${spec.js}.${method}`);
    }
  }
  assert.deepEqual(missing, [], `missing methods: ${missing.join(', ')}`);
});

// Two-sided, like the static check: a bar builder that grew an `isReady` has
// drifted as far as an indicator that lost one.
test('isReady and warmupPeriod are present exactly where the catalogue says', () => {
  const wrong = [];
  for (const spec of MANIFEST) {
    for (const method of STATEFUL) {
      const present = typeof W[spec.js].prototype[method] === 'function';
      if (present === isBarBuilder(spec)) {
        wrong.push(`${spec.js}.${method} ${present ? 'present on a bar builder' : 'absent'}`);
      }
    }
  }
  assert.deepEqual(wrong, [], wrong.join(', '));
});

test('an exported class that is neither catalogued nor data-layer is unexpected', () => {
  const catalogued = new Set(MANIFEST.map((s) => s.js));
  const stray = Object.keys(W).filter((name) => {
    const value = W[name];
    return (
      typeof value === 'function' &&
      value.prototype &&
      typeof value.prototype.update === 'function' &&
      !catalogued.has(name) &&
      !DATA_LAYER.has(name)
    );
  });
  assert.deepEqual(stray, [], `exported but not in the catalogue: ${stray.join(', ')}`);
});

test('a freshly constructed indicator reports not-ready with a positive warmup', () => {
  // Indicators that need constructor arguments are covered by the golden, batch
  // and lifecycle suites, which build every one of them from the manifest.
  let checked = 0;
  for (const spec of MANIFEST) {
    if (isBarBuilder(spec) || spec.ctor.length > 0) continue;
    const instance = new W[spec.js]();
    assert.equal(instance.isReady(), false, `${spec.js} should start un-ready`);
    assert.ok(instance.warmupPeriod() >= 1, `${spec.js} warmup must be >= 1`);
    checked += 1;
  }
  assert.ok(checked > 0, 'expected at least one zero-arg indicator to check');
});
