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
    ADX,
    ATR,
    Aroon,
    AwesomeOscillator,
    BollingerBands,
    CCI,
    DEMA,
    Donchian,
    EMA,
    HMA,
    KAMA,
    Keltner,
    MACD,
    MFI,
    OBV,
    PSAR,
    ROC,
    RSI,
    SMA,
    Stochastic,
    TEMA,
    TRIX,
    VWAP,
    WilliamsR,
    WMA,
)

__all__ = [
    "__version__",
    "SMA",
    "EMA",
    "WMA",
    "RSI",
    "MACD",
    "BollingerBands",
    "ATR",
    "Stochastic",
    "OBV",
    "DEMA",
    "TEMA",
    "HMA",
    "KAMA",
    "CCI",
    "ROC",
    "WilliamsR",
    "ADX",
    "MFI",
    "TRIX",
    "PSAR",
    "Keltner",
    "Donchian",
    "VWAP",
    "AwesomeOscillator",
    "Aroon",
]
