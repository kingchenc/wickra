"""Compute indicators on multiple timeframes from a single 1-minute feed.

Wickra exposes resampling on the Rust side (in `wickra-data`); from Python we
roll up bars manually using NumPy because most users already have their data
in a NumPy array and don't need the streaming-resample infrastructure for
offline analysis.

Run with::

    python -m examples.python.multi_timeframe path/to/1m.csv
"""

from __future__ import annotations

import argparse
import csv
from typing import Iterable, List, Tuple

import numpy as np

import wickra as ta


# Columns the OHLCV layout requires; the CSV header must name every one.
REQUIRED_COLUMNS = ("timestamp", "open", "high", "low", "close", "volume")


def read_csv(path: str) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    """Read an OHLCV CSV into typed columns.

    Raises:
        ValueError: if the file has no header, is missing a required column,
            holds no data rows, or contains a non-numeric value (the message
            names the offending row).
    """
    ts, o, h, l, c, v = [], [], [], [], [], []
    with open(path, newline="") as f:
        reader = csv.DictReader(f)
        if reader.fieldnames is None:
            raise ValueError(f"{path}: CSV has no header row")
        missing = [col for col in REQUIRED_COLUMNS if col not in reader.fieldnames]
        if missing:
            raise ValueError(
                f"{path}: CSV is missing required column(s): "
                f"{', '.join(missing)}; found: {', '.join(reader.fieldnames)}"
            )
        for line_no, row in enumerate(reader, start=2):
            try:
                ts.append(int(row["timestamp"]))
                o.append(float(row["open"]))
                h.append(float(row["high"]))
                l.append(float(row["low"]))
                c.append(float(row["close"]))
                v.append(float(row["volume"]))
            except (TypeError, ValueError) as exc:
                raise ValueError(
                    f"{path}: row {line_no} has a non-numeric value: {exc}"
                ) from exc
    if not ts:
        raise ValueError(f"{path}: CSV has a header but no data rows")
    return (
        np.asarray(ts, dtype=np.int64),
        np.asarray(o, dtype=np.float64),
        np.asarray(h, dtype=np.float64),
        np.asarray(l, dtype=np.float64),
        np.asarray(c, dtype=np.float64),
        np.asarray(v, dtype=np.float64),
    )


def resample(
    ts: np.ndarray,
    o: np.ndarray,
    h: np.ndarray,
    l: np.ndarray,
    c: np.ndarray,
    v: np.ndarray,
    bucket: int,
) -> Tuple[np.ndarray, ...]:
    """Aggregate bar-level columns into coarser buckets of size `bucket`.

    Raises:
        ValueError: if the input series is empty.
    """
    if ts.size == 0:
        raise ValueError("resample received an empty series")
    bucket_index = (ts // bucket).astype(np.int64)
    boundaries = np.diff(bucket_index, prepend=bucket_index[0] - 1) != 0
    group_ids = np.cumsum(boundaries) - 1
    n_groups = int(group_ids.max()) + 1

    new_ts = np.empty(n_groups, dtype=np.int64)
    new_o = np.empty(n_groups, dtype=np.float64)
    new_h = np.empty(n_groups, dtype=np.float64)
    new_l = np.empty(n_groups, dtype=np.float64)
    new_c = np.empty(n_groups, dtype=np.float64)
    new_v = np.empty(n_groups, dtype=np.float64)

    for g in range(n_groups):
        mask = group_ids == g
        new_ts[g] = ts[mask][0]
        new_o[g] = o[mask][0]
        new_h[g] = h[mask].max()
        new_l[g] = l[mask].min()
        new_c[g] = c[mask][-1]
        new_v[g] = v[mask].sum()
    return new_ts, new_o, new_h, new_l, new_c, new_v


def summarize(label: str, close: np.ndarray, high: np.ndarray, low: np.ndarray) -> None:
    if close.size == 0:
        print(f"  {label:<10} (empty series — nothing to summarize)")
        return
    rsi = ta.RSI(14).batch(close)
    macd = ta.MACD().batch(close)
    adx = ta.ADX(14).batch(high, low, close)
    last_macd_hist = macd[~np.isnan(macd[:, 2])][-1, 2] if np.any(~np.isnan(macd[:, 2])) else float("nan")
    last_adx = adx[~np.isnan(adx[:, 2])][-1, 2] if np.any(~np.isnan(adx[:, 2])) else float("nan")
    valid_rsi = rsi[~np.isnan(rsi)]
    last_rsi = valid_rsi[-1] if valid_rsi.size else float("nan")
    print(
        f"  {label:<10} bars={close.size:>5}  "
        f"last_close={close[-1]:>10.4f}  rsi={last_rsi:>6.2f}  "
        f"macd_hist={last_macd_hist:+.4f}  adx={last_adx:>6.2f}"
    )


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0] if __doc__ else None)
    p.add_argument("path", help="Path to a 1-minute OHLCV CSV (timestamps in milliseconds)")
    args = p.parse_args()

    ts, o, h, l, c, v = read_csv(args.path)
    one_minute = 60_000

    print(f"Multi-timeframe view of {args.path}")
    summarize("1m", c, h, l)
    for label, bucket in [("5m", 5), ("15m", 15), ("1h", 60), ("4h", 240), ("1d", 1440)]:
        if ts.size < bucket:
            continue
        rs_ts, rs_o, rs_h, rs_l, rs_c, rs_v = resample(ts, o, h, l, c, v, bucket * one_minute)
        summarize(label, rs_c, rs_h, rs_l)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
