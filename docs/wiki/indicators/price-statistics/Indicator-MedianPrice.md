# MedianPrice

> Median Price — the bar's `(high + low) / 2`, the midpoint of its range.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Price Statistics |
| Input type | `Candle` (uses `high`, `low`) |
| Output type | `f64` |
| Output range | unbounded (price scale) |
| Default parameters | none (no parameters) |
| Warmup period | `1` |
| Interpretation | The midpoint of the bar's range, ignoring open and close. |

## Formula

```
MedianPrice = (high + low) / 2
```

The median price is the centre of the bar's range — it discards where the bar
opened and closed entirely. It is the price series Bill Williams'
[`AwesomeOscillator`](../momentum-oscillators/Indicator-AwesomeOscillator.md) is built on,
and a useful close substitute when the close is noisy relative to the range.

## Parameters

`MedianPrice` takes **no parameters** — `MedianPrice::new()` in Rust,
`wickra.MedianPrice()` in Python, `new ta.MedianPrice()` in Node.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/median_price.rs`:

```rust
impl Indicator for MedianPrice {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`MedianPrice` is a **candle-input** indicator that reads `high` and `low`. In
Python the streaming `update` accepts a 6-tuple or a dict; the batch helper
takes `high`, `low` numpy arrays. Node and WASM expose `update(high, low)` and
the matching `batch`.

## Warmup

`MedianPrice::new().warmup_period() == 1`. It is a stateless per-bar transform
— it emits a value from the very first candle.

## Edge cases

- **No warmup.** Every candle produces a value immediately.
- **Reset.** `mp.reset()` only clears the `is_ready` flag; there is no
  rolling state to discard.

## Examples

### Rust

```rust
use wickra::{Candle, Indicator, MedianPrice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mp = MedianPrice::new();
    let v = mp.update(Candle::new(10.0, 12.0, 8.0, 11.0, 1.0, 0)?);
    println!("{:?}", v);
    Ok(())
}
```

Output:

```
Some(10.0)
```

`(12 + 8) / 2 = 10`. This matches the `reference_value` test in
`crates/wickra-core/src/indicators/median_price.rs`.

### Python

```python
import numpy as np
import wickra as ta

mp = ta.MedianPrice()
print(mp.batch(np.array([12.0]), np.array([8.0])))
```

Output:

```
[10.]
```

### Node

```javascript
const ta = require('wickra');
const mp = new ta.MedianPrice();
console.log(mp.batch([12], [8]));
```

Output:

```
[ 10 ]
```

## Interpretation

The median price is the most range-centric of the three transforms — it is
blind to the close. Use it when the question is "where did this bar trade?"
rather than "where did it settle?", or as the input to a Bill Williams setup.

## Common pitfalls

- **Expecting the close to matter.** It does not — by definition the median
  price ignores both the open and the close.

## References

The Median Price; the `(H + L) / 2` definition is standard (TA-Lib's
`MEDPRICE`).

## See also

- [Indicator-TypicalPrice.md](../price-statistics/Indicator-TypicalPrice.md) — `(H + L + C) / 3`.
- [Indicator-WeightedClose.md](../price-statistics/Indicator-WeightedClose.md) — `(H + L + 2C) / 4`.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
