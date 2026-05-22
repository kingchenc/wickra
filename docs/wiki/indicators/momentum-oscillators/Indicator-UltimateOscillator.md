# UltimateOscillator

> Ultimate Oscillator — Larry Williams' momentum oscillator that blends
> three lookback periods into one bounded `[0, 100]` reading.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum Oscillators |
| Input type | `Candle` (uses `high`, `low`, `close`) |
| Output type | `f64` |
| Output range | `[0, 100]` |
| Default parameters | `(short = 7, mid = 14, long = 28)` (Python) |
| Warmup period | `max(short, mid, long) + 1` |
| Interpretation | Weighted three-timeframe buying pressure; `50` is neutral. |

## Formula

```
true_low_t = min(low_t, close_{t−1})
BP_t       = close_t − true_low_t                       (buying pressure)
TR_t       = max(high_t, close_{t−1}) − true_low_t      (true range)
avg_n      = Σ BP over n / Σ TR over n
UO         = 100 · (4·avg_short + 2·avg_mid + avg_long) / 7
```

A single-timeframe momentum oscillator can show false divergences when
its lookback does not match the swing being measured. The Ultimate
Oscillator averages buying pressure over *three* windows and weights the
fastest (`4×`) above the medium (`2×`) and slow (`1×`), which damps those
false signals while keeping the response quick.

## Parameters

| Name    | Type    | Default       | Valid range | Description |
|---------|---------|---------------|-------------|-------------|
| `short` | `usize` | `7` (Python)  | `>= 1`      | Fast lookback (weight `4`). `0` errors with `Error::PeriodZero`. |
| `mid`   | `usize` | `14` (Python) | `>= 1`      | Medium lookback (weight `2`). |
| `long`  | `usize` | `28` (Python) | `>= 1`      | Slow lookback (weight `1`). |

The Python binding defaults the trio to `(7, 14, 28)` via
`#[pyo3(signature = (short=7, mid=14, long=28))]`. Node and WASM take all
three explicitly. The `periods` property returns `(short, mid, long)`.
`UltimateOscillator::classic()` is the conventional `(7, 14, 28)`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/ultimate_oscillator.rs`:

```rust
impl Indicator for UltimateOscillator {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`UltimateOscillator` is a **candle-input** indicator: it reads `high`,
`low` and `close`. In Python the streaming `update` accepts a 6-tuple or
a dict; the batch helper takes `high`, `low`, `close` numpy arrays. Node
and WASM expose `update(high, low, close)` and `batch(high, low, close)`.

## Warmup

`warmup_period() == max(short, mid, long) + 1`. The first bar has no
previous close, so the first `BP`/`TR` pair forms on bar 2; the longest
window must then fill, so the first non-`None` output lands on input
`max(short, mid, long) + 1`.

## Edge cases

- **Pure uptrend.** Bars that each close higher have `BP == TR`, so every
  ratio is `1` and UO saturates at `100`
  (`pure_uptrend_saturates_at_100` pins this).
- **Pure downtrend.** Bars that each close lower have `BP == 0`, so UO is
  `0` (`pure_downtrend_saturates_at_0` pins this).
- **Flat market.** Identical bars have zero true range; each window
  contributes the neutral ratio `0.5`, so UO reads `50`
  (`flat_market_reads_50` pins this).
- **Bounds.** The output is always within `[0, 100]`
  (`output_stays_within_0_100` pins this).
- **Candle validation.** `Candle::new` rejects NaN/infinite fields, so
  `update` never sees an invalid bar.
- **Reset.** `uo.reset()` clears the previous close, the rolling window
  and all six running sums.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, UltimateOscillator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut uo = UltimateOscillator::classic(); // (7, 14, 28)
    // 30 flat candles, each closing one tick higher than the last.
    let candles: Vec<Candle> = (0..40)
        .map(|i| {
            let p = 100.0 + f64::from(i);
            Candle::new(p, p, p, p, 1.0, i64::from(i)).unwrap()
        })
        .collect();
    let out = uo.batch(&candles);
    println!("warmup_period = {}", uo.warmup_period());
    println!("last = {:?}", out.last().unwrap());
    Ok(())
}
```

Output:

```
warmup_period = 29
last = Some(100.0)
```

Every bar closes higher with `BP == TR`, so UO saturates at `100`. This
matches the `pure_uptrend_saturates_at_100` test in
`crates/wickra-core/src/indicators/ultimate_oscillator.rs`.

### Python

```python
import numpy as np
import wickra as ta

uo = ta.UltimateOscillator()  # (7, 14, 28)
high = np.full(40, 100.0)
low = np.full(40, 100.0)
close = np.full(40, 100.0)  # perfectly flat market
print(uo.batch(high, low, close)[-1])
```

Output:

```
50.0
```

### Node

```javascript
const ta = require('wickra');
const uo = new ta.UltimateOscillator(7, 14, 28);
const flat = Array.from({ length: 40 }, () => 100);
console.log(uo.batch(flat, flat, flat).at(-1)); // 50
```

## Interpretation

`UltimateOscillator` is read with the usual overbought/oversold lens —
above `70` is stretched, below `30` is washed out — but Larry Williams'
canonical signal is *divergence with confirmation*: price makes a new
extreme while UO does not, then UO breaks the level of the divergence.
The three-timeframe blend makes those divergences more reliable than a
single-period oscillator.

## Common pitfalls

- **Feeding it scalar prices.** It needs `high`/`low`/`close`; it takes a
  `Candle`, not an `f64`.
- **Reordering the periods.** The `4 / 2 / 1` weights assume `short` is
  the fastest window — keep `short < mid < long`. Any positive periods
  are accepted, but mis-ordering them inverts the intended weighting.

## References

Larry Williams, "The Ultimate Oscillator", *Technical Analysis of Stocks
& Commodities* (1985). The buying-pressure / true-range definition and the
`4 / 2 / 1` weighting follow Williams' original.

## See also

- [Indicator-Stochastic.md](../momentum-oscillators/Indicator-Stochastic.md) — single-timeframe
  bounded oscillator.
- [Indicator-Rsi.md](../momentum-oscillators/Indicator-Rsi.md) — the canonical momentum oscillator.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
