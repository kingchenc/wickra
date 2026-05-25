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


# --- Family 15: Risk / Performance ---------------------------------------


def test_sharpe_ratio_known_window():
    # returns [0.01, 0.02, 0.03, 0.04], rf = 0; mean = 0.025;
    # sample-var = 0.000166...; Sharpe = 0.025 / sqrt(var).
    out = ta.SharpeRatio(4, 0.0).batch(np.array([0.01, 0.02, 0.03, 0.04]))
    expected = 0.025 / math.sqrt(0.000_166_666_666_666_666_67)
    assert math.isclose(out[3], expected, rel_tol=1e-9)


def test_sortino_ratio_known_window():
    # returns [-0.02, 0.01, -0.01, 0.03], mar = 0; mean = 0.0025;
    # downside_sq = 0.0005; dd = sqrt(0.0005/4); Sortino = 0.0025/dd.
    out = ta.SortinoRatio(4, 0.0).batch(np.array([-0.02, 0.01, -0.01, 0.03]))
    expected = 0.0025 / math.sqrt(0.000_125)
    assert math.isclose(out[3], expected, rel_tol=1e-9)


def test_max_drawdown_known_window():
    # window [100, 120, 90] -> peak 120, trough 90 -> 25% drawdown.
    out = ta.MaxDrawdown(3).batch(np.array([100.0, 120.0, 90.0]))
    assert math.isclose(out[2], 0.25, abs_tol=1e-12)


def test_pain_index_known_window():
    # dd[0..2] = 0, 0, 0.25; mean = 0.25/3.
    out = ta.PainIndex(3).batch(np.array([100.0, 120.0, 90.0]))
    assert math.isclose(out[2], 0.25 / 3.0, abs_tol=1e-12)


def test_profit_factor_known_window():
    # gains 0.05, losses 0.03 -> PF = 5/3.
    out = ta.ProfitFactor(4).batch(np.array([0.02, -0.01, 0.03, -0.02]))
    assert math.isclose(out[3], 5.0 / 3.0, rel_tol=1e-9)


def test_gain_loss_ratio_known_window():
    # avg_win 0.03, avg_loss 0.02 -> GLR = 1.5.
    out = ta.GainLossRatio(4).batch(np.array([0.02, -0.01, 0.04, -0.03]))
    assert math.isclose(out[3], 1.5, rel_tol=1e-9)


def test_omega_ratio_known_window():
    # gains 0.04, losses 0.03 -> Omega = 4/3.
    out = ta.OmegaRatio(4, 0.0).batch(np.array([-0.02, 0.01, -0.01, 0.03]))
    assert math.isclose(out[3], 4.0 / 3.0, rel_tol=1e-9)


def test_kelly_criterion_known_window():
    # n_win=n_loss=2, payoff=2 -> Kelly = 0.5 - 0.5/2 = 0.25.
    out = ta.KellyCriterion(4).batch(np.array([0.02, 0.04, -0.01, -0.02]))
    assert math.isclose(out[3], 0.25, rel_tol=1e-9)


def test_drawdown_duration_under_water_counter():
    out = ta.DrawdownDuration().batch(np.array([100.0, 95.0, 90.0, 85.0]))
    np.testing.assert_allclose(out, [0.0, 1.0, 2.0, 3.0])


def test_recovery_factor_known_path():
    # Start 100, peak 110, trough 88 -> max_dd = 0.20; end 130 ->
    # net_return = 0.30 -> Recovery = 1.5.
    prices = np.array([100.0, 110.0, 105.0, 95.0, 88.0, 100.0, 120.0, 130.0])
    out = ta.RecoveryFactor().batch(prices)
    assert math.isclose(out[-1], 1.5, rel_tol=1e-9)


def test_alpha_perfect_capm_fit_yields_zero():
    bench = np.array([0.01 * i for i in range(1, 21)])
    asset = 2.0 * bench
    out = ta.Alpha(20, 0.0).batch(asset, bench)
    assert math.isclose(out[-1], 0.0, abs_tol=1e-12)


def test_alpha_additive_offset_recovered():
    bench = np.array([0.01 * i for i in range(1, 21)])
    asset = bench + 0.005
    out = ta.Alpha(20, 0.0).batch(asset, bench)
    assert math.isclose(out[-1], 0.005, rel_tol=1e-9)


def test_treynor_ratio_known_window():
    bench = np.array([0.01 * i for i in range(1, 21)])
    asset = 2.0 * bench
    out = ta.TreynorRatio(20, 0.0).batch(asset, bench)
    assert math.isclose(out[-1], bench.mean(), rel_tol=1e-9)


def test_information_ratio_known_window():
    asset = np.array([0.02, 0.04, 0.06, 0.08])
    bench = np.array([0.01, 0.02, 0.03, 0.04])
    out = ta.InformationRatio(4).batch(asset, bench)
    expected = 0.025 / math.sqrt(0.000_166_666_666_666_666_67)
    assert math.isclose(out[-1], expected, rel_tol=1e-9)


def test_value_at_risk_known_window():
    # returns -5..4 *0.01; q=0.05*9=0.45 -> -0.0455; VaR = 0.0455.
    returns = np.array([i * 0.01 for i in range(-5, 5)])
    out = ta.ValueAtRisk(10, 0.95).batch(returns)
    assert math.isclose(out[-1], 0.0455, rel_tol=1e-9)


def test_conditional_value_at_risk_known_window():
    # tail = {-0.10}; CVaR = 0.10.
    returns = np.array([i * 0.01 for i in range(-10, 10)])
    out = ta.ConditionalValueAtRisk(20, 0.95).batch(returns)
    assert math.isclose(out[-1], 0.10, rel_tol=1e-9)


def test_calmar_ratio_known_path():
    # returns [0.10, -0.20, 0.05]; equity 1.0->1.10->0.88->0.924;
    # mdd = 0.20; mean = -0.01666...; Calmar = mean / 0.20.
    out = ta.CalmarRatio(3).batch(np.array([0.10, -0.20, 0.05]))
    expected = ((0.10 - 0.20 + 0.05) / 3.0) / 0.20
    assert math.isclose(out[-1], expected, rel_tol=1e-9)


def test_average_drawdown_known_window():
    # window [100, 120, 90, 110]: dd = 0, 0, 0.25, 10/120;
    # mean = (0.25 + 10/120) / 4.
    out = ta.AverageDrawdown(4).batch(np.array([100.0, 120.0, 90.0, 110.0]))
    expected = (0.25 + 10.0 / 120.0) / 4.0
    assert math.isclose(out[-1], expected, rel_tol=1e-12)
