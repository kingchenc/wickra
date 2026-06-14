"""Generic golden replay for the profile family: time/volume seasonality
histograms (`bins`) and price-binned market profiles (`price_low, price_high,
bins`). Each profile emits a fixed-width row once warm and `NaN`s during warmup.

The shared `testdata/golden/input.csv` candle series is replayed through the
Python FFI and the flattened histogram is checked bit-for-bit against the
Rust-generated `g_<Canonical>.csv`.
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


with open(os.path.join(GOLDEN, "profile_manifest.json")) as _mf:
    MANIFEST = json.load(_mf)
ROWS = _input()


@pytest.mark.parametrize("spec", MANIFEST, ids=[m["canonical"] for m in MANIFEST])
def test_profile_matches_golden(spec):
    ind = getattr(ta, spec["native"])(*spec["params"])
    expected = _rows("g_" + spec["canonical"])
    width = spec["width"]
    for i, (o, h, l, c, v) in enumerate(ROWS):
        got = ind.update((o, h, l, c, v, i))
        want = expected[i]
        assert len(want) == width, f"{spec['canonical']} row {i}: fixture width {len(want)} != {width}"
        if got is None:
            assert all(math.isnan(w) for w in want), f"{spec['canonical']} row {i}: want {want} got None"
            continue
        if spec["kind"] == "pricebins":
            price_low, price_high, bins = got
            vals = [price_low, price_high, *list(bins)]
        else:
            vals = list(got)
        assert len(vals) == width, f"{spec['canonical']} row {i}: arity {len(vals)} != {width}"
        for gv, w in zip(vals, want):
            if math.isnan(w):
                assert math.isnan(gv), f"{spec['canonical']} row {i}: want NaN got {gv}"
            else:
                assert abs(gv - w) <= 1e-6 * max(1.0, abs(w)), f"{spec['canonical']} row {i}: got {gv} want {w}"
