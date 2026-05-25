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
    "Vortex": (
        lambda: ta.Vortex(14),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
        2,
    ),
    "SuperTrend": (
        lambda: ta.SuperTrend(10, 3.0),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
        2,
    ),
    "ChandelierExit": (
        lambda: ta.ChandelierExit(22, 3.0),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
        2,
    ),
    "ChandeKrollStop": (
        lambda: ta.ChandeKrollStop(10, 1.0, 9),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
        2,
    ),
    "ClassicPivots": (
        lambda: ta.ClassicPivots(),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
        7,
    ),
    "FibonacciPivots": (
        lambda: ta.FibonacciPivots(),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
        7,
    ),
    "Camarilla": (
        lambda: ta.Camarilla(),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
        9,
    ),
    "WoodiePivots": (
        lambda: ta.WoodiePivots(),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
        5,
    ),
    "DemarkPivots": (
        # batch needs open; pass close in for open since the synthetic OHLCV
        # streaming feeds open=close as well.
        lambda: ta.DemarkPivots(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
        3,
    ),
    "WilliamsFractals": (
        lambda: ta.WilliamsFractals(),
        lambda ind, h, l, c, v: ind.batch(h, l),
        2,
    ),
    "ZigZag": (
        lambda: ta.ZigZag(0.02),
        lambda ind, h, l, c, v: ind.batch(h, l),
        2,
    ),
}


@pytest.mark.parametrize("name", list(MULTI))
def test_multi_streaming_matches_batch(name, ohlcv):
    high, low, close, volume = ohlcv
    make, batch_call, k = MULTI[name]

    batch = batch_call(make(), high, low, close, volume)
    assert batch.shape == (close.size, k)

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
        if v is None:
            rows.append([math.nan] * k)
        else:
            # WilliamsFractals returns (Optional[float], Optional[float]); the
            # batch helper writes NaN for None. Normalise tuple/list entries
            # the same way before the equality check.
            rows.append([math.nan if x is None else float(x) for x in v])
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


def test_classic_pivots_reference():
    # H=110, L=90, C=105 -> PP = 305/3, R1 = 2·PP − L, S1 = 2·PP − H.
    cp = ta.ClassicPivots()
    pp, r1, r2, r3, s1, s2, s3 = cp.update((105.0, 110.0, 90.0, 105.0, 1.0, 0))
    expected_pp = 305.0 / 3.0
    assert pp == pytest.approx(expected_pp)
    assert r1 == pytest.approx(2 * expected_pp - 90.0)
    assert s1 == pytest.approx(2 * expected_pp - 110.0)
    assert r2 == pytest.approx(expected_pp + 20.0)
    assert s2 == pytest.approx(expected_pp - 20.0)
    assert r3 > r2 and s3 < s2


def test_fibonacci_pivots_reference():
    # H=110, L=90, C=100 -> PP=100, range=20, R1=PP+0.382·range, etc.
    fp = ta.FibonacciPivots()
    pp, r1, r2, r3, s1, s2, s3 = fp.update((100.0, 110.0, 90.0, 100.0, 1.0, 0))
    assert pp == pytest.approx(100.0)
    assert r1 == pytest.approx(100.0 + 0.382 * 20.0)
    assert r2 == pytest.approx(100.0 + 0.618 * 20.0)
    assert r3 == pytest.approx(100.0 + 20.0)
    assert s1 == pytest.approx(100.0 - 0.382 * 20.0)
    assert s3 == pytest.approx(100.0 - 20.0)


def test_camarilla_pivots_reference():
    # H=110, L=90, C=105 -> R4 = C + range · 1.1 / 2 = 105 + 11 = 116.
    cm = ta.Camarilla()
    pp, r1, r2, r3, r4, s1, s2, s3, s4 = cm.update((105.0, 110.0, 90.0, 105.0, 1.0, 0))
    range_ = 20.0
    assert r1 == pytest.approx(105.0 + range_ * 1.1 / 12.0)
    assert r4 == pytest.approx(105.0 + range_ * 1.1 / 2.0)
    assert s4 == pytest.approx(105.0 - range_ * 1.1 / 2.0)
    assert pp == pytest.approx(305.0 / 3.0)
    # Strict widening with index.
    assert r4 > r3 > r2 > r1
    assert s4 < s3 < s2 < s1


def test_woodie_pivots_reference():
    # H=110, L=90, C=108 -> PP = (110 + 90 + 216) / 4 = 104.
    wp = ta.WoodiePivots()
    pp, r1, r2, s1, s2 = wp.update((108.0, 110.0, 90.0, 108.0, 1.0, 0))
    assert pp == pytest.approx(104.0)
    assert r1 == pytest.approx(2 * 104.0 - 90.0)
    assert s1 == pytest.approx(2 * 104.0 - 110.0)
    assert r2 == pytest.approx(104.0 + 20.0)
    assert s2 == pytest.approx(104.0 - 20.0)


def test_demark_pivots_up_bar_reference():
    # Up bar: O=100, H=120, L=80, C=110 -> X = H + 2L + C = 390, PP = 97.5.
    dp = ta.DemarkPivots()
    pp, r1, s1 = dp.update((100.0, 120.0, 80.0, 110.0, 1.0, 0))
    assert pp == pytest.approx(97.5)
    assert r1 == pytest.approx(195.0 - 80.0)
    assert s1 == pytest.approx(195.0 - 120.0)


def test_williams_fractals_isolated_peak():
    # Highs 1, 2, 5, 2, 1; the centre is strictly above both neighbours.
    wf = ta.WilliamsFractals()
    last = None
    for i, h in enumerate([1.0, 2.0, 5.0, 2.0, 1.0]):
        last = wf.update((h, h, h - 0.5, h, 1.0, i))
    assert last is not None
    up, down = last
    assert up == pytest.approx(5.0)
    assert down is None


def test_zigzag_confirms_after_threshold_reversal():
    # 100 -> 120 (uptrend pivot) -> 100 (16.7% drop confirms swing high).
    zz = ta.ZigZag(0.10)
    assert zz.update((100.0, 100.5, 99.5, 100.0, 1.0, 0)) is None
    assert zz.update((120.0, 120.5, 119.5, 120.0, 1.0, 1)) is None
    confirmed = zz.update((100.0, 100.5, 99.5, 100.0, 1.0, 2))
    assert confirmed is not None
    swing, direction = confirmed
    assert swing == pytest.approx(120.5)
    assert direction == 1.0


# --- Lifecycle ------------------------------------------------------------


def test_new_indicators_expose_lifecycle():
    instances = [make() for make, _ in CANDLE_SCALAR.values()]
    instances += [t[0]() for t in MULTI.values()]
    instances += [cls(*args) for cls, args in SCALAR]
    for ind in instances:
        assert ind.is_ready() is False
        assert ind.warmup_period() >= 1
        ind.reset()
        assert ind.is_ready() is False
