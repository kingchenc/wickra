# MassIndex

> Mass Index — Donald Dorsey's range-expansion indicator: it watches the
> high–low range widen and contract to anticipate reversals.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Trend & Directional |
| Input type | `Candle` (uses `high`, `low`) |
| Output type | `f64` |
| Output range | `> 0`, oscillates around `sum_period` |
| Default parameters | `(ema_period = 9, sum_period = 25)` (Python) |
| Warmup period | `2·ema_period + sum_period − 2` |
| Interpretation | A rise above `27` then fall below `26.5` flags a reversal. |

## Formula

```
range_t   = high_t − low_t
single_t  = EMA(range, ema_period)_t
double_t  = EMA(single, ema_period)_t
ratio_t   = single_t / double_t
MassIndex = Σ ratio over sum_period
```

The Mass Index ignores direction entirely — it tracks **volatility shape**.
When the high–low range widens, the single EMA pulls ahead of the double
EMA, the ratio climbs above `1`, and the windowed sum rises. Dorsey's
"reversal bulge" is the classic pattern: the Mass Index rising above `27`
and then falling back below `26.5` warns that a range expansion is about
to resolve — often into a trend reversal.

## Parameters

| Name         | Type    | Default       | Valid range | Description |
|--------------|---------|---------------|-------------|-------------|
| `ema_period` | `usize` | `9` (Python)  | `>= 1`      | Period of both EMAs in the cascade. `0` errors with `Error::PeriodZero`. |
| `sum_period` | `usize` | `25` (Python) | `>= 1`      | Length of the summation window. |

The Python binding defaults the pair to `(9, 25)`. The `periods` property
returns `(ema_period, sum_period)`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/mass_index.rs`:

```rust
impl Indicator for MassIndex {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`MassIndex` is a **candle-input** indicator: it reads `high` and `low`. In
Python the streaming `update` accepts a 6-tuple or a dict; the batch
helper takes `high` and `low` numpy arrays. Node and WASM expose
`update(high, low)` and `batch(high, low)`.

## Warmup

`warmup_period() == 2·ema_period + sum_period − 2`. The first EMA seeds at
input `ema_period`; the second EMA, stacked on it, seeds at
`2·ema_period − 1`; the summation window then needs `sum_period` ratios.
For the default `(9, 25)` that is `41` bars.

## Edge cases

- **Constant range.** When every bar has the same high–low range, both
  EMAs converge to the same value, every ratio is `1`, and the Mass Index
  equals `sum_period` (`constant_range_sums_to_sum_period` pins this).
- **Zero-range market.** A flat market (`high == low`) drives both EMAs to
  `0`; the `0 / 0` is guarded with the neutral ratio `1`, so the Mass
  Index again equals `sum_period`
  (`zero_range_market_sums_to_sum_period` pins this).
- **Candle validation.** `Candle::new` rejects invalid bars upstream.
- **Reset.** `mi.reset()` clears both EMAs, the window and the sum.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, MassIndex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mi = MassIndex::new(3, 4)?;
    // Constant high-low range of 2.0; the Mass Index settles at sum_period.
    let candles: Vec<Candle> = (0..40)
        .map(|i| {
            let mid = 100.0 + f64::from(i);
            Candle::new(mid, mid + 1.0, mid - 1.0, mid, 1.0, i64::from(i)).unwrap()
        })
        .collect();
    let out = mi.batch(&candles);
    println!("warmup_period = {}", mi.warmup_period());
    println!("last = {:?}", out.last().unwrap());
    Ok(())
}
```

Output:

```
warmup_period = 8
last = Some(4.0)
```

A constant range makes every ratio `1`, so the sum equals `sum_period`
(`4`). This matches the `constant_range_sums_to_sum_period` test in
`crates/wickra-core/src/indicators/mass_index.rs`.

### Python

```python
import numpy as np
import wickra as ta

mi = ta.MassIndex()  # (ema_period=9, sum_period=25)
mid = np.arange(100.0, 160.0)
high = mid + 1.0
low = mid - 1.0
print(mi.batch(high, low)[-1])  # constant range -> 25
```

Output:

```
25.0
```

### Node

```javascript
const ta = require('wickra');
const mi = new ta.MassIndex(9, 25);
const mid = Array.from({ length: 60 }, (_, i) => 100 + i);
const high = mid.map((m) => m + 1);
const low = mid.map((m) => m - 1);
console.log(mi.batch(high, low).at(-1)); // 25
```

## Interpretation

`MassIndex` is a *reversal-warning* tool, not a direction tool — it never
tells you which way price will go, only that a turn is likely. The textbook
use is the "reversal bulge" on the default `(9, 25)` settings: watch for
the index to push above `27`, then act when it drops back under `26.5`,
using a directional indicator (a moving average, ADX) to pick the side.

## Common pitfalls

- **Expecting a direction.** The Mass Index is direction-blind; always
  pair it with a trend indicator.
- **Feeding it scalar prices.** It needs `high`/`low`; it takes a
  `Candle`, not an `f64`.

## References

Donald Dorsey, "The Mass Index", *Technical Analysis of Stocks &
Commodities* (1992). The double-EMA-of-range construction and the `(9,
25)` defaults follow Dorsey's original.

## See also

- [Indicator-Atr.md](../volatility-bands/Indicator-Atr.md) — directional-free
  volatility in price units.
- [Indicator-BollingerBands.md](../volatility-bands/Indicator-BollingerBands.md)
  — another range-expansion lens.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
