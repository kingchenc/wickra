# LinRegAngle

> Linear Regression Angle — the slope of the rolling least-squares fit,
> expressed as an angle in degrees.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Price Statistics |
| Input type | `f64` (price) |
| Output type | `f64` |
| Output range | `(−90°, +90°)` |
| Default parameters | `period = 14` (Python) |
| Warmup period | `period` |
| Interpretation | Steepness of the trend; sign is direction, magnitude is pitch. |

## Formula

```
LinRegAngle = atan(LinRegSlope) · 180 / π
```

The angle carries exactly the same information as
[`LinRegSlope`](../price-statistics/Indicator-LinRegSlope.md) — positive while price trends up,
negative while it trends down — but maps the unbounded slope through `atan`
onto `(−90°, +90°)`. That bounded, price-unit-free scale makes "how steep is
the trend" comparable at a glance and across instruments. This is TA-Lib's
`LINEARREG_ANGLE`.

## Parameters

`period` — the regression window. Must be at least `2` (a line needs two
points). The Python binding defaults it to `14`; the Rust and Node
constructors require it explicitly.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/linreg_angle.rs`:

```rust
impl Indicator for LinRegAngle {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

`LinRegAngle` is a **scalar** indicator: it consumes one `f64` price per step.
Because `Input = f64` it can sit inside a [`Chain`](../../Indicator-Chaining.md).

## Warmup

`LinRegAngle::new(14).warmup_period() == 14`. The first value lands once the
window holds a full `period` prices.

## Edge cases

- **`period < 2`.** Rejected at construction — a regression line is undefined
  for fewer than two points.
- **Unit slope.** A series rising by exactly `1` per step has slope `1`, and
  `atan(1) = 45°`.
- **Flat series.** A constant input has slope `0` and therefore angle `0`.
- **Reset.** `angle.reset()` clears the rolling regression window.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, LinRegAngle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut angle = LinRegAngle::new(5)?;
    // Closes rising by 1 per step -> slope 1 -> atan(1) = 45 degrees.
    let out = angle.batch(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[None, None, None, None, Some(45.0), Some(45.0)]
```

### Python

```python
import numpy as np
import wickra as ta

angle = ta.LinRegAngle(5)
print(angle.batch(np.array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0])))
```

Output:

```
[ nan  nan  nan  nan 45. 45.]
```

### Node

```javascript
const ta = require('wickra');
const angle = new ta.LinRegAngle(5);
console.log(angle.batch([1, 2, 3, 4, 5, 6]));
```

Output:

```
[ NaN, NaN, NaN, NaN, 45, 45 ]
```

## Interpretation

The angle is read like a slope: sign gives trend direction, magnitude gives
how steeply price is pitched. Because it is bounded to `±90°` it is convenient
for thresholds — e.g. "only trade with the trend while the angle exceeds
`30°`" — and for comparing trend pitch across instruments with different price
scales, which the raw [`LinRegSlope`](../price-statistics/Indicator-LinRegSlope.md) cannot do.

## Common pitfalls

- **Reading degrees as a price quantity.** The angle depends on the chart's
  implicit scaling; treat it as a relative steepness gauge, not an absolute.
- **Tiny periods.** `period = 2` reduces the fit to the last difference.

## References

The angle of an ordinary least-squares fit to a rolling price window; matches
TA-Lib's `LINEARREG_ANGLE`.

## See also

- [Indicator-LinRegSlope.md](../price-statistics/Indicator-LinRegSlope.md) — the same fit's slope,
  in raw price-per-bar units.
- [Indicator-LinearRegression.md](../price-statistics/Indicator-LinearRegression.md) — the
  endpoint of the same rolling fit.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
