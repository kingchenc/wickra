# TEMA

> Triple Exponential Moving Average — Mulloy's
> `3·EMA1 − 3·EMA2 + EMA3` (where each EMA is fed from the previous one),
> the second-order lag-reduction sibling of DEMA.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Trend |
| Sub-category | Exponential family |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded; tracks the input price scale |
| Default parameters | `period` is required (no default in either binding) |
| Warmup period | `3·period − 2` |
| Interpretation | Even less lag than `Dema`, at the cost of more noise sensitivity. |

## Formula

Let `EMA1 = EMA(price, period)`, `EMA2 = EMA(EMA1, period)`,
`EMA3 = EMA(EMA2, period)`. Then:

```
TEMA_t = 3 * EMA1_t - 3 * EMA2_t + EMA3_t
```

All three EMAs share the same `period`, hence the same
`α = 2 / (period + 1)`. The coefficients `(3, −3, 1)` are the
second-order finite-difference correction that removes both the
first-order and second-order EMA lag terms — they come from expanding
`(1 − L)^{-3}` where `L` is the lag operator.

## Parameters

| Name     | Type    | Default | Valid range | Description |
|----------|---------|---------|-------------|-------------|
| `period` | `usize` | none    | `>= 1`      | Period shared by all three internal EMAs. `period = 0` errors with `Error::PeriodZero`. |

(Python class `wickra.TEMA(period)` has no `#[pyo3(signature)]` default;
pass `period` explicitly.)

## Inputs / Outputs

From `crates/wickra-core/src/indicators/tema.rs`:

```rust
impl Indicator for Tema {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

Python `update` returns `float | None`, `batch` returns a 1-D
`numpy.ndarray` (`float64`, `NaN` for warmup). Node `update` returns
`number | null`, `batch` returns `Array<number>` with `NaN`
placeholders.

## Warmup

`Tema::new(period).warmup_period() == 3 * period - 2`. Each stacked EMA
adds `period − 1` more inputs to the warmup count:

- `ema1` emits first at input `period`.
- `ema2`, fed from `ema1`, emits first at input `period + (period − 1) = 2·period − 1`.
- `ema3`, fed from `ema2`, emits first at input `(2·period − 1) + (period − 1) = 3·period − 2`.

For `Tema::new(14)` this gives `40` (matches the table in
[Warmup Periods](../../Warmup-Periods.md)); for `Tema::new(5)` (the example
below) it gives `13`. The implementation uses `?` short-circuit on every
stage, so each inner EMA is only fed once the previous one emits.

## Edge cases

- **Constant series.** Feeding `[42.0; n]` produces `Some(42.0)` once all
  three EMAs have converged: `3·42 − 3·42 + 42 = 42`. The unit test
  `constant_series_yields_constant_tema` pins this with `Tema::new(5)`
  over 80 constants.
- **NaN / infinity inputs.** Inherited from the inner `Ema`: non-finite
  inputs are silently dropped at the `ema1` boundary and never reach the
  `3·EMA1 − 3·EMA2 + EMA3` arithmetic.
- **Reset.** `tema.reset()` resets all three internal EMAs; the next
  `update` starts a full `3·period − 2` warmup countdown.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Tema};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tema = Tema::new(5)?;
    let prices: Vec<f64> = (1..=20).map(f64::from).collect();
    let out: Vec<Option<f64>> = tema.batch(&prices);
    println!("warmup_period = {}", tema.warmup_period());
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
warmup_period = 13
[None, None, None, None, None, None, None, None, None, None, None, None, Some(13.0), Some(14.0), Some(15.000000000000002), Some(16.000000000000004), Some(17.000000000000007), Some(18.000000000000007), Some(19.000000000000007), Some(20.0)]
```

The first `Some` lands at index 12 (the 13th input), matching
`3·5 − 2 = 13`. On the linear ramp `1, 2, …, 20`, TEMA tracks the input
ramp essentially exactly because both first- and second-order lag have
been cancelled; the floating-point tail
(`15.000000000000002`, `16.000000000000004`, …) is ordinary IEEE-754
drift from the recursive subtractions.

### Python

```python
import numpy as np
import wickra as ta

tema = ta.TEMA(5)
out = tema.batch(np.arange(1.0, 21.0))
print("warmup_period =", tema.warmup_period())
print(out)
```

Output:

```
warmup_period = 13
[nan nan nan nan nan nan nan nan nan nan nan nan 13. 14. 15. 16. 17. 18.
 19. 20.]
```

### Node

```javascript
const ta = require('D:/Coding/Wickra/bindings/node');
const tema = new ta.TEMA(5);
const prices = Array.from({ length: 20 }, (_, i) => i + 1);
console.log(tema.batch(prices));
console.log('warmupPeriod:', tema.warmupPeriod());
```

Output:

```
[
                 NaN,                NaN,
                 NaN,                NaN,
                 NaN,                NaN,
                 NaN,                NaN,
                 NaN,                NaN,
                 NaN,                NaN,
                  13,                 14,
  15.000000000000002, 16.000000000000004,
  17.000000000000007, 18.000000000000007,
  19.000000000000007,                 20
]
warmupPeriod: 13
```

## Interpretation

`Tema` removes more lag than `Dema` and noticeably more than `Ema`.
On a clean trending series the line stays glued to price; on a noisy
or sideways series the same lag-cancellation amplifies the noise — TEMA
overshoots and reverses faster than DEMA, and very much faster than EMA.

The signals are the same crossover patterns: price-vs-TEMA and
fast-TEMA-vs-slow-TEMA. The `(3, −3, 1)` coefficient pattern is also
what makes `Trix` (also in this family) work — `Trix` is the percentage
change of `EMA3`, the triple-smoothed series.

Prefer `Tema` when `Dema` still feels too laggy and your data is clean
enough to tolerate the extra noise sensitivity. Prefer `Hma` if you want
a similar lag profile but with a built-in smoothing step (WMA chain
instead of EMA chain), which behaves more gracefully on noisy data.

## Common pitfalls

- **Forgetting the `3·period − 2` warmup.** `Tema::new(50)` will not
  emit until input 148. That is a significant chunk of any short-term
  backtest. If you are running a side-by-side panel of indicators with
  different warmups, filter rows on `~np.isnan(...)` (Python) /
  `is_some()` (Rust) per indicator rather than picking one global
  warmup cutoff.
- **Using TEMA for noisy intraday data without a smoothing step.** The
  same lag-cancellation that makes TEMA attractive on clean data turns
  into whipsaws on tick-by-tick feeds. Either raise `period` materially
  or switch to `Hma`, which has a final WMA smoothing pass built in.

## References

Patrick G. Mulloy, *"Smoothing Data with Less Lag"*, **Technical Analysis
of Stocks & Commodities**, February 1994 (TEMA). The coefficient pattern
`(3, −3, 1)` for cancelling first- and second-order EMA lag is derived
in the same article.

## See also

- [Indicator-Ema.md](Indicator-Ema.md) — the building block.
- [Indicator-Dema.md](Indicator-Dema.md) — second-order's sibling.
- [Indicator-Hma.md](Indicator-Hma.md) — similar lag profile, built on WMAs.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
