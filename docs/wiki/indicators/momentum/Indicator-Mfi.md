# MFI

> Money Flow Index — a volume-weighted RSI built on typical price times
> volume.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum |
| Sub-category | bounded oscillator (volume-driven) |
| Input type | `Candle` (volume needed) |
| Output type | `f64` |
| Output range | `[0, 100]` |
| Default parameters | `period = 14` (Python) |
| Warmup period | `period` (14 for `period = 14`) |
| Interpretation | overbought above 80, oversold below 20 |

## Formula

For each new candle:

```
TP_t         = (high_t + low_t + close_t) / 3          (typical price)
MF_t         = TP_t · volume_t                          (money flow)

positive MF  = MF_t   if TP_t > TP_{t-1}, else 0
negative MF  = MF_t   if TP_t < TP_{t-1}, else 0
                (both zero when TP_t == TP_{t-1})
```

Maintain rolling sums of positive and negative money flow over the last
`period` bars. Then:

```
MR_t   = positive_sum / negative_sum
MFI_t  = 100 − 100 / (1 + MR_t)
```

The implementation guards both special cases: when both rolling sums are
zero, MFI returns `50` (neutral); when only `negative_sum == 0`, MFI
returns `100`; otherwise the standard formula.

## Parameters

| Name | Type | Default (Python) | Valid range | Description |
|------|------|------------------|-------------|-------------|
| `period` | `usize` | `14` | `>= 1` | Rolling window length for the positive/negative money-flow sums. |

`Mfi::new(0)` returns `Error::PeriodZero`.

## Inputs / Outputs

From `impl Indicator for Mfi`:

```rust
type Input  = Candle;
type Output = f64;
fn update(&mut self, candle: Candle) -> Option<f64>;
```

Volume is consumed via `candle.volume` — it is not optional. Calling
the indicator with a zero-volume candle is legal (every money flow on
that bar is zero), but mass zero-volume bars will dilute the sums.

Python's `MFI.batch(high, low, close, volume)` returns a 1-D `float64`
`np.ndarray` (warmup → `NaN`). Node's `MFI.batch(high, low, close,
volume)` returns a flat `number[]` (warmup → `NaN`); only `batch` is
exposed on the Node binding.

## Warmup

`warmup_period()` returns `period`. The first candle has no previous
`TP` to compare against, so its money flow is classified as neither
positive nor negative — it sits in the window as a `0 / 0` slot but
still counts toward filling the window. The first `Some` is therefore
emitted at the `period`-th `update`, exactly when the rolling positive
and negative sums first contain `period − 1` real comparisons.

## Edge cases

- **Pure uptrend.** Every `TP_t > TP_{t-1}`, so `negative_sum == 0` and
  the implementation returns `100` directly (test
  `pure_uptrend_yields_high_mfi`). Pure downtrend mirrors at `0` (test
  `pure_downtrend_yields_low_mfi`).
- **Flat input (all `TP` equal).** Both sums stay at zero; the
  implementation returns `50` (the same neutral convention as RSI on
  flat input).
- **Zero-volume candle.** Money flow on that bar is zero. The window
  still advances; the indicator just gets one less data point of
  influence.
- **Reset.** `reset()` clears `prev_tp`, both rolling windows, and both
  sums.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, Mfi};

let candles: Vec<Candle> = (1..=20)
    .map(|i| Candle::new(i as f64, i as f64, i as f64, i as f64, 100.0, 0).unwrap())
    .collect();
let mut mfi = Mfi::new(14)?;
let out = mfi.batch(&candles);
println!("row 13 = {}", out[13].unwrap());
println!("row 19 = {}", out[19].unwrap());
# Ok::<(), wickra::Error>(())
```

Verified output:

```
row 13 = 100
row 19 = 100
```

### Python

```python
import numpy as np
import wickra as ta

n = 20
i = np.arange(1, n + 1, dtype=float)
high = low = close = i
volume = np.full(n, 100.0)
mfi = ta.MFI(14)
out = mfi.batch(high, low, close, volume)
print('warmup:', mfi.warmup_period())
print('row 13:', out[13])
print('row 19:', out[19])
```

Verified output:

```
warmup: 14
row 13: 100.0
row 19: 100.0
```

### Node

```javascript
const wickra = require('wickra');

const n = 20;
const high = [], low = [], close = [], vol = [];
for (let i = 1; i <= n; i++) {
  high.push(i); low.push(i); close.push(i); vol.push(100);
}
const m = new wickra.MFI(14);
const out = m.batch(high, low, close, vol);
console.log('row 13:', out[13]);
console.log('row 19:', out[19]);
```

Verified output:

```
row 13: 100
row 19: 100
```

## Interpretation

- **Overbought / oversold.** The conventional MFI thresholds are
  `80 / 20` — tighter than RSI's `70 / 30` because the volume weighting
  amplifies sustained one-way moves.
- **Divergence.** MFI divergences are read like RSI divergences: a new
  price high without a confirming MFI high is bearish, and vice versa.
  Because volume is in the mix, MFI divergences are often interpreted
  as "the move is happening on weak participation" — i.e. structurally
  more meaningful than a pure-price divergence.
- **Compare with OBV.** OBV (the unsmoothed cumulative volume) tells
  you accumulated participation; MFI tells you participation pressure
  over a fixed horizon. The two often diverge interestingly near
  trend exhaustion.

## Common pitfalls

- **MFI requires volume.** Unlike RSI (close only) or Stochastic
  (high/low/close), MFI's per-bar money flow is `TP × volume`. Passing
  a candle stream with `volume == 0` throughout will collapse MFI to
  `50` regardless of price action. Validate your data source before
  reaching for MFI.
- **Same flat-input convention as RSI.** A perfectly flat window yields
  `50` (not `NaN`, not "no value"). Treat the value as informational
  only until the underlying TP series starts moving.

## References

- Gene Quong and Avrum Soudack, "Volume-Weighted RSI: Money Flow",
  *Technical Analysis of Stocks & Commodities*, March 1989 — the
  original publication of the MFI as a volume-weighted RSI variant.

## See also

- [Indicator: Rsi](Indicator-Rsi.md) — the price-only ancestor.
- [Indicator: Adx](Indicator-Adx.md) — directional/trend strength to
  pair with MFI's overbought/oversold reading.
- [Warmup Periods](../../Warmup-Periods.md) — bare `period` (no off-by-one).
