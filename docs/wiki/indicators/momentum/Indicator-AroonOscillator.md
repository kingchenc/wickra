# AroonOscillator

> Aroon Oscillator — the single-line difference `AroonUp − AroonDown`,
> condensing the two Aroon lines into one trend gauge.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum (trend strength) |
| Sub-category | Bounded oscillators |
| Input type | `Candle` (uses `high`, `low`) |
| Output type | `f64` |
| Output range | `[−100, 100]` |
| Default parameters | `period = 14` (Python) |
| Warmup period | `period + 1` |
| Interpretation | Positive = up-trend, negative = down-trend, near zero = range. |

## Formula

```
AroonOscillator = AroonUp − AroonDown
```

where [`Aroon`](Indicator-Aroon.md) reports two `[0, 100]` lines measuring
how recently the window's highest high and lowest low occurred. Their
difference lives in `[−100, 100]`: strongly positive means the most recent
high is much fresher than the most recent low (an up-trend); strongly
negative is the mirror image; near zero means neither extreme is recent.

## Parameters

| Name     | Type    | Default       | Valid range | Description |
|----------|---------|---------------|-------------|-------------|
| `period` | `usize` | `14` (Python) | `>= 1`      | Aroon lookback window. `0` errors with `Error::PeriodZero`. |

The Python binding defaults `period` to `14`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/aroon_oscillator.rs`:

```rust
impl Indicator for AroonOscillator {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`AroonOscillator` is a **candle-input** indicator: it reads `high` and
`low`. In Python the streaming `update` accepts a 6-tuple or a dict; the
batch helper takes `high` and `low` numpy arrays. Node and WASM expose
`update(high, low)` and `batch(high, low)`.

## Warmup

`AroonOscillator::new(period).warmup_period() == period + 1` — identical
to the underlying `Aroon`, which needs a `period + 1`-bar window before
the first reading.

## Edge cases

- **Pure trend.** A series of fresh highs gives `AroonUp = 100`,
  `AroonDown = 0`, so the oscillator is `+100`; a series of fresh lows is
  `−100` (`pure_uptrend_yields_plus_100` /
  `pure_downtrend_yields_minus_100` pin this).
- **Bounds.** The output is always within `[−100, 100]`
  (`output_stays_within_minus_100_and_100` pins this).
- **Candle validation.** `Candle::new` rejects invalid bars before
  `update` ever sees them.
- **Reset.** `osc.reset()` clears the underlying Aroon window.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, AroonOscillator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut osc = AroonOscillator::new(5)?;
    // 30 bars, each a fresh high.
    let candles: Vec<Candle> = (0..30)
        .map(|i| {
            let p = 100.0 + f64::from(i);
            Candle::new(p, p + 1.0, p - 1.0, p, 1.0, i64::from(i)).unwrap()
        })
        .collect();
    let out = osc.batch(&candles);
    println!("last = {:?}", out.last().unwrap());
    Ok(())
}
```

Output:

```
last = Some(100.0)
```

Every bar is a fresh high and never a fresh low, so the oscillator pins at
`+100`. This matches the `pure_uptrend_yields_plus_100` test in
`crates/wickra-core/src/indicators/aroon_oscillator.rs`.

### Python

```python
import numpy as np
import wickra as ta

osc = ta.AroonOscillator(14)
high = np.arange(100.0, 140.0)
low = high - 2.0
print(osc.batch(high, low)[-1])  # steady uptrend -> 100
```

Output:

```
100.0
```

### Node

```javascript
const ta = require('wickra');
const osc = new ta.AroonOscillator(14);
const high = Array.from({ length: 40 }, (_, i) => 100 + i);
const low = high.map((h) => h - 2);
console.log(osc.batch(high, low).at(-1)); // 100
```

## Interpretation

`AroonOscillator` is a compact trend gauge. The two canonical reads are
the zero-line cross (`AroonUp` overtaking `AroonDown` or vice versa — a
trend change) and the magnitude (values pinned near `±100` confirm a
strong, uninterrupted trend; values oscillating near zero confirm a
range). Use it where the two-line `Aroon` is more detail than you need.

## Common pitfalls

- **Feeding it scalar prices.** It needs `high`/`low`; it takes a
  `Candle`, not an `f64`.
- **Expecting the `[0, 100]` Aroon scale.** The oscillator is signed and
  spans `[−100, 100]`.

## References

Tushar Chande's Aroon system (1995); the oscillator is the standard
`AroonUp − AroonDown` difference.

## See also

- [Indicator-Aroon.md](Indicator-Aroon.md) — the two-line indicator this
  collapses.
- [Indicator-Adx.md](Indicator-Adx.md) — another trend-strength gauge.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
