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
import sys
from dataclasses import dataclass

import numpy as np

import wickra as ta


def columns(matrix) -> np.ndarray:
    """A multi-output ``batch`` returns a ``Matrix``, not a NumPy array; ``tolist``
    is the documented bridge to one."""
    return np.asarray(matrix.tolist(), dtype=np.float64)


@dataclass
class History:
    timestamp: np.ndarray
    open: np.ndarray
    high: np.ndarray
    low: np.ndarray
    close: np.ndarray
    volume: np.ndarray


def read_history(path: str) -> History:
    """Load an OHLCV CSV into typed NumPy columns with Wickra's native
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
    # Transpose into contiguous 1-D columns (batch() needs C-contiguous arrays).
    o, h, l, c, v, ts = (np.array(col, dtype=np.float64) for col in zip(*candles))
    return History(
        timestamp=ts.astype(np.int64),
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
    # A scalar `batch` returns `array.array('d')`, which NumPy reads through the
    # buffer protocol but cannot be boolean-indexed directly.
    valid = np.asarray(values, dtype=np.float64)
    valid = valid[~np.isnan(valid)]
    if valid.size == 0:
        print(f"  {name:<12} (no valid samples — series too short)")
        return
    print(
        f"  {name:<12} mean={valid.mean():>10.4f}  min={valid.min():>10.4f}  "
        f"max={valid.max():>10.4f}  last={valid[-1]:>10.4f}"
    )


def main() -> int:
    args = parse_args()
    history = read_history(args.path)

    rsi = ta.RSI(args.rsi).batch(history.close)
    ema = ta.EMA(args.ema).batch(history.close)
    macd = columns(ta.MACD().batch(history.close))  # shape (n, 3)
    bb = columns(ta.BollingerBands(args.bb_period, args.bb_mult).batch(history.close))  # (n, 4)
    atr = ta.ATR(14).batch(history.high, history.low, history.close)
    adx = columns(ta.ADX(14).batch(history.high, history.low, history.close))  # (n, 3)
    obv = ta.OBV().batch(history.close, history.volume)

    print(f"Backtest summary for {args.path} ({history.close.size} bars)")
    summarize(f"RSI({args.rsi})", rsi)
    summarize(f"EMA({args.ema})", ema)
    summarize("MACD line", macd[:, 0])
    summarize("MACD hist", macd[:, 2])
    summarize(f"BB upper", bb[:, 0])
    summarize(f"BB lower", bb[:, 2])
    summarize("ATR(14)", atr)
    summarize("ADX(14)", adx[:, 2])
    summarize("OBV", obv)

    return 0


if __name__ == "__main__":
    sys.exit(main())
