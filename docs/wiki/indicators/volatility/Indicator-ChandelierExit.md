# ChandelierExit

> Chandelier Exit — an ATR trailing stop hung a fixed number of ATRs off
> the highest high (for longs) or the lowest low (for shorts) of a window.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volatility |
| Sub-category | Trailing stop |
| Input type | `Candle` (uses `high`, `low`, `close`) |
| Output type | `(long_stop, short_stop)` |
| Output range | unbounded (price scale) |
| Default parameters | `period = 22`, `multiplier = 3.0` (Python) |
| Warmup period | `period` |
| Interpretation | Long/short trailing-stop levels; a close past one exits the trade. |

## Formula

```
long_stop  = highest_high(period) − multiplier · ATR(period)
short_stop = lowest_low(period)   + multiplier · ATR(period)
```

Chuck LeBeau's Chandelier Exit hangs the stop off the extreme of the lookback
window — like a chandelier off a ceiling — a fixed `multiplier · ATR` below the
highest high (for a long) or above the lowest low (for a short). Because the
extreme only moves favourably while a trend runs, the stop trails price up
(or down) and never loosens. A long is exited when price closes below
`long_stop`; a short when it closes above `short_stop`. The classic
configuration is a `22`-bar window with a `3.0` multiplier.

## Parameters

- `period` — the window for both the highest high / lowest low and the ATR
  (Python default `22`).
- `multiplier` — how many ATRs the stop hangs off the extreme (default `3.0`).

`ChandelierExit::classic()` returns the `(22, 3.0)` configuration.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/chandelier_exit.rs`:

```rust
impl Indicator for ChandelierExit {
    type Input = Candle;
    type Output = ChandelierExitOutput;   // { long_stop: f64, short_stop: f64 }
    // update(&mut self, input: Candle) -> Option<ChandelierExitOutput>
}
```

`ChandelierExit` is a **candle-input** indicator (it reads `high`, `low`,
`close`). Python's streaming `update` returns a `(long_stop, short_stop)`
tuple; the batch helper returns an `(n, 2)` array with columns
`[long_stop, short_stop]`. Node's `update` returns `{ longStop, shortStop }`
and `batch` a flat `[l0, s0, l1, s1, …]` array; WASM matches Node.

## Warmup

`ChandelierExit::classic().warmup_period() == 22`. The highest-high / lowest-low
window and the inner ATR become ready on the same candle — input index
`period − 1`.

## Edge cases

- **Window bound.** `long_stop` never exceeds the window's highest high, and
  `short_stop` never drops below its lowest low
  (`long_stop_below_highest_short_stop_above_lowest` pins this).
- **Flat market.** Constant candles give constant `ATR` and equal extremes, so
  both stops sit a fixed `multiplier · ATR` from the price.
- **Reset.** `ce.reset()` clears the ATR and both extreme windows.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, ChandelierExit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ce = ChandelierExit::new(5, 3.0)?;
    // Flat market: ATR = 2, HH = 11, LL = 9.
    let candles: Vec<Candle> = (0..20)
        .map(|i| Candle::new(10.0, 11.0, 9.0, 10.0, 1.0, i).unwrap())
        .collect();
    let out = ce.batch(&candles);
    println!("{:?}", out.last().unwrap());
    Ok(())
}
```

Output:

```
Some(ChandelierExitOutput { long_stop: 5.0, short_stop: 15.0 })
```

`long_stop = 11 − 3·2 = 5`, `short_stop = 9 + 3·2 = 15`. This matches the
`reference_values_flat_market` test in
`crates/wickra-core/src/indicators/chandelier_exit.rs`.

### Python

```python
import numpy as np
import wickra as ta

ce = ta.ChandelierExit(5, 3.0)
n = 20
high = np.full(n, 11.0)
low = np.full(n, 9.0)
close = np.full(n, 10.0)
print(ce.batch(high, low, close)[-1])   # [long_stop, short_stop]
```

Output:

```
[ 5. 15.]
```

### Node

```javascript
const ta = require('wickra');
const ce = new ta.ChandelierExit(5, 3.0);
const n = 20;
const high = Array(n).fill(11), low = Array(n).fill(9), close = Array(n).fill(10);
const out = ce.batch(high, low, close);
console.log(out.slice(-2));   // [long_stop, short_stop] of the last bar
```

Output:

```
[ 5, 15 ]
```

## Interpretation

While long, watch `long_stop`: it climbs as new highs print and never falls,
so a close beneath it is a disciplined exit. While short, `short_stop` is the
mirror. The `3.0` multiplier is wide enough to ride a trend through normal
pullbacks; tightening it exits sooner at the cost of more whipsaws.

## Common pitfalls

- **Using the wrong stop for the position.** `long_stop` only applies to
  longs, `short_stop` only to shorts — they are not a channel.
- **Feeding it scalar prices.** It needs the full `high`/`low`/`close` bar.

## References

Chuck LeBeau's Chandelier Exit; the highest-high-minus-ATR formulation here
matches the standard definition.

## See also

- [Indicator-SuperTrend.md](Indicator-SuperTrend.md) — an ATR trailing stop
  with explicit flip logic and a single line.
- [Indicator-ChandeKrollStop.md](Indicator-ChandeKrollStop.md) — a two-stage
  ATR stop that smooths the preliminary level.
- [Indicator-Atr.md](Indicator-Atr.md) — the volatility measure underneath.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
