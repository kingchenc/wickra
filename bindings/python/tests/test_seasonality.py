"""Streaming-vs-batch equivalence and reference values for the Seasonality &
Session family.

These indicators read the full candle (including ``timestamp``), so they have a
dedicated test rather than joining the timestamp-less parametrize harness in
``test_new_indicators.py``.
"""

import numpy as np
import pytest

import wickra as ta

HOUR_MS = 3_600_000


def _to_np(x) -> np.ndarray:
    """Normalize a Wickra batch result (``array.array`` or ``Matrix``) to an ndarray."""
    if isinstance(x, np.ndarray):
        return x.astype(np.float64)
    if hasattr(x, "tolist") and hasattr(x, "shape"):
        return np.asarray(x.tolist(), dtype=np.float64)
    return np.asarray(x, dtype=np.float64)


@pytest.fixture(scope="module")
def candle_columns():
    """240 hourly candles (10 days) with valid OHLCV and epoch-ms timestamps."""
    n = 240
    t = np.arange(n, dtype=np.float64)
    close = 100.0 + np.sin(t * 0.3) * 5.0 + np.cos(t * 0.1) * 3.0
    open_ = close + np.sin(t * 0.5) * 0.5
    high = np.maximum(open_, close) + 1.0
    low = np.minimum(open_, close) - 1.0
    volume = 1000.0 + (t % 24) * 50.0
    timestamp = (np.arange(n, dtype=np.int64)) * HOUR_MS
    return open_, high, low, close, volume, timestamp


def _candles(cols):
    open_, high, low, close, volume, timestamp = cols
    return [
        (open_[i], high[i], low[i], close[i], volume[i], int(timestamp[i]))
        for i in range(len(close))
    ]


def _check_scalar(make, cols):
    candles = _candles(cols)
    a, b = make(), make()
    stream = np.array(
        [np.nan if (v := a.update(c)) is None else v for c in candles],
        dtype=np.float64,
    )
    batch = _to_np(b.batch(*cols))
    np.testing.assert_allclose(stream, batch, equal_nan=True, rtol=1e-9, atol=1e-9)


def _check_matrix(make, k, cols):
    candles = _candles(cols)
    a, b = make(), make()
    rows = []
    for c in candles:
        out = a.update(c)
        rows.append(np.full(k, np.nan) if out is None else np.asarray(out, dtype=float))
    stream = np.vstack(rows)
    batch = _to_np(b.batch(*cols))
    assert batch.shape == (len(candles), k)
    np.testing.assert_allclose(stream, batch, equal_nan=True, rtol=1e-9, atol=1e-9)


SCALAR = [
    lambda: ta.SessionVwap(0),
    lambda: ta.OvernightGap(0),
    lambda: ta.SeasonalZScore(0),
    lambda: ta.AverageDailyRange(3, 0),
    lambda: ta.TurnOfMonth(3, 1, 0),
]

MATRIX = [
    (lambda: ta.SessionHighLow(0), 2),
    (lambda: ta.SessionRange(0), 3),
    (lambda: ta.OvernightIntradayReturn(0), 2),
    (lambda: ta.TimeOfDayReturnProfile(24, 0), 24),
    (lambda: ta.IntradayVolatilityProfile(12, 0), 12),
    (lambda: ta.VolumeByTimeProfile(24, 0), 24),
    (lambda: ta.DayOfWeekProfile(0), 7),
]


@pytest.mark.parametrize("make", SCALAR)
def test_scalar_streaming_equals_batch(make, candle_columns):
    _check_scalar(make, candle_columns)


@pytest.mark.parametrize("make,k", MATRIX)
def test_matrix_streaming_equals_batch(make, k, candle_columns):
    _check_matrix(make, k, candle_columns)


def test_session_vwap_reference():
    vwap = ta.SessionVwap(0)
    # typical = close for a flat candle; volume-weighted within the day.
    v1 = vwap.update((100.0, 100.0, 100.0, 100.0, 10.0, 0))
    assert v1 == pytest.approx(100.0)
    v2 = vwap.update((110.0, 110.0, 110.0, 110.0, 30.0, HOUR_MS))
    assert v2 == pytest.approx(107.5)
    # New day re-anchors.
    v3 = vwap.update((200.0, 200.0, 200.0, 200.0, 5.0, 24 * HOUR_MS))
    assert v3 == pytest.approx(200.0)


def test_overnight_gap_reference():
    gap = ta.OvernightGap(0)
    assert gap.update((99.0, 101.0, 98.0, 100.0, 1.0, 0)) is None
    g = gap.update((105.0, 106.0, 104.0, 105.5, 1.0, 24 * HOUR_MS))
    assert g == pytest.approx(0.05)


def test_session_high_low_reference():
    shl = ta.SessionHighLow(0)
    shl.update((100.0, 105.0, 99.0, 101.0, 1.0, 0))
    out = shl.update((101.0, 108.0, 100.0, 107.0, 1.0, HOUR_MS))
    assert out == (108.0, 99.0)


def test_volume_by_time_profile_reference():
    prof = ta.VolumeByTimeProfile(24, 0)
    out = prof.update((100.0, 100.0, 100.0, 100.0, 500.0, HOUR_MS))  # 01:00 -> bucket 1
    assert out[1] == pytest.approx(500.0)
    assert out[0] == pytest.approx(0.0)


def test_rejects_zero_buckets():
    with pytest.raises(ValueError):
        ta.TimeOfDayReturnProfile(0, 0)


def test_average_daily_range_rejects_zero_period():
    with pytest.raises(ValueError):
        ta.AverageDailyRange(0, 0)
