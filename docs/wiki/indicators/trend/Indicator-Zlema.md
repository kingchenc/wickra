# ZLEMA

> Zero-Lag Exponential Moving Average — an EMA fed a de-lagged price series
> so it tracks turns with almost no group delay.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Trend |
| Sub-category | Exponential family |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded; tracks the input price scale |
| Default parameters | `period` is required (no default in either binding) |
| Warmup period | `lag + period` where `lag = (period − 1) / 2` |
| Interpretation | Low-lag trend line; crossings of price react far sooner than a plain EMA. |

## Formula

```
lag        = (period − 1) / 2          (integer division)
de_lagged_t = 2·price_t − price_{t−lag}
ZLEMA_t     = EMA_period(de_lagged)_t
```

The trick (Ehlers & Way, 2010): `price_t − price_{t−lag}` is a momentum
term. Adding it to the current price *over-shoots* in the direction of the
recent move by exactly enough to cancel the EMA's lag. The inner EMA then
smooths that de-lagged series with the usual `α = 2 / (period + 1)`.

## Parameters

| Name     | Type    | Default | Valid range | Description |
|----------|---------|---------|-------------|-------------|
| `period` | `usize` | none    | `>= 1`      | EMA length. `period = 0` errors with `Error::PeriodZero`. The lag offset is derived as `(period − 1) / 2`. |

There is no Python `#[pyo3(signature = …)]` default for `ZLEMA`, so
`wickra.ZLEMA(period)` requires the period explicitly. The derived `lag`
is exposed as a read-only property.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/zlema.rs`:

```rust
impl Indicator for Zlema {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`Zlema::new(period).warmup_period() == lag + period`. The de-lagged series
is undefined until `lag` prior inputs exist, so it produces its first
value on input `lag + 1`; the inner EMA then needs `period` de-lagged
values to seed. The first non-`None` output therefore lands on input
`lag + period`.

## Edge cases

- **Constant series.** De-lagging a constant gives the same constant
  (`2c − c = c`), so `ZLEMA` of a flat series is flat
  (`constant_series_yields_the_constant` pins this).
- **NaN / infinity inputs.** Non-finite inputs are silently dropped: the
  rolling lag buffer is not advanced and the inner EMA is not fed, so the
  previous valid value (if any) is returned.
- **`period = 1`.** `lag = 0`, the de-lagged series equals the raw price,
  and `ZLEMA(1)` degenerates to a pass-through.
- **Reset.** `zlema.reset()` clears the lag buffer and the inner EMA.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Zlema};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut zlema = Zlema::new(3)?;
    let out: Vec<Option<f64>> = zlema.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    println!("{:?}", out);
    println!("lag = {}, warmup_period = {}", zlema.lag(), zlema.warmup_period());
    Ok(())
}
```

Output:

```
[None, None, None, Some(4.0), Some(5.0)]
lag = 1, warmup_period = 4
```

`ZLEMA(3)` has `lag = 1`. The de-lagged series of `[1,2,3,4,5]` is
`[_, 3, 4, 5, 6]`; `EMA(3)` of that seeds at `mean(3,4,5) = 4.0`, then
`0.5·6 + 0.5·4 = 5.0`. This matches the `reference_values` test in
`crates/wickra-core/src/indicators/zlema.rs`.

### Python

```python
import numpy as np
import wickra as ta

zlema = ta.ZLEMA(3)
print(zlema.batch(np.array([1.0, 2.0, 3.0, 4.0, 5.0])))
print("lag =", zlema.lag, "warmup_period =", zlema.warmup_period())
```

Output:

```
[nan nan nan  4.  5.]
lag = 1 warmup_period = 4
```

### Node

```javascript
const ta = require('wickra');
const zlema = new ta.ZLEMA(3);
console.log(zlema.batch([1, 2, 3, 4, 5]));
console.log('warmupPeriod:', zlema.warmupPeriod());
```

Output:

```
[ NaN, NaN, NaN, 4, 5 ]
warmupPeriod: 4
```

## Interpretation

`Zlema` is a low-lag trend line. Use it where an `Ema` would lag too much
into a reversal — for example as the fast leg of a crossover system, or
as a trailing reference that should react quickly. The momentum injection
that removes the lag also makes `Zlema` overshoot on sharp spikes, so it
is noisier than the `Ema` it is built on; pair it with a slower filter if
whipsaws are a concern.

## Common pitfalls

- **Expecting `Ema`-identical values.** `Zlema` is deliberately *not* an
  `Ema` — it leads price. The two only coincide for `period = 1`.
- **Forgetting the extra warmup.** Warmup is `lag + period`, not `period`;
  budget `(period − 1) / 2` extra bars before the first output.

## References

John Ehlers and Ric Way, "Zero Lag (Well, Almost)", *Technical Analysis
of Stocks & Commodities* (2010). The implementation here uses the standard
`lag = (period − 1) / 2` and an SMA-seeded inner EMA.

## See also

- [Indicator-Ema.md](Indicator-Ema.md) — the inner average ZLEMA de-lags.
- [Indicator-Hma.md](Indicator-Hma.md) — another low-lag average, via WMAs.
- [Indicator-T3.md](Indicator-T3.md) — low-lag average via a six-EMA cascade.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
