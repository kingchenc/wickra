# CCI

> Commodity Channel Index — measures how far the current typical price
> deviates from its rolling mean, in units of mean absolute deviation
> scaled by Lambert's constant.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum |
| Sub-category | unbounded oscillator |
| Input type | `Candle` |
| Output type | `f64` |
| Output range | unbounded (typically `[−200, +200]` thanks to the 0.015 factor) |
| Default parameters | `period = 20` (Python) |
| Warmup period | `period` (20 for `period = 20`) |
| Interpretation | `> +100` overbought, `< −100` oversold (Lambert) |

## Formula

For each candle, compute the typical price `TP = (high + low + close) / 3`,
then over the rolling `period`-bar window:

```
SMA_TP_t = (TP_{t-period+1} + … + TP_t) / period
MAD_t    = (1 / period) · Σ |TP_i − SMA_TP_t|    for i = t-period+1 … t

CCI_t    = (TP_t − SMA_TP_t) / (factor · MAD_t)
```

The default `factor` is Lambert's `0.015`, chosen empirically so that
roughly 70–80 % of values fall inside `[−100, +100]`. The implementation
exposes the factor through `Cci::with_factor(period, factor)` if you want
to retune it for an asset with very different volatility characteristics.

When `MAD == 0` (a perfectly flat window), the implementation returns `0`
rather than dividing by zero.

## Parameters

| Name | Type | Default (Python) | Valid range | Description |
|------|------|------------------|-------------|-------------|
| `period` | `usize` | `20` | `>= 1` | Rolling window length for both the SMA of typical price and the MAD. |
| `factor` | `f64` | `0.015` (`Cci::new`) | `> 0`, finite | Lambert's scaling constant; configurable via `Cci::with_factor`. |

`Cci::new(0)` returns `Error::PeriodZero`. `Cci::with_factor(_, factor)`
returns `Error::NonPositiveMultiplier` when `factor <= 0` or non-finite.

## Inputs / Outputs

From `impl Indicator for Cci`:

```rust
type Input  = Candle;
type Output = f64;
fn update(&mut self, candle: Candle) -> Option<f64>;
```

Python's `CCI.batch(high, low, close)` returns a 1-D `float64` `np.ndarray`
with `NaN` during warmup. Node's `CCI.batch(high, low, close)` returns a
flat `number[]` (also `NaN` during warmup); the Node binding does not
expose a streaming `update()` (`bindings/node/index.d.ts` lists only
`constructor` and `batch`).

## Warmup

`warmup_period()` returns exactly `period`. CCI does not consume diffs —
it only needs `period` typical-price samples to populate its rolling
window before it can compute an SMA and MAD. In streaming terms, calls
`1..period` return `None`; the `period`-th call returns the first value.

## Edge cases

- **Flat input.** Every `TP` is the SMA, so `MAD == 0` and the
  implementation returns `0.0` (test `flat_candles_yield_zero`). This
  avoids the divide-by-zero that would otherwise produce `NaN` /
  `±∞`.
- **Custom factor.** `Cci::with_factor(period, factor)` lets you replace
  Lambert's `0.015`. Picking a smaller factor widens the typical range
  of CCI values; picking a larger one compresses them.
- **Reset.** `reset()` clears the rolling window and the running sum,
  returning the indicator to the freshly-constructed state.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Cci, Indicator};

let candles: Vec<Candle> = (0..25)
    .map(|i| {
        let m = 50.0 + i as f64;
        Candle::new(m, m + 1.0, m - 1.0, m, 1.0, 0).unwrap()
    })
    .collect();
let mut cci = Cci::new(20)?;
let out = cci.batch(&candles);
println!("row 19 = {}", out[19].unwrap());
println!("row 24 = {}", out[24].unwrap());
# Ok::<(), wickra::Error>(())
```

Verified output:

```
row 19 = 126.66666666666667
row 24 = 126.66666666666667
```

### Python

```python
import numpy as np
import wickra as ta

i = np.arange(25, dtype=float)
m = 50.0 + i
high  = m + 1.0
low   = m - 1.0
close = m
cci = ta.CCI(20)
out = cci.batch(high, low, close)
print('row 19:', out[19])
print('row 24:', out[24])
```

Verified output:

```
row 19: 126.66666666666667
row 24: 126.66666666666667
```

### Node

```javascript
const wickra = require('wickra');

const n = 25;
const high = [], low = [], close = [];
for (let i = 0; i < n; i++) {
  const m = 50 + i;
  high.push(m + 1);
  low.push(m - 1);
  close.push(m);
}
const cci = new wickra.CCI(20);
const out = cci.batch(high, low, close);
console.log('row 19:', out[19]);
console.log('row 24:', out[24]);
```

Verified output:

```
row 19: 126.66666666666667
row 24: 126.66666666666667
```

## Interpretation

- **±100 threshold.** Lambert's published convention is to treat values
  above `+100` as overbought and below `−100` as oversold. The choice
  of `0.015` for the divisor is what makes the threshold meaningful;
  changing the factor changes the threshold.
- **Zero-line cross.** `CCI` crossing zero says the typical price has
  moved through its `period`-bar mean — sometimes used as a
  trend-direction filter.
- **Divergence.** As with RSI/Stochastic, a price making a new high
  while CCI makes a lower high is a classic bearish divergence.

## Common pitfalls

- **CCI is unbounded.** Unlike RSI or Stochastic, CCI can spike well
  outside `±100` in volatile markets. Threshold-based rules should be
  paired with a maximum-absolute-value guard, or you will mis-classify
  legitimate breakouts as "extreme overbought".
- **The 0.015 factor is empirical, not derived.** It was chosen by
  Lambert in 1980 for commodity futures markets. Modern equities and
  crypto have wider distributions; if your `|CCI|` distribution sits
  almost entirely outside `±100`, retune via `Cci::with_factor` rather
  than rewriting downstream thresholds.

## References

- Donald Lambert, "Commodity Channel Index: Tools for Trading Cyclical
  Trends", *Commodities Magazine*, October 1980 — the original
  publication, including the empirical choice of `0.015`.

## See also

- [Indicator: Rsi](Indicator-Rsi.md) — bounded sibling for comparison.
- [Indicator: WilliamsR](Indicator-WilliamsR.md) — another candle-input
  oscillator, range-based rather than deviation-based.
- [Indicator: Mfi](Indicator-Mfi.md) — volume-weighted RSI; useful as a
  confirmation alongside CCI.
- [Warmup Periods](../../Warmup-Periods.md) — `period` (no off-by-one).
