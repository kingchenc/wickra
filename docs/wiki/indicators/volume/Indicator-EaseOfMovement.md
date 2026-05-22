# EaseOfMovement

> Ease of Movement (EOM) — Richard Arms' measure of how far price travels
> per unit of volume, averaged over a window.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volume |
| Input type | `Candle` (uses `high`, `low`, `volume`) |
| Output type | `f64` |
| Output range | unbounded around zero (scaled by `divisor`) |
| Default parameters | `period = 14`, `divisor = 1e8` (Python) |
| Warmup period | `period + 1` |
| Interpretation | Light-volume moves push it away from zero; sign tracks direction. |

## Formula

```
distance_t = (high_t + low_t)/2 − (high_{t−1} + low_{t−1})/2
EMV_t      = distance_t · (high_t − low_t) · divisor / volume_t
EOM_t      = SMA(EMV, period)_t
```

Arms' question is *how easily did price move?* A bar whose midpoint jumped a
long way on a wide range but light volume gets a large `EMV`; a bar that
needed heavy volume to budge gets a small one. The `divisor` is a pure
output-scaling constant — the conventional `1e8` keeps `EMV` readable for
typical share volumes; smaller markets want a smaller divisor. The window SMA
smooths the noisy per-bar `EMV` into a tradeable line.

## Parameters

- `period` — the SMA averaging window (Python default `14`).
- `divisor` — the volume-scaling constant (Python default `1e8`). Rust exposes
  `EaseOfMovement::new(period)` for the `1e8` default and
  `EaseOfMovement::with_divisor(period, divisor)` for an explicit value.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/ease_of_movement.rs`:

```rust
impl Indicator for EaseOfMovement {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`EaseOfMovement` is a **candle-input** indicator that reads `high`, `low` and
`volume`. In Python the streaming `update` accepts a 6-tuple or a dict; the
batch helper takes `high`, `low`, `volume` numpy arrays. Node and WASM expose
`update(high, low, volume)` and the matching `batch`.

## Warmup

`EaseOfMovement::new(14).warmup_period() == 15`. The first candle only seeds
the previous midpoint, so the first `EMV` appears on candle 2 and the first
averaged value on candle `period + 1`.

## Edge cases

- **Zero-volume bar.** Contributes `EMV = 0` instead of dividing by zero
  (`zero_volume_contributes_zero` pins this).
- **Zero-range bar.** `high == low` makes the `(high − low)` factor zero, so
  `EMV = 0`.
- **Constant series.** Unchanging midpoints give zero distance, so EOM stays
  at `0.0` (`constant_series_yields_zero` pins this).
- **Reset.** `eom.reset()` clears the previous midpoint and the SMA window.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, EaseOfMovement};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // EOM(period = 1, divisor = 1): one EMV value is its own average.
    let mut eom = EaseOfMovement::with_divisor(1, 1.0)?;
    let out = eom.batch(&[
        Candle::new(9.0, 10.0, 8.0, 9.0, 50.0, 0)?,      // seeds the previous midpoint (9)
        Candle::new(12.0, 14.0, 10.0, 12.0, 100.0, 1)?,  // mid 12, distance 3, range 4
    ]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[None, Some(0.12)]
```

Bar 2: `EMV = distance · range · divisor / volume = 3 · 4 · 1 / 100 = 0.12`.
This matches the `reference_values` test in
`crates/wickra-core/src/indicators/ease_of_movement.rs`.

### Python

```python
import numpy as np
import wickra as ta

eom = ta.EaseOfMovement(1, 1.0)
high = np.array([10.0, 14.0])
low = np.array([8.0, 10.0])
volume = np.array([50.0, 100.0])
print(eom.batch(high, low, volume))
```

Output:

```
[ nan 0.12]
```

### Node

```javascript
const ta = require('wickra');
const eom = new ta.EaseOfMovement(1, 1.0);
console.log(eom.batch([10, 14], [8, 10], [50, 100]));
```

Output:

```
[ NaN, 0.12 ]
```

## Interpretation

EOM crossing above zero says price is drifting up *without* needing much
volume — an easy, low-resistance advance; below zero is the same for a
decline. A reading hovering near zero means volume is heavy relative to the
distance covered, i.e. price is grinding. The sign tracks direction; the
distance from zero tracks how freely the move is happening.

## Common pitfalls

- **Reading the raw magnitude.** It depends entirely on the `divisor` you
  chose — only the sign and relative size are portable.
- **Feeding it scalar prices.** It needs `high`, `low` *and* `volume`.

## References

Richard W. Arms Jr.'s Ease of Movement; the box-ratio formulation here matches
the standard definition.

## See also

- [Indicator-ForceIndex.md](../volume/Indicator-ForceIndex.md) — a different
  price-change-vs-volume gauge.
- [Indicator-ChaikinMoneyFlow.md](../volume/Indicator-ChaikinMoneyFlow.md) — bounded
  money-flow balance.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
