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
    assert len(out) == len(sine_prices)
    assert out.typecode == "d"


def test_macd_batch_returns_n_by_3(sine_prices):
    out = ta.MACD().batch(sine_prices)
    assert out.shape == (sine_prices.size, 3)


def test_bollinger_batch_returns_n_by_4(sine_prices):
    out = ta.BollingerBands().batch(sine_prices)
    assert out.shape == (sine_prices.size, 4)


def test_atr_batch_shape(ohlc_series):
    high, low, close = ohlc_series
    out = ta.ATR(14).batch(high, low, close)
    assert len(out) == len(close)


def test_stochastic_batch_shape(ohlc_series):
    high, low, close = ohlc_series
    out = ta.Stochastic(14, 3).batch(high, low, close)
    assert out.shape == (close.size, 2)


def test_obv_batch_shape(ohlc_series):
    _, _, close = ohlc_series
    volume = np.ones_like(close)
    out = ta.OBV().batch(close, volume)
    assert len(out) == len(close)


def test_value_area_batch_shape(ohlc_series):
    high, low, close = ohlc_series
    volume = np.ones_like(close)
    out = ta.ValueArea(20, 50, 0.70).batch(high, low, volume)
    assert out.shape == (close.size, 3)


def test_initial_balance_batch_shape(ohlc_series):
    high, low, _close = ohlc_series
    out = ta.InitialBalance(12).batch(high, low)
    assert out.shape == (high.size, 2)


def test_opening_range_batch_shape(ohlc_series):
    high, low, close = ohlc_series
    out = ta.OpeningRange(6).batch(high, low, close)
    assert out.shape == (close.size, 3)


def test_ichimoku_batch_returns_n_by_5(ohlc_series):
    high, low, close = ohlc_series
    out = ta.Ichimoku().batch(high, low, close)
    assert out.shape == (close.size, 5)


def test_heikin_ashi_batch_returns_n_by_4(ohlc_series):
    high, low, close = ohlc_series
    open_ = (high + low) / 2.0
    out = ta.HeikinAshi().batch(open_, high, low, close)
    assert out.shape == (close.size, 4)


def test_ehlers_super_smoother_batch_shape(sine_prices):
    out = ta.SuperSmoother(10).batch(sine_prices)
    assert len(out) == len(sine_prices)


def test_mama_batch_shape(sine_prices):
    out = ta.MAMA().batch(sine_prices)
    assert out.shape == (sine_prices.size, 2)


def test_orderbook_indicators_construct_and_emit():
    # All five order-book indicators accept a four-array snapshot and emit a float.
    snapshot = ([100.0, 99.0], [2.0, 1.0], [101.0, 102.0], [1.0, 1.0])
    indicators = [
        ta.OrderBookImbalanceTop1(),
        ta.OrderBookImbalanceTopN(2),
        ta.OrderBookImbalanceFull(),
        ta.Microprice(),
        ta.QuotedSpread(),
        ta.DepthSlope(),
    ]
    for ind in indicators:
        out = ind.update(*snapshot)
        assert isinstance(out, float)


def test_orderbook_batch_returns_one_value_per_snapshot():
    snapshots = [([100.0], [3.0], [101.0], [1.0])] * 5
    out = ta.OrderBookImbalanceTop1().batch(snapshots)
    assert len(out) == 5
    assert out.typecode == "d"


def test_tradeflow_indicators_construct_and_emit():
    # SignedVolume and CVD emit from the first trade; TradeImbalance(1) too.
    assert isinstance(ta.SignedVolume().update(100.0, 2.0, True), float)
    assert isinstance(ta.CumulativeVolumeDelta().update(100.0, 2.0, True), float)
    assert isinstance(ta.TradeImbalance(1).update(100.0, 2.0, True), float)


def test_tradeflow_batch_returns_one_value_per_trade():
    price = np.full(6, 100.0)
    size = np.array([1.0, 2.0, 3.0, 1.0, 2.0, 3.0])
    is_buy = [True, False, True, False, True, False]
    out = ta.CumulativeVolumeDelta().batch(price, size, is_buy)
    assert len(out) == 6
    assert out.typecode == "d"


def test_price_impact_indicators_construct_and_emit():
    # Price-impact indicators take a trade paired with the prevailing mid.
    assert isinstance(ta.EffectiveSpread().update(100.05, 1.0, True, 100.0), float)
    # RealizedSpread buffers until its horizon elapses.
    assert ta.RealizedSpread(1).update(100.05, 1.0, True, 100.0) is None


def test_price_impact_batch_returns_one_value_per_trade():
    price = np.array([100.05, 99.95, 100.10, 99.90])
    size = np.array([1.0, 2.0, 1.0, 2.0])
    is_buy = [True, False, True, False]
    mid = np.full(4, 100.0)
    for ind in (ta.EffectiveSpread(), ta.RealizedSpread(2), ta.KylesLambda(2)):
        out = ind.batch(price, size, is_buy, mid)
        assert len(out) == 4
        assert out.typecode == "d"


def test_footprint_constructs_and_emits():
    out = ta.Footprint(1.0).update(100.2, 2.0, True)
    assert out.shape == (1, 3)
    assert isinstance(out[0, 0], float)


def test_footprint_batch_returns_list_of_arrays():
    res = ta.Footprint(1.0).batch([100.2, 100.7], [2.0, 3.0], [True, False])
    assert isinstance(res, list)
    assert len(res) == 2
    assert res[-1].shape[1] == 3
