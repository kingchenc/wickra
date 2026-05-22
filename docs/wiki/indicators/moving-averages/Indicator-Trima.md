# TRIMA

> Triangular Moving Average — a simple moving average applied twice, which
> triangular-weights the window so the middle bars carry the most weight.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Moving Averages |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded; tracks the input price scale |
| Default parameters | `period` is required (no default in either binding) |
| Warmup period | `period` |
| Interpretation | Very smooth price level; the triangular weighting suppresses edge bars. |

## Formula

`TRIMA(n)` is `SMA` stacked on `SMA`. For period `n` the two lengths are:

```
odd  n:  n1 = n2 = (n + 1) / 2
even n:  n1 = n / 2,  n2 = n / 2 + 1
TRIMA_t = SMA_{n2}( SMA_{n1}(price) )_t
```

Composing two equal-weight means convolves two rectangular windows, which
yields a triangular weight profile over the original `n` closes — the
centre bar gets the largest weight, the two edges the smallest. Both
stacked SMAs are O(1), so `update` is O(1) regardless of `period`.

## Parameters

| Name     | Type    | Default | Valid range | Description |
|----------|---------|---------|-------------|-------------|
| `period` | `usize` | none    | `>= 1`      | Window length. `period = 0` errors with `Error::PeriodZero`. `period = 1` and `period = 2` degenerate to short SMAs. |

There is no Python `#[pyo3(signature = …)]` default for `TRIMA`, so
`wickra.TRIMA(period)` requires the period explicitly.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/trima.rs`:

```rust
impl Indicator for Trima {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`Trima::new(period).warmup_period() == period`. The inner SMA emits after
`n1` inputs; the outer SMA then needs `n2 − 1` more, and `n1 + n2 − 1 = n`
for both the odd and even splits. So the first non-`None` output lands on
exactly the `period`-th `update()`.

## Edge cases

- **Constant series.** `[42.0; n]` returns `Some(42.0)` from input
  `period` onward — both SMAs are exact for constants
  (`constant_series_yields_the_constant` pins this).
- **NaN / infinity inputs.** `update` returns `self.outer.value()` for a
  non-finite input *without* feeding either SMA, so the inner SMA's stale
  value is never double-counted into the outer SMA. State is left
  untouched.
- **Reset.** `trima.reset()` resets both inner and outer SMAs, restarting
  the warmup countdown.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Trima};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut trima = Trima::new(5)?;
    let out: Vec<Option<f64>> = trima.batch(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
    println!("{:?}", out);
    println!("warmup_period = {}", trima.warmup_period());
    Ok(())
}
```

Output:

```
[None, None, None, None, Some(3.0), Some(4.0), Some(5.0)]
warmup_period = 5
```

`TRIMA(5)` is `SMA(3)` of `SMA(3)`. `SMA(3)` of `1..=7` is
`[_, _, 2, 3, 4, 5, 6]`; `SMA(3)` of that is `[_, _, _, _, 3, 4, 5]`. This
matches the `odd_period_reference_values` test in
`crates/wickra-core/src/indicators/trima.rs`.

### Python

```python
import numpy as np
import wickra as ta

trima = ta.TRIMA(5)
print(trima.batch(np.array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0])))
print("warmup_period =", trima.warmup_period())
```

Output:

```
[nan nan nan nan  3.  4.  5.]
warmup_period = 5
```

### Node

```javascript
const ta = require('wickra');
const trima = new ta.TRIMA(5);
console.log(trima.batch([1, 2, 3, 4, 5, 6, 7]));
console.log('warmupPeriod:', trima.warmupPeriod());
```

Output:

```
[ NaN, NaN, NaN, NaN, 3, 4, 5 ]
warmupPeriod: 5
```

## Interpretation

`Trima` is one of the smoothest single-line averages in the library: the
triangular weight profile damps the most recent bar far more than a plain
`Sma` does, so whipsaws are rare. The cost is lag — a `Trima(n)` lags
roughly like an `Sma(n/2)` doubled. Use it as a slow trend filter where a
clean, low-noise line matters more than fast reaction; prefer
[`Ema`](../moving-averages/Indicator-Ema.md) or [`Hma`](../moving-averages/Indicator-Hma.md) when responsiveness
matters.

## Common pitfalls

- **Expecting `Sma`-like lag.** Stacking two means roughly doubles the
  effective lag; size the period accordingly.
- **Treating `period = 0` as "use a default".** `Trima::new(0)` returns
  `Err(Error::PeriodZero)` in Rust and a `ValueError` in Python.

## References

The triangular moving average is a standard double-smoothed SMA; the
odd/even split used here (`n1`, `n2`) matches TA-Lib's `TRIMA`.

## See also

- [Indicator-Sma.md](../moving-averages/Indicator-Sma.md) — the building block applied twice.
- [Indicator-Wma.md](../moving-averages/Indicator-Wma.md) — linear (not triangular) weights.
- [Indicator-Smma.md](../moving-averages/Indicator-Smma.md) — the other F1 average.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
