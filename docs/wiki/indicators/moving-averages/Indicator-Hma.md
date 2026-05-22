# HMA

> Hull Moving Average — Alan Hull's
> `WMA(2·WMA(n/2) − WMA(n), √n)`, a near-lag-free trend filter that
> combines a fast `Wma(n/2)`, a slow `Wma(n)`, and a final smoothing pass.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Moving Averages |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded; tracks the input price scale |
| Default parameters | `period` is required (no default in either binding) |
| Warmup period (`warmup_period()`) | `period + round(√period).max(1) − 1` — exact first-emission index |
| Interpretation | Near-zero-lag trend line with an inherent smoothing step. |

## Formula

```
half   = max(period / 2, 1)                       // integer division
smooth = max(round(sqrt(period)), 1)              // nearest integer, floor at 1

raw_t  = 2 * WMA(price, half)_t  -  WMA(price, period)_t
HMA_t  = WMA(raw, smooth)_t
```

The "magic" is the `2·WMA(n/2) − WMA(n)` step: the fast WMA leads the
slow WMA on a trend, so doubling the fast and subtracting the slow
produces a series that is *ahead* of the input by roughly the WMA lag.
The final `WMA(…, √n)` then smooths the resulting overshoot back down
to a clean line. For `period = 9` this gives `half = 4`, `smooth = 3`;
for `period = 14`, `half = 7`, `smooth = 4`.

## Parameters

| Name     | Type    | Default | Valid range | Description |
|----------|---------|---------|-------------|-------------|
| `period` | `usize` | none    | `>= 1`      | Top-level lookback. The inner WMA periods are derived from it. `period = 0` errors with `Error::PeriodZero`. |

(Python class `wickra.HMA(period)` has no `#[pyo3(signature)]` default;
pass `period` explicitly.)

## Inputs / Outputs

From `crates/wickra-core/src/indicators/hma.rs`:

```rust
impl Indicator for Hma {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

Python returns `float | None` (streaming) / `numpy.ndarray` (batch,
`NaN` for warmup). Node returns `number | null` / `Array<number>` with
`NaN`.

## Warmup

`warmup_period()` returns:

```
period + round(sqrt(period)).max(1) - 1
```

which gives `11` for `Hma::new(9)`, `17` for `Hma::new(14)`,
`19` for `Hma::new(16)`. This figure is **exact**: the first non-`None`
output lands on input `warmup_period()` (index `warmup_period() - 1`).

The number reflects how the three inner WMAs warm up *in parallel*: the
slow `WMA(period)` emits at input `period`, then the smoothing
`WMA(√period)` needs `√period − 1` more inputs on top.

```rust
fn update(&mut self, input: f64) -> Option<f64> {
    // Both raw WMAs are fed unconditionally so neither delays the other.
    let h = self.half_wma.update(input);
    let f = self.full_wma.update(input);
    match (h, f) {
        (Some(h), Some(f)) => self.smooth_wma.update(2.0 * h - f),
        _ => None,
    }
}
```

`half_wma` and `full_wma` receive every input, so `full_wma` emits at
input `period` (not later). The `2·half − full` diff then flows into
`smooth_wma`, which needs `round(√period)` of those — giving a first
emission at exactly `period + round(√period) − 1`.

| `period` | `round(√period)` | `warmup_period()` | First emission (input #) |
|----------|------------------|-------------------|--------------------------|
| 9        | 3                | 11                | 11                       |
| 14       | 4                | 17                | 17                       |
| 16       | 4                | 19                | 19                       |

This is pinned by the `first_emission_matches_warmup_period` test in
`hma.rs`: the first call that returns `Some` is exactly at
`warmup_period() - 1` (0-indexed).

## Edge cases

- **Constant series.** Feeding `[10.0; n]` produces `Some(10.0)` once
  the chain is warm. All three WMAs converge to `10`, so
  `raw = 2·10 − 10 = 10`, then `WMA(10, smooth) = 10`. The unit test
  `constant_series_yields_constant_hma` pins this with `Hma::new(9)`
  over 80 constants.
- **NaN / infinity inputs.** Inherited from the inner `Wma`: non-finite
  inputs are silently dropped at the half/full WMA boundary and never
  reach the `2·h − f` arithmetic.
- **Reset.** `hma.reset()` resets all three internal WMAs; the next
  `update` starts a full warmup countdown.

## Examples

### Rust

```rust
use wickra::{BatchExt, Hma, Indicator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut hma = Hma::new(9)?;
    let prices: Vec<f64> = (1..=20).map(f64::from).collect();
    let out: Vec<Option<f64>> = hma.batch(&prices);
    println!("warmup_period = {}", hma.warmup_period());
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
warmup_period = 11
[None, None, None, None, None, None, None, None, None, None, Some(11.0), Some(12.0), Some(13.0), Some(14.0), Some(15.0), Some(16.0), Some(17.0), Some(18.0), Some(19.0), Some(20.0)]
```

The first `Some` lands at index 10 (the 11th input) — exactly
`warmup_period() - 1`, as the [Warmup](#warmup) section explains. On the
linear ramp `1, 2, …, 20`, HMA tracks price exactly with no visible lag.

### Python

```python
import numpy as np
import wickra as ta

hma = ta.HMA(9)
out = hma.batch(np.arange(1.0, 21.0))
print("warmup_period =", hma.warmup_period())
print(out)
```

Output:

```
warmup_period = 11
[nan nan nan nan nan nan nan nan nan nan 11. 12. 13. 14. 15. 16. 17. 18.
 19. 20.]
```

### Node

```javascript
const ta = require('wickra');
const hma = new ta.HMA(9);
const prices = Array.from({ length: 20 }, (_, i) => i + 1);
console.log(hma.batch(prices));
console.log('warmupPeriod:', hma.warmupPeriod());
```

Output:

```
[
  NaN, NaN, NaN, NaN, NaN, NaN,
  NaN, NaN, NaN, NaN,  11,  12,
   13,  14,  15,  16,  17,  18,
   19,  20
]
warmupPeriod: 11
```

## Interpretation

`Hma` is the lag-reduction trend filter that does *not* require you to
choose between responsiveness and noise: the final `WMA(√period)` pass
is a built-in smoothing step that prevents the kind of whipsaw a `Tema`
of the same period would produce on noisy data. On clean trending data
it sits effectively on top of price; on choppy data the smoothing pass
keeps the line readable.

The textbook signal is colour-coded slope: HMA turning up = uptrend,
turning down = downtrend. Crossover patterns (`Hma(9)` vs `Hma(20)`)
also work and tend to be cleaner than the equivalent EMA pair.

Prefer `Hma` over `Dema` / `Tema` when your data is noisy enough that
the lag-reduction in those would manifest as whipsaws. Prefer `Tema` /
`Dema` on cleaner data where you want one fewer smoothing step.

## Common pitfalls

- **Mis-reading the warmup as a lag.** `warmup_period()` is the exact
  first-emission index (`Hma::new(9).warmup_period() == 11`, first
  `Some` at the 11th input), so it can be used directly for `Chain`
  alignment. The leading `None`/`NaN` values are warmup, not lag — once
  HMA emits it tracks price with near-zero lag.
- **Picking `period = 2` or `3`.** The inner `half = period / 2` is an
  integer division floored at 1. For `period = 2`, `half = 1`,
  `smooth = 1`, and you essentially end up with `Wma(2·price − WMA(2))`
  which is a sharp, noisy line. HMA is designed for `period >= 9` or so;
  for shorter lookbacks reach for `Ema(period)` or `Wma(period)` instead.

## References

Alan Hull, *"How to Reduce Lag in a Moving Average"*, 2005 — the
original HMA derivation, hosted on Hull's site at
<https://alanhull.com/hull-moving-average>.

## See also

- [Indicator-Wma.md](../moving-averages/Indicator-Wma.md) — the building block.
- [Indicator-Tema.md](../moving-averages/Indicator-Tema.md) — same lag-reduction goal, EMA-based.
- [Indicator-Kama.md](../moving-averages/Indicator-Kama.md) — adaptive smoothing instead of fixed.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
