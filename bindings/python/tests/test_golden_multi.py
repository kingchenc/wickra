"""Generic golden replay for multi-output indicators: each entry in
multi_manifest.json is reconstructed by its native name and every output field
is checked against the Rust-generated g_<Canonical>.csv (one column per field).
"""
import json
import math
import os

import pytest
import wickra as ta

HERE = os.path.dirname(__file__)
GOLDEN = os.path.normpath(os.path.join(HERE, "..", "..", "..", "testdata", "golden"))


def _cell(s):
    return math.nan if s == "nan" else float(s)


def _rows(name):
    with open(os.path.join(GOLDEN, name + ".csv")) as f:
        return [[_cell(x) for x in line.split(",")] for line in f.read().splitlines()[1:] if line.strip()]


def _input():
    with open(os.path.join(GOLDEN, "input.csv")) as f:
        return [[float(x) for x in line.split(",")] for line in f.read().splitlines()[1:] if line.strip()]


with open(os.path.join(GOLDEN, "multi_manifest.json")) as _mf:
    MANIFEST = json.load(_mf)
ROWS = _input()


@pytest.mark.parametrize("spec", MANIFEST, ids=[m["canonical"] for m in MANIFEST])
def test_multi_matches_golden(spec):
    ind = getattr(ta, spec["native"])(*spec["params"])
    expected = _rows("g_" + spec["canonical"])
    inp = spec["input"]
    for i, (o, h, l, c, v) in enumerate(ROWS):
        if inp == "f64":
            got = ind.update(c)
        elif inp == "Candle":
            got = ind.update((o, h, l, c, v, i))
        else:
            got = ind.update(c, o)
        want = expected[i]
        if got is None:
            assert all(math.isnan(w) for w in want), f"{spec['canonical']} row {i}: want {want} got None"
            continue
        vals = list(got)
        assert len(vals) == len(want), f"{spec['canonical']} row {i}: arity {len(vals)} vs {len(want)}"
        for gv, w in zip(vals, want):
            gv = math.nan if gv is None else gv
            if math.isnan(w):
                assert math.isnan(gv), f"{spec['canonical']} row {i}: want NaN got {gv}"
            elif math.isinf(w):
                assert math.isinf(gv) and (gv > 0) == (w > 0)
            else:
                assert abs(gv - w) <= 1e-6 * max(1.0, abs(w)), f"{spec['canonical']} row {i}: got {gv} want {w}"
