# VolumePriceTrend

> Volume-Price Trend (VPT) — a cumulative volume line where each bar's
> contribution is scaled by its percentage price change.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volume |
| Input type | `Candle` (uses `close`, `volume`) |
| Output type | `f64` |
| Output range | unbounded (drifts with cumulative volume) |
| Default parameters | none (no parameters) |
| Warmup period | `1` |
| Interpretation | Running volume flow; slope and divergence matter. |

## Formula

```
VPT_t = VPT_{t−1} + volume_t · (close_t − close_{t−1}) / close_{t−1}
```

VPT is a close relative of [`Obv`](../volume/Indicator-Obv.md). Where OBV adds the
*entire* bar volume on any up-close, VPT adds volume scaled by the **size**
of the move: a 2 % gain on a given volume moves the line twice as far as a
1 % gain on the same volume. That makes VPT more sensitive to the
conviction behind a move. The first bar establishes the baseline at `0`.

## Parameters

`VolumePriceTrend` takes **no parameters** — `VolumePriceTrend::new()` in
Rust, `wickra.VolumePriceTrend()` in Python, `new ta.VolumePriceTrend()`
in Node.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/vpt.rs`:

```rust
impl Indicator for VolumePriceTrend {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`VolumePriceTrend` is a **candle-input** indicator: it reads `close` and
`volume`. In Python the streaming `update` accepts a 6-tuple or a dict;
the batch helper takes `close` and `volume` numpy arrays. Node and WASM
expose `update(close, volume)` and `batch(close, volume)`.

## Warmup

`warmup_period() == 1`. VPT is cumulative — it emits the baseline `0` from
the first candle, then accumulates from the second onward.

## Edge cases

- **Constant close.** With no price change every bar contributes `0`, so
  the line stays flat regardless of volume
  (`constant_close_keeps_line_flat` pins this).
- **First bar.** The first candle has no previous close; VPT emits the
  baseline `0.0` (`emits_from_first_candle_at_zero` pins this).
- **Zero previous close.** A percentage change against a `0.0` prior
  close is undefined and is treated as `0`.
- **Candle validation.** `Candle::new` rejects invalid bars upstream.
- **Reset.** `vpt.reset()` returns the running total to `0`.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, VolumePriceTrend};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vpt = VolumePriceTrend::new();
    // closes 10 -> 11 -> 9, volumes 100, 200, 300.
    let out = vpt.batch(&[
        Candle::new(10.0, 10.0, 10.0, 10.0, 100.0, 0)?,
        Candle::new(11.0, 11.0, 11.0, 11.0, 200.0, 1)?,
        Candle::new(9.0, 9.0, 9.0, 9.0, 300.0, 2)?,
    ]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[Some(0.0), Some(20.0), Some(-34.54545454545455)]
```

Bar 1 is the baseline `0`. Bar 2 adds `200 · (11−10)/10 = 20`. Bar 3 adds
`300 · (9−11)/11 = −600/11`, leaving `20 − 600/11 ≈ −34.545`. This matches
the `reference_values` test in `crates/wickra-core/src/indicators/vpt.rs`.

### Python

```python
import numpy as np
import wickra as ta

vpt = ta.VolumePriceTrend()
close = np.array([10.0, 11.0, 9.0])
volume = np.array([100.0, 200.0, 300.0])
print(vpt.batch(close, volume))
```

Output:

```
[  0.          20.         -34.54545455]
```

### Node

```javascript
const ta = require('wickra');
const vpt = new ta.VolumePriceTrend();
console.log(vpt.batch([10, 11, 9], [100, 200, 300]));
```

Output:

```
[ 0, 20, -34.54545454545455 ]
```

## Interpretation

`VolumePriceTrend` is read like OBV — by **slope** and by **divergence**,
never by absolute level. A VPT rising in step with price confirms the
trend is volume-supported; VPT flattening or falling while price climbs
is a bearish divergence warning that the move lacks participation. Versus
OBV, VPT gives proportionally more weight to large moves and less to a
string of tiny up-closes, so it tracks the *magnitude* of conviction, not
just its direction.

## Common pitfalls

- **Reading the absolute value.** Only slope and divergences carry
  meaning; the level depends on the stream's start point.
- **Expecting OBV-identical behaviour.** VPT scales by percentage change,
  so the two lines diverge — especially across large single-bar moves.

## References

The Volume-Price Trend (also "Price-Volume Trend") is a standard
cumulative volume study; the `volume · ROC` accumulation here matches the
common definition.

## See also

- [Indicator-Obv.md](../volume/Indicator-Obv.md) — cumulative signed volume, the
  closest relative.
- [Indicator-Adl.md](../volume/Indicator-Adl.md) — cumulative range-weighted volume.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
