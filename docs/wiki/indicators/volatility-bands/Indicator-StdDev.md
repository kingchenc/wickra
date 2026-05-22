# StdDev

> Rolling population standard deviation — the dispersion of the last
> `period` prices around their mean.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volatility & Bands |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | `[0, ∞)` (price-difference scale) |
| Default parameters | `period = 20` (Python) |
| Warmup period | `period` |
| Interpretation | Spread of recent prices; the raw volatility behind Bollinger Bands. |

## Formula

```
mean     = (1/n) · Σ price
variance = (1/n) · Σ price² − mean²
StdDev   = √variance
```

This is the **population** standard deviation (divisor `n`, not `n − 1`)
— the exact dispersion measure that drives the band width of
[`BollingerBands`](../volatility-bands/Indicator-BollingerBands.md). It is maintained as an
O(1) state machine: a running sum and a running sum-of-squares, each
updated by one add and one subtract per bar. Floating-point cancellation
can leave the computed variance very slightly negative; it is clamped to
zero before the square root.

## Parameters

| Name     | Type    | Default       | Valid range | Description |
|----------|---------|---------------|-------------|-------------|
| `period` | `usize` | `20` (Python) | `>= 1`      | Rolling window length. `0` errors with `Error::PeriodZero`. `period = 1` always yields `0`. |

The Python binding defaults `period` to `20`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/std_dev.rs`:

```rust
impl Indicator for StdDev {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`StdDev::new(period).warmup_period() == period`. The first non-`None`
value is emitted once the window holds `period` prices.

## Edge cases

- **Constant series.** A flat series has zero dispersion, so the output
  is `0.0` (`constant_series_yields_zero` pins this).
- **NaN / infinity inputs.** Non-finite inputs are silently dropped; the
  window and the running sums are left untouched.
- **Reset.** `sd.reset()` clears the window and both running sums.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, StdDev};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sd = StdDev::new(3)?;
    let out: Vec<Option<f64>> = sd.batch(&[2.0, 4.0, 6.0]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[None, None, Some(1.6329931618554525)]
```

The window `[2, 4, 6]` has mean `4` and variance `(4 + 0 + 4) / 3 = 8/3`,
so the standard deviation is `√(8/3) ≈ 1.633`. This matches the
`reference_value` test in `crates/wickra-core/src/indicators/std_dev.rs`.

### Python

```python
import numpy as np
import wickra as ta

sd = ta.StdDev(3)
print(sd.batch(np.array([2.0, 4.0, 6.0])))
```

Output:

```
[      nan       nan 1.6329932]
```

### Node

```javascript
const ta = require('wickra');
const sd = new ta.StdDev(3);
console.log(sd.batch([2, 4, 6]));
```

Output:

```
[ NaN, NaN, 1.6329931618554525 ]
```

## Interpretation

`StdDev` is the most direct volatility measure in the library: large
values mean prices are scattered widely around their mean, small values
mean a tight, quiet market. Use it on its own as a volatility filter, or
recognise it as the engine inside `BollingerBands` — multiplying `StdDev`
by the band multiplier and adding it to an `Sma` reproduces the bands
exactly.

## Common pitfalls

- **Expecting the sample standard deviation.** `StdDev` divides by `n`,
  not `n − 1`. For the unbiased return-based estimator use
  [`HistoricalVolatility`](../volatility-bands/Indicator-HistoricalVolatility.md).
- **Comparing across instruments.** The output is in price units; a
  `StdDev` of `5` is not comparable between a $10 and a $1000 asset.

## References

The population standard deviation is standard statistics; this
implementation matches the dispersion term of John Bollinger's Bollinger
Bands and pandas' `rolling(period).std(ddof=0)`.

## See also

- [Indicator-BollingerBands.md](../volatility-bands/Indicator-BollingerBands.md) — bands built
  from this dispersion measure.
- [Indicator-HistoricalVolatility.md](../volatility-bands/Indicator-HistoricalVolatility.md) —
  annualised volatility of log returns.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
