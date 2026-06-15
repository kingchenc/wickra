// Generic golden-fixture parity for the WASM (wasm-bindgen) binding, run under
// Node's test runner against the nodejs-target build in ../pkg.
//
// Every one of the 514 indicators is reconstructed from wasm_manifest.json (JS
// class, constructor params, ordered update args parsed from the generated
// .d.ts), fed the synthetic stream derived from the shared golden input — the
// same construction gen_golden uses — and checked bit-for-bit against the Rust
// reference fixtures g_<Canonical>.csv.
//
//   wasm-pack build --target nodejs --out-dir pkg
//   node --test tests/

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const W = require('../pkg/wickra_wasm.js');

const GOLDEN = path.resolve(__dirname, '..', '..', '..', 'testdata', 'golden');

function cell(s) {
  if (s === 'nan') return NaN;
  if (s === 'inf') return Infinity;
  if (s === '-inf') return -Infinity;
  return Number(s);
}

function readCsv(name) {
  const lines = fs.readFileSync(path.join(GOLDEN, name + '.csv'), 'utf8').split('\n');
  lines.shift();
  return lines.filter((l) => l.length > 0).map((l) => l.split(',').map(cell));
}

function readBarRows(name) {
  const lines = fs.readFileSync(path.join(GOLDEN, name + '.csv'), 'utf8').split('\n');
  lines.shift();
  if (lines.length && lines[lines.length - 1] === '') lines.pop();
  return lines.map((l) => (l.length === 0 ? [] : l.split(',').map(cell)));
}

const MANIFEST = JSON.parse(fs.readFileSync(path.join(GOLDEN, 'wasm_manifest.json'), 'utf8'));
// Canonical Indicator::name() per indicator, the same string every binding returns.
const NAMES = JSON.parse(fs.readFileSync(path.join(GOLDEN, 'names.json'), 'utf8'));
const ROWS = readCsv('input');

function deriv(o, h, l, c, v) {
  return {
    funding_rate: ((c - o) / c) * 0.01,
    mark_price: c,
    index_price: c - 0.5,
    futures_price: c + 1.0,
    open_interest: v * 10.0,
    long_size: v * 0.6,
    short_size: v * 0.4,
    taker_buy_volume: v * 0.55,
    taker_sell_volume: v * 0.45,
    long_liquidation: h - c,
    short_liquidation: c - l,
  };
}

function resolveArg(arg, o, h, l, c, v, i) {
  const n = arg.name;
  if (arg.array) {
    const j5 = [0, 1, 2, 3, 4];
    switch (n) {
      case 'change': return Float64Array.from(j5.map((j) => c - o + j));
      case 'volume': return Float64Array.from(j5.map((j) => v + j * 10.0));
      case 'new_high': return Float64Array.from(j5.map((j) => (j % 2 === 0 ? 1 : 0)));
      case 'new_low': return Float64Array.from(j5.map((j) => (j % 3 === 0 ? 1 : 0)));
      case 'above_ma': return Float64Array.from(j5.map((j) => (j % 2 === 0 ? 1 : 0)));
      case 'on_buy_signal': return Float64Array.from(j5.map((j) => (j % 3 === 0 ? 1 : 0)));
      case 'bid_px': return Float64Array.from(j5.map((k) => c - 0.1 * (k + 1)));
      case 'bid_sz': return Float64Array.from(j5.map((k) => v / (k + 1)));
      case 'ask_px': return Float64Array.from(j5.map((k) => c + 0.1 * (k + 1)));
      case 'ask_sz': return Float64Array.from(j5.map((k) => (v * 0.9) / (k + 1)));
      default: throw new Error('array arg ' + n);
    }
  }
  if (arg.bigint) return BigInt(i);
  switch (n) {
    case 'value': case 'close': case 'price': case 'x': case 'a': case 'asset': return c;
    case 'y': case 'b': case 'open': case 'benchmark': return o;
    case 'high': return h;
    case 'low': return l;
    case 'volume': case 'size': return v;
    case 'is_buy': return c >= o;
    case 'mid': return (h + l) / 2.0;
    case 'timestamp': return BigInt(i);
    default: {
      const d = deriv(o, h, l, c, v);
      if (n in d) return d[n];
      throw new Error('scalar arg ' + n);
    }
  }
}

// Recursively flatten a WASM output (number, object with field props, typed/array
// of either) into a flat number list. Returns null for warmup (null/undefined).
function flat(v) {
  if (v === null || v === undefined) return null;
  if (typeof v === 'number' || typeof v === 'bigint') return [Number(v)];
  if (Array.isArray(v) || ArrayBuffer.isView(v)) {
    const out = [];
    for (const e of v) { const f = flat(e); if (f) out.push(...f); }
    return out;
  }
  if (typeof v === 'object') {
    const out = [];
    for (const val of Object.values(v)) { const f = flat(val); if (f) out.push(...f); }
    return out;
  }
  return [Number(v)];
}

function nanRow(n) {
  return Array.from({ length: n }, () => NaN);
}

function widthOf(spec) {
  if (spec.out.startsWith('multi') || spec.out === 'deriv_multi') return spec.n;
  if (spec.out.startsWith('profile')) return spec.width;
  return 1; // scalar archetypes
}

function closeEq(got, want, label) {
  if (Number.isNaN(want)) { assert.ok(Number.isNaN(got), `${label}: want NaN got ${got}`); return; }
  if (!Number.isFinite(want)) { assert.ok(got === want, `${label}: want ${want} got ${got}`); return; }
  const tol = 1e-6 * Math.max(1.0, Math.abs(want));
  assert.ok(Math.abs(got - want) <= tol, `${label}: got ${got} want ${want}`);
}

for (const spec of MANIFEST) {
  test(`wasm golden: ${spec.canonical}`, () => {
    const Cls = W[spec.js];
    assert.ok(Cls, `missing WASM class ${spec.js}`);
    const ind = new Cls(...spec.ctor);
    assert.equal(ind.name(), NAMES[spec.canonical], `${spec.canonical}: name()`);
    const isBars = spec.out === 'bars' || spec.out === 'footprint';
    const expected = isBars ? readBarRows('g_' + spec.canonical) : readCsv('g_' + spec.canonical);

    for (let i = 0; i < ROWS.length; i++) {
      const [o, h, l, c, v] = ROWS[i];
      const args = spec.args.map((a) => resolveArg(a, o, h, l, c, v, i));
      const raw = ind.update(...args);
      const want = expected[i];
      const label = `${spec.canonical} row ${i}`;

      let got = flat(raw);
      if (isBars) {
        if (got === null) got = [];
      } else if (got === null) {
        got = nanRow(widthOf(spec));
      }
      assert.equal(got.length, want.length, `${label}: arity ${got.length} vs ${want.length}`);
      for (let k = 0; k < want.length; k++) closeEq(got[k], want[k], `${label} col ${k}`);
    }
  });
}
