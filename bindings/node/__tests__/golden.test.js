// Generic golden-fixture parity for the Node binding.
//
// Every one of the 514 indicators is reconstructed from `node_manifest.json`
// (native class, constructor params, ordered update args), fed the synthetic
// stream derived from the shared `testdata/golden/input.csv` — the exact same
// construction the Rust `gen_golden` binary uses — and checked bit-for-bit
// against the Rust-generated `g_<Canonical>.csv`. This pins the Node FFI to the
// Rust reference for the whole indicator catalogue, not just a few archetypes.
//
//   cd bindings/node && npm run build && npm test

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const wickra = require('..');

const GOLDEN = path.resolve(__dirname, '..', '..', '..', 'testdata', 'golden');

function cell(s) {
  if (s === 'nan') return NaN;
  if (s === 'inf') return Infinity;
  if (s === '-inf') return -Infinity;
  return Number(s);
}

function readCsv(name) {
  // Split on \r?\n so a CRLF checkout (Windows core.autocrlf) parses identically
  // to LF — otherwise `cell('inf\r')` falls through to Number() and becomes NaN.
  const lines = fs.readFileSync(path.join(GOLDEN, name + '.csv'), 'utf8').split(/\r?\n/);
  lines.shift(); // header
  return lines.filter((l) => l.length > 0).map((l) => l.split(',').map(cell));
}

// Bars keep blank lines (one row per candle, blank == no bar closed).
function readBarRows(name) {
  const lines = fs.readFileSync(path.join(GOLDEN, name + '.csv'), 'utf8').split(/\r?\n/);
  lines.shift();
  // Drop only the single trailing-newline artifact, keeping legitimate blank
  // rows (a candle on which no bar closed) so rows stay aligned to the input.
  if (lines.length && lines[lines.length - 1] === '') lines.pop();
  return lines.map((l) => (l.length === 0 ? [] : l.split(',').map(cell)));
}

const MANIFEST = JSON.parse(fs.readFileSync(path.join(GOLDEN, 'node_manifest.json'), 'utf8'));
// Canonical Indicator::name() per indicator (the same string every binding must
// return). Keyed by the Rust canonical; values are the core names, which may
// differ from the registered class name (e.g. ChaikinMoneyFlow -> "CMF").
const NAMES = JSON.parse(fs.readFileSync(path.join(GOLDEN, 'names.json'), 'utf8'));
const ROWS = readCsv('input');

function derivFields(o, h, l, c, v) {
  return {
    fundingRate: ((c - o) / c) * 0.01,
    markPrice: c,
    indexPrice: c - 0.5,
    futuresPrice: c + 1.0,
    openInterest: v * 10.0,
    longSize: v * 0.6,
    shortSize: v * 0.4,
    takerBuyVolume: v * 0.55,
    takerSellVolume: v * 0.45,
    longLiquidation: h - c,
    shortLiquidation: c - l,
  };
}

function resolveArg(arg, o, h, l, c, v, i) {
  const name = arg.name;
  if (arg.array) {
    switch (name) {
      case 'change':
        return [0, 1, 2, 3, 4].map((j) => c - o + j);
      case 'volume':
        return [0, 1, 2, 3, 4].map((j) => v + j * 10.0);
      case 'newHigh':
        return [0, 1, 2, 3, 4].map((j) => j % 2 === 0);
      case 'newLow':
        return [0, 1, 2, 3, 4].map((j) => j % 3 === 0);
      case 'aboveMa':
        return [0, 1, 2, 3, 4].map((j) => j % 2 === 0);
      case 'onBuySignal':
        return [0, 1, 2, 3, 4].map((j) => j % 3 === 0);
      case 'bidPx':
        return [0, 1, 2, 3, 4].map((k) => c - 0.1 * (k + 1));
      case 'bidSz':
        return [0, 1, 2, 3, 4].map((k) => v / (k + 1));
      case 'askPx':
        return [0, 1, 2, 3, 4].map((k) => c + 0.1 * (k + 1));
      case 'askSz':
        return [0, 1, 2, 3, 4].map((k) => (v * 0.9) / (k + 1));
      default:
        throw new Error('unknown array arg ' + name);
    }
  }
  switch (name) {
    case 'value':
    case 'close':
    case 'price':
    case 'x':
    case 'a':
    case 'asset':
      return c;
    case 'y':
    case 'b':
    case 'benchmark':
    case 'open':
      return o;
    case 'high':
      return h;
    case 'low':
      return l;
    case 'volume':
    case 'size':
      return v;
    case 'timestamp':
      return i;
    case 'isBuy':
      return c >= o;
    case 'mid':
      return (h + l) / 2.0;
    default: {
      const d = derivFields(o, h, l, c, v);
      if (name in d) return d[name];
      throw new Error('unknown scalar arg ' + name);
    }
  }
}

function closeEq(got, want, label) {
  if (Number.isNaN(want)) {
    assert.ok(Number.isNaN(got), `${label}: want NaN got ${got}`);
    return;
  }
  if (!Number.isFinite(want)) {
    assert.ok(got === want, `${label}: want ${want} got ${got}`);
    return;
  }
  const tol = 1e-6 * Math.max(1.0, Math.abs(want));
  assert.ok(Math.abs(got - want) <= tol, `${label}: got ${got} want ${want}`);
}

for (const spec of MANIFEST) {
  test(`golden: ${spec.canonical}`, () => {
    const Cls = wickra[spec.native];
    assert.ok(Cls, `missing Node class ${spec.native}`);
    const ind = new Cls(...spec.ctor);
    // name() must report the canonical core Indicator::name(), the same string
    // every other binding returns for this indicator.
    assert.equal(ind.name(), NAMES[spec.canonical], `${spec.canonical}: name()`);
    const isBars = spec.out === 'bars' || spec.out === 'footprint';
    const expected = isBars ? readBarRows('g_' + spec.canonical) : readCsv('g_' + spec.canonical);

    for (let i = 0; i < ROWS.length; i++) {
      const [o, h, l, c, v] = ROWS[i];
      const args = spec.args.map((a) => resolveArg(a, o, h, l, c, v, i));
      const got = ind.update(...args);
      const want = expected[i];
      const label = `${spec.canonical} row ${i}`;

      if (spec.out === 'scalar') {
        closeEq(got === null || got === undefined ? NaN : got, want[0], label);
      } else if (spec.out === 'multi') {
        if (got === null || got === undefined) {
          assert.ok(want.every(Number.isNaN), `${label}: want ${want} got null`);
          continue;
        }
        // napi serialises the output struct's fields in declaration order, which
        // matches the CSV column order — compare positionally to avoid relying on
        // the exact camelCase of each field name.
        const vals = Object.values(got);
        assert.equal(vals.length, want.length, `${label}: arity ${vals.length} vs ${want.length}`);
        vals.forEach((gv, k) => {
          closeEq(gv === null || gv === undefined ? NaN : gv, want[k], `${label} col ${k}`);
        });
      } else if (spec.out === 'profile_bins') {
        if (got === null || got === undefined) {
          assert.ok(want.every(Number.isNaN), `${label}: want all-NaN got null`);
          continue;
        }
        assert.equal(got.length, want.length, `${label}: width ${got.length} vs ${want.length}`);
        got.forEach((gv, k) => closeEq(gv, want[k], `${label} bin ${k}`));
      } else if (spec.out === 'profile_pricebins') {
        if (got === null || got === undefined) {
          assert.ok(want.every(Number.isNaN), `${label}: want all-NaN got null`);
          continue;
        }
        const flat = [got.priceLow, got.priceHigh, ...got[spec.arrayField]];
        assert.equal(flat.length, want.length, `${label}: width ${flat.length} vs ${want.length}`);
        flat.forEach((gv, k) => closeEq(gv, want[k], `${label} col ${k}`));
      } else {
        // bars / footprint: flatten array-of-objects in declared field order.
        const flat = [];
        for (const bar of got) {
          for (const f of spec.fields) flat.push(Number(bar[f]));
        }
        assert.equal(flat.length, want.length, `${label}: arity ${flat.length} vs ${want.length}`);
        flat.forEach((gv, k) => closeEq(gv, want[k], `${label} col ${k}`));
      }
    }
  });
}
