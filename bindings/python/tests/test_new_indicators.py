"""Streaming-vs-batch, shape and reference-value tests for the F1-F12 families.

Every indicator added since the original 25 is exercised here. The central
contract is the same as the rest of the suite: ``batch(...)`` must equal
repeated streaming ``update(...)`` across the whole warmup -> steady-state
transition, and batch shapes must match the input length.
"""

from __future__ import annotations

import math

import numpy as np
import pytest

import wickra as ta


def _eq_nan(a: np.ndarray, b: np.ndarray, tol: float = 1e-9) -> bool:
    """Compare two float arrays treating NaN positions as equal."""
    a = np.asarray(a, dtype=np.float64)
    b = np.asarray(b, dtype=np.float64)
    if a.shape != b.shape:
        return False
    both_nan = np.isnan(a) & np.isnan(b)
    return bool(np.all(np.where(both_nan, 0.0, np.abs(a - b)) <= tol))


@pytest.fixture
def ohlcv() -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    """Synthetic high / low / close / volume series, 200 bars."""
    t = np.arange(200, dtype=np.float64)
    close = 100.0 + np.sin(t * 0.15) * 8.0 + np.cos(t * 0.32) * 3.0
    spread = 0.5 + np.abs(np.sin(t * 0.07))
    high = close + spread
    low = close - spread
    volume = 1000.0 + (t % 7) * 50.0
    return high, low, close, volume


# --- Scalar (f64 -> f64) indicators ---------------------------------------

SCALAR = [
    (ta.SMMA, (14,)),
    (ta.TRIMA, (20,)),
    (ta.ZLEMA, (14,)),
    (ta.T3, (5, 0.7)),
    (ta.MOM, (10,)),
    (ta.CMO, (14,)),
    (ta.TSI, (25, 13)),
    (ta.PMO, (35, 20)),
    (ta.StochRSI, (14, 14)),
    (ta.PPO, (12, 26)),
    (ta.DPO, (20,)),
    (ta.Coppock, (14, 11, 10)),
    (ta.StdDev, (20,)),
    (ta.UlcerIndex, (14,)),
    (ta.HistoricalVolatility, (20, 252)),
    (ta.BollingerBandwidth, (20, 2.0)),
    (ta.PercentB, (20, 2.0)),
    (ta.LinearRegression, (14,)),
    (ta.LinRegSlope, (14,)),
    (ta.VerticalHorizontalFilter, (28,)),
    (ta.ZScore, (20,)),
    (ta.LinRegAngle, (14,)),
    (ta.Variance, (20,)),
    (ta.CoefficientOfVariation, (20,)),
    (ta.Skewness, (20,)),
    (ta.Kurtosis, (20,)),
    (ta.StandardError, (14,)),
    (ta.DetrendedStdDev, (14,)),
    (ta.RSquared, (14,)),
    (ta.MedianAbsoluteDeviation, (20,)),
    (ta.Autocorrelation, (20, 1)),
    (ta.HurstExponent, (40, 4)),
]


@pytest.mark.parametrize("cls, args", SCALAR, ids=[c.__name__ for c, _ in SCALAR])
def test_scalar_streaming_matches_batch(cls, args, sine_prices):
    batch = cls(*args).batch(sine_prices)
    assert batch.shape == sine_prices.shape
    assert batch.dtype == np.float64

    streamer = cls(*args)
    streamed = []
    for p in sine_prices:
        v = streamer.update(float(p))
        streamed.append(math.nan if v is None else float(v))
    assert _eq_nan(batch, np.array(streamed, dtype=np.float64))


# --- Candle-input, single-output indicators -------------------------------
#
# Each entry is (factory, batch-call). Streaming always feeds the full
# 6-tuple candle; the batch helper takes only the columns it needs.

CANDLE_SCALAR = {
    "VWMA": (lambda: ta.VWMA(20), lambda ind, h, l, c, v: ind.batch(c, v)),
    "UltimateOscillator": (
        lambda: ta.UltimateOscillator(7, 14, 28),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "AroonOscillator": (
        lambda: ta.AroonOscillator(14),
        lambda ind, h, l, c, v: ind.batch(h, l),
    ),
    "NATR": (lambda: ta.NATR(14), lambda ind, h, l, c, v: ind.batch(h, l, c)),
    "MassIndex": (lambda: ta.MassIndex(9, 25), lambda ind, h, l, c, v: ind.batch(h, l)),
    "ADL": (lambda: ta.ADL(), lambda ind, h, l, c, v: ind.batch(h, l, c, v)),
    "VolumePriceTrend": (
        lambda: ta.VolumePriceTrend(),
        lambda ind, h, l, c, v: ind.batch(c, v),
    ),
    "ChaikinMoneyFlow": (
        lambda: ta.ChaikinMoneyFlow(20),
        lambda ind, h, l, c, v: ind.batch(h, l, c, v),
    ),
    "ChaikinOscillator": (
        lambda: ta.ChaikinOscillator(3, 10),
        lambda ind, h, l, c, v: ind.batch(h, l, c, v),
    ),
    "ForceIndex": (
        lambda: ta.ForceIndex(13),
        lambda ind, h, l, c, v: ind.batch(c, v),
    ),
    "EaseOfMovement": (
        lambda: ta.EaseOfMovement(14),
        lambda ind, h, l, c, v: ind.batch(h, l, v),
    ),
    "AtrTrailingStop": (
        lambda: ta.AtrTrailingStop(14, 3.0),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "TypicalPrice": (
        lambda: ta.TypicalPrice(),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "MedianPrice": (
        lambda: ta.MedianPrice(),
        lambda ind, h, l, c, v: ind.batch(h, l),
    ),
    "WeightedClose": (
        lambda: ta.WeightedClose(),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "AcceleratorOscillator": (
        lambda: ta.AcceleratorOscillator(5, 34, 5),
        lambda ind, h, l, c, v: ind.batch(h, l),
    ),
    "BalanceOfPower": (
        # The streaming 6-tuple feeds open == close, so batch matches with
        # the close column standing in for open.
        lambda: ta.BalanceOfPower(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "ChoppinessIndex": (
        lambda: ta.ChoppinessIndex(14),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "TrueRange": (
        lambda: ta.TrueRange(),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "ChaikinVolatility": (
        lambda: ta.ChaikinVolatility(10, 10),
        lambda ind, h, l, c, v: ind.batch(h, l),
    ),
}


@pytest.mark.parametrize("name", list(CANDLE_SCALAR))
def test_candle_scalar_streaming_matches_batch(name, ohlcv):
    high, low, close, volume = ohlcv
    make, batch_call = CANDLE_SCALAR[name]

    batch = batch_call(make(), high, low, close, volume)
    assert batch.shape == close.shape

    streamer = make()
    streamed = []
    for i in range(close.size):
        candle = (
            float(close[i]),
            float(high[i]),
            float(low[i]),
            float(close[i]),
            float(volume[i]),
            i,
        )
        v = streamer.update(candle)
        streamed.append(math.nan if v is None else float(v))
    assert _eq_nan(batch, np.array(streamed, dtype=np.float64)), f"{name} mismatch"


# --- Candle-input, multi-output indicators --------------------------------

MULTI = {
    "Vortex": (lambda: ta.Vortex(14), lambda ind, h, l, c, v: ind.batch(h, l, c)),
    "SuperTrend": (
        lambda: ta.SuperTrend(10, 3.0),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "ChandelierExit": (
        lambda: ta.ChandelierExit(22, 3.0),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "ChandeKrollStop": (
        lambda: ta.ChandeKrollStop(10, 1.0, 9),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
}


@pytest.mark.parametrize("name", list(MULTI))
def test_multi_streaming_matches_batch(name, ohlcv):
    high, low, close, volume = ohlcv
    make, batch_call = MULTI[name]

    batch = batch_call(make(), high, low, close, volume)
    assert batch.shape == (close.size, 2)

    streamer = make()
    rows = []
    for i in range(close.size):
        candle = (
            float(close[i]),
            float(high[i]),
            float(low[i]),
            float(close[i]),
            float(volume[i]),
            i,
        )
        v = streamer.update(candle)
        rows.append([math.nan, math.nan] if v is None else list(v))
    assert _eq_nan(batch, np.array(rows, dtype=np.float64)), f"{name} mismatch"


# --- Reference values -----------------------------------------------------


def test_typical_price_reference():
    # (high + low + close) / 3 = (12 + 6 + 9) / 3 = 9.
    assert ta.TypicalPrice().update((9.0, 12.0, 6.0, 9.0, 1.0, 0)) == pytest.approx(9.0)


def test_median_price_reference():
    # (high + low) / 2 = (12 + 8) / 2 = 10.
    assert ta.MedianPrice().update((10.0, 12.0, 8.0, 11.0, 1.0, 0)) == pytest.approx(10.0)


def test_weighted_close_reference():
    # (high + low + 2*close) / 4 = (12 + 8 + 22) / 4 = 10.5.
    assert ta.WeightedClose().update((10.0, 12.0, 8.0, 11.0, 1.0, 0)) == pytest.approx(
        10.5
    )


def test_chaikin_money_flow_reference():
    cmf = ta.ChaikinMoneyFlow(2)
    assert cmf.update((8.0, 10.0, 8.0, 10.0, 100.0, 0)) is None
    assert cmf.update((10.0, 12.0, 8.0, 10.0, 100.0, 1)) == pytest.approx(0.5)


def test_linear_regression_reference():
    out = ta.LinearRegression(3).batch(np.array([1.0, 2.0, 9.0]))
    assert math.isnan(out[0]) and math.isnan(out[1])
    assert out[2] == pytest.approx(8.0)


def test_linreg_slope_reference():
    out = ta.LinRegSlope(3).batch(np.array([1.0, 2.0, 9.0]))
    assert math.isnan(out[0]) and math.isnan(out[1])
    assert out[2] == pytest.approx(4.0)


def test_balance_of_power_reference():
    # (close - open) / (high - low) = (12 - 10) / (14 - 10) = 0.5.
    bop = ta.BalanceOfPower()
    assert bop.update((10.0, 14.0, 10.0, 12.0, 1.0, 0)) == pytest.approx(0.5)


def test_true_range_reference():
    tr = ta.TrueRange()
    assert tr.update((11.0, 12.0, 8.0, 11.0, 1.0, 0)) == pytest.approx(4.0)
    assert tr.update((9.5, 10.0, 9.0, 9.5, 1.0, 1)) == pytest.approx(2.0)


def test_linreg_angle_reference():
    # A series rising by 1 per step has slope 1, and atan(1) = 45 degrees.
    out = ta.LinRegAngle(5).batch(np.array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]))
    assert out[4] == pytest.approx(45.0)


def test_z_score_reference():
    # Window [1, 3]: mean 2, population stddev 1; latest 3 -> z = 1.
    out = ta.ZScore(2).batch(np.array([1.0, 3.0]))
    assert math.isnan(out[0])
    assert out[1] == pytest.approx(1.0)


# --- Family 12: Statistik / Regression reference values ------------------


def test_variance_reference():
    # Variance(3) of [2, 4, 6]: mean 4, variance (4 + 0 + 4) / 3 = 8/3.
    out = ta.Variance(3).batch(np.array([2.0, 4.0, 6.0]))
    assert math.isnan(out[1])
    assert out[2] == pytest.approx(8.0 / 3.0)


def test_coefficient_of_variation_reference():
    # CV(3) of [2, 4, 6]: sd / mean = sqrt(8/3) / 4.
    out = ta.CoefficientOfVariation(3).batch(np.array([2.0, 4.0, 6.0]))
    assert out[2] == pytest.approx(math.sqrt(8.0 / 3.0) / 4.0)


def test_skewness_symmetric_window_is_zero():
    # Symmetric window has zero Pearson skewness.
    out = ta.Skewness(5).batch(np.array([-2.0, -1.0, 0.0, 1.0, 2.0]))
    assert out[4] == pytest.approx(0.0, abs=1e-9)


def test_kurtosis_two_point_distribution_minimum():
    # Alternating {-1, 1} has m4/m2² = 1, so excess kurtosis = -2.
    out = ta.Kurtosis(4).batch(np.array([-1.0, 1.0, -1.0, 1.0]))
    assert out[3] == pytest.approx(-2.0, abs=1e-9)


def test_standard_error_perfect_line_is_zero():
    # Residuals are zero on a perfectly linear series.
    out = ta.StandardError(5).batch(np.linspace(1.0, 20.0, num=20, dtype=np.float64))
    finite = out[~np.isnan(out)]
    assert np.allclose(finite, 0.0, atol=1e-9)


def test_detrended_std_dev_perfect_line_is_zero():
    out = ta.DetrendedStdDev(5).batch(np.linspace(1.0, 20.0, num=20, dtype=np.float64))
    finite = out[~np.isnan(out)]
    assert np.allclose(finite, 0.0, atol=1e-9)


def test_r_squared_perfect_line_is_one():
    out = ta.RSquared(5).batch(np.linspace(1.0, 20.0, num=20, dtype=np.float64))
    finite = out[~np.isnan(out)]
    assert np.allclose(finite, 1.0, atol=1e-9)


def test_median_absolute_deviation_ignores_single_outlier():
    # 9 equal values + 1 huge outlier: MAD is still 0 (more than half agree).
    prices = np.array([5.0] * 9 + [1000.0], dtype=np.float64)
    out = ta.MedianAbsoluteDeviation(10).batch(prices)
    assert out[9] == pytest.approx(0.0, abs=1e-12)


def test_autocorrelation_alternating_series_negative():
    # ±1 alternating: lag-1 ACF must be strongly negative.
    prices = np.array([-1.0 if i % 2 == 0 else 1.0 for i in range(20)], dtype=np.float64)
    out = ta.Autocorrelation(10, 1).batch(prices)
    assert out[-1] < -0.5


def test_hurst_exponent_trending_above_half():
    # A clean monotone ramp is the textbook persistent series.
    prices = np.arange(200, dtype=np.float64)
    out = ta.HurstExponent(100, 4).batch(prices)
    assert out[-1] > 0.5


def test_pearson_correlation_perfect_positive_is_one():
    x = np.arange(10, dtype=np.float64)
    y = 2.0 * x + 3.0
    out = ta.PearsonCorrelation(5).batch(x, y)
    assert out[-1] == pytest.approx(1.0, abs=1e-9)


def test_beta_perfect_two_to_one():
    benchmark = np.arange(10, dtype=np.float64)
    asset = 2.0 * benchmark
    out = ta.Beta(5).batch(asset, benchmark)
    assert out[-1] == pytest.approx(2.0, abs=1e-9)


def test_spearman_correlation_monotone_nonlinear_is_one():
    # y = x^3 is monotone non-linear; Spearman = 1 (Pearson would not be).
    x = np.arange(1.0, 11.0, dtype=np.float64)
    y = x**3
    out = ta.SpearmanCorrelation(5).batch(x, y)
    assert out[-1] == pytest.approx(1.0, abs=1e-9)


def test_pair_indicators_streaming_matches_batch():
    rng = np.linspace(0.0, 10.0, num=60, dtype=np.float64)
    x = np.sin(rng) + 0.1 * rng
    y = np.cos(rng * 0.3) + 0.05 * rng

    for cls, args in [(ta.PearsonCorrelation, (14,)), (ta.Beta, (14,)), (ta.SpearmanCorrelation, (14,))]:
        batch = cls(*args).batch(x, y)
        streamer = cls(*args)
        streamed = []
        for i in range(x.size):
            v = streamer.update(float(x[i]), float(y[i]))
            streamed.append(math.nan if v is None else float(v))
        assert _eq_nan(batch, np.array(streamed, dtype=np.float64)), f"{cls.__name__} mismatch"


# --- Lifecycle ------------------------------------------------------------


def test_new_indicators_expose_lifecycle():
    instances = [make() for make, _ in CANDLE_SCALAR.values()]
    instances += [make() for make, _ in MULTI.values()]
    instances += [cls(*args) for cls, args in SCALAR]
    for ind in instances:
        assert ind.is_ready() is False
        assert ind.warmup_period() >= 1
        ind.reset()
        assert ind.is_ready() is False
