"""Process indicators for many symbols in parallel.

Python's GIL makes pure-Python parallelism a poor fit, but Wickra's heavy
lifting happens inside the Rust extension — which releases the GIL during
batch computation. That means a `ThreadPoolExecutor` actually delivers
multi-core speedup here.

Run with::

    python -m examples.python.parallel_assets --assets 1000 --bars 5000
"""

from __future__ import annotations

import argparse
import concurrent.futures
import time
from typing import Tuple

import numpy as np

import wickra as ta


def synthesize_panel(n_assets: int, n_bars: int, seed: int = 0xBADC0FFEE0DDF00D) -> np.ndarray:
    """Build an `(n_assets, n_bars)` matrix of synthetic prices."""
    rng = np.random.default_rng(seed & 0xFFFF_FFFF)
    drift = rng.standard_normal((n_assets, n_bars)) * 0.4
    return 100.0 + np.cumsum(drift, axis=1)


def compute_one(prices: np.ndarray) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Return (RSI, EMA, MACD-line) for one asset."""
    rsi = ta.RSI(14).batch(prices)
    ema = ta.EMA(20).batch(prices)
    macd = ta.MACD().batch(prices)
    return rsi, ema, macd[:, 0]


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0] if __doc__ else None)
    p.add_argument("--assets", type=int, default=200)
    p.add_argument("--bars", type=int, default=5000)
    p.add_argument(
        "--workers",
        type=int,
        default=0,
        help="thread pool size; 0 means use os.cpu_count()",
    )
    args = p.parse_args()

    print(f"Generating {args.assets}x{args.bars} synthetic panel…")
    panel = synthesize_panel(args.assets, args.bars)

    # Serial baseline.
    t0 = time.perf_counter()
    serial_results = [compute_one(panel[i]) for i in range(args.assets)]
    t_serial = time.perf_counter() - t0
    print(f"Serial:   {t_serial:.3f} s  ({args.assets} assets)")

    # Parallel.
    workers = args.workers or None
    t0 = time.perf_counter()
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
        parallel_results = list(pool.map(compute_one, panel))
    t_parallel = time.perf_counter() - t0
    used = workers or 0
    print(
        f"Parallel: {t_parallel:.3f} s  (workers={used or 'os.cpu_count()'}, "
        f"speedup ~{t_serial / max(t_parallel, 1e-9):.2f}x)"
    )

    # Sanity-check that the parallel results match the serial baseline.
    for i in range(args.assets):
        for s, p_ in zip(serial_results[i], parallel_results[i]):
            np.testing.assert_array_equal(s, p_)
    print("Parallel results match serial results — OK.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
