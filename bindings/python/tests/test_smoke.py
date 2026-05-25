"""Smoke tests: every public class can be constructed and emits the right shape."""

from __future__ import annotations

import numpy as np
import pytest

import wickra as ta


def test_version_is_a_nonempty_string():
    assert isinstance(ta.__version__, str)
    assert ta.__version__


@pytest.mark.parametrize(
    "cls, args",
    [
        (ta.SMA, (14,)),
        (ta.EMA, (14,)),
        (ta.WMA, (14,)),
        (ta.RSI, (14,)),
    ],
)
def test_scalar_batch_returns_same_length(cls, args, sine_prices):
    out = cls(*args).batch(sine_prices)
    assert out.shape == sine_prices.shape
    assert out.dtype == np.float64


def test_macd_batch_returns_n_by_3(sine_prices):
    out = ta.MACD().batch(sine_prices)
    assert out.shape == (sine_prices.size, 3)


def test_bollinger_batch_returns_n_by_4(sine_prices):
    out = ta.BollingerBands().batch(sine_prices)
    assert out.shape == (sine_prices.size, 4)


def test_atr_batch_shape(ohlc_series):
    high, low, close = ohlc_series
    out = ta.ATR(14).batch(high, low, close)
    assert out.shape == close.shape


def test_stochastic_batch_shape(ohlc_series):
    high, low, close = ohlc_series
    out = ta.Stochastic(14, 3).batch(high, low, close)
    assert out.shape == (close.size, 2)


def test_obv_batch_shape(ohlc_series):
    _, _, close = ohlc_series
    volume = np.ones_like(close)
    out = ta.OBV().batch(close, volume)
    assert out.shape == close.shape


def test_ehlers_super_smoother_batch_shape(sine_prices):
    out = ta.SuperSmoother(10).batch(sine_prices)
    assert out.shape == sine_prices.shape


def test_mama_batch_shape(sine_prices):
    out = ta.MAMA().batch(sine_prices)
    assert out.shape == (sine_prices.size, 2)
