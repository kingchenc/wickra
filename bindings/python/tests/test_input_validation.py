"""Input-validation tests: malformed NumPy inputs raise ValueError, not panics."""

from __future__ import annotations

import numpy as np
import pytest

import wickra as ta


def test_non_contiguous_array_raises_value_error():
    # A strided view is not C-contiguous; batch() must reject it cleanly.
    base = np.linspace(1.0, 100.0, 60)
    non_contiguous = base[::2]
    assert not non_contiguous.flags["C_CONTIGUOUS"]
    with pytest.raises(ValueError):
        ta.SMA(5).batch(non_contiguous)


def test_ascontiguousarray_recovers():
    base = np.linspace(1.0, 100.0, 60)
    fixed = np.ascontiguousarray(base[::2])
    out = ta.SMA(5).batch(fixed)
    assert out.shape == fixed.shape


def test_unequal_length_candle_batch_raises(ohlc_series):
    high, low, close = ohlc_series
    short = low[:-1]
    with pytest.raises(ValueError):
        ta.ATR(14).batch(high, short, close)
    with pytest.raises(ValueError):
        ta.WilliamsR(14).batch(high, short, close)
    with pytest.raises(ValueError):
        ta.Aroon(14).batch(high, short)


def test_roc_and_trix_have_default_periods():
    # ROC/TRIX gained constructor defaults matching the TA-Lib convention.
    assert ta.ROC().period == 10
    assert ta.TRIX() is not None


def test_ichimoku_rejects_zero_and_non_increasing_periods():
    with pytest.raises(ValueError):
        ta.Ichimoku(0, 26, 52, 26)
    with pytest.raises(ValueError):
        ta.Ichimoku(9, 26, 52, 0)
    # Periods must satisfy tenkan < kijun < senkou_b.
    with pytest.raises(ValueError):
        ta.Ichimoku(26, 9, 52, 26)
    with pytest.raises(ValueError):
        ta.Ichimoku(9, 52, 52, 26)
