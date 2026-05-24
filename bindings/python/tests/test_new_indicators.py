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


# Family 05 band/channel indicators with scalar input and multi-output.
# `cols` is the expected number of band columns from `batch`.
SCALAR_MULTI = {
    "MaEnvelope": (lambda: ta.MaEnvelope(20, 0.025), 3),
    "LinRegChannel": (lambda: ta.LinRegChannel(20, 2.0), 3),
    "StandardErrorBands": (lambda: ta.StandardErrorBands(21, 2.0), 3),
    "DoubleBollinger": (lambda: ta.DoubleBollinger(20, 1.0, 2.0), 5),
}


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
    # Family 05 candle-input bands. Each entry is
    # `(factory, batch_call, output_arity, streaming_fields)` where
    # `streaming_fields` is the tuple shape returned by `update(...)`.
    "TtmSqueeze": (
        lambda: ta.TtmSqueeze(20, 2.0, 1.5),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "FractalChaosBands": (
        lambda: ta.FractalChaosBands(2),
        lambda ind, h, l, c, v: ind.batch(h, l),
    ),
}


# Bands with 3 outputs upper/middle/lower from a candle (h, l, c).
HLC_BAND3 = {
    "AccelerationBands": lambda: ta.AccelerationBands(20, 0.001),
    "StarcBands": lambda: ta.StarcBands(6, 15, 2.0),
    "AtrBands": lambda: ta.AtrBands(14, 3.0),
    "HurstChannel": lambda: ta.HurstChannel(10, 0.5),
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


# --- Family 05: scalar-input multi-output band/channel indicators ----------


@pytest.mark.parametrize("name", list(SCALAR_MULTI))
def test_scalar_multi_streaming_matches_batch(name, sine_prices):
    make, cols = SCALAR_MULTI[name]
    batch = make().batch(sine_prices)
    assert batch.shape == (sine_prices.size, cols)

    streamer = make()
    rows = []
    for p in sine_prices:
        v = streamer.update(float(p))
        rows.append([math.nan] * cols if v is None else list(v))
    assert _eq_nan(batch, np.array(rows, dtype=np.float64)), f"{name} mismatch"


# --- Family 05: 3-band candle-input indicators ------------------------------


@pytest.mark.parametrize("name", list(HLC_BAND3))
def test_hlc_band3_streaming_matches_batch(name, ohlcv):
    high, low, close, _ = ohlcv
    make = HLC_BAND3[name]
    batch = make().batch(high, low, close)
    assert batch.shape == (close.size, 3)

    streamer = make()
    rows = []
    for i in range(close.size):
        candle = (
            float(close[i]),
            float(high[i]),
            float(low[i]),
            float(close[i]),
            1.0,
            i,
        )
        v = streamer.update(candle)
        rows.append([math.nan] * 3 if v is None else list(v))
    assert _eq_nan(batch, np.array(rows, dtype=np.float64)), f"{name} mismatch"


# --- VWAP StdDev Bands (4 outputs, needs volume) ----------------------------


def test_vwap_stddev_bands_streaming_matches_batch(ohlcv):
    high, low, close, volume = ohlcv
    batch = ta.VwapStdDevBands(2.0).batch(high, low, close, volume)
    assert batch.shape == (close.size, 4)

    streamer = ta.VwapStdDevBands(2.0)
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
        rows.append([math.nan] * 4 if v is None else list(v))
    assert _eq_nan(batch, np.array(rows, dtype=np.float64))


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


# --- Family 05 reference values ---------------------------------------------


def test_ma_envelope_reference():
    # SMA([10, 20, 30]) = 20; with percent = 0.10: upper = 22, lower = 18.
    out = ta.MaEnvelope(3, 0.10).batch(np.array([10.0, 20.0, 30.0]))
    assert math.isnan(out[0, 0]) and math.isnan(out[1, 0])
    assert out[2, 0] == pytest.approx(22.0)  # upper
    assert out[2, 1] == pytest.approx(20.0)  # middle
    assert out[2, 2] == pytest.approx(18.0)  # lower


def test_acceleration_bands_reference():
    # Single bar: high=12, low=8, close=10, factor=0.5, period=1.
    # ratio = 4/20 = 0.2; raw_up = 12·1.1 = 13.2; raw_lo = 8·0.9 = 7.2.
    v = ta.AccelerationBands(1, 0.5).update((10.0, 12.0, 8.0, 10.0, 1.0, 0))
    assert v == pytest.approx((13.2, 10.0, 7.2))


def test_atr_bands_reference():
    # Five identical bars (h=11, l=9, c=10) → ATR=2, close=10, mult=3:
    # upper=16, middle=10, lower=4.
    out = ta.AtrBands(5, 3.0).batch(
        np.array([11.0] * 5), np.array([9.0] * 5), np.array([10.0] * 5)
    )
    assert math.isnan(out[3, 0])
    assert out[4, 0] == pytest.approx(16.0)
    assert out[4, 1] == pytest.approx(10.0)
    assert out[4, 2] == pytest.approx(4.0)


def test_hurst_channel_reference():
    # Five identical (h=12, l=8, c=10): SMA(close)=10, range=4, mult=0.5.
    out = ta.HurstChannel(5, 0.5).batch(
        np.array([12.0] * 5), np.array([8.0] * 5), np.array([10.0] * 5)
    )
    assert out[4, 0] == pytest.approx(12.0)
    assert out[4, 1] == pytest.approx(10.0)
    assert out[4, 2] == pytest.approx(8.0)


def test_linreg_channel_reference():
    # period 3 over [1, 2, 9]: line y=4x, endpoint=8, residuals=[1, -2, 1],
    # population sigma=sqrt(2); mult=2 → upper=8+2√2, lower=8-2√2.
    out = ta.LinRegChannel(3, 2.0).batch(np.array([1.0, 2.0, 9.0]))
    s = math.sqrt(2.0)
    assert out[2, 0] == pytest.approx(8.0 + 2.0 * s)
    assert out[2, 1] == pytest.approx(8.0)
    assert out[2, 2] == pytest.approx(8.0 - 2.0 * s)


def test_standard_error_bands_reference():
    # Same [1, 2, 9] with n=3: SSE=6, n-2=1, stderr=sqrt(6); mult=2 →
    # upper=8+2√6, lower=8-2√6.
    out = ta.StandardErrorBands(3, 2.0).batch(np.array([1.0, 2.0, 9.0]))
    s = math.sqrt(6.0)
    assert out[2, 0] == pytest.approx(8.0 + 2.0 * s)
    assert out[2, 1] == pytest.approx(8.0)
    assert out[2, 2] == pytest.approx(8.0 - 2.0 * s)


def test_double_bollinger_orders_bands():
    # On a non-trivial dispersion, outer >= inner >= middle >= -inner >= -outer.
    out = ta.DoubleBollinger(5, 1.0, 2.0).batch(
        np.array([1.0, 5.0, 2.0, 4.0, 3.0, 6.0])
    )
    v = out[5]
    assert v[0] >= v[1] >= v[2] >= v[3] >= v[4]


def test_vwap_stddev_bands_reference():
    # Two equal-volume bars with tp=8, tp=12: vwap=10, σ=2, mult=1.5 →
    # upper=13, lower=7.
    v = ta.VwapStdDevBands(1.5)
    v.update((8.0, 8.0, 8.0, 8.0, 1.0, 0))
    out = v.update((12.0, 12.0, 12.0, 12.0, 1.0, 1))
    assert out[0] == pytest.approx(13.0)
    assert out[1] == pytest.approx(10.0)
    assert out[2] == pytest.approx(7.0)
    assert out[3] == pytest.approx(2.0)


def test_ttm_squeeze_flat_market():
    # Zero volatility: BB and KC both collapse to a point → squeeze=1.0,
    # momentum=0.0.
    candles_h = np.array([10.0] * 25)
    out = ta.TtmSqueeze(20, 2.0, 1.5).batch(candles_h, candles_h, candles_h)
    assert out[24, 0] == pytest.approx(1.0)
    assert out[24, 1] == pytest.approx(0.0)


def test_fractal_chaos_bands_detects_peak_and_trough():
    # Sequence that creates one fractal high (i=2) and one low (i=3).
    h = np.array([1.0, 2.0, 5.0, 3.0, 2.0, 1.0, 2.0])
    l = np.array([1.0, 2.0, 3.0, 0.5, 2.0, 1.0, 2.0])
    out = ta.FractalChaosBands(2).batch(h, l)
    # First bar with both bands set is index 5.
    assert math.isnan(out[4, 0])
    assert out[5, 0] == pytest.approx(5.0)
    assert out[5, 1] == pytest.approx(0.5)


# --- Lifecycle ------------------------------------------------------------


def test_new_indicators_expose_lifecycle():
    instances = [make() for make, _ in CANDLE_SCALAR.values()]
    instances += [make() for make, _ in MULTI.values()]
    instances += [cls(*args) for cls, args in SCALAR]
    instances += [make() for make, _ in SCALAR_MULTI.values()]
    instances += [make() for make in HLC_BAND3.values()]
    instances += [ta.VwapStdDevBands(2.0)]
    for ind in instances:
        assert ind.is_ready() is False
        assert ind.warmup_period() >= 1
        ind.reset()
        assert ind.is_ready() is False
