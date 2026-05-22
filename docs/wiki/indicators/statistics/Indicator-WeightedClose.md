# WeightedClose

> Weighted Close — the bar's `(high + low + 2·close) / 4`, a per-bar price
> that gives the close double weight.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Statistics |
| Sub-category | Price transforms |
| Input type | `Candle` (uses `high`, `low`, `close`) |
| Output type | `f64` |
| Output range | unbounded (price scale) |
| Default parameters | none (no parameters) |
| Warmup period | `1` |
| Interpretation | A representative per-bar price that leans on the close. |

## Formula

```
WeightedClose = (high + low + 2·close) / 4
```

Like the [`TypicalPrice`](Indicator-TypicalPrice.md), the weighted close
collapses an OHLC bar to one number — but it counts the close twice, so the
result sits closer to where the bar settled than to its range. Reach for it
when the closing print carries more signal than the extremes.

## Parameters

`WeightedClose` takes **no parameters** — `WeightedClose::new()` in Rust,
`wickra.WeightedClose()` in Python, `new ta.WeightedClose()` in Node.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/weighted_close.rs`:

```rust
impl Indicator for WeightedClose {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`WeightedClose` is a **candle-input** indicator that reads `high`, `low` and
`close`. In Python the streaming `update` accepts a 6-tuple or a dict; the
batch helper takes `high`, `low`, `close` numpy arrays. Node and WASM expose
`update(high, low, close)` and the matching `batch`.

## Warmup

`WeightedClose::new().warmup_period() == 1`. It is a stateless per-bar
transform — it emits a value from the very first candle.

## Edge cases

- **No warmup.** Every candle produces a value immediately.
- **Reset.** `wc.reset()` only clears the `is_ready` flag; there is no
  rolling state to discard.

## Examples

### Rust

```rust
use wickra::{Candle, Indicator, WeightedClose};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut wc = WeightedClose::new();
    let v = wc.update(Candle::new(10.0, 12.0, 8.0, 11.0, 1.0, 0)?);
    println!("{:?}", v);
    Ok(())
}
```

Output:

```
Some(10.5)
```

`(12 + 8 + 2·11) / 4 = 42 / 4 = 10.5`. This matches the `reference_value`
test in `crates/wickra-core/src/indicators/weighted_close.rs`.

### Python

```python
import numpy as np
import wickra as ta

wc = ta.WeightedClose()
print(wc.batch(np.array([12.0]), np.array([8.0]), np.array([11.0])))
```

Output:

```
[10.5]
```

### Node

```javascript
const ta = require('wickra');
const wc = new ta.WeightedClose();
console.log(wc.batch([12], [8], [11]));
```

Output:

```
[ 10.5 ]
```

## Interpretation

The weighted close sits on the spectrum between the raw close and the
[`TypicalPrice`](Indicator-TypicalPrice.md): closer to the close, but still
nudged by the bar's range. Use it as a drop-in close replacement when you want
the settlement to dominate without ignoring the extremes entirely.

## Common pitfalls

- **Feeding it scalar prices.** It needs the full `high`/`low`/`close` bar.

## References

The Weighted Close; the `(H + L + 2C) / 4` definition is standard (TA-Lib's
`WCLPRICE`).

## See also

- [Indicator-TypicalPrice.md](Indicator-TypicalPrice.md) — `(H + L + C) / 3`.
- [Indicator-MedianPrice.md](Indicator-MedianPrice.md) — `(H + L) / 2`.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
