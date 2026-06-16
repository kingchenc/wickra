"""Live Binance feed: stream Binance kline ticks -> incremental indicators -> signals.

This example streams Binance's public kline feed (no API key needed) through
Wickra's **native** ``BinanceFeed`` and runs RSI / MACD / Bollinger Bands on the
close prices coming in. When the RSI crosses common overbought / oversold
thresholds *and* the MACD histogram confirms the direction, a signal is printed.
No orders are placed.

There is **no third-party dependency** — the WebSocket client is built into
Wickra. Just::

    python -m examples.python.live_binance --symbol BTCUSDT --interval 1m
"""

from __future__ import annotations

import argparse
import logging
import re
import sys
from dataclasses import dataclass
from typing import Optional

import wickra as ta

# Binance kline interval -> the integer code BinanceFeed expects (the same order
# in every Wickra binding).
INTERVAL_CODES = {
    "1s": 0, "1m": 1, "3m": 2, "5m": 3, "15m": 4, "30m": 5,
    "1h": 6, "2h": 7, "4h": 8, "6h": 9, "8h": 10, "12h": 11,
    "1d": 12, "3d": 13, "1w": 14, "1M": 15,
}

# A Binance symbol is strictly alphanumeric (e.g. BTCUSDT).
_SYMBOL_RE = re.compile(r"^[A-Za-z0-9]+$")


def validate_args(symbol: str, interval: str) -> int:
    """Validate the symbol/interval and return the interval's integer code.

    Raises:
        ValueError: if ``symbol`` is not strictly alphanumeric, or ``interval``
            is not one Binance recognises.
    """
    if not _SYMBOL_RE.match(symbol):
        raise ValueError(
            f"invalid --symbol {symbol!r}: expected only letters and digits, e.g. BTCUSDT"
        )
    if interval not in INTERVAL_CODES:
        raise ValueError(
            f"invalid --interval {interval!r}: expected one of "
            + ", ".join(INTERVAL_CODES)
        )
    return INTERVAL_CODES[interval]


@dataclass
class Snapshot:
    rsi: Optional[float]
    macd_hist: Optional[float]
    bb_upper: Optional[float]
    bb_middle: Optional[float]
    bb_lower: Optional[float]
    close: float


class StrategyState:
    """Holds one streaming instance of each indicator and computes signals."""

    def __init__(self) -> None:
        self.rsi = ta.RSI(14)
        self.macd = ta.MACD()
        self.bb = ta.BollingerBands(20, 2.0)

    def update(self, close: float) -> Snapshot:
        rsi = self.rsi.update(float(close))
        macd_v = self.macd.update(float(close))
        bb_v = self.bb.update(float(close))
        return Snapshot(
            rsi=rsi,
            macd_hist=macd_v[2] if macd_v else None,
            bb_upper=bb_v[0] if bb_v else None,
            bb_middle=bb_v[1] if bb_v else None,
            bb_lower=bb_v[2] if bb_v else None,
            close=float(close),
        )


def emit_signal(snap: Snapshot, log: logging.Logger) -> None:
    if snap.rsi is None or snap.macd_hist is None or snap.bb_upper is None:
        return
    if snap.rsi > 70 and snap.macd_hist < 0 and snap.close >= snap.bb_upper:
        log.warning(
            "SELL candidate: rsi=%.1f hist=%.4f close=%.4f >= bb_upper=%.4f",
            snap.rsi, snap.macd_hist, snap.close, snap.bb_upper,
        )
    elif snap.rsi < 30 and snap.macd_hist > 0 and snap.close <= snap.bb_lower:
        log.warning(
            "BUY candidate: rsi=%.1f hist=%.4f close=%.4f <= bb_lower=%.4f",
            snap.rsi, snap.macd_hist, snap.close, snap.bb_lower,
        )


def run(symbol: str, interval_code: int) -> None:
    log = logging.getLogger("wickra-live")
    state = StrategyState()
    feed = ta.BinanceFeed(symbol, interval_code)
    log.info("Streaming %s klines from Binance", symbol)
    while True:
        event = feed.next(1000)  # blocks up to 1s; None on timeout (Ctrl+C between polls)
        if event is None:
            continue
        _symbol, _open, _high, _low, close, _volume, _open_time, is_closed = event
        snap = state.update(close)
        log.info(
            "%s close=%.4f rsi=%s hist=%s bb=%s",
            "BAR" if is_closed else "tick",
            close,
            f"{snap.rsi:.1f}" if snap.rsi is not None else "--",
            f"{snap.macd_hist:+.4f}" if snap.macd_hist is not None else "--",
            f"{snap.bb_lower:.2f}/{snap.bb_middle:.2f}/{snap.bb_upper:.2f}"
            if snap.bb_upper is not None
            else "--",
        )
        emit_signal(snap, log)


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0] if __doc__ else None)
    p.add_argument("--symbol", default="BTCUSDT", help="trading pair")
    p.add_argument("--interval", default="1m", help="Binance kline interval")
    p.add_argument("--verbose", action="store_true")
    return p.parse_args()


def main() -> int:
    args = parse_args()
    try:
        interval_code = validate_args(args.symbol, args.interval)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    logging.basicConfig(
        level=logging.DEBUG if args.verbose else logging.INFO,
        format="%(asctime)s %(levelname)s %(message)s",
    )
    try:
        run(args.symbol, interval_code)
    except KeyboardInterrupt:
        pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
