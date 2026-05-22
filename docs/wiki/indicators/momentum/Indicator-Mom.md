# MOM

> Momentum — the raw price change over a fixed lookback,
> `price_t − price_{t−period}`, in absolute price units.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum |
| Sub-category | Unbounded oscillators |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded around zero (price-difference scale) |
| Default parameters | `period = 10` (Python) |
| Warmup period | `period + 1` |
| Interpretation | Sign and size of the move over the last `period` bars. |

## Formula

```
MOM_t = price_t − price_{t−period}
```

The simplest momentum primitive. Positive output means price is higher
than it was `period` bars ago, negative means lower, and the magnitude is
the change in raw price units. [`Roc`](Indicator-Roc.md) is the same idea
expressed as a percentage of the old price.

## Parameters

| Name     | Type    | Default        | Valid range | Description |
|----------|---------|----------------|-------------|-------------|
| `period` | `usize` | `10` (Python)  | `>= 1`      | Lookback distance in bars. `period = 0` errors with `Error::PeriodZero`. |

The Python binding defaults `period` to `10` via `#[pyo3(signature = (period=10))]`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/mom.rs`:

```rust
impl Indicator for Mom {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`Mom::new(period).warmup_period() == period + 1`. The output needs both
the current price and the price `period` bars back, so the window must
hold `period + 1` values — the first non-`None` output lands on input
`period + 1`.

## Edge cases

- **Constant series.** A flat series yields `0.0` from input `period + 1`
  onward (`constant_series_yields_zero` pins this).
- **NaN / infinity inputs.** Non-finite inputs are silently dropped: the
  rolling window is not advanced and the previous value is returned. The
  next finite input still references the correct historical price.
- **Reset.** `mom.reset()` clears the window and restarts the warmup.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Mom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mom = Mom::new(3)?;
    let out: Vec<Option<f64>> = mom.batch(&[1.0, 2.0, 3.0, 4.0, 7.0]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[None, None, None, Some(3.0), Some(5.0)]
```

`MOM(3)` first emits on input 4: `4 − 1 = 3`. The fifth input gives
`7 − 2 = 5`. This matches the `reference_values` test in
`crates/wickra-core/src/indicators/mom.rs`.

### Python

```python
import numpy as np
import wickra as ta

mom = ta.MOM(3)
print(mom.batch(np.array([1.0, 2.0, 3.0, 4.0, 7.0])))
```

Output:

```
[nan nan nan  3.  5.]
```

### Node

```javascript
const ta = require('wickra');
const mom = new ta.MOM(3);
console.log(mom.batch([1, 2, 3, 4, 7]));
```

Output:

```
[ NaN, NaN, NaN, 3, 5 ]
```

## Interpretation

`Mom` is a zero-centred oscillator. The textbook reads are the zero-line
cross (momentum flipping sign) and divergence (price making a new high
while `Mom` makes a lower high — a stalling trend). Because the output is
in price units, `Mom` values are not comparable across instruments at
different price levels; use [`Roc`](Indicator-Roc.md) when you need a
scale-free percentage instead.

## Common pitfalls

- **Comparing `Mom` across instruments.** A `Mom` of `5` means very
  different things on a $10 stock and a $5000 index. Normalise with `Roc`
  for cross-asset work.
- **Forgetting the `+1` warmup.** `warmup_period()` is `period + 1`, not
  `period`.

## References

Momentum is one of the oldest technical studies; the implementation here
is the standard `price − price[period]` difference, matching TA-Lib's
`MOM`.

## See also

- [Indicator-Roc.md](Indicator-Roc.md) — the percentage-scaled counterpart.
- [Indicator-Cmo.md](Indicator-Cmo.md) — bounded momentum from summed changes.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
