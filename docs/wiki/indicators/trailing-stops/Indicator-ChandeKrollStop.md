# ChandeKrollStop

> Chande Kroll Stop — a two-stage ATR stop: an ATR stop off the recent
> extreme, then smoothed by taking the most extreme such stop over a
> shorter window.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Trailing Stops |
| Input type | `Candle` (uses `high`, `low`, `close`) |
| Output type | `(stop_long, stop_short)` |
| Output range | unbounded (price scale) |
| Default parameters | `atr_period = 10`, `atr_multiplier = 1.0`, `stop_period = 9` (Python) |
| Warmup period | `atr_period + stop_period − 1` |
| Interpretation | Smoothed long/short stop levels, less prone to single-bar whipsaw. |

## Formula

```
preliminary (window p = atr_period, x = atr_multiplier):
  high_stop = highest_high(p) − x · ATR(p)
  low_stop  = lowest_low(p)   + x · ATR(p)

final (window q = stop_period):
  stop_short = highest(high_stop, q)
  stop_long  = lowest(low_stop,  q)
```

Tushar Chande and Stanley Kroll's stop runs in two stages. The first builds a
preliminary ATR stop off the recent extreme — the same idea as a
[`ChandelierExit`](../trailing-stops/Indicator-ChandelierExit.md). The second smooths it: rather
than use that preliminary stop directly, it takes the *most extreme*
preliminary stop seen over a shorter window `q`. That second pass keeps a
single unusually wide bar from yanking the stop around. The classic
configuration from *The New Technical Trader* is `ATR(10)`, multiplier `1.0`,
smoothing window `9`.

## Parameters

- `atr_period` — window for the preliminary ATR and the highest high / lowest
  low (Python default `10`).
- `atr_multiplier` — how many ATRs the preliminary stop sits off the extreme
  (default `1.0`).
- `stop_period` — the smoothing window `q` (default `9`).

`ChandeKrollStop::classic()` returns the `(10, 1.0, 9)` configuration.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/chande_kroll_stop.rs`:

```rust
impl Indicator for ChandeKrollStop {
    type Input = Candle;
    type Output = ChandeKrollStopOutput;   // { stop_long: f64, stop_short: f64 }
    // update(&mut self, input: Candle) -> Option<ChandeKrollStopOutput>
}
```

`ChandeKrollStop` is a **candle-input** indicator (it reads `high`, `low`,
`close`). Python's streaming `update` returns a `(stop_long, stop_short)`
tuple; the batch helper returns an `(n, 2)` array with columns
`[stop_long, stop_short]`. Node's `update` returns `{ stopLong, stopShort }`
and `batch` a flat `[l0, s0, l1, s1, …]` array; WASM matches Node.

## Warmup

`ChandeKrollStop::classic().warmup_period() == 18` (`atr_period + stop_period −
1`). The preliminary stop first appears on candle `atr_period`; the smoothing
window then needs `stop_period` of them.

## Edge cases

- **Two-stage warmup.** Nothing is emitted until both the preliminary window
  and the smoothing window have filled.
- **Flat market.** Constant candles collapse both stages to fixed levels.
- **Reset.** `cks.reset()` clears the ATR and all four windows.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, ChandeKrollStop};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cks = ChandeKrollStop::new(5, 1.0, 3)?;
    // Flat market: ATR = 2, HH = 11, LL = 9.
    let candles: Vec<Candle> = (0..20)
        .map(|i| Candle::new(10.0, 11.0, 9.0, 10.0, 1.0, i).unwrap())
        .collect();
    let out = cks.batch(&candles);
    println!("{:?}", out.last().unwrap());
    Ok(())
}
```

Output:

```
Some(ChandeKrollStopOutput { stop_long: 11.0, stop_short: 9.0 })
```

`high_stop = 11 − 1·2 = 9`, `low_stop = 9 + 1·2 = 11`; the smoothing pass over
constant values leaves `stop_short = 9` and `stop_long = 11`. This matches the
`reference_values_flat_market` test in
`crates/wickra-core/src/indicators/chande_kroll_stop.rs`.

### Python

```python
import numpy as np
import wickra as ta

cks = ta.ChandeKrollStop(5, 1.0, 3)
n = 20
high = np.full(n, 11.0)
low = np.full(n, 9.0)
close = np.full(n, 10.0)
print(cks.batch(high, low, close)[-1])   # [stop_long, stop_short]
```

Output:

```
[11.  9.]
```

### Node

```javascript
const ta = require('wickra');
const cks = new ta.ChandeKrollStop(5, 1.0, 3);
const n = 20;
const high = Array(n).fill(11), low = Array(n).fill(9), close = Array(n).fill(10);
const out = cks.batch(high, low, close);
console.log(out.slice(-2));   // [stop_long, stop_short] of the last bar
```

Output:

```
[ 11, 9 ]
```

## Interpretation

Use `stop_long` to trail a long position and `stop_short` to trail a short.
Compared with a one-stage [`ChandelierExit`](../trailing-stops/Indicator-ChandelierExit.md), the
extra smoothing window makes the Chande Kroll Stop steadier — it will not lurch
on a single wide-range bar — at the cost of reacting a little slower to a
genuine trend change.

## Common pitfalls

- **Forgetting the longer warmup.** Two stacked windows mean `atr_period +
  stop_period − 1` bars before the first value.
- **Confusing the labels.** `stop_short` is generally the lower level and
  `stop_long` the higher — they bracket recent price, but each only applies to
  its own side.

## References

Tushar Chande and Stanley Kroll's stop, from *The New Technical Trader* (1994);
the two-stage formulation here matches the common TradingView implementation.

## See also

- [Indicator-ChandelierExit.md](../trailing-stops/Indicator-ChandelierExit.md) — the one-stage
  ATR stop this smooths.
- [Indicator-SuperTrend.md](../trailing-stops/Indicator-SuperTrend.md) — an ATR trailing stop
  with explicit flip logic.
- [Indicator-Atr.md](../volatility-bands/Indicator-Atr.md) — the volatility measure underneath.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
