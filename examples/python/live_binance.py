"""Live Binance feed: stream Binance kline ticks → incremental indicators → signals.

This example connects to Binance's public WebSocket feed (no API key needed)
and runs RSI / MACD / Bollinger Bands on the close prices coming in. When the
RSI crosses common overbought / oversold thresholds *and* the MACD histogram
confirms the direction, a `Signal` event is printed. No orders are placed.

Run with::

    python -m examples.python.live_binance --symbol BTCUSDT --interval 1m

Dependencies::

    pip install websockets
"""

from __future__ import annotations

import argparse
import asyncio
import json
import logging
import re
import signal
import sys
from dataclasses import dataclass
from typing import Optional

import wickra as ta

try:
    import websockets
except ImportError:
    print("This example needs the `websockets` package: pip install websockets", file=sys.stderr)
    raise


BINANCE_WS = "wss://stream.binance.com:9443/stream"

# Kline intervals the public Binance WebSocket API recognises.
VALID_INTERVALS = frozenset(
    {
        "1s", "1m", "3m", "5m", "15m", "30m",
        "1h", "2h", "4h", "6h", "8h", "12h",
        "1d", "3d", "1w", "1M",
    }
)

# A Binance symbol is strictly alphanumeric (e.g. BTCUSDT).
_SYMBOL_RE = re.compile(r"^[A-Za-z0-9]+$")


def validate_args(symbol: str, interval: str) -> None:
    """Reject a symbol or interval that is not safe to splice into the WS URL.

    Both values are interpolated straight into the stream name and URL, so an
    unexpected value would otherwise produce a confusing connection failure.
    Validating up front keeps the URL well-formed without needing to escape it.

    Raises:
        ValueError: if ``symbol`` is not strictly alphanumeric, or ``interval``
            is not one Binance recognises.
    """
    if not _SYMBOL_RE.match(symbol):
        raise ValueError(
            f"invalid --symbol {symbol!r}: expected only letters and digits, "
            "e.g. BTCUSDT"
        )
    if interval not in VALID_INTERVALS:
        raise ValueError(
            f"invalid --interval {interval!r}: expected one of "
            + ", ".join(sorted(VALID_INTERVALS))
        )


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


async def run(symbol: str, interval: str) -> None:
    stream = f"{symbol.lower()}@kline_{interval}"
    url = f"{BINANCE_WS}?streams={stream}"
    log = logging.getLogger("wickra-live")
    state = StrategyState()
    log.info("Connecting to %s", url)
    async with websockets.connect(url, ping_interval=20) as ws:
        log.info("Connected, listening for %s klines", stream)
        async for raw in ws:
            envelope = json.loads(raw)
            payload = envelope.get("data", {})
            k = payload.get("k", {})
            if not k or "c" not in k:
                # Subscription acks, heartbeats and error frames carry no
                # kline payload — skip them instead of crashing on
                # float(None) the moment the stream opens.
                log.debug("skipping non-kline frame: %s", raw)
                continue
            close = float(k.get("c"))
            is_closed = bool(k.get("x"))
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
        validate_args(args.symbol, args.interval)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    logging.basicConfig(
        level=logging.DEBUG if args.verbose else logging.INFO,
        format="%(asctime)s %(levelname)s %(message)s",
    )
    loop = asyncio.new_event_loop()
    # Translate Ctrl+C into a clean loop stop.
    for sig in (signal.SIGINT, signal.SIGTERM):
        try:
            loop.add_signal_handler(sig, loop.stop)
        except NotImplementedError:
            pass  # Windows does not support add_signal_handler for SIGTERM.
    try:
        loop.run_until_complete(run(args.symbol, args.interval))
    except KeyboardInterrupt:
        pass
    finally:
        loop.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
