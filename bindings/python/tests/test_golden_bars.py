"""Generic golden replay for the alt-chart bar builders and the footprint.

Each builder turns one candle into 0..n completed bars, so the fixture stores
one CSV line per input candle holding every bar flattened (an empty line means
no bar closed on that candle). Close-driven builders (Renko, Kagi, P&F, Range,
Three-Line-Break) receive a flat candle, mirroring the binding's `update(close)`.
Values are checked bit-for-bit against the Rust-generated `g_<Canonical>.csv`.
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


def _bar_rows(name):
    # Keep blank lines: one row per input candle, blank == no bar closed.
    with open(os.path.join(GOLDEN, name + ".csv")) as f:
        lines = f.read().splitlines()[1:]
    return [[] if not ln.strip() else [_cell(x) for x in ln.split(",")] for ln in lines]


def _input():
    with open(os.path.join(GOLDEN, "input.csv")) as f:
        return [[float(x) for x in line.split(",")] for line in f.read().splitlines()[1:] if line.strip()]


with open(os.path.join(GOLDEN, "bars_manifest.json")) as _mf:
    MANIFEST = json.load(_mf)
ROWS = _input()


def _flatten(bars):
    out = []
    for bar in bars:
        out.extend(float(x) for x in bar)
    return out


@pytest.mark.parametrize("spec", MANIFEST, ids=[m["canonical"] for m in MANIFEST])
def test_bars_match_golden(spec):
    ind = getattr(ta, spec["native"])(*spec["params"])
    expected = _bar_rows("g_" + spec["canonical"])
    feed = spec["feed"]
    assert len(expected) == len(ROWS), f"{spec['canonical']}: {len(expected)} rows vs {len(ROWS)} inputs"
    for i, (o, h, l, c, v) in enumerate(ROWS):
        if feed == "close":
            produced = ind.update(c)
        elif feed == "candle4":
            produced = ind.update(o, h, l, c)
        elif feed == "candle5":
            produced = ind.update(o, h, l, c, v)
        else:  # trade footprint
            produced = ind.update(c, v, c >= o)
        got = _flatten(produced)
        want = expected[i]
        assert len(got) == len(want), f"{spec['canonical']} row {i}: arity {len(got)} vs {len(want)}"
        for gv, w in zip(got, want):
            if math.isnan(w):
                assert math.isnan(gv), f"{spec['canonical']} row {i}: want NaN got {gv}"
            else:
                assert abs(gv - w) <= 1e-6 * max(1.0, abs(w)), f"{spec['canonical']} row {i}: got {gv} want {w}"
