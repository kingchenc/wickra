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
    """Compare two float arrays treating NaN and matching-sign inf positions as equal."""
    a = np.asarray(a, dtype=np.float64)
    b = np.asarray(b, dtype=np.float64)
    if a.shape != b.shape:
        return False
    both_nan = np.isnan(a) & np.isnan(b)
    both_inf_same = np.isinf(a) & np.isinf(b) & (np.sign(a) == np.sign(b))
    skip = both_nan | both_inf_same
    with np.errstate(invalid="ignore"):
        diff = np.abs(a - b)
    return bool(np.all(np.where(skip, 0.0, diff) <= tol))


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
    (ta.ALMA, (9, 0.85, 6.0)),
    (ta.McGinleyDynamic, (10,)),
    (ta.FRAMA, (16,)),
    (ta.VIDYA, (14, 9)),
    (ta.JMA, (14, 0.0, 2)),
    (ta.T3, (5, 0.7)),
    (ta.MOM, (10,)),
    (ta.CMO, (14,)),
    (ta.TSI, (25, 13)),
    (ta.PMO, (35, 20)),
    (ta.TII, (20, 10)),
    (ta.StochRSI, (14, 14)),
    (ta.PPO, (12, 26)),
    (ta.APO, (12, 26)),
    (ta.CFO, (14,)),
    (ta.ElderImpulse, (13, 12, 26, 9)),
    (ta.STC, (23, 50, 10, 0.5)),
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
    (ta.PercentageTrailingStop, (5.0,)),
    (ta.StepTrailingStop, (1.0,)),
    (ta.RenkoTrailingStop, (1.0,)),
    (ta.LaguerreRSI, (0.5,)),
    (ta.ConnorsRSI, (3, 2, 100)),
    (ta.RVIVolatility, (10,)),
    # Family 10 — Ehlers / Cycle scalar indicators
    (ta.SuperSmoother, (10,)),
    (ta.FisherTransform, (10,)),
    (ta.InverseFisherTransform, (1.0,)),
    (ta.Decycler, (20,)),
    (ta.DecyclerOscillator, (10, 30)),
    (ta.RoofingFilter, (10, 48)),
    (ta.CenterOfGravity, (10,)),
    (ta.CyberneticCycle, (10,)),
    (ta.InstantaneousTrendline, (20,)),
    (ta.EhlersStochastic, (20,)),
    (ta.EmpiricalModeDecomposition, (20, 0.5)),
    (ta.HilbertDominantCycle, ()),
    (ta.AdaptiveCycle, ()),
    (ta.SineWave, ()),
    (ta.FAMA, (0.5, 0.05)),
    # Family 12 — Statistik / Regression
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
    # Family 15 — Risk / Performance (scalar f64 input = period return or
    # equity sample).
    (ta.SharpeRatio, (20, 0.0)),
    (ta.SortinoRatio, (20, 0.0)),
    (ta.CalmarRatio, (20,)),
    (ta.OmegaRatio, (20, 0.0)),
    (ta.MaxDrawdown, (20,)),
    (ta.AverageDrawdown, (20,)),
    (ta.DrawdownDuration, ()),
    (ta.PainIndex, (20,)),
    (ta.ValueAtRisk, (20, 0.95)),
    (ta.ConditionalValueAtRisk, (20, 0.95)),
    (ta.ProfitFactor, (20,)),
    (ta.GainLossRatio, (20,)),
    (ta.RecoveryFactor, ()),
    (ta.KellyCriterion, (20,)),
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


# --- Two-series (asset, benchmark) indicators -----------------------------

PAIR = [
    (ta.TreynorRatio, (20, 0.0)),
    (ta.InformationRatio, (20,)),
    (ta.Alpha, (20, 0.0)),
    (ta.PairwiseBeta, (20,)),
    (ta.PairSpreadZScore, (20, 20)),
]


@pytest.mark.parametrize("cls, args", PAIR, ids=[c.__name__ for c, _ in PAIR])
def test_pair_streaming_matches_batch(cls, args, sine_prices):
    asset = np.ascontiguousarray(sine_prices.astype(np.float64))
    bench = np.ascontiguousarray((sine_prices * 0.7 + 0.001).astype(np.float64))
    batch = cls(*args).batch(asset, bench)
    assert batch.shape == asset.shape
    assert batch.dtype == np.float64

    streamer = cls(*args)
    streamed = []
    for a, b in zip(asset, bench):
        v = streamer.update(float(a), float(b))
        streamed.append(math.nan if v is None else float(v))
    assert _eq_nan(batch, np.array(streamed, dtype=np.float64))


def _ll_signal(t):
    return math.sin(t * 0.4) + 0.4 * math.sin(t * 1.1) + 0.2 * math.cos(t * 0.27)


def test_lead_lag_detects_lead():
    n = 60
    a = np.array([_ll_signal(t) for t in range(n)])
    # b is a delayed by 3 ⇒ a leads b ⇒ lag = +3, correlation ≈ 1.
    b = np.array([_ll_signal(t - 3) for t in range(n)])
    out = ta.LeadLagCrossCorrelation(12, 5).batch(a, b)
    assert out.shape == (n, 2)
    assert int(out[-1, 0]) == 3
    assert out[-1, 1] > 0.99


def test_lead_lag_streaming_matches_batch():
    n = 60
    a = np.array([_ll_signal(t) for t in range(n)])
    b = np.array([_ll_signal(t - 2) for t in range(n)])
    ind = ta.LeadLagCrossCorrelation(12, 5)
    batch = ind.batch(a, b)
    streamer = ta.LeadLagCrossCorrelation(12, 5)
    for i in range(n):
        v = streamer.update(float(a[i]), float(b[i]))
        if v is None:
            assert math.isnan(batch[i, 0]) and math.isnan(batch[i, 1])
        else:
            lag, corr = v
            assert int(batch[i, 0]) == lag
            assert math.isclose(batch[i, 1], corr, rel_tol=1e-12, abs_tol=1e-12)


def test_cointegration_detects_mean_reverting_pair():
    n = 80
    b = np.array([50.0 + 0.5 * t for t in range(n)])
    # a tracks 2*b with a small mean-reverting wobble ⇒ cointegrated.
    a = 2.0 * b + 1.0 + 0.5 * np.sin(np.arange(n) * 0.6)
    out = ta.Cointegration(40, 1).batch(a, b)
    assert out.shape == (n, 3)
    assert abs(out[-1, 0] - 2.0) < 0.1  # hedge ratio
    assert out[-1, 2] < -2.0  # ADF statistic: strongly mean-reverting


def test_cointegration_streaming_matches_batch():
    n = 70
    b = np.array([30.0 + 0.7 * t for t in range(n)])
    a = 1.8 * b + 2.0 + 0.5 * np.sin(np.arange(n) * 0.4)
    batch = ta.Cointegration(25, 2).batch(a, b)
    streamer = ta.Cointegration(25, 2)
    for i in range(n):
        v = streamer.update(float(a[i]), float(b[i]))
        if v is None:
            assert np.all(np.isnan(batch[i]))
        else:
            hr, sp, adf = v
            assert math.isclose(batch[i, 0], hr, rel_tol=1e-12, abs_tol=1e-12)
            assert math.isclose(batch[i, 1], sp, rel_tol=1e-12, abs_tol=1e-12)
            assert math.isclose(batch[i, 2], adf, rel_tol=1e-12, abs_tol=1e-12)


def test_relative_strength_constant_ratio():
    n = 30
    a = np.full(n, 200.0)
    b = np.full(n, 100.0)  # ratio is a constant 2
    out = ta.RelativeStrengthAB(5, 5).batch(a, b)
    assert out.shape == (n, 3)
    assert math.isclose(out[-1, 0], 2.0, abs_tol=1e-12)  # ratio
    assert math.isclose(out[-1, 1], 2.0, abs_tol=1e-12)  # ratio MA
    assert math.isclose(out[-1, 2], 50.0, abs_tol=1e-9)  # flat ratio ⇒ RSI 50


def test_relative_strength_streaming_matches_batch():
    n = 60
    tt = np.arange(n)
    a = 100.0 + 5.0 * np.sin(tt * 0.3)
    b = 100.0 + 2.0 * np.cos(tt * 0.2)
    batch = ta.RelativeStrengthAB(10, 14).batch(a, b)
    streamer = ta.RelativeStrengthAB(10, 14)
    for i in range(n):
        v = streamer.update(float(a[i]), float(b[i]))
        if v is None:
            assert np.all(np.isnan(batch[i]))
        else:
            ratio, ma, rsi = v
            assert math.isclose(batch[i, 0], ratio, rel_tol=1e-12, abs_tol=1e-12)
            assert math.isclose(batch[i, 1], ma, rel_tol=1e-12, abs_tol=1e-12)
            assert math.isclose(batch[i, 2], rsi, rel_tol=1e-12, abs_tol=1e-12)


# --- Candle-input, single-output indicators -------------------------------
#
# Each entry is (factory, batch-call). Streaming always feeds the full
# 6-tuple candle; the batch helper takes only the columns it needs.

CANDLE_SCALAR = {
    "VWMA": (lambda: ta.VWMA(20), lambda ind, h, l, c, v: ind.batch(c, v)),
    "RVI": (
        # extract_candle pulls the open price from index 0 of the tuple; the
        # streaming test below already builds candles with open == close, so
        # match that here by passing close as the open column.
        lambda: ta.RVI(10),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "Inertia": (
        lambda: ta.Inertia(14, 20),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "PGO": (lambda: ta.PGO(14), lambda ind, h, l, c, v: ind.batch(h, l, c)),
    "SMI": (lambda: ta.SMI(5, 3, 3), lambda ind, h, l, c, v: ind.batch(h, l, c)),
    "EVWMA": (lambda: ta.EVWMA(20), lambda ind, h, l, c, v: ind.batch(c, v)),
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
    "KVO": (
        lambda: ta.KVO(34, 55),
        lambda ind, h, l, c, v: ind.batch(h, l, c, v),
    ),
    "VolumeOscillator": (
        lambda: ta.VolumeOscillator(14, 28),
        lambda ind, h, l, c, v: ind.batch(v),
    ),
    "NVI": (
        lambda: ta.NVI(),
        lambda ind, h, l, c, v: ind.batch(c, v),
    ),
    "PVI": (
        lambda: ta.PVI(),
        lambda ind, h, l, c, v: ind.batch(c, v),
    ),
    "WilliamsAD": (
        lambda: ta.WilliamsAD(),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "AnchoredVWAP": (
        lambda: ta.AnchoredVWAP(),
        lambda ind, h, l, c, v: ind.batch(h, l, c, v),
    ),
    "DemandIndex": (
        lambda: ta.DemandIndex(10),
        lambda ind, h, l, c, v: ind.batch(h, l, c, v),
    ),
    "TSV": (
        lambda: ta.TSV(18),
        lambda ind, h, l, c, v: ind.batch(c, v),
    ),
    "VZO": (
        lambda: ta.VZO(14),
        lambda ind, h, l, c, v: ind.batch(c, v),
    ),
    "MarketFacilitationIndex": (
        lambda: ta.MarketFacilitationIndex(),
        lambda ind, h, l, c, v: ind.batch(h, l, v),
    ),
    "AtrTrailingStop": (
        lambda: ta.AtrTrailingStop(14, 3.0),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "HiLoActivator": (
        lambda: ta.HiLoActivator(3),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "VoltyStop": (
        lambda: ta.VoltyStop(14, 2.0),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "YoyoExit": (
        lambda: ta.YoyoExit(14, 2.0),
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
    "AwesomeOscillatorHistogram": (
        lambda: ta.AwesomeOscillatorHistogram(5, 34, 5),
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
    "ADXR": (
        lambda: ta.ADXR(7),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "ParkinsonVolatility": (
        lambda: ta.ParkinsonVolatility(20, 252),
        lambda ind, h, l, c, v: ind.batch(h, l),
    ),
    "GarmanKlassVolatility": (
        # The streaming 6-tuple feeds open == close, so batch matches with
        # the close column standing in for open.
        lambda: ta.GarmanKlassVolatility(20, 252),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "RogersSatchellVolatility": (
        lambda: ta.RogersSatchellVolatility(20, 252),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "YangZhangVolatility": (
        lambda: ta.YangZhangVolatility(20, 252),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "TDSetup": (
        lambda: ta.TDSetup(4, 9),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "TDDeMarker": (
        lambda: ta.TDDeMarker(14),
        lambda ind, h, l, c, v: ind.batch(h, l),
    ),
    "TDREI": (
        lambda: ta.TDREI(5),
        lambda ind, h, l, c, v: ind.batch(h, l),
    ),
    "TDCombo": (
        lambda: ta.TDCombo(4, 9, 2, 13),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "TDCountdown": (
        lambda: ta.TDCountdown(4, 9, 2, 13),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    "TDDifferential": (
        lambda: ta.TDDifferential(),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
    ),
    # Candlestick patterns -- batch takes (open, high, low, close) and the
    # streaming side feeds close as open (same convention as BalanceOfPower).
    "Doji": (
        lambda: ta.Doji(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "Hammer": (
        lambda: ta.Hammer(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "InvertedHammer": (
        lambda: ta.InvertedHammer(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "HangingMan": (
        lambda: ta.HangingMan(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "ShootingStar": (
        lambda: ta.ShootingStar(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "Engulfing": (
        lambda: ta.Engulfing(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "Harami": (
        lambda: ta.Harami(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "MorningEveningStar": (
        lambda: ta.MorningEveningStar(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "ThreeSoldiersOrCrows": (
        lambda: ta.ThreeSoldiersOrCrows(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "PiercingDarkCloud": (
        lambda: ta.PiercingDarkCloud(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "Marubozu": (
        lambda: ta.Marubozu(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "Tweezer": (
        lambda: ta.Tweezer(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "SpinningTop": (
        lambda: ta.SpinningTop(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "ThreeInside": (
        lambda: ta.ThreeInside(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
    ),
    "ThreeOutside": (
        lambda: ta.ThreeOutside(),
        lambda ind, h, l, c, v: ind.batch(c, h, l, c),
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
    "RWI": (
        lambda: ta.RWI(14),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
        2,
    ),
    "WaveTrend": (
        lambda: ta.WaveTrend.classic(),
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
    "DonchianStop": (
        lambda: ta.DonchianStop(10),
        lambda ind, h, l, c, v: ind.batch(h, l),
        2,
    ),
    # Family 05 candle-input bands. Each entry is
    # `(factory, batch_call, output_arity)` where the third element is the
    # tuple shape returned by `update(...)`.
    "TtmSqueeze": (
        lambda: ta.TtmSqueeze(20, 2.0, 1.5),
        lambda ind, h, l, c, v: ind.batch(h, l, c),
        2,
    ),
    "FractalChaosBands": (
        lambda: ta.FractalChaosBands(2),
        lambda ind, h, l, c, v: ind.batch(h, l),
        2,
    ),
}


# Bands with 3 outputs upper/middle/lower from a candle (h, l, c).
HLC_BAND3 = {
    "AccelerationBands": lambda: ta.AccelerationBands(20, 0.001),
    "StarcBands": lambda: ta.StarcBands(6, 15, 2.0),
    "AtrBands": lambda: ta.AtrBands(14, 3.0),
    "HurstChannel": lambda: ta.HurstChannel(10, 0.5),
}

# --- Scalar-input, multi-output indicators --------------------------------
#
# Same shape contract as MULTI (batch returns (n, 2)) but streaming feeds a
# single float instead of a candle tuple.

MULTI_SCALAR_INPUT = {
    "KST": (
        lambda: ta.KST(10, 15, 20, 30, 10, 10, 10, 15, 9),
        lambda ind, c: ind.batch(c),
    ),
    "MAMA": (
        lambda: ta.MAMA(0.5, 0.05),
        lambda ind, c: ind.batch(c),
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


@pytest.mark.parametrize("name", list(MULTI_SCALAR_INPUT))
def test_multi_scalar_streaming_matches_batch(name, ohlcv):
    _, _, close, _ = ohlcv
    make, batch_call = MULTI_SCALAR_INPUT[name]

    batch = batch_call(make(), close)
    assert batch.shape == (close.size, 2)

    streamer = make()
    rows = []
    for p in close:
        v = streamer.update(float(p))
        rows.append([math.nan, math.nan] if v is None else list(v))
    assert _eq_nan(batch, np.array(rows, dtype=np.float64)), f"{name} mismatch"


# --- Market Profile (multi-output with variable shapes) ------------------


def test_value_area_shape_and_streaming(ohlcv):
    high, low, _close, volume = ohlcv
    batch = ta.ValueArea(20, 50, 0.70).batch(high, low, volume)
    assert batch.shape == (high.size, 3)
    streamer = ta.ValueArea(20, 50, 0.70)
    rows = []
    for i in range(high.size):
        mid = float((high[i] + low[i]) / 2.0)
        candle = (mid, float(high[i]), float(low[i]), mid, float(volume[i]), i)
        v = streamer.update(candle)
        rows.append([math.nan, math.nan, math.nan] if v is None else list(v))
    assert _eq_nan(batch, np.array(rows, dtype=np.float64))


def test_initial_balance_shape_and_streaming(ohlcv):
    high, low, _close, _volume = ohlcv
    batch = ta.InitialBalance(12).batch(high, low)
    assert batch.shape == (high.size, 2)
    streamer = ta.InitialBalance(12)
    rows = []
    for i in range(high.size):
        mid = float((high[i] + low[i]) / 2.0)
        candle = (mid, float(high[i]), float(low[i]), mid, 0.0, i)
        v = streamer.update(candle)
        rows.append([math.nan, math.nan] if v is None else list(v))
    assert _eq_nan(batch, np.array(rows, dtype=np.float64))


def test_opening_range_shape_and_streaming(ohlcv):
    high, low, close, _volume = ohlcv
    batch = ta.OpeningRange(6).batch(high, low, close)
    assert batch.shape == (high.size, 3)
    streamer = ta.OpeningRange(6)
    rows = []
    for i in range(high.size):
        candle = (
            float(close[i]),
            float(high[i]),
            float(low[i]),
            float(close[i]),
            0.0,
            i,
        )
        v = streamer.update(candle)
        rows.append([math.nan, math.nan, math.nan] if v is None else list(v))
    assert _eq_nan(batch, np.array(rows, dtype=np.float64))


# --- TD Pressure (OHLCV-input) -------------------------------------------


def test_td_pressure_streaming_matches_batch(ohlcv):
    high, low, close, volume = ohlcv
    open_ = close.copy()  # TD Pressure needs open; reuse close as the open column.
    batch = ta.TDPressure(5).batch(open_, high, low, close, volume)
    assert batch.shape == close.shape

    streamer = ta.TDPressure(5)
    streamed = []
    for i in range(close.size):
        candle = (
            float(open_[i]),
            float(high[i]),
            float(low[i]),
            float(close[i]),
            float(volume[i]),
            i,
        )
        v = streamer.update(candle)
        streamed.append(math.nan if v is None else float(v))
    assert _eq_nan(batch, np.array(streamed, dtype=np.float64))


# --- TD Sequential (3-column multi-output) ------------------------------


def test_td_sequential_streaming_matches_batch(ohlcv):
    high, low, close, volume = ohlcv
    batch = ta.TDSequential().batch(high, low, close)
    assert batch.shape == (close.size, 3)

    streamer = ta.TDSequential()
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
        rows.append([math.nan, math.nan, math.nan] if v is None else list(v))
    assert _eq_nan(batch, np.array(rows, dtype=np.float64))


# --- ZeroLagMACD (scalar input, 3-tuple output: macd / signal / histogram) -


def test_zero_lag_macd_streaming_matches_batch(ohlcv):
    _, _, close, _ = ohlcv
    batch = ta.ZeroLagMACD(12, 26, 9).batch(close)
    assert batch.shape == (close.size, 3)

    streamer = ta.ZeroLagMACD(12, 26, 9)
    rows = []
    for p in close:
        v = streamer.update(float(p))
        rows.append([math.nan, math.nan, math.nan] if v is None else list(v))
    assert _eq_nan(batch, np.array(rows, dtype=np.float64)), "ZeroLagMACD mismatch"


# --- Alligator (3-tuple output) -------------------------------------------


def test_alligator_streaming_matches_batch(ohlcv):
    high, low, _, _ = ohlcv
    alligator = ta.Alligator(13, 8, 5)
    batch = alligator.batch(high, low)
    assert batch.shape == (high.size, 3)

    streamer = ta.Alligator(13, 8, 5)
    rows = []
    for i in range(high.size):
        candle = (float(low[i]), float(high[i]), float(low[i]), float(low[i]), 0.0, i)
        v = streamer.update(candle)
        rows.append([math.nan, math.nan, math.nan] if v is None else list(v))
    assert _eq_nan(batch, np.array(rows, dtype=np.float64)), "Alligator mismatch"


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


def test_nvi_reference():
    # closes [10, 11], volumes [200, 100]: volume contracts -> NVI absorbs +10%.
    # 1000 * (1 + 0.1) = 1100.
    nvi = ta.NVI()
    out = nvi.batch(np.array([10.0, 11.0]), np.array([200.0, 100.0]))
    assert out[0] == pytest.approx(1000.0)
    assert out[1] == pytest.approx(1100.0)


def test_pvi_reference():
    # closes [10, 11], volumes [100, 200]: volume expands -> PVI absorbs +10%.
    pvi = ta.PVI()
    out = pvi.batch(np.array([10.0, 11.0]), np.array([100.0, 200.0]))
    assert out[0] == pytest.approx(1000.0)
    assert out[1] == pytest.approx(1100.0)


def test_volume_oscillator_reference():
    # fast=2, slow=4 over volumes [10, 20, 30, 40, 50]:
    # bar 4 -> fast=(30+40)/2=35, slow=(10+20+30+40)/4=25 -> VO = 100*(35-25)/25 = 40.
    vo = ta.VolumeOscillator(2, 4)
    out = vo.batch(np.array([10.0, 20.0, 30.0, 40.0, 50.0]))
    assert math.isnan(out[2])
    assert out[3] == pytest.approx(40.0)
    assert out[4] == pytest.approx(1000.0 / 35.0)


def test_kvo_constant_series_is_zero():
    # A flat series produces dm with no sign change; vf collapses to 0 every
    # bar and both EMAs hold at 0, so the KVO line stays at 0.
    kvo = ta.KVO(3, 6)
    high = np.full(60, 10.0)
    low = np.full(60, 10.0)
    close = np.full(60, 10.0)
    volume = np.full(60, 100.0)
    out = kvo.batch(high, low, close, volume)
    for v in out[~np.isnan(out)]:
        assert v == pytest.approx(0.0, abs=1e-12)


def test_williams_ad_reference():
    # bar 0 seeds prev_close = 10.
    # bar 1: prev=10, today high=13, low=8, close=12 (up day).
    #   TR_l = min(10, 8) = 8 -> delta = 12 - 8 = 4. AD = 4.
    # bar 2: prev=12, today high=11, low=7, close=7 (down day).
    #   TR_h = max(12, 11) = 12 -> delta = 7 - 12 = -5. AD = 4 - 5 = -1.
    ad = ta.WilliamsAD()
    high = np.array([11.0, 13.0, 11.0])
    low = np.array([9.0, 8.0, 7.0])
    close = np.array([10.0, 12.0, 7.0])
    out = ad.batch(high, low, close)
    assert math.isnan(out[0])
    assert out[1] == pytest.approx(4.0)
    assert out[2] == pytest.approx(-1.0)


def test_anchored_vwap_reference():
    # Three flat-OHLC bars: typical_price equals price.
    # 10@1, 20@1, 30@1 -> mean = 20.
    avwap = ta.AnchoredVWAP()
    high = np.array([10.0, 20.0, 30.0])
    low = np.array([10.0, 20.0, 30.0])
    close = np.array([10.0, 20.0, 30.0])
    volume = np.array([1.0, 1.0, 1.0])
    out = avwap.batch(high, low, close, volume)
    assert out[2] == pytest.approx(20.0)


def test_anchored_vwap_set_anchor_clears_window():
    # Drive a few flat bars, re-anchor, then drive a high-priced bar:
    # the new running mean must equal the new bar's typical price.
    avwap = ta.AnchoredVWAP()
    for _ in range(3):
        avwap.update((10.0, 10.0, 10.0, 10.0, 1.0, 0))
    assert avwap.is_ready()
    avwap.set_anchor()
    v = avwap.update((100.0, 100.0, 100.0, 100.0, 5.0, 1))
    assert v == pytest.approx(100.0)


def test_tsv_reference():
    # closes  = [10, 11, 13, 12, 14, 15]
    # volumes = [50, 100, 200, 150,  50, 200]
    # flows   = [None, 1*100=100, 2*200=400, -1*150=-150, 2*50=100, 1*200=200]
    # period=3: first emission at index 3.
    #   bar 3 window=[100,400,-150] -> 350
    #   bar 4 window=[400,-150,100] -> 350
    #   bar 5 window=[-150,100,200] -> 150
    tsv = ta.TSV(3)
    close = np.array([10.0, 11.0, 13.0, 12.0, 14.0, 15.0])
    volume = np.array([50.0, 100.0, 200.0, 150.0, 50.0, 200.0])
    out = tsv.batch(close, volume)
    assert math.isnan(out[0]) and math.isnan(out[1]) and math.isnan(out[2])
    assert out[3] == pytest.approx(350.0)
    assert out[4] == pytest.approx(350.0)
    assert out[5] == pytest.approx(150.0)


def test_vzo_strictly_rising_saturates_to_plus_100():
    # Every bar is an up-day with identical volume -> signed_volume == volume,
    # so the smoothed signed-volume EMA equals the smoothed total-volume EMA,
    # giving a ratio of 1 -> VZO = +100.
    vzo = ta.VZO(5)
    close = np.array([10.0 + i for i in range(60)])
    volume = np.full(60, 100.0)
    out = vzo.batch(close, volume)
    last = out[~np.isnan(out)][-1]
    assert last == pytest.approx(100.0)


def test_market_facilitation_index_reference():
    # (high - low) / volume = (12 - 8) / 200 = 0.02.
    mfi_bw = ta.MarketFacilitationIndex()
    high = np.array([12.0])
    low = np.array([8.0])
    volume = np.array([200.0])
    out = mfi_bw.batch(high, low, volume)
    assert out[0] == pytest.approx(0.02)


def test_demand_index_constant_series_is_zero():
    # Flat close -> pressure = 0 every bar -> EMA stays at 0.
    di = ta.DemandIndex(5)
    high = np.full(60, 10.0)
    low = np.full(60, 10.0)
    close = np.full(60, 10.0)
    volume = np.full(60, 100.0)
    out = di.batch(high, low, close, volume)
    for v in out[~np.isnan(out)]:
        assert v == pytest.approx(0.0, abs=1e-12)


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


def test_wave_trend_flat_market_yields_zero():
    # On a perfectly flat market the flat-tolerance guard keeps both lines
    # at exactly zero (otherwise the ratio ci = (ap - esa) / (0.015 * d)
    # would explode on the first esa ULP).
    out = ta.WaveTrend.classic().batch(
        np.full(80, 10.0), np.full(80, 10.0), np.full(80, 10.0)
    )
    last = out[~np.isnan(out[:, 0])][-1]
    assert last[0] == 0.0
    assert last[1] == 0.0


def test_kst_classic_constants_yield_zero():
    out = ta.KST.classic().batch(np.full(120, 100.0))
    last_row = out[~np.isnan(out[:, 0])][-1]
    assert last_row[0] == pytest.approx(0.0)
    assert last_row[1] == pytest.approx(0.0)


def test_tii_pure_uptrend_saturates_at_100():
    # On a strictly increasing series every close sits above the lagging
    # SMA, so every deviation is positive and TII reaches 100.
    prices = np.arange(80, dtype=np.float64) + 100.0
    out = ta.TII(10, 5).batch(prices)
    last = out[~np.isnan(out)][-1]
    assert last == pytest.approx(100.0)


def test_tii_flat_market_yields_50():
    out = ta.TII(5, 4).batch(np.full(30, 10.0))
    last = out[~np.isnan(out)][-1]
    assert last == 50.0


def test_rwi_reference_uptrend_dominates_low_line():
    # In a pure linear uptrend RWI_High >> RWI_Low.
    n = 60
    base = np.arange(n, dtype=np.float64) * 2.0 + 100.0
    high = base + 1.0
    low = base - 0.5
    close = base + 0.5
    out = ta.RWI(14).batch(high, low, close)
    last_row = out[~np.isnan(out[:, 0])][-1]
    assert last_row[0] > last_row[1], f"RWI_High {last_row[0]} must dominate RWI_Low {last_row[1]}"
    assert last_row[0] > 1.0


def test_adxr_reference_on_pure_uptrend():
    # On a pure linear uptrend ADX saturates at 100, so ADXR (average of two
    # saturated ADX values period-1 bars apart) also reads 100.
    n = 100
    base = np.arange(n, dtype=np.float64) * 2.0 + 100.0
    high = base + 1.0
    low = base - 0.5
    close = base + 0.5
    out = ta.ADXR(5).batch(high, low, close)
    last = out[~np.isnan(out)][-1]
    assert last == pytest.approx(100.0)


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


# --- Family 10 — Ehlers / Cycle ---


def test_mama_batch_shape_and_streaming_equivalence(sine_prices):
    batch = ta.MAMA().batch(sine_prices)
    assert batch.shape == (sine_prices.size, 2)

    streamer = ta.MAMA()
    rows = []
    for p in sine_prices:
        v = streamer.update(float(p))
        rows.append([math.nan, math.nan] if v is None else list(v))
    streamed = np.array(rows, dtype=np.float64)
    assert _eq_nan(batch, streamed)


def test_inverse_fisher_transform_zero_input_yields_zero():
    out = ta.InverseFisherTransform(1.0).batch(np.array([0.0, 0.0, 0.0]))
    np.testing.assert_allclose(out, [0.0, 0.0, 0.0], atol=1e-12)


def test_fisher_transform_flat_series_is_zero():
    # Zero range -> the normaliser yields 0, and tanh(0) chain stays at 0.
    out = ta.FisherTransform(5).batch(np.full(20, 42.0))
    ready = out[~np.isnan(out)]
    assert np.all(np.abs(ready) < 1e-6)


def test_decycler_flat_series_passes_through():
    # High-pass of a flat input is zero, so the decycler equals the input.
    out = ta.Decycler(20).batch(np.full(30, 100.0))
    ready = out[~np.isnan(out)]
    np.testing.assert_allclose(ready, 100.0, atol=1e-9)


def test_center_of_gravity_flat_series_is_zero():
    out = ta.CenterOfGravity(5).batch(np.full(20, 7.0))
    ready = out[~np.isnan(out)]
    np.testing.assert_allclose(ready, 0.0, atol=1e-12)


def test_super_smoother_first_two_outputs_equal_inputs():
    out = ta.SuperSmoother(10).batch(np.array([100.0, 101.0, 102.0]))
    # The 2-pole filter is seeded with raw values for the first two bars.
    assert out[0] == pytest.approx(100.0)
    assert out[1] == pytest.approx(101.0)


def test_td_setup_pure_uptrend_reaches_minus_9():
    # Every close is strictly greater than four bars ago -> sell-setup -9.
    h = np.arange(2.0, 22.0)
    l = h - 1.0
    c = h - 0.5
    out = ta.TDSetup(4, 9).batch(h, l, c)
    # Setup completes at index 12 (warmup is 5 -> first emit at index 4 with
    # value -1, increments to -9 at index 12).
    assert out[12] == pytest.approx(-9.0)


def test_td_demarker_uptrend_pegs_at_one():
    # Strictly higher highs, strictly higher lows -> DeMax > 0, DeMin == 0
    # -> indicator == 1 after warmup.
    h = np.arange(11.0, 31.0)
    l = h - 2.0
    out = ta.TDDeMarker(5).batch(h, l)
    assert out[-1] == pytest.approx(1.0)


def test_td_demarker_flat_market_emits_05():
    # All highs and lows equal -> denominator is zero -> neutral fallback 0.5.
    h = np.full(20, 11.0)
    l = np.full(20, 9.0)
    out = ta.TDDeMarker(5).batch(h, l)
    assert out[-1] == pytest.approx(0.5)


def test_td_pressure_pure_bullish_yields_100():
    # Every bar closes at its high (close == high, open == low) -> per-bar
    # pressure ratio is +1 -> indicator == 100.
    n = 20
    open_ = np.full(n, 9.0)
    high = np.full(n, 11.0)
    low = np.full(n, 9.0)
    close = np.full(n, 11.0)
    volume = np.full(n, 100.0)
    out = ta.TDPressure(5).batch(open_, high, low, close, volume)
    assert out[-1] == pytest.approx(100.0)


def test_td_combo_uptrend_saturates_at_minus_13():
    # Strictly increasing closes -> sell setup completes; combo conditions
    # (close >= high[i-2], high[i] >= high[i-1], close > close[i-1]) all
    # hold so combo saturates at -13.
    n = 40
    high = np.arange(1.0, 1.0 + n) + 0.5
    low = high - 1.0
    close = high - 0.5
    out = ta.TDCombo().batch(high, low, close)
    assert out[-1] == pytest.approx(-13.0)


def test_td_countdown_uptrend_saturates_at_minus_13():
    n = 40
    high = np.arange(1.0, 1.0 + n) + 0.5
    low = high - 1.0
    close = high - 0.5
    out = ta.TDCountdown().batch(high, low, close)
    assert out[-1] == pytest.approx(-13.0)


def test_td_lines_uptrend_sets_support_at_first_run_low():
    # Strictly rising closes -> sell setup completes at idx 12; the
    # lowest low among the setup bars (idx 4..=12) is the low at idx 4.
    n = 20
    high = np.arange(1.0, 1.0 + n) + 0.5
    low = high - 1.0
    close = high - 0.5
    out = ta.TDLines().batch(high, low, close)
    # support is column 1; resistance is NaN at -1.
    assert math.isnan(out[-1, 0])
    # low at idx 4 = 5 + 0.5 - 1.0 = 4.5.
    assert out[-1, 1] == pytest.approx(4.5)


def test_td_range_projection_bullish_bar_reference():
    # open=10, high=12, low=9, close=11 (close > open) ->
    # pivot_sum = 2*12 + 9 + 11 = 44; half = 22.
    # projHigh = 22 - 9 = 13; projLow = 22 - 12 = 10.
    out = ta.TDRangeProjection().batch(
        np.array([10.0]), np.array([12.0]), np.array([9.0]), np.array([11.0])
    )
    assert out[0, 0] == pytest.approx(13.0)
    assert out[0, 1] == pytest.approx(10.0)


def test_td_differential_buy_signal_reference():
    # Bar 0: high=10, low=8, close=9 -> warmup, returns None.
    # Bar 1: high=9, low=7, close=8.5 -> close < prev.close, more buying
    # pressure (1.5 > 1), less selling pressure (0.5 < 1) -> +1.
    td = ta.TDDifferential()
    assert td.update((9.0, 10.0, 8.0, 9.0, 1.0, 0)) is None
    assert td.update((8.5, 9.0, 7.0, 8.5, 1.0, 1)) == pytest.approx(1.0)


def test_td_open_buy_signal_reference():
    # Prev bar low=10. Curr open=9 < 10, curr high=11 > 10 -> +1.
    td = ta.TDOpen()
    assert td.update((10.0, 11.0, 10.0, 10.5, 1.0, 0)) is None
    assert td.update((9.0, 11.0, 8.5, 9.5, 1.0, 1)) == pytest.approx(1.0)


def test_td_risk_level_uptrend_sets_sell_risk():
    # Strictly rising closes -> sell setup completes at idx 12.
    # The highest high is at idx 12 (= 13.5) with true range 1.5 ->
    # sell_risk = 13.5 + 1.5 = 15.0. Subsequent setups re-ratchet the level
    # so we check the first emission at idx 12.
    n = 20
    high = np.arange(1.0, 1.0 + n) + 0.5
    low = high - 1.0
    close = high - 0.5
    out = ta.TDRiskLevel().batch(high, low, close)
    # buy_risk is column 0; sell_risk is column 1.
    assert math.isnan(out[12, 0])
    assert out[12, 1] == pytest.approx(15.0)


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
    instances += [t[0]() for t in MULTI.values()]
    instances += [make() for make, _ in MULTI_SCALAR_INPUT.values()]
    instances += [cls(*args) for cls, args in SCALAR]
    instances += [make() for make, _ in SCALAR_MULTI.values()]
    instances += [make() for make in HLC_BAND3.values()]
    instances += [ta.VwapStdDevBands(2.0)]
    instances.append(ta.Alligator(13, 8, 5))
    instances.append(ta.ZeroLagMACD(12, 26, 9))
    instances += [
        ta.TDPressure(5),
        ta.TDSequential(),
        ta.TDLines(),
        ta.TDRiskLevel(),
        ta.TDRangeProjection(),
        ta.TDOpen(),
    ]
    for ind in instances:
        assert ind.is_ready() is False
        assert ind.warmup_period() >= 1
        ind.reset()
        assert ind.is_ready() is False


# --- Ichimoku & Heikin-Ashi (Family 13) -----------------------------------


def test_ichimoku_batch_shape_and_warmup(ohlcv):
    high, low, close, _ = ohlcv
    ichi = ta.Ichimoku()
    out = ichi.batch(high, low, close)
    assert out.shape == (close.size, 5)
    # Warmup is 77 for the classic (9, 26, 52, 26) configuration.
    assert ichi.warmup_period() == 77
    # Tenkan emits from bar 9; before that, the entire column is NaN.
    assert np.all(np.isnan(out[:8, 0]))
    assert not math.isnan(out[8, 0])


def test_ichimoku_streaming_matches_batch(ohlcv):
    high, low, close, _ = ohlcv
    batch = ta.Ichimoku().batch(high, low, close)

    streamer = ta.Ichimoku()
    rows = []
    for i in range(close.size):
        candle = (
            float(close[i]),
            float(high[i]),
            float(low[i]),
            float(close[i]),
            0.0,
            i,
        )
        v = streamer.update(candle)
        if v is None:
            rows.append([math.nan] * 5)
        else:
            rows.append([math.nan if x is None else float(x) for x in v])
    assert _eq_nan(batch, np.array(rows, dtype=np.float64))


def test_ichimoku_chikou_is_close_displacement_back():
    # A constant series makes Chikou trivially equal to the close.
    n = 60
    close = np.full(n, 100.0)
    high = close + 1.0
    low = close - 1.0
    out = ta.Ichimoku().batch(high, low, close)
    # Displacement = 26, so chikou is defined from bar 25 onwards.
    for i in range(25, n):
        assert out[i, 4] == pytest.approx(100.0)


def test_heikin_ashi_seed_and_recursion():
    ha = ta.HeikinAshi()
    # Bar 1: ha_open = (open + close) / 2, ha_close = (O+H+L+C)/4.
    first = ha.update((10.0, 12.0, 9.0, 11.0, 0.0, 0))
    assert first == pytest.approx(
        (
            (10.0 + 11.0) / 2.0,
            max(12.0, (10.0 + 11.0) / 2.0, (10.0 + 12.0 + 9.0 + 11.0) / 4.0),
            min(9.0, (10.0 + 11.0) / 2.0, (10.0 + 12.0 + 9.0 + 11.0) / 4.0),
            (10.0 + 12.0 + 9.0 + 11.0) / 4.0,
        )
    )
    # Bar 2: ha_open = (prev_ha_open + prev_ha_close) / 2.
    second = ha.update((11.5, 13.0, 10.5, 12.0, 0.0, 1))
    assert second[0] == pytest.approx((first[0] + first[3]) / 2.0)
    assert second[3] == pytest.approx((11.5 + 13.0 + 10.5 + 12.0) / 4.0)


def test_heikin_ashi_streaming_matches_batch(ohlcv):
    high, low, close, _ = ohlcv
    open_ = (high + low) / 2.0
    batch = ta.HeikinAshi().batch(open_, high, low, close)

    streamer = ta.HeikinAshi()
    rows = []
    for i in range(close.size):
        candle = (
            float(open_[i]),
            float(high[i]),
            float(low[i]),
            float(close[i]),
            0.0,
            i,
        )
        v = streamer.update(candle)
        rows.append([math.nan] * 4 if v is None else list(v))
    assert _eq_nan(batch, np.array(rows, dtype=np.float64))


def test_heikin_ashi_lifecycle_and_reset():
    ha = ta.HeikinAshi()
    assert ha.is_ready() is False
    assert ha.warmup_period() == 1
    ha.update((10.0, 11.0, 9.0, 10.5, 0.0, 0))
    assert ha.is_ready() is True
    ha.reset()
    assert ha.is_ready() is False
# --- Candlestick pattern reference values --------------------------------


def test_doji_reference():
    # body 0, range 2 -> doji.
    assert ta.Doji().update((10.0, 11.0, 9.0, 10.0, 1.0, 0)) == pytest.approx(1.0)
    # Marubozu shape -> not a doji.
    assert ta.Doji().update((10.0, 12.0, 10.0, 12.0, 1.0, 0)) == pytest.approx(0.0)


def test_hammer_reference():
    # body 0.5, lower shadow 5.0, upper 0.1.
    assert ta.Hammer().update((10.0, 10.6, 5.0, 10.5, 1.0, 0)) == pytest.approx(1.0)


def test_inverted_hammer_reference():
    assert ta.InvertedHammer().update(
        (10.0, 15.0, 9.9, 10.5, 1.0, 0)
    ) == pytest.approx(1.0)


def test_hanging_man_reference():
    assert ta.HangingMan().update((10.0, 10.6, 5.0, 10.5, 1.0, 0)) == pytest.approx(
        -1.0
    )


def test_shooting_star_reference():
    assert ta.ShootingStar().update(
        (10.0, 15.0, 9.9, 10.5, 1.0, 0)
    ) == pytest.approx(-1.0)


def test_engulfing_reference():
    e = ta.Engulfing()
    assert e.update((11.0, 11.2, 9.8, 10.0, 1.0, 0)) == pytest.approx(0.0)
    assert e.update((9.5, 12.0, 9.5, 11.5, 1.0, 1)) == pytest.approx(1.0)


def test_harami_reference():
    h = ta.Harami()
    assert h.update((12.0, 12.5, 9.5, 10.0, 1.0, 0)) == pytest.approx(0.0)
    assert h.update((10.5, 11.5, 10.4, 11.0, 1.0, 1)) == pytest.approx(1.0)


def test_morning_evening_star_reference():
    m = ta.MorningEveningStar()
    assert m.update((12.0, 12.2, 9.5, 10.0, 1.0, 0)) == pytest.approx(0.0)
    assert m.update((9.9, 10.1, 9.7, 9.95, 1.0, 1)) == pytest.approx(0.0)
    assert m.update((10.1, 12.0, 10.0, 11.8, 1.0, 2)) == pytest.approx(1.0)


def test_three_soldiers_reference():
    t = ta.ThreeSoldiersOrCrows()
    assert t.update((10.0, 11.5, 9.9, 11.0, 1.0, 0)) == pytest.approx(0.0)
    assert t.update((10.5, 12.5, 10.4, 12.0, 1.0, 1)) == pytest.approx(0.0)
    assert t.update((11.5, 13.5, 11.4, 13.0, 1.0, 2)) == pytest.approx(1.0)


def test_piercing_dark_cloud_reference():
    p = ta.PiercingDarkCloud()
    assert p.update((12.0, 12.5, 10.0, 10.0, 1.0, 0)) == pytest.approx(0.0)
    assert p.update((9.8, 11.8, 9.5, 11.5, 1.0, 1)) == pytest.approx(1.0)


def test_marubozu_reference():
    # Bullish marubozu: open == low, close == high.
    assert ta.Marubozu().update(
        (10.0, 12.0, 10.0, 12.0, 1.0, 0)
    ) == pytest.approx(1.0)
    assert ta.Marubozu().update(
        (12.0, 12.0, 10.0, 10.0, 1.0, 0)
    ) == pytest.approx(-1.0)


def test_tweezer_reference():
    t = ta.Tweezer()
    assert t.update((11.0, 12.0, 9.5, 9.6, 1.0, 0)) == pytest.approx(0.0)
    assert t.update((9.7, 10.5, 9.5, 10.2, 1.0, 1)) == pytest.approx(1.0)


def test_spinning_top_reference():
    # body 0.5, both shadows 3.0, range 6.5 -> body/range ~= 0.077.
    assert ta.SpinningTop().update(
        (10.0, 13.5, 7.0, 10.5, 1.0, 0)
    ) == pytest.approx(1.0)


def test_three_inside_reference():
    t = ta.ThreeInside()
    assert t.update((12.0, 12.5, 9.5, 10.0, 1.0, 0)) == pytest.approx(0.0)
    assert t.update((10.5, 11.5, 10.4, 11.0, 1.0, 1)) == pytest.approx(0.0)
    assert t.update((11.0, 13.0, 10.9, 12.5, 1.0, 2)) == pytest.approx(1.0)


def test_three_outside_reference():
    t = ta.ThreeOutside()
    assert t.update((11.0, 11.2, 9.8, 10.0, 1.0, 0)) == pytest.approx(0.0)
    assert t.update((9.5, 12.0, 9.5, 11.5, 1.0, 1)) == pytest.approx(0.0)
    assert t.update((11.5, 13.0, 11.4, 12.5, 1.0, 2)) == pytest.approx(1.0)


# --- Lifecycle ------------------------------------------------------------


def test_new_indicators_expose_lifecycle():
    instances = [make() for make, _ in CANDLE_SCALAR.values()]
    instances += [make() for make, *_ in MULTI.values()]
    instances += [cls(*args) for cls, args in SCALAR]
    for ind in instances:
        assert ind.is_ready() is False
        assert ind.warmup_period() >= 1
        ind.reset()
        assert ind.is_ready() is False


def _orderbook_snapshots(n: int) -> list:
    """A deterministic varying sequence of order-book snapshots."""
    snaps = []
    for i in range(n):
        bid_sz = 1.0 + (i % 5)
        ask_sz = 1.0 + ((i + 2) % 4)
        snaps.append(
            (
                [100.0, 99.0],
                [bid_sz, 1.0],
                [101.0, 102.0],
                [ask_sz, 1.0],
            )
        )
    return snaps


def test_orderbook_indicators_streaming_equals_batch():
    snaps = _orderbook_snapshots(40)
    for make in (
        ta.OrderBookImbalanceTop1,
        lambda: ta.OrderBookImbalanceTopN(2),
        ta.OrderBookImbalanceFull,
        ta.Microprice,
        ta.QuotedSpread,
    ):
        batch = make().batch(snaps)
        streamer = make()
        streamed = np.array(
            [streamer.update(*snap) for snap in snaps], dtype=np.float64
        )
        assert batch.shape == (len(snaps),)
        assert _eq_nan(batch, streamed)


def test_tradeflow_indicators_streaming_equals_batch():
    n = 40
    price = np.full(n, 100.0)
    size = np.array([1.0 + (i % 5) for i in range(n)], dtype=np.float64)
    is_buy = [i % 2 == 0 for i in range(n)]
    for make in (
        ta.SignedVolume,
        ta.CumulativeVolumeDelta,
        lambda: ta.TradeImbalance(5),
    ):
        batch = make().batch(price, size, is_buy)
        streamer = make()
        streamed = np.array(
            [streamer.update(price[i], size[i], is_buy[i]) for i in range(n)],
            dtype=np.float64,
        )
        assert batch.shape == (n,)
        assert _eq_nan(batch, streamed)
