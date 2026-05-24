"""Wickra: streaming-first technical indicators.

Every indicator is available both in streaming mode (call ``update(value)`` per
new data point) and batch mode (call ``batch(numpy_array)`` over a full series).
Warmup positions in batch output are returned as ``NaN`` so the shape always
matches the input.

Example::

    import numpy as np
    import wickra as ta

    prices = np.linspace(100, 200, 1000)
    rsi = ta.RSI(14)
    values = rsi.batch(prices)        # numpy array, NaN during warmup

    # Or streaming:
    rsi = ta.RSI(14)
    for p in prices:
        v = rsi.update(p)             # None during warmup, then float

"""

from __future__ import annotations

from ._wickra import (
    __version__,
    # Trend
    SMA,
    EMA,
    WMA,
    DEMA,
    TEMA,
    HMA,
    KAMA,
    SMMA,
    TRIMA,
    ZLEMA,
    T3,
    VWMA,
    # Momentum
    RSI,
    MACD,
    Stochastic,
    CCI,
    ROC,
    WilliamsR,
    ADX,
    MFI,
    TRIX,
    AwesomeOscillator,
    Aroon,
    MOM,
    CMO,
    TSI,
    PMO,
    StochRSI,
    UltimateOscillator,
    APO,
    AwesomeOscillatorHistogram,
    CFO,
    ZeroLagMACD,
    ElderImpulse,
    STC,
    PPO,
    DPO,
    Coppock,
    AroonOscillator,
    Vortex,
    MassIndex,
    AcceleratorOscillator,
    BalanceOfPower,
    ChoppinessIndex,
    VerticalHorizontalFilter,
    # Volatility
    BollingerBands,
    ATR,
    Keltner,
    Donchian,
    PSAR,
    NATR,
    StdDev,
    UlcerIndex,
    HistoricalVolatility,
    BollingerBandwidth,
    PercentB,
    SuperTrend,
    ChandelierExit,
    ChandeKrollStop,
    AtrTrailingStop,
    TrueRange,
    ChaikinVolatility,
    # Volume
    OBV,
    VWAP,
    RollingVWAP,
    ADL,
    VolumePriceTrend,
    ChaikinMoneyFlow,
    ChaikinOscillator,
    ForceIndex,
    EaseOfMovement,
    # Statistics
    TypicalPrice,
    MedianPrice,
    WeightedClose,
    LinearRegression,
    LinRegSlope,
    ZScore,
    LinRegAngle,
)

__all__ = [
    "__version__",
    # Trend
    "SMA",
    "EMA",
    "WMA",
    "DEMA",
    "TEMA",
    "HMA",
    "KAMA",
    "SMMA",
    "TRIMA",
    "ZLEMA",
    "T3",
    "VWMA",
    # Momentum
    "RSI",
    "MACD",
    "Stochastic",
    "CCI",
    "ROC",
    "WilliamsR",
    "ADX",
    "MFI",
    "TRIX",
    "AwesomeOscillator",
    "Aroon",
    "MOM",
    "CMO",
    "TSI",
    "PMO",
    "StochRSI",
    "UltimateOscillator",
    "APO",
    "AwesomeOscillatorHistogram",
    "CFO",
    "ZeroLagMACD",
    "ElderImpulse",
    "STC",
    "PPO",
    "DPO",
    "Coppock",
    "AroonOscillator",
    "Vortex",
    "MassIndex",
    "AcceleratorOscillator",
    "BalanceOfPower",
    "ChoppinessIndex",
    "VerticalHorizontalFilter",
    # Volatility
    "BollingerBands",
    "ATR",
    "Keltner",
    "Donchian",
    "PSAR",
    "NATR",
    "StdDev",
    "UlcerIndex",
    "HistoricalVolatility",
    "BollingerBandwidth",
    "PercentB",
    "SuperTrend",
    "ChandelierExit",
    "ChandeKrollStop",
    "AtrTrailingStop",
    "TrueRange",
    "ChaikinVolatility",
    # Volume
    "OBV",
    "VWAP",
    "RollingVWAP",
    "ADL",
    "VolumePriceTrend",
    "ChaikinMoneyFlow",
    "ChaikinOscillator",
    "ForceIndex",
    "EaseOfMovement",
    # Statistics
    "TypicalPrice",
    "MedianPrice",
    "WeightedClose",
    "LinearRegression",
    "LinRegSlope",
    "ZScore",
    "LinRegAngle",
]
