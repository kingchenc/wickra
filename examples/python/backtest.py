"""Offline backtest example: compute a basket of indicators over a CSV history.

Run with::

    python -m examples.python.backtest path/to/ohlcv.csv

The CSV must have a header row with at least the columns
``timestamp, open, high, low, close, volume``. The script computes a panel of
indicators with the standard parameters used across Wickra's tests and
prints a small summary of the resulting series — enough to verify that the
indicators are wired correctly without pulling in pandas or a charting stack.
"""

from __future__ import annotations

import argparse
import math
import sys
from array import array
from dataclasses import dataclass

import wickra as ta


def column(matrix, index: int) -> list[float]:
    """One field of a multi-output ``batch``.

    A multi-output ``batch`` returns a ``Matrix``; ``tolist`` is the documented
    bridge to plain rows, and this picks one field out of each.
    """
    return [row[index] for row in matrix.tolist()]


@dataclass
class History:
    timestamp: array
    open: array
    high: array
    low: array
    close: array
    volume: array


def read_history(path: str) -> History:
    """Load an OHLCV CSV into typed columns with Wickra's native
    ``CandleReader`` — no manual CSV parsing.

    ``CandleReader`` validates the header (``timestamp,open,high,low,close,volume``),
    tolerates a UTF-8 BOM and surrounding whitespace, and raises ``ValueError`` on a
    missing column or a non-numeric / invalid OHLC row.

    Raises:
        ValueError: if the CSV header or a data row is malformed.
    """
    with open(path, encoding="utf-8") as f:
        candles = ta.CandleReader(f.read()).read()
    if not candles:
        raise ValueError(f"{path}: CSV has a header but no data rows")
    # CandleReader yields (open, high, low, close, volume, timestamp) tuples.
    # Transpose into contiguous 1-D columns; `array('d')` is the stdlib buffer
    # `batch` already hands back, so no third-party array type is involved.
    o, h, l, c, v, ts = (array("d", col) for col in zip(*candles))
    return History(
        timestamp=array("q", (int(t) for t in ts)),
        open=o,
        high=h,
        low=l,
        close=c,
        volume=v,
    )


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0] if __doc__ else None)
    p.add_argument("path", help="Path to an OHLCV CSV file")
    p.add_argument("--rsi", type=int, default=14)
    p.add_argument("--ema", type=int, default=20)
    p.add_argument("--bb-period", type=int, default=20)
    p.add_argument("--bb-mult", type=float, default=2.0)
    return p.parse_args()


def summarize(name: str, values) -> None:
    # A scalar `batch` returns `array.array('d')`; the warmup entries are NaN.
    valid = [v for v in values if not math.isnan(v)]
    if not valid:
        print(f"  {name:<12} (no valid samples — series too short)")
        return
    print(
        f"  {name:<12} mean={sum(valid) / len(valid):>10.4f}  "
        f"min={min(valid):>10.4f}  max={max(valid):>10.4f}  last={valid[-1]:>10.4f}"
    )


def main() -> int:
    args = parse_args()
    history = read_history(args.path)

    rsi = ta.RSI(args.rsi).batch(history.close)
    ema = ta.EMA(args.ema).batch(history.close)
    macd = ta.MACD().batch(history.close)  # (n, 3)
    bb = ta.BollingerBands(args.bb_period, args.bb_mult).batch(history.close)  # (n, 4)
    atr = ta.ATR(14).batch(history.high, history.low, history.close)
    adx = ta.ADX(14).batch(history.high, history.low, history.close)  # (n, 3)
    obv = ta.OBV().batch(history.close, history.volume)

    print(f"Backtest summary for {args.path} ({len(history.close)} bars)")
    summarize(f"RSI({args.rsi})", rsi)
    summarize(f"EMA({args.ema})", ema)
    summarize("MACD line", column(macd, 0))
    summarize("MACD hist", column(macd, 2))
    summarize("BB upper", column(bb, 0))
    summarize("BB lower", column(bb, 2))
    summarize("ATR(14)", atr)
    summarize("ADX(14)", column(adx, 2))
    summarize("OBV", obv)

    return 0


if __name__ == "__main__":
    sys.exit(main())
