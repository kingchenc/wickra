# ADL

> Accumulation/Distribution Line — a cumulative volume-flow line that
> weights each bar's volume by where its close fell within the range.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volume |
| Sub-category | Cumulative |
| Input type | `Candle` (uses `high`, `low`, `close`, `volume`) |
| Output type | `f64` |
| Output range | unbounded (drifts with cumulative volume) |
| Default parameters | none (no parameters) |
| Warmup period | `1` |
| Interpretation | Running buying/selling pressure; slope and divergence matter. |

## Formula

```
MFM_t = ((close − low) − (high − close)) / (high − low)   (money-flow multiplier, −1..+1)
MFV_t = MFM_t · volume_t                                  (money-flow volume)
ADL_t = ADL_{t−1} + MFV_t
```

The money-flow multiplier asks *where in the bar's range did price
close?* A close on the high gives `+1` (full accumulation), on the low
`−1` (full distribution), in the middle `0`. Scaling by volume and
running the cumulative total gives a line whose **slope** reflects
sustained buying or selling pressure. A bar with `high == low` carries no
positional information and contributes `0`.

## Parameters

`ADL` takes **no parameters** — `Adl::new()` in Rust, `wickra.ADL()` in
Python, `new ta.ADL()` in Node.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/adl.rs`:

```rust
impl Indicator for Adl {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`ADL` is a **candle-input** indicator: it reads `high`, `low`, `close` and
`volume`. In Python the streaming `update` accepts a 6-tuple or a dict;
the batch helper takes `high`, `low`, `close`, `volume` numpy arrays. Node
and WASM expose `update(high, low, close, volume)` and the matching
`batch`.

## Warmup

`Adl::new().warmup_period() == 1`. ADL is cumulative — it emits a value
from the very first candle.

## Edge cases

- **Zero-range bar.** A bar with `high == low` contributes `0` to the line
  (`zero_range_bar_contributes_nothing` pins this).
- **Close at the high.** Every bar closing on its high has `MFM = +1`, so
  ADL grows by exactly `volume` each bar
  (`close_at_high_accumulates_full_volume` pins this).
- **Candle validation.** `Candle::new` rejects invalid bars upstream.
- **Reset.** `adl.reset()` returns the running total to `0`.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, Adl};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut adl = Adl::new();
    let out = adl.batch(&[
        Candle::new(8.0, 10.0, 8.0, 10.0, 100.0, 0)?,  // close at high
        Candle::new(10.0, 12.0, 8.0, 9.0, 200.0, 1)?,
    ]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[Some(100.0), Some(0.0)]
```

Bar 1 closes at its high (`MFM = +1`), adding `+100`. Bar 2 has
`MFM = ((9−8)−(12−9))/4 = −0.5`, adding `−100`, so the line returns to
`0`. This matches the `reference_values` test in
`crates/wickra-core/src/indicators/adl.rs`.

### Python

```python
import numpy as np
import wickra as ta

adl = ta.ADL()
high = np.array([10.0, 12.0])
low = np.array([8.0, 8.0])
close = np.array([10.0, 9.0])
volume = np.array([100.0, 200.0])
print(adl.batch(high, low, close, volume))
```

Output:

```
[100.   0.]
```

### Node

```javascript
const ta = require('wickra');
const adl = new ta.ADL();
console.log(adl.batch([10, 12], [8, 8], [10, 9], [100, 200]));
```

Output:

```
[ 100, 0 ]
```

## Interpretation

`Adl` is read by slope and by divergence, never by absolute level (the
total drifts arbitrarily with cumulative volume). A rising ADL confirms
that an up-move is backed by accumulation; a *falling* ADL while price
rises is a bearish divergence — the rally is not being bought into.
[`ChaikinOscillator`](Indicator-ChaikinOscillator.md) is the standard way
to turn the ADL into a bounded, tradeable oscillator.

## Common pitfalls

- **Reading the absolute value.** Only the slope and divergences are
  meaningful; the level depends on where you started the stream.
- **Feeding it scalar prices.** It needs the full OHLCV bar.

## References

Marc Chaikin's Accumulation/Distribution Line; the money-flow-multiplier
formulation here matches the standard definition (StockCharts, TA-Lib's
`AD`).

## See also

- [Indicator-Obv.md](Indicator-Obv.md) — cumulative *signed* volume.
- [Indicator-ChaikinOscillator.md](Indicator-ChaikinOscillator.md) — an
  oscillator built on the ADL.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
