"""Streaming indicators with the Wickra Python binding.

Feeds a synthetic price series through several indicators tick by tick — the
same O(1)-per-update model a live trading bot would use — and prints a status
line whenever every indicator has warmed up. The Python counterpart of
``examples/node/streaming.js`` and ``examples/rust/src/bin/streaming.rs``.

Run with::

    python -m examples.python.streaming
"""

from __future__ import annotations

import argparse
import math

import wickra as ta


def make_series(n: int) -> list[float]:
    """Deterministic synthetic series: slow trend + two oscillations + tiny noise.

    The seeded linear-congruential generator matches the Node sibling example
    so a side-by-side run produces visibly comparable streams.
    """
    seed = 1234567
    prices: list[float] = []
    for t in range(n):
        seed = (seed * 1103515245 + 12345) & 0x7FFFFFFF
        rand = seed / 0x7FFFFFFF
        price = (
            100.0
            + t * 0.05
            + math.sin(t * 0.07) * 8.0
            + math.cos(t * 0.21) * 3.0
            + (rand - 0.5)
        )
        prices.append(price)
    return prices


def fmt(value: float | None) -> str:
    if value is None:
        return "  --  "
    if isinstance(value, float) and math.isnan(value):
        return "  --  "
    return f"{value:7.2f}"


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__.splitlines()[0] if __doc__ else None,
    )
    parser.add_argument(
        "--ticks",
        type=int,
        default=120,
        help="number of synthetic price ticks to stream (default: 120)",
    )
    args = parser.parse_args()
    if args.ticks <= 0:
        parser.error("--ticks must be positive")

    print(f"Wickra {ta.__version__} — streaming indicator demo (Python)\n")

    sma = ta.SMA(20)
    ema = ta.EMA(20)
    rsi = ta.RSI(14)
    macd = ta.MACD(12, 26, 9)

    prices = make_series(args.ticks)
    signals = 0
    for t, price in enumerate(prices):
        sma_v = sma.update(price)
        ema_v = ema.update(price)
        rsi_v = rsi.update(price)
        macd_v = macd.update(price)  # (macd, signal, histogram) or None

        # Only act once every indicator has produced a value.
        if sma_v is None or ema_v is None or rsi_v is None or macd_v is None:
            continue

        _macd_line, _signal, histogram = macd_v
        overbought = rsi_v > 70 and histogram < 0
        oversold = rsi_v < 30 and histogram > 0
        tag = "SELL?" if overbought else "BUY? " if oversold else "     "
        if overbought or oversold:
            signals += 1

        print(
            f"t={t:>3}  price={fmt(price)}  sma={fmt(sma_v)}  ema={fmt(ema_v)}  "
            f"rsi={fmt(rsi_v)}  macd_hist={fmt(histogram)}  {tag}"
        )

    print(f"\nDone — {signals} candidate signal(s) over {len(prices)} ticks.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
