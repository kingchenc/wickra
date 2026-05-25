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


# --- DeMark family ---------------------------------------------------------


def test_td_setup_buy_setup_completes_at_minus_9_uptrend():
    # Strictly rising closes -> every bar has close > close[-4] (sell setup);
    # the streak hits -9 at index 12 and caps there.
    h = np.arange(2.0, 22.0)
    l = h - 1.0
    c = h - 0.5
    out = ta.TDSetup(4, 9).batch(h, l, c)
    assert out[12] == pytest.approx(-9.0)
    assert out[-1] == pytest.approx(-9.0)


def test_td_demarker_downtrend_pegs_at_zero():
    n = 20
    h = np.arange(30.0, 30.0 - n, -1.0)
    l = h - 2.0
    out = ta.TDDeMarker(5).batch(h, l)
    assert out[-1] == pytest.approx(0.0)


def test_td_pressure_pure_bearish_yields_minus_100():
    n = 20
    open_ = np.full(n, 11.0)
    high = np.full(n, 11.0)
    low = np.full(n, 9.0)
    close = np.full(n, 9.0)
    volume = np.full(n, 100.0)
    out = ta.TDPressure(5).batch(open_, high, low, close, volume)
    assert out[-1] == pytest.approx(-100.0)


def test_td_combo_uptrend_completes_to_minus_13():
    # Pure uptrend -> setup completes, then combo conditions (close>=high[-2],
    # high>=prev.high, close>prev.close) all hold for every subsequent bar
    # -> sell combo saturates at -13.
    n = 40
    high = np.arange(1.0, 1.0 + n) + 0.5
    low = high - 1.0
    close = high - 0.5
    out = ta.TDCombo().batch(high, low, close)
    assert out[-1] == pytest.approx(-13.0)


def test_td_countdown_uptrend_completes_to_minus_13():
    n = 40
    high = np.arange(1.0, 1.0 + n) + 0.5
    low = high - 1.0
    close = high - 0.5
    out = ta.TDCountdown().batch(high, low, close)
    assert out[-1] == pytest.approx(-13.0)


def test_td_range_projection_doji_reference():
    # open=close=10, high=12, low=9 -> doji branch.
    # pivot_sum = 12 + 9 + 2*10 = 41; half = 20.5.
    # projHigh = 20.5 - 9 = 11.5; projLow = 20.5 - 12 = 8.5.
    out = ta.TDRangeProjection().batch(
        np.array([10.0]), np.array([12.0]), np.array([9.0]), np.array([10.0])
    )
    assert out[0, 0] == pytest.approx(11.5)
    assert out[0, 1] == pytest.approx(8.5)


def test_td_open_sell_signal_reference():
    # Prev high=12. Curr open=13 > 12, curr low=11 < 12 -> -1.
    td = ta.TDOpen()
    assert td.update((10.0, 12.0, 9.0, 11.0, 1.0, 0)) is None
    assert td.update((13.0, 13.5, 11.0, 11.5, 1.0, 1)) == pytest.approx(-1.0)


def test_td_differential_sell_signal_reference():
    # Prev high=10, low=8, close=9: buying=1, selling=1.
    # Curr high=12, low=9.8, close=10.5: close>prev.close, selling=1.5>1,
    # buying=0.7<1 -> sell signal -1.
    td = ta.TDDifferential()
    assert td.update((9.0, 10.0, 8.0, 9.0, 1.0, 0)) is None
    assert td.update((10.5, 12.0, 9.8, 10.5, 1.0, 1)) == pytest.approx(-1.0)


def test_td_lines_uptrend_support_reference():
    # Strictly rising series -> sell setup completes at idx 12, the
    # lowest low across bars 4..=12 is the low at idx 4 = 4.5.
    n = 20
    high = np.arange(1.0, 1.0 + n) + 0.5
    low = high - 1.0
    close = high - 0.5
    out = ta.TDLines().batch(high, low, close)
    assert math.isnan(out[-1, 0])
    assert out[-1, 1] == pytest.approx(4.5)


def test_td_risk_level_uptrend_sell_risk_reference():
    # Strictly rising series -> sell setup completes at idx 12 with high
    # 13.5 and true range 1.5 -> sell_risk = 13.5 + 1.5 = 15.0.
    n = 20
    high = np.arange(1.0, 1.0 + n) + 0.5
    low = high - 1.0
    close = high - 0.5
    out = ta.TDRiskLevel().batch(high, low, close)
    assert math.isnan(out[-1, 0])
    assert out[-1, 1] == pytest.approx(15.0)
