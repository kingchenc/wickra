"""Generic golden replay for the exotic-input families: DerivativesTick,
CrossSection, Trade, TradeQuote and OrderBook indicators.

Each family feeds a synthetic stream deterministically derived from the shared
`testdata/golden/input.csv` OHLCV rows — the exact same construction the Rust
`gen_golden` binary uses — and every value is checked bit-for-bit against the
Rust-generated `g_<Canonical>.csv`. This pins the Python FFI for indicators
whose inputs cannot be expressed as a plain close/candle/pair stream.
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


with open(os.path.join(GOLDEN, "exotic_manifest.json")) as _mf:
    MANIFEST = json.load(_mf)
ROWS = _input()


def _deriv_fields(o, h, l, c, v):
    return {
        "funding_rate": (c - o) / c * 0.01,
        "mark_price": c,
        "index_price": c - 0.5,
        "futures_price": c + 1.0,
        "open_interest": v * 10.0,
        "long_size": v * 0.6,
        "short_size": v * 0.4,
        "taker_buy_volume": v * 0.55,
        "taker_sell_volume": v * 0.45,
        "long_liquidation": h - c,
        "short_liquidation": c - l,
    }


def _cross_lists(o, h, l, c, v):
    change = [(c - o) + j for j in range(5)]
    volume = [v + j * 10.0 for j in range(5)]
    new_high = [j % 2 == 0 for j in range(5)]
    new_low = [j % 3 == 0 for j in range(5)]
    above_ma = [j % 2 == 0 for j in range(5)]
    on_buy_signal = [j % 3 == 0 for j in range(5)]
    return change, volume, new_high, new_low, above_ma, on_buy_signal


def _ob_lists(o, h, l, c, v):
    bid_px = [c - 0.1 * (k + 1) for k in range(5)]
    bid_sz = [v / (k + 1) for k in range(5)]
    ask_px = [c + 0.1 * (k + 1) for k in range(5)]
    ask_sz = [v * 0.9 / (k + 1) for k in range(5)]
    return bid_px, bid_sz, ask_px, ask_sz


def _assert_scalar(got, want, canonical, i):
    got = math.nan if got is None else got
    if math.isnan(want):
        assert math.isnan(got), f"{canonical} row {i}: want NaN got {got}"
    elif math.isinf(want):
        assert math.isinf(got) and (got > 0) == (want > 0), f"{canonical} row {i}: got {got} want {want}"
    else:
        assert abs(got - want) <= 1e-6 * max(1.0, abs(want)), f"{canonical} row {i}: got {got} want {want}"


def _specs(family):
    return [(family, s) for s in MANIFEST[family]]


@pytest.mark.parametrize(
    "family,spec",
    _specs("deriv") + _specs("cross") + _specs("trade") + _specs("trademid") + _specs("ob"),
    ids=[s["canonical"] for fam in ("deriv", "cross", "trade", "trademid", "ob") for s in MANIFEST[fam]],
)
def test_exotic_matches_golden(family, spec):
    ind = getattr(ta, spec["native"])(*spec["params"])
    expected = _rows("g_" + spec["canonical"])
    n = spec.get("n")
    for i, (o, h, l, c, v) in enumerate(ROWS):
        if family == "deriv":
            f = _deriv_fields(o, h, l, c, v)
            got = ind.update(*[f[a] for a in spec["args"]])
        elif family == "cross":
            change, volume, nh, nl, above_ma, on_buy = _cross_lists(o, h, l, c, v)
            extra = spec.get("extra")
            if extra == "above_ma":
                got = ind.update(change, volume, nh, nl, above_ma)
            elif extra == "on_buy_signal":
                got = ind.update(change, volume, nh, nl, on_buy)
            else:
                got = ind.update(change, volume, nh, nl)
        elif family == "trade":
            got = ind.update(c, v, c >= o)
        elif family == "trademid":
            got = ind.update(c, v, c >= o, (h + l) / 2.0)
        else:  # ob
            bid_px, bid_sz, ask_px, ask_sz = _ob_lists(o, h, l, c, v)
            got = ind.update(bid_px, bid_sz, ask_px, ask_sz)

        want = expected[i]
        if n:  # multi-output (LiquidationFeatures)
            if got is None:
                assert all(math.isnan(w) for w in want), f"{spec['canonical']} row {i}: want {want} got None"
                continue
            vals = list(got)
            assert len(vals) == len(want), f"{spec['canonical']} row {i}: arity {len(vals)} vs {len(want)}"
            for gv, w in zip(vals, want):
                _assert_scalar(gv, w, spec["canonical"], i)
        else:
            _assert_scalar(got, want[0], spec["canonical"], i)
