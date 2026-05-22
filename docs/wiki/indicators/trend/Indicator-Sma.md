# SMA

> Simple Moving Average — the equal-weighted rolling mean of the last
> `period` closes, maintained as an O(1) rolling-sum state machine.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Trend |
| Sub-category | Simple averages |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded; tracks the input price scale |
| Default parameters | `period` is required (no default in either binding) |
| Warmup period | `period` |
| Interpretation | Smoothed price level; price-vs-SMA crossings flag direction changes. |

## Formula

```
SMA_t = (1 / n) * Σ_{i=0}^{n-1} price_{t-i}
```

where `n = period`. Maintained incrementally as `sum -= window.pop_front();
sum += new_price; out = sum / n`, so `update` is O(1) regardless of
`period`.

## Parameters

| Name     | Type     | Default | Valid range | Description |
|----------|----------|---------|-------------|-------------|
| `period` | `usize`  | none    | `>= 1`      | Length of the rolling window. `period = 0` errors with `Error::PeriodZero`. `period = 1` is a pass-through. |

(There is no Python `#[pyo3(signature = …)]` default for `SMA`, so
`wickra.SMA(period)` requires the period explicitly.)

## Inputs / Outputs

From `crates/wickra-core/src/indicators/sma.rs`:

```rust
impl Indicator for Sma {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. The Python binding maps
this to `float | None` (streaming) or a `numpy.ndarray` of dtype
`float64` with `NaN` for warmup rows (batch). The Node binding maps it to
`number | null` / `Array<number>` with `NaN` for warmup.

## Warmup

`Sma::new(period).warmup_period() == period`. The first non-empty value
is emitted on the `period`-th `update()` call, because the window needs to
hold exactly `period` values before the mean is defined. There is no
seeding step beyond filling the window — `Sma` only ever stores its
running sum and the `VecDeque` of values, so its readiness condition is
literally `window.len() == period`.

## Edge cases

- **Constant series.** Feeding `[7.0; n]` returns `Some(7.0)` from input
  `period` onward; the running-sum bookkeeping is exact for constants
  (the unit test `constant_series_yields_constant_sma` pins this).
- **NaN / infinity inputs.** The first line of `update` is
  `if !input.is_finite() { return self.value(); }`. Non-finite inputs are
  **silently dropped** — they do not advance the window, do not corrupt
  the sum, and the previous valid value (if any) is returned. The unit
  test `ignores_non_finite_input_but_keeps_state` pins this behaviour.
- **Reset.** `sma.reset()` clears the window and the sum, returning the
  indicator to a fresh `is_ready() == false` state. The next `update`
  starts a new warmup countdown.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Sma};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sma = Sma::new(3)?;
    let out: Vec<Option<f64>> = sma.batch(&[2.0, 4.0, 6.0, 8.0, 10.0]);
    println!("{:?}", out);
    println!("warmup_period = {}", sma.warmup_period());
    Ok(())
}
```

Output:

```
[None, None, Some(4.0), Some(6.0), Some(8.0)]
warmup_period = 3
```

The first two inputs return `None` while the window fills; the third
emits `(2 + 4 + 6) / 3 = 4.0` and every subsequent input slides the
window by one. This matches the `known_reference_values` test in
`crates/wickra-core/src/indicators/sma.rs`.

### Python

```python
import numpy as np
import wickra as ta

sma = ta.SMA(3)
print(sma.batch(np.array([2.0, 4.0, 6.0, 8.0, 10.0])))
print("warmup_period =", sma.warmup_period())
```

Output:

```
[nan nan  4.  6.  8.]
warmup_period = 3
```

Warmup rows come back as `NaN` so the result aligns 1:1 with the input
array.

### Node

```javascript
const ta = require('D:/Coding/Wickra/bindings/node');
const sma = new ta.SMA(3);
console.log(sma.batch([2, 4, 6, 8, 10]));
console.log('warmupPeriod:', sma.warmupPeriod());
```

Output:

```
[ NaN, NaN, 4, 6, 8 ]
warmupPeriod: 3
```

## Interpretation

`Sma` is a smoothed price level. The two canonical signals are:

1. **Price–SMA crossover.** Close above the SMA suggests an uptrend, close
   below suggests a downtrend. The longer the SMA, the slower (and more
   trustworthy) the signal.
2. **Two-SMA crossover.** A fast SMA crossing above a slow SMA is the
   classic "golden cross"; below is the "death cross". Either of `Ema`
   or `Hma` will give earlier (but noisier) signals at the same period.

Prefer `Sma` when you want the simplest possible reference price — for
example, as the middle band of [`BollingerBands`](../../Indicators-Overview.md),
which uses an SMA by construction. Prefer `Ema` if you want the same
smoothness profile but slightly less lag on direction changes.

## Common pitfalls

- **Treating `period = 0` as "use a default".** `Sma::new(0)` returns
  `Err(Error::PeriodZero)` in Rust and a `ValueError` in Python; there is
  no implicit default. Pass an explicit period.
- **Slicing batch results with `> warmup_period` instead of
  `~np.isnan(...)`.** In Python the batch output has `NaN` for warmup
  rows; in Rust it has `None`. Use the warmup-aware mask to filter — see
  the [Quickstart: Python](../../Quickstart-Python.md#macd-a-multi-column-indicator-and-its-warmup-nans)
  pattern. Slicing by `prices.size - warmup_period` works for a single
  indicator but breaks the moment you compose two of them via `Chain`.

## References

The simple moving average predates technical analysis as a discipline.
The implementation here follows the standard "rolling sum, slide on each
update" formulation; the matching reference implementations are TA-Lib
and pandas (`rolling(period).mean()`).

## See also

- [Indicator-Ema.md](Indicator-Ema.md) — same smoothness budget, less lag.
- [Indicator-Wma.md](Indicator-Wma.md) — linear weights instead of equal.
- [Indicator-Hma.md](Indicator-Hma.md) — built on three WMAs for near-zero
  lag.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
