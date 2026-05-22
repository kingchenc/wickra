# PMO

> Price Momentum Oscillator — Carl Swenlin's DecisionPoint PMO line: a
> doubly-smoothed rate of change.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum Oscillators |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded around zero |
| Default parameters | `(smoothing1 = 35, smoothing2 = 20)` (Python) |
| Warmup period | `2` |
| Interpretation | Smoothed momentum; zero-line and signal-line crosses are the signals. |

## Formula

```
roc_t      = (price_t / price_{t−1} − 1) · 100
smoothed_t = customEMA(roc, smoothing1)_t
PMO_t      = customEMA(10 · smoothed, smoothing2)_t
```

`customEMA` is the DecisionPoint smoothing: an exponential average whose
smoothing constant is `2 / period` (not the textbook `2 / (period + 1)`),
seeded from its first input. The 1-bar percentage change is smoothed once,
scaled by `10`, then smoothed again.

The classic PMO **signal line** is a 10-period EMA of this PMO line. It is
deliberately not bundled in — compose it yourself with
[`Chain`](../../Indicator-Chaining.md) and an `Ema(10)`.

## Parameters

| Name         | Type    | Default       | Valid range | Description |
|--------------|---------|---------------|-------------|-------------|
| `smoothing1` | `usize` | `35` (Python) | `>= 2`      | First smoothing period (applied to ROC). `0` errors with `Error::PeriodZero`; `1` with `Error::InvalidPeriod`. |
| `smoothing2` | `usize` | `20` (Python) | `>= 2`      | Second smoothing period (applied to `10 · smoothed`). Same error rules. |

`smoothing = 1` is rejected because the smoothing constant `2 / 1 = 2`
would exceed `1`. The Python binding defaults the pair to `(35, 20)` via
`#[pyo3(signature = (smoothing1=35, smoothing2=20))]`. The `periods`
property returns `(smoothing1, smoothing2)`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/pmo.rs`:

```rust
impl Indicator for Pmo {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`Pmo::new(s1, s2).warmup_period() == 2`. The first ROC needs a previous
price, and both `customEMA`s seed from their very first input, so the
first non-`None` output lands on the **second** `update()`. Note this is
the first *defined* value; the doubly-smoothed series only stabilises
after many more bars, so treat early readings as unsettled.

## Edge cases

- **Constant series.** A flat series gives `roc = 0` on every bar, so both
  smoothings stay at `0` and PMO is `0.0`
  (`constant_series_yields_zero` pins this).
- **Zero previous price.** A ratio against a `0.0` prior price is
  undefined; `roc` is treated as `0` for that bar.
- **NaN / infinity inputs.** Non-finite inputs are silently dropped; the
  smoothing chains are not advanced.
- **Reset.** `pmo.reset()` clears the previous price and both EMAs.

## Examples

### Rust

```rust
use wickra::{Indicator, Pmo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut pmo = Pmo::new(35, 20)?;
    println!("{:?}", pmo.update(100.0)); // no previous price yet
    println!("{:?}", pmo.update(101.0)); // first defined PMO
    Ok(())
}
```

Output:

```
None
Some(10.0)
```

The first `update` only records the price. The second produces
`roc = 1.0%`; each `customEMA` seeds from its first input, so the inner
EMA emits `1.0`, the `×10` scaling gives `10.0`, and the outer EMA seeds
at `10.0` — hence `PMO = 10.0` on the first defined bar. Early values are
seed artefacts: the double smoothing only settles after many more bars.
This matches the `first_emission_at_second_update` test in
`crates/wickra-core/src/indicators/pmo.rs`.

### Python

```python
import numpy as np
import wickra as ta

pmo = ta.PMO()  # (smoothing1=35, smoothing2=20)
prices = 100.0 * 1.01 ** np.arange(120)  # steady uptrend
out = pmo.batch(prices)
print("last > 0:", out[-1] > 0)
```

Output:

```
last > 0: True
```

### Node

```javascript
const ta = require('wickra');
const pmo = new ta.PMO(35, 20);
const prices = Array.from({ length: 120 }, (_, i) => 100 * 1.01 ** i);
console.log('last:', pmo.batch(prices).at(-1));
```

## Interpretation

`Pmo` is a smoothed momentum line. The DecisionPoint reads are: PMO
crossing its zero line (momentum changing sign), PMO crossing its signal
line (a 10-EMA of PMO — build it with `Chain`), and PMO turning up/down
from an extreme. Because the rate of change is taken in percentage terms,
PMO values *are* comparable across instruments — unlike raw
[`Mom`](../momentum-oscillators/Indicator-Mom.md).

## Common pitfalls

- **Trusting the first few values.** `warmup_period()` is `2`, but that is
  only the first *defined* output — the double smoothing needs many bars
  to settle. Discard the early ramp.
- **Expecting a bundled signal line.** PMO here is the single PMO line;
  add `Ema(10)` via `Chain` for the signal.

## References

Carl Swenlin, DecisionPoint Price Momentum Oscillator. The
`2 / period` "custom smoothing", the `×10` scaling and the conventional
`(35, 20)` periods follow the published DecisionPoint definition.

## See also

- [Indicator-Roc.md](../momentum-oscillators/Indicator-Roc.md) — the raw rate of change PMO smooths.
- [Indicator-Tsi.md](../momentum-oscillators/Indicator-Tsi.md) — another double-smoothed momentum
  oscillator.
- [Indicator-Chaining.md](../../Indicator-Chaining.md) — how to add the
  signal-line EMA.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
