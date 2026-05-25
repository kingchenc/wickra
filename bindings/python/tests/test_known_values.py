"""Reference-value tests that pin numerical behaviour from the Python side."""

from __future__ import annotations

import math

import numpy as np
import pytest

import wickra as ta


def test_sma_constant_series():
    out = ta.SMA(5).batch(np.full(20, 42.0, dtype=np.float64))
    # First 4 are warmup -> NaN; rest equal 42.
    assert np.all(np.isnan(out[:4]))
    assert np.allclose(out[4:], 42.0)


def test_sma_known_window():
    # SMA(3) of [2, 4, 6, 8, 10] -> [_, _, 4, 6, 8]
    out = ta.SMA(3).batch(np.array([2.0, 4.0, 6.0, 8.0, 10.0]))
    assert math.isnan(out[0]) and math.isnan(out[1])
    np.testing.assert_allclose(out[2:], [4.0, 6.0, 8.0])


def test_ema_seed_equals_simple_mean_of_first_window():
    # EMA(5) seed = mean([10, 20, 30, 40, 50]) = 30
    out = ta.EMA(5).batch(np.array([10.0, 20.0, 30.0, 40.0, 50.0]))
    assert math.isnan(out[0])
    assert math.isclose(out[4], 30.0, abs_tol=1e-12)


def test_wma_known_window():
    # WMA(4) of [1, 2, 3, 4] = (1*1 + 2*2 + 3*3 + 4*4)/10 = 3
    out = ta.WMA(4).batch(np.array([1.0, 2.0, 3.0, 4.0]))
    assert math.isnan(out[0]) and math.isnan(out[1]) and math.isnan(out[2])
    assert math.isclose(out[3], 3.0, abs_tol=1e-12)


def test_rsi_pure_uptrend_is_100():
    out = ta.RSI(14).batch(np.arange(1.0, 21.0, dtype=np.float64))
    np.testing.assert_allclose(out[14:], 100.0)


def test_rsi_pure_downtrend_is_0():
    out = ta.RSI(14).batch(np.arange(20.0, 0.0, -1.0))
    np.testing.assert_allclose(out[14:], 0.0)


def test_rsi_flat_series_is_50():
    out = ta.RSI(14).batch(np.full(30, 100.0))
    np.testing.assert_allclose(out[14:], 50.0)


def test_rsi_wilder_textbook_first_value():
    """Wilder's original 14-period example, ~70.46 at the first emit."""
    prices = np.array(
        [
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08,
            45.89, 46.03, 45.61, 46.28, 46.28,
        ],
        dtype=np.float64,
    )
    out = ta.RSI(14).batch(prices)
    assert math.isclose(out[14], 70.464, abs_tol=0.05)


def test_macd_constant_series_converges_to_zero():
    out = ta.MACD().batch(np.full(200, 100.0))
    # Last row's MACD and signal must be ~0.
    last = out[-1]
    assert math.isclose(last[0], 0.0, abs_tol=1e-9)
    assert math.isclose(last[1], 0.0, abs_tol=1e-9)
    assert math.isclose(last[2], 0.0, abs_tol=1e-9)


def test_bollinger_constant_series_zero_width():
    out = ta.BollingerBands(20, 2.0).batch(np.full(50, 100.0))
    row = out[-1]
    np.testing.assert_allclose(row, [100.0, 100.0, 100.0, 0.0], atol=1e-12)


def test_bollinger_upper_middle_lower_ordering():
    out = ta.BollingerBands(20, 2.0).batch(np.linspace(50.0, 150.0, 100))
    ready = out[~np.isnan(out[:, 0])]
    assert np.all(ready[:, 0] >= ready[:, 1])
    assert np.all(ready[:, 1] >= ready[:, 2])
    assert np.all(ready[:, 3] >= 0.0)


def test_atr_constant_range_constant_output():
    high = np.full(30, 11.0)
    low = np.full(30, 9.0)
    close = np.full(30, 10.0)
    out = ta.ATR(14).batch(high, low, close)
    # Once seeded, ATR equals the constant TR of 2.
    np.testing.assert_allclose(out[13:], 2.0, atol=1e-12)


def test_stochastic_extremes():
    # Close at the top of a 3-period range -> %K = 100.
    high = np.array([10.0, 11.0, 12.0])
    low = np.array([8.0, 9.0, 10.0])
    close = np.array([9.0, 10.0, 12.0])
    out = ta.Stochastic(3, 1).batch(high, low, close)
    assert math.isclose(out[2, 0], 100.0, abs_tol=1e-12)


def test_obv_cumulative_known_sequence():
    close = np.array([10.0, 11.0, 10.5, 10.5, 12.0])
    volume = np.array([100.0, 20.0, 30.0, 40.0, 10.0])
    out = ta.OBV().batch(close, volume)
    np.testing.assert_allclose(out, [0.0, 20.0, -10.0, -10.0, 0.0])


def test_percentage_trailing_stop_seed_and_ratchet():
    # 10% trail: first close 100 -> stop 90; next 110 -> stop max(90, 99) = 99.
    s = ta.PercentageTrailingStop(10.0)
    assert math.isclose(s.update(100.0), 90.0, abs_tol=1e-12)
    assert math.isclose(s.update(110.0), 99.0, abs_tol=1e-12)


def test_step_trailing_stop_snaps_below_close():
    # step 1: floor((100.4 - 1) / 1) = 99.
    s = ta.StepTrailingStop(1.0)
    assert math.isclose(s.update(100.4), 99.0, abs_tol=1e-12)


def test_renko_trailing_stop_holds_until_full_block():
    # block 1: seed 100 -> stop 99; 100.5 still 99; 101 -> stop 100.
    s = ta.RenkoTrailingStop(1.0)
    assert math.isclose(s.update(100.0), 99.0, abs_tol=1e-12)
    assert math.isclose(s.update(100.5), 99.0, abs_tol=1e-12)
    assert math.isclose(s.update(101.0), 100.0, abs_tol=1e-12)


def test_donchian_stop_window_extremes():
    # 5-bar window of highs 1..5 and lows 0..4.
    high = np.array([1.0, 2.0, 3.0, 4.0, 5.0])
    low = np.array([0.0, 1.0, 2.0, 3.0, 4.0])
    out = ta.DonchianStop(5).batch(high, low)
    # First 4 rows NaN, fifth row: stop_long = 0, stop_short = 5.
    for i in range(4):
        assert math.isnan(out[i, 0])
        assert math.isnan(out[i, 1])
    assert math.isclose(out[4, 0], 0.0, abs_tol=1e-12)
    assert math.isclose(out[4, 1], 5.0, abs_tol=1e-12)


def test_hilo_activator_flat_market_holds_low_sma():
    # Flat candles H=11, L=9, C=10 -> close (10) sits between bands, so the
    # initial long seed is preserved: emitted stop = lo_sma = 9.
    h = np.full(15, 11.0)
    l = np.full(15, 9.0)
    c = np.full(15, 10.0)
    out = ta.HiLoActivator(3).batch(h, l, c)
    # warmup_period == period + 1 == 4, so indices 0..2 are NaN; index 3 onwards is 9.
    for i in range(3):
        assert math.isnan(out[i])
    for i in range(3, 15):
        assert math.isclose(out[i], 9.0, abs_tol=1e-12)


def test_volty_stop_flat_market_constant_level():
    # ATR=2, mult=2 -> band 4; anchor stays at close 10 -> stop = 10 - 4 = 6.
    h = np.full(20, 11.0)
    l = np.full(20, 9.0)
    c = np.full(20, 10.0)
    out = ta.VoltyStop(5, 2.0).batch(h, l, c)
    for i in range(4):
        assert math.isnan(out[i])
    for i in range(4, 20):
        assert math.isclose(out[i], 6.0, abs_tol=1e-12)


def test_yoyo_exit_flat_market_constant_level():
    # ATR=2, mult=2 -> band 4; trail = close - band = 10 - 4 = 6 and holds.
    h = np.full(20, 11.0)
    l = np.full(20, 9.0)
    c = np.full(20, 10.0)
    out = ta.YoyoExit(5, 2.0).batch(h, l, c)
    for i in range(4):
        assert math.isnan(out[i])
    for i in range(4, 20):
        assert math.isclose(out[i], 6.0, abs_tol=1e-12)
