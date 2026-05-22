# PPO

> Percentage Price Oscillator — MACD expressed as a percentage of the slow
> EMA, so readings are comparable across instruments.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum |
| Sub-category | Unbounded oscillators |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded around zero (percent) |
| Default parameters | `(fast = 12, slow = 26)` (Python) |
| Warmup period | `slow` |
| Interpretation | Percentage gap between a fast and slow EMA; zero-line crosses are signals. |

## Formula

```
PPO = 100 · (EMA_fast − EMA_slow) / EMA_slow
```

PPO is [`MacdIndicator`](Indicator-MacdIndicator.md) divided by the slow
EMA. That single change makes it **scale-free**: a `PPO` of `1.5` always
means "the fast EMA is 1.5 % above the slow EMA", whether the instrument
trades at $5 or $5000 — so PPO values can be compared across assets and
across time, which raw MACD values cannot. The classic PPO **signal
line** is a 9-period EMA of this PPO line; compose it with
[`Chain`](../Indicator-Chaining.md) and an `Ema(9)`.

## Parameters

| Name   | Type    | Default       | Valid range      | Description |
|--------|---------|---------------|------------------|-------------|
| `fast` | `usize` | `12` (Python) | `>= 1`, `< slow` | Fast EMA period. |
| `slow` | `usize` | `26` (Python) | `> fast`         | Slow EMA period. |

`fast` must be strictly less than `slow` — otherwise `new` returns
`Error::InvalidPeriod`. A zero period returns `Error::PeriodZero`. The
Python binding defaults the pair to `(12, 26)`; the `periods` property
returns `(fast, slow)`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/ppo.rs`:

```rust
impl Indicator for Ppo {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`Ppo::new(fast, slow).warmup_period() == slow`. Both EMAs are SMA-seeded;
the slow EMA is the last to seed, at input `slow`, which is also when PPO
emits its first value.

## Edge cases

- **Constant series.** Both EMAs converge to the constant, so their gap —
  and PPO — is `0` (`constant_series_yields_zero` pins this).
- **Zero slow EMA.** A `0.0` slow EMA would divide by zero; PPO reports
  `0.0` for that bar instead.
- **NaN / infinity inputs.** Non-finite inputs are silently dropped; the
  EMAs are not advanced.
- **Reset.** `ppo.reset()` clears both EMAs and the cached value.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Ppo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ppo = Ppo::new(12, 26)?;
    let prices: Vec<f64> = (1..=80).map(f64::from).collect();
    let out = ppo.batch(&prices);
    println!("warmup_period = {}", ppo.warmup_period());
    println!("last > 0: {}", out.last().unwrap().unwrap() > 0.0);
    Ok(())
}
```

Output:

```
warmup_period = 26
last > 0: true
```

In a rising series the fast EMA leads the slow EMA, so PPO is positive.

### Python

```python
import numpy as np
import wickra as ta

ppo = ta.PPO()  # (fast=12, slow=26)
prices = np.full(60, 100.0)  # flat series
print(ppo.batch(prices)[-1])  # both EMAs equal -> 0
```

Output:

```
0.0
```

### Node

```javascript
const ta = require('wickra');
const ppo = new ta.PPO(12, 26);
const prices = Array.from({ length: 80 }, (_, i) => 100 + i);
console.log('warmupPeriod:', ppo.warmupPeriod());
```

## Interpretation

`Ppo` is read exactly like MACD: the zero-line cross (fast EMA crossing
the slow EMA), the signal-line cross (PPO crossing its own 9-EMA), and
histogram-style divergence. Its advantage over MACD is comparability — a
PPO scan across a watchlist ranks instruments by *relative* trend
strength, which a MACD scan cannot do because MACD is in each
instrument's own price units.

## Common pitfalls

- **Expecting a bundled signal line.** `Ppo` here is the single PPO line;
  add `Ema(9)` via `Chain` for the signal line and histogram.
- **`fast >= slow`.** The constructor rejects it — the fast EMA must be
  the faster one.

## References

Gerald Appel's MACD, re-expressed as a percentage. The implementation
follows the standard PPO definition and matches TA-Lib's `PPO`.

## See also

- [Indicator-MacdIndicator.md](Indicator-MacdIndicator.md) — the price-unit
  original, with a bundled signal line and histogram.
- [Indicator-Ema.md](../trend/Indicator-Ema.md) — the underlying average.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
