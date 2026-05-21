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

25 streaming-first indicators across four families. Every one passes a
`batch == streaming` equivalence test and reference-value tests:

- **Trend** — SMA, EMA, WMA, DEMA, TEMA, HMA, KAMA
- **Momentum** — RSI (Wilder), MACD, Stochastic, CCI, ROC, WilliamsR, ADX,
  MFI, TRIX, AwesomeOscillator, Aroon
- **Volatility** — BollingerBands, ATR, Keltner, Donchian, PSAR
- **Volume** — OBV, VWAP

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
