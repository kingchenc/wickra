# ChoppinessIndex

> Choppiness Index — is the market trending or just chopping sideways?

## Quick reference

| Field | Value |
|-------|-------|
| Family | Trend & Directional |
| Input type | `Candle` (uses `high`, `low`, `close`) |
| Output type | `f64` |
| Output range | `[0, 100]` (typical) |
| Default parameters | `period = 14` (Python) |
| Warmup period | `period` |
| Interpretation | High = choppy/ranging, low = trending; `61.8` / `38.2` thresholds. |

## Formula

```
CI = 100 · log10( Σ(TR, n) / (highest_high(n) − lowest_low(n)) ) / log10(n)
```

The ratio compares the distance price *actually travelled* (the summed true
range) with the *net ground it covered* (the high-low span of the window). A
clean trend travels almost exactly its span, so the ratio is near `1` and `CI`
near `0`; a choppy market criss-crosses far more than its span, so the ratio
is large and `CI` climbs toward `100`. The conventional reading is `CI > 61.8`
ranging, `CI < 38.2` trending.

## Parameters

`period` — the lookback window. Must be at least `2` (the `log10(period)`
denominator is zero for `period == 1`). The Python binding defaults it to `14`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/choppiness_index.rs`:

```rust
impl Indicator for ChoppinessIndex {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`ChoppinessIndex` is a **candle-input** indicator that reads `high`, `low` and
`close` (the close drives the true range across bar gaps). Python's streaming
`update` accepts a 6-tuple or a dict; the batch helper takes `high`, `low`,
`close` numpy arrays. Node and WASM expose `update(high, low, close)` and the
matching `batch`.

## Warmup

`ChoppinessIndex::new(14).warmup_period() == 14`. The first value lands once
the window holds a full `period` bars.

## Edge cases

- **Flat window.** A window with `high == low` everywhere has a zero span;
  `CI` is defined as `100` (maximal choppiness).
- **Steady trend.** A one-directional march reads well below `50`.
- **`period < 2`.** Rejected at construction.
- **Reset.** `ci.reset()` clears the true-range and high/low windows.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, ChoppinessIndex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ci = ChoppinessIndex::new(2)?;
    // Two H=11 L=9 C=10 bars: ΣTR = 4, span = 2 -> CI = 100·log10(2)/log10(2).
    let out = ci.batch(&[
        Candle::new(10.0, 11.0, 9.0, 10.0, 1.0, 0)?,
        Candle::new(10.0, 11.0, 9.0, 10.0, 1.0, 1)?,
    ]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[None, Some(100.0)]
```

### Python

```python
import numpy as np
import wickra as ta

ci = ta.ChoppinessIndex(2)
high = np.array([11.0, 11.0])
low = np.array([9.0, 9.0])
close = np.array([10.0, 10.0])
print(ci.batch(high, low, close))
```

Output:

```
[ nan 100.]
```

### Node

```javascript
const ta = require('wickra');
const ci = new ta.ChoppinessIndex(2);
console.log(ci.batch([11, 11], [9, 9], [10, 10]));
```

Output:

```
[ NaN, 100 ]
```

## Interpretation

The Choppiness Index is not directional — it does not say *which way* price is
going, only *whether* it is going anywhere. Use it as a regime filter: above
`61.8` favour mean-reversion / range tactics; below `38.2` favour
trend-following. It pairs naturally with a directional indicator that picks
the side once a trend is confirmed.

## Common pitfalls

- **Expecting a direction.** It has none — combine it with a trend indicator.
- **Tiny periods.** `period = 2` is allowed but noisy; `14` is conventional.

## References

E. W. Dreiss' Choppiness Index; the summed-true-range formulation here is the
standard one.

## See also

- [Indicator-VerticalHorizontalFilter.md](../trend-directional/Indicator-VerticalHorizontalFilter.md)
  — the same trending-vs-ranging question on an inverted scale.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
