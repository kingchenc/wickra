// Batch/streaming equivalence for the WASM binding.
//
// `golden.test.js` replays every indicator through `update` only, so a `batch`
// that disagreed with it -- or that did not exist -- went unnoticed. This feeds
// the same manifest-derived stream through `batch` on a fresh instance and
// checks it against the reference fixtures, flattened the same way.
//
//   wasm-pack build --target nodejs --out-dir pkg
//   node --test tests/

const test = require('node:test');
const assert = require('node:assert/strict');
const W = require('../pkg/wickra_wasm.js');
const {
  MANIFEST,
  closeEq,
  column,
  flatten,
  readRows,
  tolFor,
} = require('./harness.js');

for (const spec of MANIFEST) {
  test(`wasm batch: ${spec.canonical}`, () => {
    const Cls = W[spec.js];
    assert.ok(Cls, `missing WASM class ${spec.js}`);
    assert.equal(
      typeof Cls.prototype.batch,
      'function',
      `${spec.js} has no batch`,
    );

    const columns = spec.args.map(column);
    const got = flatten(new Cls(...spec.ctor).batch(...columns), spec);

    // The reference is the same fixture `golden.test.js` checks `update`
    // against, flattened row by row; a warmup row there is already NaN.
    const want = [];
    for (const row of readRows('g_' + spec.canonical)) want.push(...row);

    const tol = tolFor(spec.canonical);
    assert.equal(got.length, want.length, `${spec.canonical}: length`);
    for (let k = 0; k < want.length; k++) {
      closeEq(got[k], want[k], `${spec.canonical} value ${k}`, tol);
    }
  });
}
