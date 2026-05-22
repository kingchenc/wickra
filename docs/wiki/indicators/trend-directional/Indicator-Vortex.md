# Vortex

> Vortex Indicator — a pair of oscillators (`VI+`, `VI−`) whose crossings
> identify the start of a new trend.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Trend & Directional |
| Input type | `Candle` (uses `high`, `low`, `close`) |
| Output type | `VortexOutput { plus, minus }` |
| Output range | each line `>= 0`, typically around `1.0` |
| Default parameters | `period = 14` (Python) |
| Warmup period | `period + 1` |
| Interpretation | `VI+` above `VI−` = up-trend; the cross marks the turn. |

## Formula

```
VM+_t = |high_t − low_{t−1}|        (positive vortex movement)
VM−_t = |low_t  − high_{t−1}|       (negative vortex movement)
TR_t  = true range
VI+   = Σ VM+ over period / Σ TR over period
VI−   = Σ VM− over period / Σ TR over period
```

Each vortex movement measures how far this bar reached against the
*opposite* extreme of the previous bar; dividing the running sums by the
running true range normalises both lines to a comparable scale around
`1.0`. `VI+` crossing above `VI−` signals a new up-trend; the reverse, a
down-trend.

## Parameters

| Name     | Type    | Default       | Valid range | Description |
|----------|---------|---------------|-------------|-------------|
| `period` | `usize` | `14` (Python) | `>= 1`      | Summation window. `0` errors with `Error::PeriodZero`. |

The Python binding defaults `period` to `14`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/vortex.rs`:

```rust
pub struct VortexOutput { pub plus: f64, pub minus: f64 }

impl Indicator for Vortex {
    type Input = Candle;
    type Output = VortexOutput;
}
```

`Vortex` is a **candle-input** indicator reading `high`, `low` and
`close`. The streaming `update` returns `VortexOutput` (Rust),
`(plus, minus)` (Python), or `{ plus, minus }` (Node/WASM). The batch
helper returns one row per input — a `(n, 2)` numpy array in Python, a
flat `[plus, minus, …]` array of length `2·n` in Node/WASM, with `NaN`
during warmup.

## Warmup

`Vortex::new(period).warmup_period() == period + 1`. The first VM/TR
triple needs a previous bar, so it forms on bar 2; the summation window
then needs `period` triples — the first output lands on input
`period + 1`.

## Edge cases

- **Flat market.** A window with zero total true range cannot be
  normalised; both lines are reported as `0.0`
  (`perfectly_flat_market_yields_zero` pins this).
- **Non-negative.** Both `VI+` and `VI−` are sums of absolute values over
  a non-negative range, so neither is ever negative
  (`outputs_are_non_negative` pins this).
- **Candle validation.** `Candle::new` rejects invalid bars upstream.
- **Reset.** `vortex.reset()` clears the previous bar, the window and the
  three running sums.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, Vortex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let candles = [
        Candle::new(9.0, 10.0, 8.0, 9.0, 1.0, 0)?,
        Candle::new(10.0, 12.0, 9.0, 11.0, 1.0, 1)?,
        Candle::new(12.0, 13.0, 11.0, 12.0, 1.0, 2)?,
    ];
    let mut v = Vortex::new(2)?;
    let out = v.batch(&candles);
    println!("{:?}", out[2]);
    Ok(())
}
```

Output:

```
Some(VortexOutput { plus: 1.6, minus: 0.4 })
```

Over the two formed bars `Σ VM+ = 8`, `Σ VM− = 2`, `Σ TR = 5`, giving
`VI+ = 1.6` and `VI− = 0.4`. This matches the `reference_values` test in
`crates/wickra-core/src/indicators/vortex.rs`.

### Python

```python
import numpy as np
import wickra as ta

v = ta.Vortex(14)
high = np.array([10.0, 12.0, 13.0])
low = np.array([8.0, 9.0, 11.0])
close = np.array([9.0, 11.0, 12.0])
# v.batch(high, low, close) -> (3, 2) array of [plus, minus], NaN during warmup
print(v.update((9.0, 10.0, 8.0, 9.0, 1.0, 0)))
```

### Node

```javascript
const ta = require('wickra');
const v = new ta.Vortex(14);
console.log(v.update(12, 9, 11)); // { plus, minus } or null during warmup
```

## Interpretation

`Vortex` is a trend-onset detector. The signal is the **crossing**: when
`VI+` rises above `VI−`, a new up-trend is starting; when `VI−` rises
above `VI+`, a down-trend. The gap between the lines measures conviction —
a wide, widening gap is a strong trend, converging lines warn of a stall.
Unlike a lagging moving-average cross, the vortex movements react to the
*reach* of each bar, so the cross tends to fire early.

## Common pitfalls

- **Reading the lines in isolation.** A `VI+` of `1.1` means nothing on
  its own — what matters is its position relative to `VI−`.
- **Feeding it scalar prices.** It needs `high`/`low`/`close`.

## References

Etienne Botes and Douglas Siepman, "The Vortex Indicator", *Technical
Analysis of Stocks & Commodities* (2010). The `VM±` / true-range
definition here follows their original.

## See also

- [Indicator-Adx.md](../trend-directional/Indicator-Adx.md) — Wilder's directional system.
- [Indicator-Atr.md](../volatility-bands/Indicator-Atr.md) — the true range
  Vortex normalises against.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
