"""Tests for the indicator lifecycle methods: reset, is_ready, warmup_period, repr."""

from __future__ import annotations

import numpy as np
import pytest

import wickra as ta

SCALAR_INDICATORS = [
    (ta.SMA, (14,)),
    (ta.EMA, (14,)),
    (ta.WMA, (14,)),
    (ta.RSI, (14,)),
    (ta.MACD, ()),
    (ta.BollingerBands, ()),
]


@pytest.mark.parametrize("cls, args", SCALAR_INDICATORS)
def test_is_ready_transitions_after_warmup(cls, args):
    ind = cls(*args)
    assert not ind.is_ready()
    series = np.linspace(1.0, 200.0, 200)
    ind.batch(series)
    assert ind.is_ready()


@pytest.mark.parametrize("cls, args", SCALAR_INDICATORS)
def test_reset_returns_to_initial_state(cls, args):
    ind = cls(*args)
    ind.batch(np.linspace(1.0, 200.0, 200))
    assert ind.is_ready()
    ind.reset()
    assert not ind.is_ready()


@pytest.mark.parametrize(
    "cls, args, period",
    [
        (ta.SMA, (14,), 14),
        (ta.EMA, (14,), 14),
        (ta.WMA, (14,), 14),
        (ta.RSI, (14,), 15),
        (ta.BollingerBands, (20, 2.0), 20),
    ],
)
def test_warmup_period(cls, args, period):
    assert cls(*args).warmup_period() == period


def test_repr_contains_class_and_parameters():
    assert "SMA" in repr(ta.SMA(14))
    assert "14" in repr(ta.SMA(14))
    assert "BollingerBands" in repr(ta.BollingerBands(20, 2.0))


def test_constructor_rejects_zero_period():
    with pytest.raises(ValueError):
        ta.SMA(0)
    with pytest.raises(ValueError):
        ta.RSI(0)


def test_macd_rejects_fast_geq_slow():
    with pytest.raises(ValueError):
        ta.MACD(fast=26, slow=12, signal=9)


def test_bollinger_rejects_non_positive_multiplier():
    with pytest.raises(ValueError):
        ta.BollingerBands(20, 0.0)
    with pytest.raises(ValueError):
        ta.BollingerBands(20, -1.0)


def test_candle_dict_input_supported():
    atr = ta.ATR(2)
    atr.update({"open": 10.0, "high": 11.0, "low": 9.0, "close": 10.5, "volume": 1.0})
    v = atr.update({"open": 10.5, "high": 12.0, "low": 10.0, "close": 11.0, "volume": 1.0})
    assert v is not None


def test_candle_tuple_input_supported():
    atr = ta.ATR(2)
    atr.update((10.0, 11.0, 9.0, 10.5, 1.0, 0))
    v = atr.update((10.5, 12.0, 10.0, 11.0, 1.0, 1))
    assert v is not None
