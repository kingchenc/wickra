# LinearRegression

> Linear Regression — the endpoint of a rolling ordinary-least-squares fit
> over the last `period` prices.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Price Statistics |
| Input type | `f64` (price) |
| Output type | `f64` |
| Output range | unbounded (price scale) |
| Default parameters | `period = 14` (Python) |
| Warmup period | `period` |
| Interpretation | A low-lag smoothed price — the trend line extrapolated to now. |

## Formula

Over the last `period` inputs, indexed `x = 0, 1, …, period − 1`:

```
b (slope)     = (n·Σxy − Σx·Σy) / (n·Σxx − (Σx)²)
a (intercept) = (Σy − b·Σx) / n
LinearReg     = a + b·(period − 1)
```

The indicator fits a straight line to the window by ordinary least squares,
then reports that line's value at the most recent bar. Because it
extrapolates the *local trend* forward rather than averaging it away, it lags
a same-period [`Sma`](../moving-averages/Indicator-Sma.md) noticeably less. This is
TA-Lib's `LINEARREG`.

## Parameters

`period` — the regression window. Must be at least `2` (a line needs two
points). The Python binding defaults it to `14`; the Rust and Node
constructors require it explicitly.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/linreg.rs`:

```rust
impl Indicator for LinearRegression {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

`LinearRegression` is a **scalar** indicator: it consumes one `f64` price per
step. Because `Input = f64` it can sit inside a [`Chain`](../../Indicator-Chaining.md).

## Warmup

`LinearRegression::new(14).warmup_period() == 14`. The first value lands once
the window holds a full `period` prices — on input index `period − 1`.

## Edge cases

- **`period < 2`.** Rejected at construction — a regression line is undefined
  for fewer than two points.
- **Perfect line.** Fed a perfectly linear series, the fit *is* that line, so
  the endpoint equals the current value (`perfect_line_returns_current_value`
  pins this).
- **Constant series.** A flat input returns that constant.
- **Reset.** `lr.reset()` clears the rolling window.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, LinearRegression};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lr = LinearRegression::new(3)?;
    // Fit over [1, 2, 9]: the least-squares line is y = 4x, endpoint 4·2 = 8.
    let out = lr.batch(&[1.0, 2.0, 9.0]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[None, None, Some(8.0)]
```

This matches the `reference_values` test in
`crates/wickra-core/src/indicators/linreg.rs`.

### Python

```python
import numpy as np
import wickra as ta

lr = ta.LinearRegression(3)
print(lr.batch(np.array([1.0, 2.0, 9.0])))
```

Output:

```
[nan nan  8.]
```

### Node

```javascript
const ta = require('wickra');
const lr = new ta.LinearRegression(3);
console.log(lr.batch([1, 2, 9]));
```

Output:

```
[ NaN, NaN, 8 ]
```

## Interpretation

Read `LinearRegression` as a low-lag moving average: it tracks price more
closely than an SMA of the same period because it projects the window's trend
to the current bar instead of centring on the window. A shorter `period`
hugs price; a longer one is a smoother trend line. Pair it with
[`LinRegSlope`](../price-statistics/Indicator-LinRegSlope.md) to read the same fit's steepness.

## Common pitfalls

- **Confusing it with an SMA.** It is a *projected* fit, not a centred
  average, so it leads an SMA of the same period.
- **Tiny periods.** `period = 2` is allowed but the "fit" just passes through
  the last two points; use a meaningful window.

## References

Ordinary least-squares linear regression applied to a rolling price window;
the endpoint formulation matches TA-Lib's `LINEARREG`.

## See also

- [Indicator-LinRegSlope.md](../price-statistics/Indicator-LinRegSlope.md) — the slope of the same
  rolling fit.
- [Indicator-Sma.md](../moving-averages/Indicator-Sma.md) — the centred average it is
  often compared against.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
