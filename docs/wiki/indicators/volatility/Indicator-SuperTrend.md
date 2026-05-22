# SuperTrend

> SuperTrend — an ATR-banded trailing stop that flips sides when price
> closes through the band, reporting both the stop level and the trend
> direction.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volatility |
| Sub-category | Trailing stop |
| Input type | `Candle` (uses `high`, `low`, `close`) |
| Output type | `(value, direction)` |
| Output range | `value`: unbounded (price scale); `direction`: `−1.0` or `+1.0` |
| Default parameters | `atr_period = 10`, `multiplier = 3.0` (Python) |
| Warmup period | `atr_period` |
| Interpretation | Trend-following stop; a direction flip marks a trend change. |

## Formula

```
hl2          = (high + low) / 2
basic_upper  = hl2 + multiplier · ATR
basic_lower  = hl2 − multiplier · ATR

final_upper  = basic_upper  if basic_upper < prev_final_upper or prev_close > prev_final_upper
               else prev_final_upper
final_lower  = basic_lower  if basic_lower > prev_final_lower or prev_close < prev_final_lower
               else prev_final_lower

downtrend: stay down while close <= final_upper, else flip up
uptrend:   stay up   while close >= final_lower, else flip down
SuperTrend = final_lower in an uptrend, final_upper in a downtrend
```

The two final bands ratchet — the upper band only moves down, the lower band
only moves up — until price closes through the active one. That close flips
the trend and hands the trailing-stop role to the opposite band. The result is
a single line that sits below price in an uptrend and above it in a downtrend,
plus a `direction` flag (`+1.0` / `-1.0`) that names which regime you are in.

## Parameters

- `atr_period` — the ATR lookback (Python default `10`).
- `multiplier` — how many ATRs wide the bands sit (Python default `3.0`).

`SuperTrend::classic()` returns Wilder's `(10, 3.0)` configuration.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/super_trend.rs`:

```rust
impl Indicator for SuperTrend {
    type Input = Candle;
    type Output = SuperTrendOutput;   // { value: f64, direction: f64 }
    // update(&mut self, input: Candle) -> Option<SuperTrendOutput>
}
```

`SuperTrend` is a **candle-input** indicator (it reads `high`, `low`, `close`).
Python's streaming `update` returns a `(value, direction)` tuple; the batch
helper returns an `(n, 2)` array with columns `[value, direction]`. Node's
`update` returns `{ value, direction }` and `batch` a flat `[v0, d0, v1, d1, …]`
array; WASM matches Node.

## Warmup

`SuperTrend::classic().warmup_period() == 10`. The first value lands once the
inner ATR is ready, on input index `atr_period − 1`. The first ATR-ready bar
seeds the trend as up; the flip logic corrects it within a few bars if the
market is actually falling.

## Edge cases

- **Seed direction.** The first emitted bar is always `direction = +1.0`; a
  genuine downtrend flips it within a handful of bars.
- **Flat market.** Constant candles give a constant ATR, so both bands and the
  line are flat and the trend never flips.
- **Reset.** `st.reset()` clears the ATR and the carried band state.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, SuperTrend};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut st = SuperTrend::new(5, 3.0)?;
    // Flat market: ATR = 2, hl2 = 10, lower band = 10 - 3·2 = 4.
    let candles: Vec<Candle> = (0..20)
        .map(|i| Candle::new(10.0, 11.0, 9.0, 10.0, 1.0, i).unwrap())
        .collect();
    let out = st.batch(&candles);
    println!("{:?}", out.last().unwrap());
    Ok(())
}
```

Output:

```
Some(SuperTrendOutput { value: 4.0, direction: 1.0 })
```

On a flat market the seeded uptrend never flips and the line holds at the
lower band, `4.0`.

### Python

```python
import numpy as np
import wickra as ta

st = ta.SuperTrend(5, 3.0)
n = 20
high = np.full(n, 11.0)
low = np.full(n, 9.0)
close = np.full(n, 10.0)
print(st.batch(high, low, close)[-1])   # [value, direction]
```

Output:

```
[4. 1.]
```

### Node

```javascript
const ta = require('wickra');
const st = new ta.SuperTrend(5, 3.0);
const n = 20;
const high = Array(n).fill(11), low = Array(n).fill(9), close = Array(n).fill(10);
const out = st.batch(high, low, close);
console.log(out.slice(-2));   // [value, direction] of the last bar
```

Output:

```
[ 4, 1 ]
```

## Interpretation

`SuperTrend` is used as a stop-and-reverse system: stay long while
`direction == +1` and the line trails below price, flip to short the bar the
`direction` turns `-1` and the line jumps above price. A larger `multiplier`
widens the bands — fewer whipsaws, later flips; a smaller one flips sooner.
The line itself doubles as a concrete stop-loss level.

## Common pitfalls

- **Expecting an exact flip bar.** The seed bar is always an uptrend; on
  genuinely falling data the flip lands a few bars in.
- **Reading `value` without `direction`.** The line means "support" in an
  uptrend and "resistance" in a downtrend — `direction` tells you which.

## References

The SuperTrend trailing stop; the final-band ratchet formulation here matches
the widely used TradingView / Olivier Seban definition.

## See also

- [Indicator-Psar.md](Indicator-Psar.md) — Wilder's parabolic stop-and-reverse.
- [Indicator-AtrTrailingStop.md](Indicator-AtrTrailingStop.md) — a plain
  ATR trailing stop without the band ratchet.
- [Indicator-Atr.md](Indicator-Atr.md) — the volatility measure underneath.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
