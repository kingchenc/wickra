# TypicalPrice

> Typical Price — the bar's `(high + low + close) / 3`, a single
> representative price per candle.

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
| Interpretation | A representative per-bar price; a smoother stand-in for the close. |

## Formula

```
TypicalPrice = (high + low + close) / 3
```

The typical price collapses a full OHLC bar to one number, giving the close
no more weight than the two extremes. It is the price series that
[`Cci`](../momentum/Indicator-Cci.md) and [`Mfi`](../momentum/Indicator-Mfi.md)
are defined on, and a common input to feed any close-driven indicator when you
want the bar's range reflected in the value.

## Parameters

`TypicalPrice` takes **no parameters** — `TypicalPrice::new()` in Rust,
`wickra.TypicalPrice()` in Python, `new ta.TypicalPrice()` in Node.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/typical_price.rs`:

```rust
impl Indicator for TypicalPrice {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`TypicalPrice` is a **candle-input** indicator that reads `high`, `low` and
`close`. In Python the streaming `update` accepts a 6-tuple or a dict; the
batch helper takes `high`, `low`, `close` numpy arrays. Node and WASM expose
`update(high, low, close)` and the matching `batch`.

## Warmup

`TypicalPrice::new().warmup_period() == 1`. It is a stateless per-bar
transform — it emits a value from the very first candle.

## Edge cases

- **No warmup.** Every candle produces a value immediately.
- **Reset.** `tp.reset()` only clears the `is_ready` flag; there is no
  rolling state to discard.

## Examples

### Rust

```rust
use wickra::{Candle, Indicator, TypicalPrice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tp = TypicalPrice::new();
    let v = tp.update(Candle::new(9.0, 12.0, 6.0, 9.0, 1.0, 0)?);
    println!("{:?}", v);
    Ok(())
}
```

Output:

```
Some(9.0)
```

`(12 + 6 + 9) / 3 = 9`. This matches the `reference_value` test in
`crates/wickra-core/src/indicators/typical_price.rs`.

### Python

```python
import numpy as np
import wickra as ta

tp = ta.TypicalPrice()
print(tp.batch(np.array([12.0]), np.array([6.0]), np.array([9.0])))
```

Output:

```
[9.]
```

### Node

```javascript
const ta = require('wickra');
const tp = new ta.TypicalPrice();
console.log(tp.batch([12], [6], [9]));
```

Output:

```
[ 9 ]
```

## Interpretation

Use it wherever you would use the close but want the bar's range to count —
feeding a moving average, an oscillator, or a band. It is marginally smoother
than the raw close because a wild close is pulled back toward the bar's mid.

## Common pitfalls

- **Feeding it scalar prices.** It needs the full `high`/`low`/`close` bar.

## References

The Typical Price (also "pivot price"); the `(H + L + C) / 3` definition is
standard (StockCharts, TA-Lib's `TYPPRICE`).

## See also

- [Indicator-MedianPrice.md](Indicator-MedianPrice.md) — `(H + L) / 2`.
- [Indicator-WeightedClose.md](Indicator-WeightedClose.md) — `(H + L + 2C) / 4`.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
