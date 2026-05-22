# Aroon

> Tushar Chande's Aroon indicator — tracks the bars-since-highest-high
> and bars-since-lowest-low inside a `period + 1`-bar window, reported
> as percentages.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Trend & Directional |
| Input type | `Candle` |
| Output type | `AroonOutput { up, down }` |
| Output range | `up, down ∈ [0, 100]` |
| Default parameters | `period = 14` (Python) |
| Warmup period | `period + 1` (15 for `period = 14`) |
| Interpretation | `up > 70 && down < 30` strong uptrend (mirror for downtrend); crossovers as turn signals |

## Formula

Scan the rolling `period + 1`-bar window for the position of the highest
high and the position of the lowest low (with `0 = oldest`,
`period = newest`):

```
hh_idx_t  =  argmax_{i in 0..period}  high_{t-period+i}
ll_idx_t  =  argmin_{i in 0..period}  low_{t-period+i}

up_t      =  100 · hh_idx_t / period
down_t    =  100 · ll_idx_t / period
```

When the highest high lands on the most-recent bar (`hh_idx == period`),
`up == 100`; when it lands on the oldest bar in the window, `up == 0`.
The same holds for `down`.

In Wickra's implementation the scan uses `>=` / `<=`, so ties go to the
*latest* matching bar — which is why a perfectly flat window produces
`up = down = 100` rather than `0` (the latest bar is always tied with
the oldest).

## Parameters

| Name | Type | Default (Python) | Valid range | Description |
|------|------|------------------|-------------|-------------|
| `period` | `usize` | `14` | `>= 1` | Lookback length. The internal window holds `period + 1` candles. |

`Aroon::new(0)` returns `Error::PeriodZero`.

## Inputs / Outputs

From `impl Indicator for Aroon`:

```rust
type Input  = Candle;
type Output = AroonOutput;
fn update(&mut self, candle: Candle) -> Option<AroonOutput>;
```

`AroonOutput`:

| Field | Description |
|-------|-------------|
| `up`   | `100 · bars_since_oldest_HH / period`, in `[0, 100]`. High = recent new high. |
| `down` | `100 · bars_since_oldest_LL / period`, in `[0, 100]`. High = recent new low. |

Python's `Aroon.batch(high, low)` returns a `(n, 2)` `float64` array
with columns `[up, down]`; warmup rows are `[NaN, NaN]`. Streaming
`update(candle)` returns a `(up, down)` tuple or `None`.

Node's `Aroon.batch(high, low)` returns a flat `number[]` of length
`n * 2`, interleaved `[up_0, down_0, up_1, down_1, …]`. Only `batch`
is exposed on the Node binding.

## Warmup

`warmup_period()` returns `period + 1`. Aroon scans `period + 1` bars
to find "bars since highest high" (which ranges over `0..period`), so
the indicator is not ready until exactly `period + 1` candles have
arrived. This is the same off-by-one as RSI and ROC, but for a
window-position reason rather than a diff reason.

## Edge cases

- **Pure uptrend.** Every new candle is a new high — `hh_idx` is always
  the latest position, `up == 100`. The lowest low is the oldest
  candle in the window, `down == 0`. Tests `pure_uptrend_aroon_up_100`
  pin this.
- **Constant input.** Every candle's high is equal to every other
  candle's high. The `>=` tiebreak in the scan means the most-recent
  candle always wins both the HH and LL positions — both `up` and
  `down` end up at `100`. (Be careful: this is *not* a neutral
  reading; it is an artefact of the tiebreak rule.)
- **Reset.** `reset()` clears the candle buffer; the next `period + 1`
  updates return `None`.

## Examples

### Rust

```rust
use wickra::{Aroon, BatchExt, Candle, Indicator};

let candles: Vec<Candle> = (1..=15)
    .map(|i| Candle::new(i as f64, i as f64 + 1.0, i as f64 - 1.0, i as f64, 1.0, 0).unwrap())
    .collect();
let mut aroon = Aroon::new(14)?;
let out = aroon.batch(&candles);
let v = out[14].unwrap();
println!("uptrend row 14  up={} down={}", v.up, v.down);
# Ok::<(), wickra::Error>(())
```

Verified output:

```
uptrend row 14  up=100 down=0
```

### Python

```python
import numpy as np
import wickra as ta

i = np.arange(1, 16, dtype=float)
high = i + 1.0
low  = i - 1.0
aroon = ta.Aroon(14)
out = aroon.batch(high, low)
print('warmup:', aroon.warmup_period())
print('shape :', out.shape)
print('row 14:', out[14])
```

Verified output:

```
warmup: 15
shape : (15, 2)
row 14: [100.   0.]
```

### Node

```javascript
const wickra = require('wickra');

const high = [], low = [];
for (let i = 1; i <= 15; i++) {
  high.push(i + 1);
  low.push(i - 1);
}
const a = new wickra.Aroon(14);
const out = a.batch(high, low);
console.log('len   :', out.length);
console.log('row 14:', { up: out[14 * 2], down: out[14 * 2 + 1] });
```

Verified output:

```
len   : 30
row 14: { up: 100, down: 0 }
```

## Interpretation

- **Strong trend bands.** `up > 70` with `down < 30` indicates a strong
  uptrend (new highs are recent, new lows are old); mirror for a
  downtrend.
- **Crossover.** `up` crossing above `down` is a bullish trend-shift
  signal; the mirror is bearish. Crossovers near `50/50` are weak
  (the window has no clear leader); crossovers from `0`/`100` extremes
  are strong.
- **Consolidation.** Both lines wandering near `50` means neither
  recent highs nor recent lows are dominating — typical of a
  range-bound market.

## Common pitfalls

- **Constant input gives `up == down == 100`, not `0` or `50`.** The
  `>=` / `<=` tiebreak in the scan rewards the most-recent candle.
  Treat constant or near-constant windows as a degenerate case; a
  reading of `(100, 100)` is *not* a strong trend in both directions
  — it is "no information".
- **`period + 1` warmup, not `period`.** Same off-by-one trap as RSI:
  the indicator looks at a window of size `period + 1` so that
  `bars_since_high` can range from `0` to `period`. Indexing your
  output array as if it were ready at the `period`-th input gives you
  one `NaN` / `None` row at the start you didn't expect.

## References

- Tushar Chande, "A New Tool for Technical Traders: The Aroon
  Indicator", *Technical Analysis of Stocks & Commodities*, September
  1995 — the original publication.

## See also

- [Indicator: Adx](../trend-directional/Indicator-Adx.md) — alternative trend-strength
  measure with explicit `+DI` / `−DI` direction.
- [Indicator: Stochastic](../momentum-oscillators/Indicator-Stochastic.md) — also range-window
  based, but reports close position rather than extremum age.
- [Warmup Periods](../../Warmup-Periods.md) — the `period + 1` family.
