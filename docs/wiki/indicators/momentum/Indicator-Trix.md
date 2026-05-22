# TRIX

> Triple-EMA percent rate of change — applies three EMAs in sequence to
> smooth out short-term noise, then reports the one-bar percent change
> of the resulting series.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum |
| Sub-category | unbounded oscillator (zero-centred) |
| Input type | `f64` (close) |
| Output type | `f64` |
| Output range | unbounded (typically a few percent, centred on 0) |
| Default parameters | none — `period` is required in every binding |
| Warmup period | `3 · period − 1` (44 for `period = 15`) |
| Interpretation | zero-line crossings as trend-change cues; magnitude as momentum |

## Formula

Let `EMA_n(·)` denote Wickra's EMA over `n` periods (seeded from the
simple mean of the first `n` inputs, then recursive with `α = 2/(n+1)`).
For each input close, build a triple-smoothed series:

```
TR_t  =  EMA_period( EMA_period( EMA_period( close ) ) )_t
```

Then TRIX is the one-bar percent rate of change of `TR`:

```
TRIX_t  =  100 · (TR_t − TR_{t-1}) / TR_{t-1}
```

When `TR_{t-1} == 0` exactly, the implementation returns `0.0` rather
than dividing by zero.

## Parameters

| Name | Type | Default | Valid range | Description |
|------|------|---------|-------------|-------------|
| `period` | `usize` | required | `>= 1` | Period shared by all three EMAs. |

`Trix::new(0)` returns `Error::PeriodZero` (via the inner `Ema::new`).
The Python and Node bindings expose no default for `period`; you must
pass it explicitly.

## Inputs / Outputs

From `impl Indicator for Trix`:

```rust
type Input  = f64;
type Output = f64;
fn update(&mut self, input: f64) -> Option<f64>;
```

Python's `TRIX.batch(prices)` returns a 1-D `float64` `np.ndarray`
(warmup → `NaN`). Node's `TRIX.batch(prices)` returns a flat
`number[]` (warmup → `NaN`). Both also expose streaming `update(price)`.

## Warmup

`warmup_period()` returns `3 · period − 1`. Three stacked EMAs of the
same period seed at input `3 · period − 2`; once `TR` exists, TRIX
itself needs one more input to form the `TR_t − TR_{t-1}` difference,
which lands at input `3 · period − 1`. For `period = 15` this is
`3 · 15 − 1 = 44`, verified above.

## Edge cases

- **Constant input.** All three EMAs converge to the constant value, so
  `TR_t − TR_{t-1} == 0` and TRIX returns `0` (test
  `constant_series_yields_zero_trix`).
- **`TR_{t-1} == 0`.** The implementation returns `0` rather than
  producing `NaN` / `±∞`. This is the `Some(_)` branch with `prev !=
  0.0`-failed in `Trix::update`.
- **Reset.** `reset()` resets all three EMAs and clears `prev_tr`.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Trix};

let prices: Vec<f64> = (1..=50).map(|i| i as f64).collect();
let mut trix = Trix::new(15)?;
let out = trix.batch(&prices);
println!("row 43 = {}", out[43].unwrap());
println!("row 49 = {}", out[49].unwrap());
# Ok::<(), wickra::Error>(())
```

Verified output:

```
row 43 = 4.545454545454546
row 49 = 3.5714285714285716
```

(The series decays toward zero as a ramp gets longer because the
percent change of an arithmetic ramp shrinks as the level grows.)

### Python

```python
import wickra as ta

trix = ta.TRIX(15)
print('warmup:', trix.warmup_period())
vals = []
for i in range(1, 51):
    vals.append(trix.update(float(i)))
print('vals[43]:', vals[43])
print('vals[49]:', vals[49])
```

Verified output:

```
warmup: 44
vals[43]: 4.545454545454546
vals[49]: 3.5714285714285716
```

### Node

```javascript
const wickra = require('wickra');

const trix = new wickra.TRIX(15);
console.log('warmup:', trix.warmupPeriod());
const vals = [];
for (let i = 1; i <= 50; i++) vals.push(trix.update(i));
console.log('vals[43]:', vals[43]);
console.log('vals[49]:', vals[49]);
```

Verified output:

```
warmup: 44
vals[43]: 4.545454545454546
vals[49]: 3.5714285714285716
```

## Interpretation

- **Zero-line cross.** TRIX crossing above zero suggests the
  triple-smoothed trend is turning up; crossing below, turning down.
  Because of the triple smoothing, these crosses are deliberately
  late and deliberately stable.
- **Magnitude.** A larger absolute TRIX value means the smoothed series
  is changing faster per bar. There is no canonical "overbought" band
  — TRIX is interpreted by its sign and slope, not by threshold.
- **Compare to MACD.** Both are EMA-based momentum oscillators on a
  zero-centred scale. MACD reacts faster (two EMAs, one diff); TRIX
  reacts slower (three EMAs, one rate of change), making it a
  cleaner long-horizon trend filter.

## Common pitfalls

- **Long warmup.** `3 · period − 1` is one of the largest warmups in
  the library (44 for the canonical `period = 15`). Sizing your input
  buffer to `period` and expecting values immediately will hand you
  `None` / `NaN` for a full 44 bars.
- **Triple smoothing kills small wiggles.** TRIX deliberately ignores
  short-term noise. Do not use it for entry-timing inside a fast
  oscillator strategy; use it as a long-term trend filter on top of a
  faster signal.

## References

- Jack Hutson, "Good TRIX", *Technical Analysis of Stocks &
  Commodities*, July 1983 — the original publication popularising the
  triple-EMA rate-of-change oscillator.

## See also

- [Indicator: MacdIndicator](Indicator-MacdIndicator.md) — faster
  EMA-based momentum oscillator, useful as a confirmation against
  TRIX zero-line crosses.
- [Indicator: Roc](Indicator-Roc.md) — the raw, one-stage rate of
  change TRIX is built on top of.
- [Warmup Periods](../../Warmup-Periods.md) — `3 · period − 1` entry.
