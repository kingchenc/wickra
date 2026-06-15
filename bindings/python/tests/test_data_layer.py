"""Cross-language data-layer parity for the Python binding: replay the shared
golden tick stream through the TickAggregator and check the candles against the
Rust-generated fixtures, with and without gap filling. Fixtures are produced by
``cargo run -p wickra-examples --bin gen_golden``.
"""
import csv
import os

import pytest
import wickra as ta

HERE = os.path.dirname(__file__)
GOLDEN = os.path.normpath(os.path.join(HERE, "..", "..", "..", "testdata", "golden"))


def _read(name):
    with open(os.path.join(GOLDEN, name + ".csv"), newline="") as f:
        rows = list(csv.reader(f))
    return [[float(x) for x in r] for r in rows[1:] if r]


TICKS = _read("data_ticks")


def _run(gap_fill):
    agg = ta.TickAggregator(1000, gap_fill=gap_fill)
    out = []
    for price, size, ts in TICKS:
        out.extend(agg.push(price, size, int(ts)))
    return out


@pytest.mark.parametrize(
    "gap_fill,fixture",
    [(False, "data_candles"), (True, "data_candles_gap")],
)
def test_tick_aggregator_matches_golden(gap_fill, fixture):
    got = _run(gap_fill)
    want = _read(fixture)
    assert len(got) == len(want)
    for i, (g, w) in enumerate(zip(got, want)):
        for j in range(6):
            tol = 1e-9 * max(1.0, abs(w[j]))
            assert abs(g[j] - w[j]) <= tol, f"row {i} col {j}: {g[j]} vs {w[j]}"


def test_candle_reader_matches_golden():
    with open(os.path.join(GOLDEN, "data_csv.csv")) as f:
        text = f.read()
    got = ta.CandleReader(text).read()
    want = _read("data_csv_candles")
    assert len(got) == len(want)
    for i, (g, w) in enumerate(zip(got, want)):
        for j in range(6):
            tol = 1e-9 * max(1.0, abs(w[j]))
            assert abs(g[j] - w[j]) <= tol, f"row {i} col {j}: {g[j]} vs {w[j]}"


INPUT = _read("input")  # open,high,low,close,volume (timestamp = row index)


def test_resampler_matches_golden():
    r = ta.Resampler(5)
    got = []
    for i, (o, h, l, c, v) in enumerate(INPUT):
        candle = r.update(o, h, l, c, v, i)
        if candle is not None:
            got.append(candle)
    f = r.flush()
    if f is not None:
        got.append(f)
    want = _read("data_resampled")
    assert len(got) == len(want)
    for i, (g, w) in enumerate(zip(got, want)):
        for j in range(6):
            tol = 1e-9 * max(1.0, abs(w[j]))
            assert abs(g[j] - w[j]) <= tol, f"row {i} col {j}: {g[j]} vs {w[j]}"
