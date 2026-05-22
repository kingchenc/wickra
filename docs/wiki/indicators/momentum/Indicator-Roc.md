# ROC

> Rate of Change — the percent change between the current close and the
> close `period` bars ago.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum |
| Sub-category | unbounded oscillator |
| Input type | `f64` (close) |
| Output type | `f64` |
| Output range | unbounded (centred on 0; expressed as a percent) |
| Default parameters | none — `period` is required in every binding |
| Warmup period | `period + 1` (13 for `period = 12`) |
| Interpretation | sign and magnitude of momentum; zero-line crossover for direction changes |

## Formula

```
ROC_t  =  (close_t − close_{t − period}) / close_{t − period} · 100
```

When `close_{t − period}` is exactly zero, the implementation returns
`0.0` rather than dividing by zero. The unit test `known_value` pins the
basic case: with `period = 3`, inputs `[100, 105, 108, 110]` produce
ROC `= 10` at index 3 (because `(110 − 100) / 100 · 100 = 10`).

## Parameters

| Name | Type | Default | Valid range | Description |
|------|------|---------|-------------|-------------|
| `period` | `usize` | required | `>= 1` | Lookback distance for the comparison close. |

`Roc::new(0)` returns `Error::PeriodZero`. The Python and Node bindings
do **not** assign a default for `period`; you must pass it explicitly.

## Inputs / Outputs

From `impl Indicator for Roc`:

```rust
type Input  = f64;
type Output = f64;
fn update(&mut self, input: f64) -> Option<f64>;
```

Python's `ROC.batch(prices)` returns a 1-D `float64` `np.ndarray`. Node's
`ROC.batch(prices)` returns a flat `number[]`. Streaming `update(price)`
returns a scalar (`float` / `number`) or `None` / `null` during warmup.

## Warmup

`warmup_period()` returns `period + 1`. The reason is the same off-by-one
as RSI: ROC compares against the close `period` bars ago, so at the
`period`-th input we still have nothing to look back at — the `(period +
1)`-th input is the first one for which `close_{t − period}` exists.
Internally the rolling buffer is sized `period + 1`.

## Edge cases

- **Constant input.** Every diff is zero, so `ROC == 0` for every emitted
  value (test `constant_series_yields_zero`).
- **Reference close of zero.** Treated as `0.0` rather than producing
  `NaN`/`±∞` — see the `prev == 0.0` early return in `update`. This
  matters for assets quoted with zero as a legitimate value (rare for
  prices, but possible for, e.g., yield spreads).
- **Non-finite input.** `update(NaN)` or `update(±∞)` returns `None`
  without advancing the rolling buffer.
- **Reset.** `reset()` clears the rolling buffer; the next `period + 1`
  updates return `None`.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Roc};

let mut roc = Roc::new(3)?;
let out = roc.batch(&[100.0, 105.0, 108.0, 110.0]);
println!("ROC(3) at idx 3 = {}", out[3].unwrap());
# Ok::<(), wickra::Error>(())
```

Verified output:

```
ROC(3) at idx 3 = 10
```

### Python

```python
import wickra as ta

roc = ta.ROC(3)
print('warmup:', roc.warmup_period())
for p in [100.0, 105.0, 108.0, 110.0]:
    print(p, '->', roc.update(p))
```

Verified output:

```
warmup: 4
100.0 -> None
105.0 -> None
108.0 -> None
110.0 -> 10.0
```

### Node

```javascript
const wickra = require('wickra');

const roc = new wickra.ROC(3);
console.log('warmup:', roc.warmupPeriod());
for (const p of [100, 105, 108, 110]) {
  console.log(p, '->', roc.update(p));
}
```

Verified output:

```
warmup: 4
100 -> null
105 -> null
108 -> null
110 -> 10
```

## Interpretation

- **Sign.** Positive ROC means price is higher than `period` bars ago;
  negative means lower. The magnitude is the percent move.
- **Zero-line crossover.** A move through zero signals a regime change
  in the `period`-bar horizon. Combined with a longer-period ROC, this
  gives you a poor-man's trend filter.
- **Divergence.** A new price high paired with a lower ROC high is the
  same bearish-divergence pattern as RSI/Stochastic, with the
  unbounded-oscillator caveat that "lower high" is unambiguous (no
  saturation against a `100` ceiling).

## Common pitfalls

- **ROC is unbounded.** A 10× price spike over `period` bars produces
  `ROC = 900`. Don't pipe ROC directly into rule sets designed for
  bounded oscillators (RSI, %K, %R) without an explicit clamp or a
  log-return transformation upstream.
- **Off-by-one on the warmup.** The first non-`None` value lands at the
  `(period + 1)`-th input, not the `period`-th. A common bug is sizing
  an output array as `len(prices) - period` and getting an off-by-one
  empty row at the end.

## References

- Robert Colby, *The Encyclopedia of Technical Market Indicators*,
  2nd ed., McGraw-Hill, 2002 — Chapter on Rate of Change / Momentum,
  covering the canonical percent and ratio formulations.

## See also

- [Indicator: Rsi](Indicator-Rsi.md) — same `period + 1` warmup, but
  bounded.
- [Indicator: Trix](Indicator-Trix.md) — also a rate of change, but on
  a triple-smoothed EMA.
- [Indicator: MacdIndicator](Indicator-MacdIndicator.md) — momentum
  cousin operating on EMA differences instead of raw close differences.
- [Warmup Periods](../../Warmup-Periods.md) — the `period + 1` family.
