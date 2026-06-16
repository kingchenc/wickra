"""Compute indicators on multiple timeframes from a single 1-minute feed.

Loads the 1-minute history with Wickra's native ``CandleReader`` and rolls it up
to coarser timeframes with the native ``Resampler`` — no manual CSV parsing and
no hand-written bucketing. NumPy is used only to run the batch indicators over
each resampled series.

Run with::

    python -m examples.python.multi_timeframe path/to/1m.csv
"""

from __future__ import annotations

import argparse
from typing import List, Tuple

import numpy as np

import wickra as ta

# A candle as the data layer yields it: (open, high, low, close, volume, timestamp).
Candle = Tuple[float, float, float, float, float, int]


def load(path: str) -> List[Candle]:
    """Read an OHLCV CSV into candles with ``CandleReader`` (validates the
    ``timestamp,open,high,low,close,volume`` header; raises ``ValueError`` on a
    malformed header or row)."""
    with open(path, encoding="utf-8") as f:
        return ta.CandleReader(f.read()).read()


def resample(candles: List[Candle], bucket_ms: int) -> List[Candle]:
    """Aggregate 1-minute candles into ``bucket_ms``-sized candles with the
    native streaming ``Resampler``."""
    r = ta.Resampler(bucket_ms)
    out: List[Candle] = []
    for o, h, l, c, v, ts in candles:
        agg = r.update(o, h, l, c, v, ts)
        if agg is not None:
            out.append(agg)
    tail = r.flush()
    if tail is not None:
        out.append(tail)
    return out


def summarize(label: str, candles: List[Candle]) -> None:
    if not candles:
        print(f"  {label:<10} (empty series — nothing to summarize)")
        return
    # Transpose into contiguous 1-D columns (batch() needs C-contiguous arrays).
    _o, high, low, close, _v, _ts = (np.array(col, dtype=np.float64) for col in zip(*candles))
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

    candles = load(args.path)
    one_minute = 60_000

    print(f"Multi-timeframe view of {args.path}")
    summarize("1m", candles)
    for label, bucket in [("5m", 5), ("15m", 15), ("1h", 60), ("4h", 240), ("1d", 1440)]:
        if len(candles) < bucket:
            continue
        summarize(label, resample(candles, bucket * one_minute))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
