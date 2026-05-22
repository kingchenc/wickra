"""Download real BTCUSDT spot candles from the Binance REST API and write
them as CSV datasets under ``examples/data/``.

The Python counterpart of ``examples/rust/src/bin/fetch_btcusdt.rs`` and
``examples/node/fetch_btcusdt.js`` — same pagination, same validity check,
same output layout. Uses only the standard library; zero third-party
dependencies.

Run with::

    python -m examples.python.fetch_btcusdt
"""

from __future__ import annotations

import argparse
import json
import math
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

# Binance Spot REST endpoint for historical klines.
KLINES_URL = "https://api.binance.com/api/v3/klines"
SYMBOL = "BTCUSDT"
# Binance caps a single klines response at 1000 rows.
PAGE_LIMIT = 1000
REQUEST_PAUSE_S = 0.2

# (Binance interval code, output file name, target candle count).
# The monthly file is btcusdt-1month.csv, not btcusdt-1M.csv, so it does not
# collide with btcusdt-1m.csv on case-insensitive filesystems.
DATASETS: list[tuple[str, str, int]] = [
    ("1m", "btcusdt-1m.csv", 50_000),
    ("5m", "btcusdt-5m.csv", 10_000),
    ("15m", "btcusdt-15m.csv", 10_000),
    ("1h", "btcusdt-1h.csv", 10_000),
    ("12h", "btcusdt-12h.csv", 5_000),
    ("1d", "btcusdt-1d.csv", 5_000),
    ("1M", "btcusdt-1month.csv", 5_000),
]


@dataclass
class Kline:
    open_time: int
    close_time: int
    open: float
    high: float
    low: float
    close: float
    volume: float


def parse_kline(raw: list) -> Optional[Kline]:
    """Parse one Binance kline array. Returns None for any row that is
    malformed or fails the same OHLC validity check ``Candle::new`` applies
    in the Rust core."""
    if not isinstance(raw, list) or len(raw) < 7:
        return None
    try:
        open_time = int(raw[0])
        close_time = int(raw[6])
        o = float(raw[1])
        h = float(raw[2])
        lo = float(raw[3])
        c = float(raw[4])
        v = float(raw[5])
    except (TypeError, ValueError):
        return None
    if not all(math.isfinite(x) for x in (o, h, lo, c, v)):
        return None
    if h < lo or h < o or h < c or lo > o or lo > c or v < 0:
        return None
    return Kline(open_time, close_time, o, h, lo, c, v)


def fetch_page(interval: str, end_time: Optional[int]) -> list:
    """Fetch one page of klines (up to ``PAGE_LIMIT`` rows)."""
    params: dict[str, str | int] = {
        "symbol": SYMBOL,
        "interval": interval,
        "limit": PAGE_LIMIT,
    }
    if end_time is not None:
        params["endTime"] = end_time
    url = f"{KLINES_URL}?{urllib.parse.urlencode(params)}"
    try:
        with urllib.request.urlopen(url, timeout=30) as resp:
            body = resp.read().decode("utf-8")
    except urllib.error.URLError as exc:
        raise RuntimeError(f"urlopen failed for {url}: {exc}") from exc
    data = json.loads(body)
    if not isinstance(data, list):
        raise RuntimeError(
            f"expected a JSON array from Binance, got: {body[:160]}"
        )
    return data


def collect(interval: str, target: int, now_ms: int) -> list[Kline]:
    """Paginate the Binance REST API backwards until ``target`` closed
    candles have been collected (or the exchange runs out of history),
    then return them in ascending open-time order."""
    by_open: dict[int, Kline] = {}
    end_time: Optional[int] = None
    pages = 0
    while True:
        page = fetch_page(interval, end_time)
        pages += 1
        if not page:
            break
        oldest: Optional[int] = None
        for raw in page:
            k = parse_kline(raw)
            if k is None:
                continue
            if oldest is None or k.open_time < oldest:
                oldest = k.open_time
            # Keep only fully closed candles — drop the in-progress bucket.
            if k.close_time < now_ms:
                by_open[k.open_time] = k
        sys.stderr.write(
            f"\r  {interval}: collected {len(by_open)} candles over "
            f"{pages} page(s)…"
        )
        sys.stderr.flush()
        if len(by_open) >= target or len(page) < PAGE_LIMIT:
            break
        if oldest is None:
            raise RuntimeError(
                f"Binance page for {interval} held no parseable klines"
            )
        end_time = oldest - 1
        time.sleep(REQUEST_PAUSE_S)
    sys.stderr.write("\n")

    candles = sorted(by_open.values(), key=lambda k: k.open_time)
    if len(candles) > target:
        candles = candles[-target:]
    return candles


def format_number(v: float) -> str:
    """Match Rust's f64 ``Display``: shortest round-trip, no trailing ``.0``.

    Python's :func:`repr` already gives the shortest round-trip
    representation; Rust additionally drops the ``.0`` suffix for
    whole-number floats. Strip that here so the CSVs produced by the
    Rust, Python and Node fetchers are byte-for-byte identical on the
    same Binance snapshot.
    """
    s = repr(v)
    if s.endswith(".0"):
        return s[:-2]
    return s


def write_csv(path: Path, candles: list[Kline]) -> None:
    """Write candles to ``path`` in the standard OHLCV layout the
    ``CandleReader`` accepts."""
    with path.open("w", encoding="utf-8", newline="") as f:
        f.write("timestamp,open,high,low,close,volume\n")
        for k in candles:
            f.write(
                f"{k.open_time},"
                f"{format_number(k.open)},"
                f"{format_number(k.high)},"
                f"{format_number(k.low)},"
                f"{format_number(k.close)},"
                f"{format_number(k.volume)}\n"
            )


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__.splitlines()[0] if __doc__ else None,
    )
    parser.parse_args()  # accept --help; no other CLI flags today.

    now_ms = int(time.time() * 1000)
    data_dir = Path(__file__).resolve().parent.parent / "data"
    data_dir.mkdir(parents=True, exist_ok=True)

    print(f"Fetching {SYMBOL} klines from Binance into {data_dir}")
    for interval, filename, target in DATASETS:
        candles = collect(interval, target, now_ms)
        if not candles:
            print(
                f"error: Binance returned no closed candles for {interval}",
                file=sys.stderr,
            )
            return 1
        write_csv(data_dir / filename, candles)
        print(
            f"  {interval:>3}  {len(candles):>6} candles  ->  "
            f"examples/data/{filename}"
        )
    print(f"Done — {len(DATASETS)} datasets written.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
