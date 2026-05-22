# Wickra — Python bindings

Streaming-first technical indicators powered by a Rust core.

```bash
pip install wickra
```

## Quick start

```python
import numpy as np
import wickra as ta

# Batch — TA-Lib-style usage
prices = np.linspace(100, 200, 1000)
rsi = ta.RSI(14).batch(prices)            # NumPy array; NaN during warmup

# Streaming — feed ticks one at a time
rsi = ta.RSI(14)
for price in live_prices:
    v = rsi.update(price)                 # O(1) per tick
    if v is not None and v > 70:
        ...
```

## What's included

71 streaming-first indicators across eight families. Every one passes a
`batch == streaming` equivalence test and reference-value tests:

- **Moving Averages** — SMA, EMA, WMA, DEMA, TEMA, HMA, KAMA, SMMA, TRIMA,
  ZLEMA, T3, VWMA
- **Momentum Oscillators** — RSI (Wilder), Stochastic, CCI, ROC, Williams %R,
  MFI, Awesome Oscillator, MOM, CMO, TSI, PMO, StochRSI, Ultimate Oscillator
- **Trend & Directional** — MACD, ADX (+DI/-DI), Aroon, TRIX, Aroon
  Oscillator, Vortex, Mass Index, Choppiness Index, Vertical Horizontal Filter
- **Price Oscillators** — PPO, DPO, Coppock, Accelerator Oscillator, Balance
  of Power
- **Volatility & Bands** — ATR, Bollinger Bands, Keltner Channels, Donchian
  Channels, NATR, StdDev, Ulcer Index, Historical Volatility, Bollinger
  Bandwidth, %B, True Range, Chaikin Volatility
- **Trailing Stops** — Parabolic SAR, SuperTrend, Chandelier Exit, Chande
  Kroll Stop, ATR Trailing Stop
- **Volume** — OBV, VWAP (cumulative + rolling), ADL, Volume-Price Trend,
  Chaikin Money Flow, Chaikin Oscillator, Force Index, Ease of Movement
- **Price Statistics** — Typical Price, Median Price, Weighted Close, Linear
  Regression, Linear Regression Slope, Z-Score, Linear Regression Angle

## Why streaming-first matters

Classic TA libraries are batch-only: every live tick triggers a full
recomputation over the entire history. Wickra updates indicator state in
O(1) per tick. On a 5K-bar history the streaming RSI gap is ~17× over the
nearest peer with a streaming API and 100×+ over batch-only libraries.

## Full project

See <https://github.com/kingchenc/wickra> for benchmarks, the Rust core,
Node.js and WebAssembly bindings, examples, and CI.

## License

Licensed under the **PolyForm Noncommercial License 1.0.0**. Personal,
research, educational, and non-profit use are all permitted. Commercial
sale requires a separate license — contact via the GitHub repo.
