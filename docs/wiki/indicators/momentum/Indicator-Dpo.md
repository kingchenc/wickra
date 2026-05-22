# DPO

> Detrended Price Oscillator — removes the trend from price by comparing a
> shifted past price to the moving average, exposing the underlying cycle.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum |
| Sub-category | Unbounded oscillators |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded around zero (price-difference scale) |
| Default parameters | `period = 20` (Python) |
| Warmup period | `max(period, period / 2 + 2)` |
| Interpretation | Detrended price; peak-to-peak spacing reveals the cycle length. |

## Formula

```
shift = period / 2 + 1
DPO_t = price_{t − shift} − SMA(period)_t
```

A normal oscillator compares price to a *current* average and therefore
still carries the trend. DPO instead subtracts the average from a price
taken `period / 2 + 1` bars **back** — roughly half a cycle. The dominant
trend cancels, and what is left swings around zero with the same period
as the price's shorter cycles, so the distance between DPO peaks reads off
the cycle length directly.

DPO is **not** a momentum or signal indicator: by construction it is
shifted into the past and is not meant to track the latest bar.

## Parameters

| Name     | Type    | Default       | Valid range | Description |
|----------|---------|---------------|-------------|-------------|
| `period` | `usize` | `20` (Python) | `>= 1`      | SMA length; also sets the look-back `shift = period / 2 + 1`. `0` errors with `Error::PeriodZero`. |

The Python binding defaults `period` to `20`. The derived `shift` is
exposed as a read-only property.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/dpo.rs`:

```rust
impl Indicator for Dpo {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`warmup_period() == max(period, period / 2 + 2)`. The output needs both a
full `period`-bar SMA window and a price `shift` bars back; the indicator
becomes ready once the rolling window holds enough bars for both. For the
usual `period >= 4` this simplifies to `period`.

## Edge cases

- **Constant series.** On a flat series the shifted price equals the SMA,
  so DPO is `0` (`constant_series_yields_zero` pins this).
- **NaN / infinity inputs.** Non-finite inputs are silently dropped; the
  window is not advanced.
- **Reset.** `dpo.reset()` clears the window and the rolling sum.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Dpo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dpo = Dpo::new(4)?;
    let out: Vec<Option<f64>> = dpo.batch(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    println!("{:?}", out);
    println!("shift = {}, warmup_period = {}", dpo.shift(), dpo.warmup_period());
    Ok(())
}
```

Output:

```
[None, None, None, Some(-1.5), Some(-1.5), Some(-1.5)]
shift = 3, warmup_period = 4
```

`DPO(4)` has `shift = 3`. At input 4 the SMA of `[1,2,3,4]` is `2.5` and
the price 3 bars back is `1`, giving `1 − 2.5 = −1.5`. On a pure ramp the
detrended value is constant. This matches the `reference_values` test in
`crates/wickra-core/src/indicators/dpo.rs`.

### Python

```python
import numpy as np
import wickra as ta

dpo = ta.DPO(4)
print(dpo.batch(np.array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0])))
```

Output:

```
[ nan  nan  nan -1.5 -1.5 -1.5]
```

### Node

```javascript
const ta = require('wickra');
const dpo = new ta.DPO(4);
console.log(dpo.batch([1, 2, 3, 4, 5, 6]));
```

Output:

```
[ NaN, NaN, NaN, -1.5, -1.5, -1.5 ]
```

## Interpretation

`Dpo` is a cycle-measurement tool, not a trading trigger. Read it for the
*spacing* of its peaks and troughs: regular spacing reveals the dominant
cycle length, which you can then feed back into the periods of other
indicators. Crossing zero is not a signal — because the series is shifted
into the past, the latest DPO value does not correspond to the latest bar.

## Common pitfalls

- **Trading the zero cross.** DPO is detrended *and* time-shifted; its
  latest value is historical. Use it to size cycles, not to time entries.
- **Reading it as momentum.** It is a detrended price, not a rate of
  change — see [`Roc`](Indicator-Roc.md) or [`Mom`](Indicator-Mom.md) for
  momentum.

## References

The Detrended Price Oscillator is a standard cycle-analysis study; the
`period / 2 + 1` look-back shift used here matches the common definition
(StockCharts, TA-Lib-compatible implementations).

## See also

- [Indicator-Sma.md](../trend/Indicator-Sma.md) — the moving average DPO
  detrends against.
- [Indicator-Roc.md](Indicator-Roc.md) — momentum, the indicator DPO is
  often confused with.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
