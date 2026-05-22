# ChaikinVolatility

> Chaikin Volatility — the rate of change of a smoothed high-low spread;
> is the trading range widening or narrowing?

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volatility & Bands |
| Input type | `Candle` (uses `high`, `low`) |
| Output type | `f64` |
| Output range | unbounded around zero (percent) |
| Default parameters | `ema_period = 10`, `roc_period = 10` (Python) |
| Warmup period | `ema_period + roc_period` |
| Interpretation | Positive = ranges expanding, negative = ranges contracting. |

## Formula

```
spread_t   = high_t − low_t
smoothed_t = EMA(spread, ema_period)_t
ChaikinVol = 100 · (smoothed_t − smoothed_{t−roc_period}) / smoothed_{t−roc_period}
```

Marc Chaikin's volatility measure tracks not the *level* of the trading range
but how fast it is *widening or narrowing*. The bar's high-low spread is
EMA-smoothed, then run through a rate-of-change: a rising value means ranges
are expanding (often near a market top, as fear spikes), a falling value means
they are contracting (a quiet, complacent market). The classic configuration
smooths the spread with a `10`-period EMA and takes its `10`-period rate of
change.

## Parameters

- `ema_period` — the EMA that smooths the high-low spread (`10`).
- `roc_period` — the rate-of-change lookback over the smoothed spread (`10`).

`ChaikinVolatility::classic()` returns the `(10, 10)` configuration.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/chaikin_volatility.rs`:

```rust
impl Indicator for ChaikinVolatility {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`ChaikinVolatility` is a **candle-input** indicator that reads `high` and
`low`. Python's streaming `update` accepts a 6-tuple or a dict; the batch
helper takes `high`, `low` numpy arrays. Node and WASM expose
`update(high, low)` and the matching `batch`.

## Warmup

`ChaikinVolatility::classic().warmup_period() == 20`. The EMA emits at candle
`ema_period`; the rate-of-change then needs `roc_period` more smoothed values.

## Edge cases

- **Constant range.** A constant high-low spread smooths to a constant EMA,
  whose rate of change is `0`.
- **Expanding range.** A monotonically widening range reads positive.
- **Reset.** `cv.reset()` clears the inner EMA and ROC.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, ChaikinVolatility};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cv = ChaikinVolatility::new(10, 10)?;
    // A constant 2-wide range -> constant EMA -> zero rate of change.
    let candles: Vec<Candle> = (0..40)
        .map(|i| {
            let base = 100.0 + f64::from(i);
            Candle::new(base, base + 1.0, base - 1.0, base, 1.0, i).unwrap()
        })
        .collect();
    println!("{:?}", cv.batch(&candles).last().unwrap());
    Ok(())
}
```

Output:

```
Some(0.0)
```

### Python

```python
import numpy as np
import wickra as ta

cv = ta.ChaikinVolatility(10, 10)
n = 40
base = np.arange(n, dtype=float) + 100.0
print(cv.batch(base + 1.0, base - 1.0)[-1])
```

Output:

```
0.0
```

### Node

```javascript
const ta = require('wickra');
const cv = new ta.ChaikinVolatility(10, 10);
const base = Array.from({ length: 40 }, (_, i) => 100 + i);
const out = cv.batch(base.map((b) => b + 1), base.map((b) => b - 1));
console.log(out[out.length - 1]);
```

Output:

```
0
```

## Interpretation

A rising Chaikin Volatility warns that ranges are expanding fast — Chaikin
associated sharp rises with market tops, where panic widens bars. A low or
falling reading is the calm, range-contracting market that often precedes a
move. It complements [`Atr`](../volatility-bands/Indicator-Atr.md): ATR gives the level of
volatility, Chaikin Volatility gives its momentum.

## Common pitfalls

- **Reading it as a volatility level.** It is a *rate of change* — zero means
  steady ranges, not zero volatility.
- **Feeding it scalar prices.** It needs the `high`/`low` bar.

## References

Marc Chaikin's Chaikin Volatility; the EMA-of-spread rate-of-change definition
here is the standard one.

## See also

- [Indicator-Atr.md](../volatility-bands/Indicator-Atr.md) — the level of per-bar volatility.
- [Indicator-TrueRange.md](../volatility-bands/Indicator-TrueRange.md) — raw single-bar range.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
