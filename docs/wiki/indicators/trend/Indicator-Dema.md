# DEMA

> Double Exponential Moving Average — Patrick Mulloy's `2·EMA − EMA(EMA)`,
> a single-line trend filter that removes the first-order lag of a plain
> EMA.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Trend |
| Sub-category | Exponential family |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded; tracks the input price scale |
| Default parameters | `period` is required (no default in either binding) |
| Warmup period | `2·period − 1` |
| Interpretation | EMA-style smoothing with less lag; sits ahead of `Ema` on a sustained trend. |

## Formula

Let `EMA1 = EMA(price, period)` and `EMA2 = EMA(EMA1, period)`. Then:

```
DEMA_t = 2 * EMA1_t - EMA2_t
```

Both inner EMAs use the same `period`, hence the same
`α = 2 / (period + 1)`. The subtraction is a finite-difference
approximation of "remove the lag introduced by single EMA smoothing":
if EMA lags the true series by `L`, then EMA(EMA) lags by roughly `2L`,
so `2·EMA − EMA(EMA)` cancels most of the first-order error.

## Parameters

| Name     | Type    | Default | Valid range | Description |
|----------|---------|---------|-------------|-------------|
| `period` | `usize` | none    | `>= 1`      | Period shared by both internal EMAs. `period = 0` errors with `Error::PeriodZero`. |

(Python class `wickra.DEMA(period)` has no `#[pyo3(signature)]` default;
pass `period` explicitly.)

## Inputs / Outputs

From `crates/wickra-core/src/indicators/dema.rs`:

```rust
impl Indicator for Dema {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

Python `update` returns `float | None`, `batch` returns a 1-D
`numpy.ndarray` (`float64`, `NaN` for warmup). Node `update` returns
`number | null`, `batch` returns `Array<number>` with `NaN` placeholders.

## Warmup

`Dema::new(period).warmup_period() == 2 * period - 1`. The comment in
the source explains it cleanly:

> EMA1 seeds at `period`, then EMA2 needs another `period − 1` values to
> seed.

`Ema::new(period)` only starts producing output once it has seen
`period` inputs. So `ema1` emits its first value at input `period`. From
that point on, `ema2` starts receiving inputs (the outputs of `ema1`)
and itself needs `period` of them to seed — first emission at "input
`period` of `ema1`" = input `2·period − 1` of `Dema`. For
`Dema::new(14)` this gives `27`, matching the table in
[Warmup Periods](../../Warmup-Periods.md).

The implementation uses the `?` operator to short-circuit:
`let e1 = self.ema1.update(input)?; let e2 = self.ema2.update(e1)?;`,
so `ema2` is only fed once `ema1` actually emits — which is exactly
what the warmup arithmetic above models.

## Edge cases

- **Constant series.** Feeding `[100.0; n]` eventually produces
  `Some(100.0)`: once both EMAs converge to `100.0`, the output is
  `2 · 100 − 100 = 100`. The unit test `constant_series_yields_constant_dema`
  pins this with `Dema::new(5)` over 60 constants.
- **NaN / infinity inputs.** Inherited from the inner `Ema`: non-finite
  inputs are silently dropped and the previously emitted value (if any)
  is preserved. Inputs that fail to pass `is_finite()` never reach the
  `2·EMA1 − EMA2` arithmetic.
- **Reset.** `dema.reset()` resets both internal EMAs. The next `update`
  starts a full `2·period − 1` warmup countdown.

## Examples

### Rust

```rust
use wickra::{BatchExt, Dema, Indicator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dema = Dema::new(5)?;
    let prices: Vec<f64> = (1..=20).map(f64::from).collect();
    let out: Vec<Option<f64>> = dema.batch(&prices);
    println!("warmup_period = {}", dema.warmup_period());
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
warmup_period = 9
[None, None, None, None, None, None, None, None, Some(9.0), Some(10.0), Some(11.0), Some(12.0), Some(13.000000000000002), Some(14.000000000000002), Some(15.000000000000002), Some(16.000000000000004), Some(17.0), Some(18.0), Some(19.0), Some(20.0)]
```

The first `Some` arrives at index 8 (the 9th input), exactly as
predicted by `2·5 − 1 = 9`. On a linear ramp `1, 2, …, 20`, DEMA tracks
the input ramp almost perfectly because the lag has been cancelled to
first order — the floating-point tail of `13.000000000000002` is
ordinary IEEE-754 drift. The unit test
`linear_uptrend_dema_above_ema_eventually` pins the property that
`Dema` exceeds `Ema` of the same period on a sustained uptrend.

### Python

```python
import numpy as np
import wickra as ta

dema = ta.DEMA(5)
out = dema.batch(np.arange(1.0, 21.0))
print("warmup_period =", dema.warmup_period())
print(out)
```

Output:

```
warmup_period = 9
[nan nan nan nan nan nan nan nan  9. 10. 11. 12. 13. 14. 15. 16. 17. 18.
 19. 20.]
```

### Node

```javascript
const ta = require('D:/Coding/Wickra/bindings/node');
const dema = new ta.DEMA(5);
const prices = Array.from({ length: 20 }, (_, i) => i + 1);
console.log(dema.batch(prices));
console.log('warmupPeriod:', dema.warmupPeriod());
```

Output:

```
[
                 NaN,                NaN,
                 NaN,                NaN,
                 NaN,                NaN,
                 NaN,                NaN,
                   9,                 10,
                  11,                 12,
  13.000000000000002, 14.000000000000002,
  15.000000000000002, 16.000000000000004,
                  17,                 18,
                  19,                 20
]
warmupPeriod: 9
```

## Interpretation

`Dema` is the canonical "I want EMA, but with less lag" answer. On a
sustained directional trend the DEMA line sits ahead of an `Ema` of the
same period (the unit test pins this). The same signals you use for
`Ema` — price-vs-MA crossover, fast-vs-slow MA crossover — apply, and
they fire earlier. In return for the lower lag you accept more
sensitivity to noise: on choppy data DEMA will whipsaw earlier than EMA
of the same period.

Prefer `Dema` over `Ema` when you want a faster trend filter without
moving to a smaller `period` (which would also amplify noise). Prefer
`Tema` for *even* less lag at the cost of further noise sensitivity, or
`Hma` if you want lag reduction *plus* an inherent smoothing step.

## Common pitfalls

- **Picking a `period` that's too short for a noisy market.** Because
  `Dema` removes lag rather than adding smoothing, on choppy series it
  amplifies high-frequency oscillations. If you reach for `Dema(5)` on
  a tick-by-tick feed and get a jittery line, the fix is to *raise*
  `period` — `Dema(20)` is often a better compromise than `Dema(5)`.
- **Assuming the first `Dema` value lines up with the first `Ema`
  value at the same period.** `Ema(14)` first emits at input 14;
  `Dema(14)` first emits at input 27. If you align a DEMA series to an
  EMA series in a backtest, account for the offset or use the
  `~np.isnan(...)` mask (Python) / `is_some()` filter (Rust) to drop the
  warmup rows.

## References

Patrick G. Mulloy, *"Smoothing Data with Faster Moving Averages"*,
**Technical Analysis of Stocks & Commodities**, January 1994 (DEMA), and
*"Smoothing Data with Less Lag"*, **Technical Analysis of Stocks &
Commodities**, February 1994 (TEMA).

## See also

- [Indicator-Ema.md](Indicator-Ema.md) — the building block.
- [Indicator-Tema.md](Indicator-Tema.md) — three-EMA version, less lag still.
- [Indicator-Hma.md](Indicator-Hma.md) — same lag-reduction goal, built on WMAs.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
