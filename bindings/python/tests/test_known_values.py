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


def test_laguerre_rsi_constant_series_stays_at_mid_band():
    # All four Laguerre stages seed to the first input, so subsequent flat
    # inputs keep them equal and the up/down accumulator is 0 — Wickra maps
    # that to the neutral 50.
    out = ta.LaguerreRSI(0.5).batch(np.full(40, 42.0, dtype=np.float64))
    np.testing.assert_allclose(out, 50.0, atol=1e-12)


def test_smi_close_at_centre_yields_zero():
    # Close at the midpoint of a flat high/low range -> displacement is
    # always zero -> SMI converges to 0.
    n = 60
    out = ta.SMI(5, 3, 3).batch(np.full(n, 11.0), np.full(n, 9.0), np.full(n, 10.0))
    # warmup_period = 5 + 3 + 3 - 2 = 9.
    np.testing.assert_allclose(out[8:], 0.0, atol=1e-12)


def test_kst_constant_series_yields_zero():
    # ROC is zero on a flat input, so every RCMA is zero, so KST and its
    # signal SMA are both zero after warmup.
    kst = ta.KST(10, 15, 20, 30, 10, 10, 10, 15, 9)
    out = kst.batch(np.full(80, 42.0, dtype=np.float64))
    warmup = kst.warmup_period()
    # Use NaN-safe comparison on the post-warmup tail.
    tail = out[warmup - 1 :]
    assert np.all(np.isfinite(tail))
    np.testing.assert_allclose(tail, 0.0, atol=1e-12)


def test_pgo_flat_close_yields_zero():
    # On a constant close the numerator (close − SMA) is zero, so PGO emits 0
    # regardless of the TR-EMA in the denominator.
    n = 20
    high = np.full(n, 11.0)
    low = np.full(n, 9.0)
    close = np.full(n, 10.0)
    out = ta.PGO(5).batch(high, low, close)
    assert np.all(np.isnan(out[:4]))
    np.testing.assert_allclose(out[4:], 0.0, atol=1e-12)


def test_rvi_reference_value_period_2():
    # Two bars: (open, high, low, close) = (10, 11, 9, 10.5), (10.5, 11.5, 10, 11).
    #   num = (0.5 + 0.5) = 1.0; den = (2.0 + 1.5) = 3.5; RVI = 1 / 3.5.
    out = ta.RVI(2).batch(
        np.array([10.0, 10.5]),
        np.array([11.0, 11.5]),
        np.array([9.0, 10.0]),
        np.array([10.5, 11.0]),
    )
    assert math.isnan(out[0])
    assert math.isclose(out[1], 1.0 / 3.5, abs_tol=1e-12)


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
