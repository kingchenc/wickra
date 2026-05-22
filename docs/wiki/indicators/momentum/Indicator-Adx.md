# ADX

> Wilder's Average Directional Index — the smoothed strength of a trend,
> plus the two directional components (`+DI`, `−DI`) that say which
> direction the trend is going.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum (directional) |
| Sub-category | directional / trend-strength |
| Input type | `Candle` |
| Output type | `AdxOutput { plus_di, minus_di, adx }` |
| Output range | each field in `[0, 100]` |
| Default parameters | `period = 14` (Python) |
| Warmup period | `2 · period` (28 for `period = 14`) |
| Interpretation | `adx > 25` means a meaningful trend; the dominant DI gives its direction |

## Formula

For each new candle at time `t` (with previous candle `t-1`):

```
+DM_t  =  high_t − high_{t-1}          if  (high_t − high_{t-1}) > (low_{t-1} − low_t)
                                       and (high_t − high_{t-1}) > 0
       =  0                            otherwise

−DM_t  =  low_{t-1} − low_t            if  (low_{t-1} − low_t) > (high_t − high_{t-1})
                                       and (low_{t-1} − low_t) > 0
       =  0                            otherwise

TR_t   =  max(high_t − low_t,
              |high_t − close_{t-1}|,
              |low_t  − close_{t-1}|)
```

Wilder's smoothing is applied to all three series. Seeding is a simple
sum over the first `period` post-prev candles; after seeding the update
rule for any of these is

```
S_t   =  S_{t-1}  −  S_{t-1} / period  +  X_t
```

where `X_t` is `TR_t`, `+DM_t`, or `−DM_t`. The directional indicators
and DX then are

```
+DI_t  =  100 · (+DM smoothed)_t / (TR smoothed)_t
−DI_t  =  100 · (−DM smoothed)_t / (TR smoothed)_t
DX_t   =  100 · |+DI_t − −DI_t| / (+DI_t + −DI_t)
```

`ADX_t` is itself a Wilder-smoothed `DX` series, seeded as the mean of
the first `period` `DX` values, and then updated with `α = 1/period`:

```
ADX_t  =  (ADX_{t-1} · (period − 1) + DX_t) / period
```

When `+DI + −DI == 0`, `DX` is `0`; when `TR == 0`, both DI lines are
`0`. These are the divide-by-zero guards in `Adx::update`.

## Parameters

| Name | Type | Default (Python) | Valid range | Description |
|------|------|------------------|-------------|-------------|
| `period` | `usize` | `14` | `>= 1` | Wilder smoothing length shared by `+DM`, `−DM`, `TR`, and `ADX`. |

`Adx::new(0)` returns `Error::PeriodZero`.

## Inputs / Outputs

From `impl Indicator for Adx`:

```rust
type Input  = Candle;
type Output = AdxOutput;
fn update(&mut self, candle: Candle) -> Option<AdxOutput>;
```

`AdxOutput`:

| Field | Description |
|-------|-------------|
| `plus_di` | Plus Directional Indicator (`+DI`) — strength of upward movement. |
| `minus_di` | Minus Directional Indicator (`−DI`) — strength of downward movement. |
| `adx` | Average Directional Index — smoothed `|DX|`, a directionless trend-strength measure. |

Python's `ADX.batch(high, low, close)` returns a `(n, 3)` `float64` array
with columns `[plus_di, minus_di, adx]`; warmup rows are entirely `NaN`.
The streaming `update(candle)` returns a `(plus_di, minus_di, adx)`
tuple or `None`.

Node's `ADX.batch(high, low, close)` returns a flat `number[]` of length
`n * 3`, interleaved `[plus_di_0, minus_di_0, adx_0, plus_di_1, …]`.
Only `batch` is exposed on the Node binding — no `update`.

## Warmup

`warmup_period()` returns `2 · period`. The first candle just provides a
"previous" reference (no DM/TR can be computed yet); the next `period`
candles seed the smoothed `+DM`, `−DM`, and `TR` sums; the next `period`
candles after that produce `DX` values that seed `ADX`. For `period =
14` that's `1 + 14 + 13 = 28` candles before the first full
`AdxOutput`, which matches `2 · 14 = 28`.

## Edge cases

- **Strong unidirectional trend.** If every candle is strictly higher
  than the last (with `+DM` always positive, `−DM` always zero), `+DI`
  saturates at `100`, `−DI` at `0`, and `ADX` climbs toward `100`. The
  example below produces exactly that.
- **Flat market (no high/low movement).** Every `TR`, `+DM`, `−DM` is
  zero, so the divide-by-zero guards return `+DI = −DI = 0` and `DX =
  0`; `ADX` then sits at `0` indefinitely.
- **Reset.** `reset()` clears `prev`, all seed sums and counts, all
  smoothed values, the DX buffer, and `adx_value`.

## Examples

### Rust

```rust
use wickra::{Adx, BatchExt, Candle, Indicator};

let candles: Vec<Candle> = (0..40)
    .map(|i| {
        let base = 100.0 + i as f64 * 2.0;
        Candle::new(base + 0.5, base + 1.0, base - 0.5, base + 0.5, 1.0, 0).unwrap()
    })
    .collect();
let mut adx = Adx::new(14)?;
let out = adx.batch(&candles);
let v = out[27].unwrap();
println!("row 27  +DI={} -DI={} ADX={}", v.plus_di, v.minus_di, v.adx);
let v = out[39].unwrap();
println!("row 39  +DI={} -DI={} ADX={}", v.plus_di, v.minus_di, v.adx);
# Ok::<(), wickra::Error>(())
```

Verified output:

```
row 27  +DI=80 -DI=0 ADX=100
row 39  +DI=80 -DI=0 ADX=100
```

### Python

```python
import numpy as np
import wickra as ta

n = 40
i = np.arange(n, dtype=float)
base = 100.0 + i * 2.0
high  = base + 1.0
low   = base - 0.5
close = base + 0.5
adx = ta.ADX(14)
out = adx.batch(high, low, close)
print('warmup:', adx.warmup_period())
print('shape :', out.shape)
print('row 27:', out[27])
print('row 39:', out[39])
```

Verified output:

```
warmup: 28
shape : (40, 3)
row 27: [ 80.   0. 100.]
row 39: [ 80.   0. 100.]
```

### Node

```javascript
const wickra = require('wickra');

const n = 40;
const high = [], low = [], close = [];
for (let i = 0; i < n; i++) {
  const b = 100 + i * 2;
  high.push(b + 1);
  low.push(b - 0.5);
  close.push(b + 0.5);
}
const adx = new wickra.ADX(14);
const out = adx.batch(high, low, close);
console.log('len   :', out.length);
console.log('row 27:', { plusDi: out[27 * 3], minusDi: out[27 * 3 + 1], adx: out[27 * 3 + 2] });
console.log('row 39:', { plusDi: out[39 * 3], minusDi: out[39 * 3 + 1], adx: out[39 * 3 + 2] });
```

Verified output:

```
len   : 120
row 27: { plusDi: 80, minusDi: 0, adx: 100 }
row 39: { plusDi: 80, minusDi: 0, adx: 100 }
```

## Interpretation

- **Trend-strength bands.** `ADX < 20` is typically read as a ranging
  market; `ADX > 25` as a "real" trend; `ADX > 40` as a strong trend.
  ADX itself is direction-agnostic — you need `+DI` vs `−DI` to know
  which way the trend points.
- **DI crossover.** `+DI` crossing above `−DI` is a bullish directional
  signal; the mirror is bearish. Many traders only act on a crossover
  when `ADX > 25` to filter out crossovers in a ranging market.
- **ADX peaks.** A rising ADX confirms trend continuation; a falling
  ADX from a high level suggests the current trend is exhausting (even
  if `+DI` still dominates `−DI`).

## Common pitfalls

- **Long warmup.** ADX needs `2 · period` candles before the first
  emission — twice as many as most other Wilder indicators. A common
  bug is reusing an "RSI fits in `period + 1` bars" mental model and
  reading garbage during the ADX warmup; check `is_ready()` or test
  for `NaN` on the `adx` column.
- **Plotted on the same axis as DI.** `+DI`, `−DI`, and `ADX` all live
  in `[0, 100]` and are typically overlaid. The crossover signal is
  between `+DI` and `−DI` only — `ADX` does not cross either of them
  for any directional meaning.

## References

- J. Welles Wilder, *New Concepts in Technical Trading Systems*, Trend
  Research, 1978 — the original publication of `+DI`, `−DI`, `DX`,
  `ADX`, and the smoothing scheme they share with RSI and ATR.

## See also

- [Indicator: Rsi](Indicator-Rsi.md) — shares Wilder smoothing.
- [Indicator: Aroon](Indicator-Aroon.md) — alternative trend-strength
  measure, range-based.
- [Indicator: MacdIndicator](Indicator-MacdIndicator.md) — trend-following
  momentum, useful as a confirmation against `+DI` / `−DI`.
- [Warmup Periods](../../Warmup-Periods.md) — the `2 · period` ADX entry.
