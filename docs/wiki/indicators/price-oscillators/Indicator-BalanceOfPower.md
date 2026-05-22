# BalanceOfPower

> Balance of Power (BOP) — where the bar closed within its range relative
> to where it opened.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Price Oscillators |
| Input type | `Candle` (uses `open`, `high`, `low`, `close`) |
| Output type | `f64` |
| Output range | `[−1, +1]` |
| Default parameters | none (no parameters) |
| Warmup period | `1` |
| Interpretation | Intrabar buyer/seller control; `+1` buyers, `−1` sellers. |

## Formula

```
BOP = (close − open) / (high − low)
```

Balance of Power asks a single question per bar: did buyers or sellers win it?
A bar that opened on its low and closed on its high scores `+1` (buyers in
total control); the mirror image scores `−1`. It is a stateless per-bar
reading. A zero-range bar carries no information and yields `0`.

## Parameters

`BalanceOfPower` takes **no parameters** — `BalanceOfPower::new()` in Rust,
`wickra.BalanceOfPower()` in Python, `new ta.BalanceOfPower()` in Node.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/balance_of_power.rs`:

```rust
impl Indicator for BalanceOfPower {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`BalanceOfPower` is a **candle-input** indicator that reads all four of
`open`, `high`, `low`, `close`. Python's streaming `update` accepts a 6-tuple
or a dict; the batch helper takes `open`, `high`, `low`, `close` numpy arrays.
Node and WASM expose `update(open, high, low, close)` and the matching
`batch`.

## Warmup

`BalanceOfPower::new().warmup_period() == 1`. It is a stateless per-bar
transform — it emits a value from the very first candle.

## Edge cases

- **Zero-range bar.** `high == low` yields `0` instead of dividing by zero.
- **Close on high, open on low.** Scores exactly `+1`.
- **Reset.** `bop.reset()` only clears the `is_ready` flag.

## Examples

### Rust

```rust
use wickra::{Candle, Indicator, BalanceOfPower};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bop = BalanceOfPower::new();
    // open 10, high 14, low 10, close 12 -> (12 - 10) / (14 - 10) = 0.5.
    let v = bop.update(Candle::new(10.0, 14.0, 10.0, 12.0, 1.0, 0)?);
    println!("{:?}", v);
    Ok(())
}
```

Output:

```
Some(0.5)
```

### Python

```python
import numpy as np
import wickra as ta

bop = ta.BalanceOfPower()
print(bop.batch(
    np.array([10.0]), np.array([14.0]), np.array([10.0]), np.array([12.0])
))
```

Output:

```
[0.5]
```

### Node

```javascript
const ta = require('wickra');
const bop = new ta.BalanceOfPower();
console.log(bop.batch([10], [14], [10], [12]));
```

Output:

```
[ 0.5 ]
```

## Interpretation

A BOP holding above zero says buyers are consistently winning the bars — a
healthy uptrend; below zero is the seller's mirror. Because the raw per-bar
value is noisy, it is commonly smoothed with a short moving average before
trading the zero-line crossings, or read for divergence against price.

## Common pitfalls

- **Using the raw value as a trend signal.** Per-bar BOP whipsaws; smooth it.
- **Feeding it scalar prices.** It needs the full OHLC bar — including `open`.

## References

Balance of Power, popularised by Igor Livshin; the `(close − open) /
(high − low)` definition is the standard one.

## See also

- [Indicator-AwesomeOscillator.md](../momentum-oscillators/Indicator-AwesomeOscillator.md) — another
  Bill Williams-era price oscillator.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
