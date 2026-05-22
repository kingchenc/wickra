# Coppock

> Coppock Curve — a long-horizon momentum indicator: a weighted moving
> average of two rates of change, designed to flag major bottoms.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum |
| Sub-category | Unbounded oscillators |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded around zero |
| Default parameters | `(roc_long = 14, roc_short = 11, wma_period = 10)` (Python) |
| Warmup period | `max(roc_long, roc_short) + wma_period` |
| Interpretation | Long-term momentum; an upturn from below zero is the buy signal. |

## Formula

```
Coppock = WMA( ROC(roc_long) + ROC(roc_short), wma_period )
```

Edwin Coppock built this in 1962 as a long-horizon buy signal for stock
indices. The two rates of change blend a slightly longer and a slightly
shorter momentum horizon; the [`Wma`](../trend/Indicator-Wma.md) smooths
their sum. On a **monthly** chart with the conventional
`(14, 11, 10)` settings, the curve turning *up from below zero* has
historically marked the start of a new bull phase.

## Parameters

| Name         | Type    | Default       | Valid range | Description |
|--------------|---------|---------------|-------------|-------------|
| `roc_long`   | `usize` | `14` (Python) | `>= 1`      | Longer ROC period. `0` errors with `Error::PeriodZero`. |
| `roc_short`  | `usize` | `11` (Python) | `>= 1`      | Shorter ROC period. |
| `wma_period` | `usize` | `10` (Python) | `>= 1`      | WMA smoothing length. |

The Python binding defaults the trio to `(14, 11, 10)`. The `periods`
property returns `(roc_long, roc_short, wma_period)`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/coppock.rs`:

```rust
impl Indicator for Coppock {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`warmup_period() == max(roc_long, roc_short) + wma_period`. Each ROC emits
its first value at input `roc_period + 1`; the longer ROC is the last to
become ready, and the WMA then needs `wma_period` of the summed ROC
values — so the first non-`None` output lands on input
`max(roc_long, roc_short) + wma_period`.

## Edge cases

- **Constant series.** Both ROCs are `0` on a flat series, so the WMA of
  zeros — and the curve — is `0` (`constant_series_yields_zero` pins
  this).
- **NaN / infinity inputs.** Non-finite inputs are silently dropped; no
  component is advanced.
- **Reset.** `coppock.reset()` clears both ROCs and the WMA.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Coppock};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut coppock = Coppock::new(14, 11, 10)?;
    let prices: Vec<f64> = (1..=120).map(|i| 100.0 * 1.01_f64.powi(i)).collect();
    let out = coppock.batch(&prices);
    println!("warmup_period = {}", coppock.warmup_period());
    println!("last > 0: {}", out.last().unwrap().unwrap() > 0.0);
    Ok(())
}
```

Output:

```
warmup_period = 24
last > 0: true
```

A steady uptrend keeps both ROCs positive, so the Coppock Curve stays
above zero.

### Python

```python
import numpy as np
import wickra as ta

coppock = ta.Coppock()  # (roc_long=14, roc_short=11, wma_period=10)
prices = np.full(60, 100.0)  # flat series
print(coppock.batch(prices)[-1])  # ROCs are 0 -> 0
```

Output:

```
0.0
```

### Node

```javascript
const ta = require('wickra');
const coppock = new ta.Coppock(14, 11, 10);
const prices = Array.from({ length: 120 }, (_, i) => 100 * 1.01 ** i);
console.log('warmupPeriod:', coppock.warmupPeriod());
```

## Interpretation

`Coppock` is a long-horizon signal, traditionally read on **monthly**
data. The canonical rule is a single one: when the curve has been below
zero and turns up, that is a long-term buy. It was not designed to give
sell signals — Coppock left exits to other tools. On faster timeframes it
behaves as a smoothed momentum oscillator, but its statistical edge is
specifically the monthly bottom call.

## Common pitfalls

- **Using it for sell signals.** The Coppock Curve is a buy-only
  indicator by design; pair it with a separate exit rule.
- **Applying it intraday and expecting the historical edge.** The
  documented behaviour is for monthly index charts.

## References

E. S. Coppock, "Practical Relative Strength Charting", *Barron's* (1962).
The `WMA(ROC(14) + ROC(11), 10)` construction here is Coppock's original.

## See also

- [Indicator-Roc.md](Indicator-Roc.md) — the rate-of-change building block.
- [Indicator-Wma.md](../trend/Indicator-Wma.md) — the smoothing average.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
