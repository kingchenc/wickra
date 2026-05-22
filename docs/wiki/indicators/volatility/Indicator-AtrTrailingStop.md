# AtrTrailingStop

> ATR Trailing Stop — a single stop level that trails price by a fixed ATR
> multiple, ratcheting toward the trend and flipping on a close through it.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volatility |
| Sub-category | Trailing stop |
| Input type | `Candle` (uses `high`, `low`, `close`) |
| Output type | `f64` |
| Output range | unbounded (price scale) |
| Default parameters | `atr_period = 14`, `multiplier = 3.0` (Python) |
| Warmup period | `atr_period` |
| Interpretation | One trailing stop line; price closing through it flips the trade. |

## Formula

```
loss = multiplier · ATR

stop_t = max(stop_{t−1}, close − loss)   while price holds above the stop
       = min(stop_{t−1}, close + loss)   while price holds below the stop
       = close − loss                    on a fresh break above the stop
       = close + loss                    on a fresh break below the stop
```

This is the trailing stop popularised by the "UT Bot": a single line that sits
`multiplier · ATR` away from the close. While price holds on one side of the
stop the level only ratchets *toward* price — up in an uptrend, down in a
downtrend — and never away from it. When a close crosses the stop the level
snaps to the opposite side of the new close, flipping the trade. Unlike the
[`ChandelierExit`](Indicator-ChandelierExit.md), it hangs off the close
itself, not the window's extreme, and reports one line rather than two.

## Parameters

- `atr_period` — the ATR lookback (Python default `14`).
- `multiplier` — the ATR multiple the stop trails by (Python default `3.0`).

`AtrTrailingStop::classic()` returns the `(14, 3.0)` configuration.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/atr_trailing_stop.rs`:

```rust
impl Indicator for AtrTrailingStop {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`AtrTrailingStop` is a **candle-input** indicator (it reads `high`, `low`,
`close`). In Python the streaming `update` accepts a 6-tuple or a dict; the
batch helper takes `high`, `low`, `close` numpy arrays. Node and WASM expose
`update(high, low, close)` and the matching `batch`.

## Warmup

`AtrTrailingStop::classic().warmup_period() == 14`. The first value lands once
the inner ATR is ready, on input index `atr_period − 1`. That first bar seeds
the stop below price (a long).

## Edge cases

- **Seed bar.** The first emitted stop is `close − loss` — the indicator
  starts on the long side.
- **Ratchet.** While price holds above the stop it never moves down, and
  while price holds below it never moves up
  (`uptrend_stop_ratchets_up_and_stays_below_price` pins this).
- **Flat market.** Constant candles hold the stop at a fixed `close − loss`.
- **Reset.** `ts.reset()` clears the ATR and the carried stop / close.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, AtrTrailingStop};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ts = AtrTrailingStop::new(5, 3.0)?;
    // Flat market: ATR = 2, loss = 3·2 = 6, stop = 10 - 6 = 4.
    let candles: Vec<Candle> = (0..20)
        .map(|i| Candle::new(10.0, 11.0, 9.0, 10.0, 1.0, i).unwrap())
        .collect();
    let out = ts.batch(&candles);
    println!("{:?}", out.last().unwrap());
    Ok(())
}
```

Output:

```
Some(4.0)
```

On a flat market the seeded long stop holds at `close − loss = 10 − 6 = 4`.
This matches the `reference_values_flat_market` test in
`crates/wickra-core/src/indicators/atr_trailing_stop.rs`.

### Python

```python
import numpy as np
import wickra as ta

ts = ta.AtrTrailingStop(5, 3.0)
n = 20
high = np.full(n, 11.0)
low = np.full(n, 9.0)
close = np.full(n, 10.0)
print(ts.batch(high, low, close)[-1])
```

Output:

```
4.0
```

### Node

```javascript
const ta = require('wickra');
const ts = new ta.AtrTrailingStop(5, 3.0);
const n = 20;
const high = Array(n).fill(11), low = Array(n).fill(9), close = Array(n).fill(10);
const out = ts.batch(high, low, close);
console.log(out[out.length - 1]);
```

Output:

```
4
```

## Interpretation

Read it as a stop-and-reverse line: while the stop sits below the close you
are long and it trails your profit up; the bar a close prints below the stop,
it flips above the new close and you are short. A larger `multiplier` gives
the trade more room — fewer flips, wider risk; a smaller one flips sooner.

## Common pitfalls

- **Expecting it off the window high.** It trails the *close*, so it can sit
  closer to price than a [`ChandelierExit`](Indicator-ChandelierExit.md).
- **Feeding it scalar prices.** It needs the full `high`/`low`/`close` bar to
  drive the ATR.

## References

The ATR Trailing Stop used by the well-known "UT Bot"; the four-branch ratchet
here matches the common Sylvain Vervoort formulation.

## See also

- [Indicator-SuperTrend.md](Indicator-SuperTrend.md) — an ATR trailing stop
  with band ratcheting and an explicit direction flag.
- [Indicator-ChandelierExit.md](Indicator-ChandelierExit.md) — an ATR stop hung
  off the window's extreme instead of the close.
- [Indicator-Atr.md](Indicator-Atr.md) — the volatility measure underneath.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
