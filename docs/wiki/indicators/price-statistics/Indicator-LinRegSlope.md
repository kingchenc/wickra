# LinRegSlope

> Linear Regression Slope — the slope of a rolling ordinary-least-squares
> fit over the last `period` prices.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Price Statistics |
| Input type | `f64` (price) |
| Output type | `f64` |
| Output range | unbounded around zero (price units per bar) |
| Default parameters | `period = 14` (Python) |
| Warmup period | `period` |
| Interpretation | How steeply price trends; positive up, negative down, zero flat. |

## Formula

Over the last `period` inputs, indexed `x = 0, 1, …, period − 1`:

```
b = (n·Σxy − Σx·Σy) / (n·Σxx − (Σx)²)
```

`LinRegSlope` fits a straight line to the window by ordinary least squares —
the same fit as [`LinearRegression`](../price-statistics/Indicator-LinearRegression.md) — but
reports the *slope* `b` instead of the endpoint. The slope is in price units
per bar: positive while price trends up, negative while it trends down, near
zero when it is ranging. This is TA-Lib's `LINEARREG_SLOPE`.

## Parameters

`period` — the regression window. Must be at least `2` (a line needs two
points). The Python binding defaults it to `14`; the Rust and Node
constructors require it explicitly.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/linreg_slope.rs`:

```rust
impl Indicator for LinRegSlope {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

`LinRegSlope` is a **scalar** indicator: it consumes one `f64` price per step.
Because `Input = f64` it can sit inside a [`Chain`](../../Indicator-Chaining.md).

## Warmup

`LinRegSlope::new(14).warmup_period() == 14`. The first value lands once the
window holds a full `period` prices — on input index `period − 1`.

## Edge cases

- **`period < 2`.** Rejected at construction — a regression line is undefined
  for fewer than two points.
- **Perfect line.** Fed a series rising by a fixed step, the slope is exactly
  that step (`perfect_line_returns_its_step` pins this).
- **Constant series.** A flat input returns a slope of `0`.
- **Falling series.** A descending input returns a negative slope.
- **Reset.** `ls.reset()` clears the rolling window.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, LinRegSlope};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ls = LinRegSlope::new(3)?;
    // Fit over [1, 2, 9]: the least-squares line is y = 4x, slope 4.
    let out = ls.batch(&[1.0, 2.0, 9.0]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[None, None, Some(4.0)]
```

This matches the `reference_values` test in
`crates/wickra-core/src/indicators/linreg_slope.rs`.

### Python

```python
import numpy as np
import wickra as ta

ls = ta.LinRegSlope(3)
print(ls.batch(np.array([1.0, 2.0, 9.0])))
```

Output:

```
[nan nan  4.]
```

### Node

```javascript
const ta = require('wickra');
const ls = new ta.LinRegSlope(3);
console.log(ls.batch([1, 2, 9]));
```

Output:

```
[ NaN, NaN, 4 ]
```

## Interpretation

`LinRegSlope` is a momentum gauge: its sign is the trend direction and its
magnitude is the trend's steepness in price-per-bar. A slope crossing zero
marks a trend change; a slope that flattens while price still rises warns the
trend is losing pace. Unlike a difference-based oscillator it uses every bar
in the window, so it is less jumpy.

## Common pitfalls

- **Comparing slopes across instruments.** The slope is in the instrument's
  own price units per bar — normalise (e.g. divide by price) to compare.
- **Tiny periods.** `period = 2` reduces the slope to the last simple
  difference; use a meaningful window.

## References

The slope of an ordinary least-squares fit to a rolling price window; matches
TA-Lib's `LINEARREG_SLOPE`.

## See also

- [Indicator-LinearRegression.md](../price-statistics/Indicator-LinearRegression.md) — the
  endpoint of the same rolling fit.
- [Indicator-Mom.md](../momentum-oscillators/Indicator-Mom.md) — raw price-difference
  momentum, the unsmoothed cousin.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
