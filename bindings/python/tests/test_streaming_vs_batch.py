"""For every indicator, batch(prices) must equal repeated update(price).

This is the central correctness contract of Wickra: the two APIs share one
implementation, so they cannot disagree. These tests verify it from Python
across the entire warmup → steady-state transition.
"""

from __future__ import annotations

import math

import numpy as np
import pytest

import wickra as ta


def _equal_with_nan(a: np.ndarray, b: np.ndarray, tol: float = 1e-9) -> bool:
    """NumPy ``==`` treats NaN as not-equal; emulate ``equal_nan`` for floats."""
    if a.shape != b.shape:
        return False
    both_nan = np.isnan(a) & np.isnan(b)
    diff_ok = np.where(both_nan, 0.0, np.abs(a - b))
    return bool(np.all(diff_ok <= tol))


@pytest.mark.parametrize(
    "cls, args",
    [
        (ta.SMA, (14,)),
        (ta.EMA, (14,)),
        (ta.WMA, (14,)),
        (ta.RSI, (14,)),
    ],
)
def test_scalar_streaming_matches_batch(cls, args, sine_prices):
    batch = cls(*args).batch(sine_prices)

    streamer = cls(*args)
    streamed = np.array(
        [streamer.update(float(p)) if streamer is not None else None for p in sine_prices],
        dtype=object,
    )
    # Map None -> NaN to compare against batch.
    streamed = np.array(
        [math.nan if v is None else float(v) for v in streamed], dtype=np.float64
    )

    assert _equal_with_nan(batch, streamed)


def test_macd_streaming_matches_batch(sine_prices):
    batch = ta.MACD().batch(sine_prices)

    streamer = ta.MACD()
    rows = []
    for p in sine_prices:
        v = streamer.update(float(p))
        if v is None:
            rows.append([math.nan, math.nan, math.nan])
        else:
            rows.append(list(v))
    streamed = np.array(rows, dtype=np.float64)
    assert _equal_with_nan(batch, streamed)


def test_bollinger_streaming_matches_batch(sine_prices):
    batch = ta.BollingerBands().batch(sine_prices)

    streamer = ta.BollingerBands()
    rows = []
    for p in sine_prices:
        v = streamer.update(float(p))
        if v is None:
            rows.append([math.nan, math.nan, math.nan, math.nan])
        else:
            rows.append(list(v))
    streamed = np.array(rows, dtype=np.float64)
    assert _equal_with_nan(batch, streamed)


def test_atr_streaming_matches_batch(ohlc_series):
    high, low, close = ohlc_series
    batch = ta.ATR(14).batch(high, low, close)

    streamer = ta.ATR(14)
    rows = []
    for h, l, c in zip(high, low, close):
        rows.append(streamer.update((float(c), float(h), float(l), float(c), 0.0, 0)))
    streamed = np.array([math.nan if v is None else v for v in rows], dtype=np.float64)
    assert _equal_with_nan(batch, streamed)


def test_stochastic_streaming_matches_batch(ohlc_series):
    high, low, close = ohlc_series
    batch = ta.Stochastic(14, 3).batch(high, low, close)

    streamer = ta.Stochastic(14, 3)
    rows = []
    for h, l, c in zip(high, low, close):
        v = streamer.update((float(c), float(h), float(l), float(c), 0.0, 0))
        rows.append([math.nan, math.nan] if v is None else list(v))
    streamed = np.array(rows, dtype=np.float64)
    assert _equal_with_nan(batch, streamed)


def test_obv_streaming_matches_batch(ohlc_series):
    _, _, close = ohlc_series
    volume = np.ones_like(close)
    batch = ta.OBV().batch(close, volume)

    streamer = ta.OBV()
    rows = []
    for c, v in zip(close, volume):
        rows.append(streamer.update((float(c), float(c), float(c), float(c), float(v), 0)))
    streamed = np.array([math.nan if x is None else x for x in rows], dtype=np.float64)
    assert _equal_with_nan(batch, streamed)


def test_rolling_vwap_streaming_matches_batch(ohlc_series):
    # RollingVWAP(20) on the shared OHLC series. Provides finite-memory VWAP
    # parity coverage now that the indicator is exposed across all bindings.
    high, low, close = ohlc_series
    volume = np.linspace(100.0, 200.0, num=close.size, dtype=np.float64)
    batch = ta.RollingVWAP(20).batch(high, low, close, volume)

    streamer = ta.RollingVWAP(20)
    rows = []
    for h, l, c, v in zip(high, low, close, volume):
        rows.append(streamer.update((float(c), float(h), float(l), float(c), float(v), 0)))
    streamed = np.array([math.nan if x is None else x for x in rows], dtype=np.float64)
    assert _equal_with_nan(batch, streamed)
    assert streamer.period == 20
    assert streamer.warmup_period() == 20
    assert streamer.is_ready()
    streamer.reset()
    assert not streamer.is_ready()
