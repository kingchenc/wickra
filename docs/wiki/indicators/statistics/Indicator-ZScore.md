# ZScore

> Z-Score — how many standard deviations the latest price sits from its
> rolling mean.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Price Statistics |
| Input type | `f64` (price) |
| Output type | `f64` |
| Output range | unbounded around zero (standard deviations) |
| Default parameters | `period = 20` (Python) |
| Warmup period | `period` |
| Interpretation | Large magnitude = stretched; a return toward `0` = reversion. |

## Formula

```
ZScore = (price − SMA(price, n)) / population_stddev(price, n)
```

The Z-Score normalises price against its own recent behaviour: it subtracts
the rolling mean and divides by the rolling population standard deviation. A
reading of `+2` means price is two standard deviations above its `n`-bar
average — statistically stretched to the upside; `−2` is the mirror. It is the
standard input to mean-reversion strategies.

## Parameters

`period` — the rolling window for the mean and standard deviation. The Python
binding defaults it to `20`; the Rust and Node constructors require it.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/z_score.rs`:

```rust
impl Indicator for ZScore {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

`ZScore` is a **scalar** indicator: it consumes one `f64` price per step.
Because `Input = f64` it can sit inside a [`Chain`](../../Indicator-Chaining.md).

## Warmup

`ZScore::new(20).warmup_period() == 20`. The first value lands once the window
holds a full `period` prices.

## Edge cases

- **Zero dispersion.** A flat window has a zero standard deviation; `ZScore`
  is defined as `0` rather than dividing by zero.
- **Rising series.** A monotonically rising price always scores above its
  trailing mean (positive).
- **Reset.** `z.reset()` clears the rolling window.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, ZScore};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut z = ZScore::new(2)?;
    // Window [1, 3]: mean 2, population stddev 1; latest 3 -> (3 - 2) / 1.
    println!("{:?}", z.batch(&[1.0, 3.0]));
    Ok(())
}
```

Output:

```
[None, Some(1.0)]
```

### Python

```python
import numpy as np
import wickra as ta

z = ta.ZScore(2)
print(z.batch(np.array([1.0, 3.0])))
```

Output:

```
[nan  1.]
```

### Node

```javascript
const ta = require('wickra');
const z = new ta.ZScore(2);
console.log(z.batch([1, 3]));
```

Output:

```
[ NaN, 1 ]
```

## Interpretation

Z-Score is the workhorse of mean-reversion: a common rule enters against the
move when `|ZScore| > 2` and exits as it crosses back through `0`. Read
together with a trend filter — a high Z-Score in a strong trend is often
continuation, not exhaustion, so the reversion edge is best in ranging
regimes.

## Common pitfalls

- **Trading extremes blindly.** A trending market can hold a high Z-Score for
  a long time; pair it with a regime filter.
- **Tiny periods.** A short window makes the mean and stddev jumpy.

## References

The standard statistical Z-Score (standard score) applied to a rolling price
window.

## See also

- [Indicator-StdDev.md](../volatility/Indicator-StdDev.md) — the rolling
  standard deviation in the denominator.
- [Indicator-LinearRegression.md](Indicator-LinearRegression.md) — another
  rolling statistical fit.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
