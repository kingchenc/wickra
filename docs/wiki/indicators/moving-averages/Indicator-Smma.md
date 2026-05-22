# SMMA

> Smoothed Moving Average — Wilder's running moving average (RMA): an
> SMA-seeded exponential average with a slow `1 / period` smoothing factor.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Moving Averages |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded; tracks the input price scale |
| Default parameters | `period` is required (no default in either binding) |
| Warmup period | `period` |
| Interpretation | Heavily smoothed price level; the average underlying Wilder's RSI and ATR. |

## Formula

```
SMMA_period = SMA(price_1 … price_period)                       (seed)
SMMA_t      = (SMMA_{t-1} * (period - 1) + price_t) / period     (t > period)
```

This is algebraically an exponential moving average with smoothing factor
`alpha = 1 / period` — substantially slower than the `Ema` factor of
`2 / (period + 1)` at the same `period`. The recurrence is O(1): each
`update` touches only the previous value.

## Parameters

| Name     | Type    | Default | Valid range | Description |
|----------|---------|---------|-------------|-------------|
| `period` | `usize` | none    | `>= 1`      | Smoothing length. `period = 0` errors with `Error::PeriodZero`. `period = 1` is a pass-through. |

There is no Python `#[pyo3(signature = …)]` default for `SMMA`, so
`wickra.SMMA(period)` requires the period explicitly.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/smma.rs`:

```rust
impl Indicator for Smma {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` (streaming) or a `numpy.ndarray` with `NaN` warmup rows
(batch); Node maps it to `number | null` / `Array<number>` with `NaN`
warmup.

## Warmup

`Smma::new(period).warmup_period() == period`. The first `period - 1`
inputs are buffered while the seed accumulates; the `period`-th `update()`
emits the simple average of those inputs as `SMMA_period`. Every later
input applies the `(prev·(n−1)+x)/n` recurrence.

## Edge cases

- **Constant series.** Feeding `[7.0; n]` returns `Some(7.0)` from input
  `period` onward — the recurrence is a fixed point for constants
  (`constant_series_yields_the_constant` pins this).
- **NaN / infinity inputs.** The first line of `update` is
  `if !input.is_finite() { return self.current; }`. Non-finite inputs are
  **silently dropped** — they neither advance the seed nor perturb the
  recurrence, and the previous valid value (if any) is returned.
- **Reset.** `smma.reset()` clears the seed buffer and the current value,
  restarting the warmup countdown.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Smma};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut smma = Smma::new(3)?;
    let out: Vec<Option<f64>> = smma.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    println!("{:?}", out);
    println!("warmup_period = {}", smma.warmup_period());
    Ok(())
}
```

Output:

```
[None, None, Some(2.0), Some(2.6666666666666665), Some(3.4444444444444446)]
warmup_period = 3
```

The third input emits the seed `(1 + 2 + 3) / 3 = 2.0`; the fourth applies
`(2.0·2 + 4) / 3 = 8/3`; the fifth `(8/3·2 + 5) / 3 = 31/9`. This matches
the `warmup_then_recurrence` test in
`crates/wickra-core/src/indicators/smma.rs`.

### Python

```python
import numpy as np
import wickra as ta

smma = ta.SMMA(3)
print(smma.batch(np.array([1.0, 2.0, 3.0, 4.0, 5.0])))
print("warmup_period =", smma.warmup_period())
```

Output:

```
[      nan       nan 2.        2.6666667 3.4444444]
warmup_period = 3
```

### Node

```javascript
const ta = require('wickra');
const smma = new ta.SMMA(3);
console.log(smma.batch([1, 2, 3, 4, 5]));
console.log('warmupPeriod:', smma.warmupPeriod());
```

Output:

```
[ NaN, NaN, 2, 2.6666666666666665, 3.4444444444444446 ]
warmupPeriod: 3
```

## Interpretation

`Smma` is a very smooth, lag-heavy price level. Because its smoothing
factor is `1 / period` rather than `2 / (period + 1)`, an `Smma(n)` is
roughly as smooth as an `Ema(2n − 1)` — useful when you want maximum
noise rejection from a single line. Its main role in this library,
however, is structural: it is the exact smoothing kernel inside
[`Rsi`](../momentum-oscillators/Indicator-Rsi.md) and [`Atr`](../volatility-bands/Indicator-Atr.md),
so reaching for `Smma` directly lets you reproduce Wilder-style averages
on any series.

## Common pitfalls

- **Confusing it with `Ema` at the same period.** `Smma(n)` and `Ema(n)`
  are *not* interchangeable — `Smma` lags far more. Match `Ema(2n − 1)`
  if you need comparable smoothness.
- **Treating `period = 0` as "use a default".** `Smma::new(0)` returns
  `Err(Error::PeriodZero)` in Rust and a `ValueError` in Python; pass an
  explicit period.

## References

The smoothed moving average is J. Welles Wilder Jr.'s running average
from *New Concepts in Technical Trading Systems* (1978); it is the
averaging step in his RSI, ATR and ADX. The implementation here follows
the standard SMA-seeded formulation, matching TA-Lib's `RMA`.

## See also

- [Indicator-Ema.md](../moving-averages/Indicator-Ema.md) — faster exponential average.
- [Indicator-Sma.md](../moving-averages/Indicator-Sma.md) — the equal-weighted mean used as
  the SMMA seed.
- [Indicator-Trima.md](../moving-averages/Indicator-Trima.md) — the other F1 average.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
