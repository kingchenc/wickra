# T3

> Tillson T3 — a six-fold cascaded EMA recombined with a volume factor `v`
> to give a smooth, low-lag trend line.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Trend |
| Sub-category | Exponential family |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded; tracks the input price scale |
| Default parameters | `period` required; `v = 0.7` (Python default) |
| Warmup period | `6·period − 5` |
| Interpretation | Smooth trend line with less lag than a same-period EMA. |

## Formula

T3 is the *generalised DEMA* (`GD`) applied three times. Tim Tillson's
expansion of `GD(GD(GD(price)))` over six chained EMAs — `e1 … e6`, each
of the same `period`, where `e2 = EMA(e1)`, `e3 = EMA(e2)`, … — is:

```
v2 = v²,  v3 = v³
c1 = −v3
c2 = 3·v2 + 3·v3
c3 = −6·v2 − 3·v − 3·v3
c4 = 1 + 3·v + v3 + 3·v2
T3 = c1·e6 + c2·e5 + c3·e4 + c4·e3
```

The four coefficients always sum to `1`, so a constant price series maps
to itself. The volume factor `v` controls the lag/overshoot trade-off:
`v = 0` collapses T3 to the plain triple-cascaded EMA `e3`; the
conventional `v = 0.7` adds a corrective hump that sharpens turns.

## Parameters

| Name     | Type    | Default        | Valid range | Description |
|----------|---------|----------------|-------------|-------------|
| `period` | `usize` | none           | `>= 1`      | Length of every EMA in the cascade. `period = 0` errors with `Error::PeriodZero`. |
| `v`      | `f64`   | `0.7` (Python) | `[0.0, 1.0]`| Volume factor. Non-finite or out-of-range values error with `Error::InvalidPeriod`. |

The Python binding defaults `v` to `0.7` via `#[pyo3(signature = (period, v=0.7))]`;
`period` is always explicit. The Node and WASM constructors take both
arguments explicitly.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/t3.rs`:

```rust
impl Indicator for T3 {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`T3::new(period, v).warmup_period() == 6·period − 5`. Each stage of the
SMA-seeded EMA cascade adds `period − 1` bars of delay: `e1` seeds at
input `period`, `e2` at `2·period − 1`, …, `e6` at `6·period − 5`. T3
emits its first value once `e6` is ready, since the output formula needs
`e3` through `e6`.

## Edge cases

- **Constant series.** Because `c1 + c2 + c3 + c4 = 1` for any `v`, a flat
  input series produces a flat output equal to the constant
  (`coefficients_sum_to_one` and `constant_series_yields_the_constant`
  pin this).
- **`v = 0`.** The coefficients become `c1 = c2 = c3 = 0`, `c4 = 1`, so
  `T3` is exactly the third stage of the EMA cascade
  (`zero_volume_factor_collapses_to_triple_cascaded_ema` pins this).
- **NaN / infinity inputs.** Non-finite inputs are silently dropped — the
  cascade is not advanced — and the previous valid value is returned.
- **Reset.** `t3.reset()` clears all six EMAs and the cached value.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, T3};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prices: Vec<f64> = (1..=40).map(f64::from).collect();
    let mut t3 = T3::new(3, 0.7)?;
    let out = t3.batch(&prices);
    println!("warmup_period = {}", t3.warmup_period());
    println!("first ready index = {:?}", out.iter().position(Option::is_some));
    Ok(())
}
```

Output:

```
warmup_period = 13
first ready index = Some(12)
```

`T3(3, 0.7)` warms up after `6·3 − 5 = 13` inputs, so the first non-`None`
output sits at index `12`. On a pure ramp the output then tracks the input
trend with a smooth, near-constant offset.

### Python

```python
import numpy as np
import wickra as ta

t3 = ta.T3(5)  # v defaults to 0.7
prices = np.linspace(100.0, 140.0, 60)
out = t3.batch(prices)
print("warmup_period =", t3.warmup_period())
print("ready values:", np.count_nonzero(~np.isnan(out)))
```

Output:

```
warmup_period = 25
ready values: 36
```

### Node

```javascript
const ta = require('wickra');
const t3 = new ta.T3(5, 0.7);
const prices = Array.from({ length: 60 }, (_, i) => 100 + i);
console.log('warmupPeriod:', t3.warmupPeriod());
console.log('last:', t3.batch(prices).at(-1));
```

## Interpretation

`T3` is a "best of both" trend line — close to `Tema` in lag reduction but
visibly smoother, because the six-EMA cascade filters noise the
three-EMA `Tema` lets through. Use it as a single trend filter or as the
slow leg of a crossover where you want a clean line. Raise `v` toward `1`
for sharper turns (more overshoot), lower it toward `0` for maximum
smoothness (`v = 0` is just a triple EMA).

## Common pitfalls

- **Treating `v` as optional outside Python.** Only the Python binding
  defaults `v` to `0.7`; the Rust, Node and WASM constructors require it.
- **Underestimating warmup.** `6·period − 5` grows fast — a `T3(20)` needs
  `115` bars before its first value.

## References

Tim Tillson, "Better Moving Averages", *Technical Analysis of Stocks &
Commodities* (1998). The six-EMA expansion and coefficient formulas here
match Tillson's published derivation and TA-Lib's `T3`.

## See also

- [Indicator-Tema.md](Indicator-Tema.md) — the three-EMA relative.
- [Indicator-Dema.md](Indicator-Dema.md) — the two-EMA relative.
- [Indicator-Zlema.md](Indicator-Zlema.md) — low-lag average via de-lagging.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
