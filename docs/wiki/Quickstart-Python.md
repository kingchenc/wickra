# Quickstart: Python

A five-minute tour of the Wickra Python binding. By the end you will have run
a batch RSI over a NumPy array, fed the same indicator one tick at a time,
and read a multi-column MACD result correctly during warmup.

## Install

```bash
pip install wickra
```

The published wheels target Python 3.9 – 3.12 on Linux x86_64, macOS
(Intel + Apple Silicon), and Windows x86_64. The only runtime dependency is
`numpy >= 1.22`. No system compiler, no C headers, no Rust toolchain are
needed to install — Wickra ships pre-built native wheels.

Verify the install:

```python
import wickra as ta
print(ta.__version__)
```

## Batch: RSI over a NumPy array

`Indicator.batch(prices)` takes a 1-D `numpy.ndarray` of `float64` closes and
returns a 1-D `numpy.ndarray` of `float64` outputs. Warmup steps come back as
`NaN` so the result aligns 1:1 with your input prices and slots straight into
a pandas column or a NumPy mask.

The first 15 prices below are the classic Wilder textbook example. RSI(14)
emits its first value at index 14 (the 15th input) because it needs 14
diffs to seed Wilder's smoothing.

```python
import numpy as np
import wickra as ta

prices = np.array([
    44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42,
    45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.00,
    46.03, 46.41, 46.22, 45.64,
], dtype=float)

rsi = ta.RSI(14)
values = rsi.batch(prices)

print(values.dtype, values.shape)
print("warmup count:", int(np.isnan(values).sum()))
print("first value :", float(values[14]))
print("last value  :", float(values[-1]))
```

Running this prints:

```
float64 (20,)
warmup count: 14
first value : 70.46413502109705
last value  : 57.91502067008556
```

The exact first value `70.464` matches Wilder's published table; this is the
same input/output pair the Rust test suite pins as `classic_wilder_textbook_values`
in `crates/wickra-core/src/indicators/rsi.rs`.

## Streaming: feed one price at a time

The same `RSI` instance can be driven tick-by-tick with `update()`. Each call
is O(1) and returns either a `float` or `None` while the indicator is still
warming up.

```python
import wickra as ta

rsi = ta.RSI(14)
prices = [
    44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42,
    45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.00,
    46.03, 46.41,
]

for tick, price in enumerate(prices, start=1):
    value = rsi.update(price)
    if value is not None:
        print(f"tick {tick:2d}  close={price:.2f}  rsi={value:.4f}")
```

Output:

```
tick 15  close=46.28  rsi=70.4641
tick 16  close=46.00  rsi=66.2496
tick 17  close=46.03  rsi=66.4809
tick 18  close=46.41  rsi=69.3469
```

Tick 15 is the first emission because `RSI(14).warmup_period() == 15`. Before
that, `update()` returns `None`. After warmup the indicator never goes back
to `None`: each subsequent tick produces a steady value.

The full set of streaming-state methods is:

| Method                | Returns        | Notes                                      |
|-----------------------|----------------|--------------------------------------------|
| `update(price)`       | `float`/`None` | O(1) state transition, `None` during warmup |
| `batch(prices)`       | `np.ndarray`   | replays `update`, `NaN` during warmup       |
| `reset()`             | `None`         | returns to a freshly-constructed state      |
| `is_ready()`          | `bool`         | `True` once the first value has been emitted |
| `warmup_period()`     | `int`          | inputs required before the first value      |

## MACD: a multi-column indicator and its warmup NaNs

Some indicators emit several values at once. `MACD` returns three: the MACD
line, the signal line, and the histogram. The Python `batch` reflects that
shape directly — instead of a 1-D `float64` vector you get a 2-D
`(n_rows, n_columns)` array, and each warmup row is filled with `NaN` across
every column. (`Stochastic` follows the same pattern with two columns,
`Bollinger Bands` with four, `Keltner`/`Donchian`/`ADX` with three.)

```python
import numpy as np
import wickra as ta

prices = np.linspace(100.0, 120.0, 40)
macd = ta.MACD(12, 26, 9)
out = macd.batch(prices)

print("shape       :", out.shape)
print("warmup rows :", int(np.isnan(out[:, 0]).sum()))
print("first ready :", int(np.argmax(~np.isnan(out[:, 0]))))
print("row 33      :", out[33])
print("row 39      :", out[39])
```

Output:

```
shape       : (40, 3)
warmup rows : 33
first ready : 33
row 33      : [ 3.58974359e+00  3.58974359e+00 -1.77635684e-15]
row 39      : [3.58974359e+00 3.58974359e+00 6.21724894e-15]
```

Two things to notice:

1. `MACD(12, 26, 9).warmup_period()` is `slow + signal - 1 = 34`, and indeed
   row `34 - 1 = 33` is the first row where every column is finite. Earlier
   rows are entirely `NaN`; you should not slice a partial row out and use,
   say, the `signal` column independently of the `macd` column.
2. Columns are positional — `out[:, 0]` is MACD, `out[:, 1]` is signal,
   `out[:, 2]` is histogram. The streaming form returns the same triple as a
   plain Python tuple: `(macd, signal, histogram)`.

To filter a warmup-aware mask cleanly:

```python
ready = ~np.isnan(out[:, 0])
clean_rows = out[ready]
```

`ready` is a single boolean column you can apply to every column at once
because the warmup pattern is identical across all of them.

## A deeper example

`examples/python/backtest.py` in the repo runs a full panel of indicators
(RSI, EMA, Bollinger, MACD, ATR, ADX, OBV) over an OHLCV CSV and prints a
summary. It's a good template for "I have historical data on disk, give me a
table of indicator values" workflows; for live workflows, see
`examples/python/live_trading.py`.

## See also

- [Quickstart: Rust](Quickstart-Rust.md) — same API surface in Rust.
- [Streaming vs Batch](Streaming-vs-Batch.md) — why the streaming path is
  the primary one, not a convenience.
- [Warmup Periods](Warmup-Periods.md) — the full table of warmup counts.
- Source: <https://github.com/kingchenc/wickra>
