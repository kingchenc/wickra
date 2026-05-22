# ForceIndex

> Force Index — Alexander Elder's price change scaled by volume, then
> smoothed with an EMA.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volume |
| Sub-category | Oscillators |
| Input type | `Candle` (uses `close`, `volume`) |
| Output type | `f64` |
| Output range | unbounded around zero |
| Default parameters | `period = 13` (Python) |
| Warmup period | `period + 1` |
| Interpretation | Conviction behind a move; sign and zero-crossings are the signal. |

## Formula

```
raw_t   = (close_t − close_{t−1}) · volume_t
Force_t = EMA(raw, period)_t
```

The raw force is positive on an up-close and negative on a down-close, with a
magnitude that grows with the volume backing the move — a large move on heavy
volume registers a large force, a large move on thin volume does not.
Smoothing the raw series with an EMA turns the noisy per-bar reading into a
tradeable line; Elder's classic period is `13`.

## Parameters

`period` — the EMA smoothing period. The Python binding defaults it to `13`;
the Rust and Node constructors require it explicitly.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/force_index.rs`:

```rust
impl Indicator for ForceIndex {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`ForceIndex` is a **candle-input** indicator that reads `close` and `volume`.
In Python the streaming `update` accepts a 6-tuple or a dict; the batch helper
takes `close`, `volume` numpy arrays. Node and WASM expose
`update(close, volume)` and the matching `batch`.

## Warmup

`ForceIndex::new(13).warmup_period() == 14`. The first candle only establishes
the previous close, so the first raw force appears on candle 2 and the first
smoothed value on candle `period + 1`.

## Edge cases

- **First candle.** Establishes the previous close only; emits `None`.
- **Up- vs down-trend.** A strictly rising series gives a positive force, a
  strictly falling series a negative one (`pure_uptrend_is_positive` and
  `pure_downtrend_is_negative` pin this).
- **`period = 1`.** `EMA(1)` has `alpha = 1`, so the Force Index passes the
  raw force through unsmoothed.
- **Reset.** `fi.reset()` clears the previous close and the EMA.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, ForceIndex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ForceIndex(1): EMA(1) passes the raw force through.
    let mut fi = ForceIndex::new(1)?;
    let out = fi.batch(&[
        Candle::new(10.0, 10.0, 10.0, 10.0, 100.0, 0)?,  // seeds the previous close
        Candle::new(12.0, 12.0, 12.0, 12.0, 100.0, 1)?,  // raw = (12-10)·100
        Candle::new(11.0, 11.0, 11.0, 11.0, 200.0, 2)?,  // raw = (11-12)·200
    ]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[None, Some(200.0), Some(-200.0)]
```

This matches the `reference_values` test in
`crates/wickra-core/src/indicators/force_index.rs`.

### Python

```python
import numpy as np
import wickra as ta

fi = ta.ForceIndex(1)
close = np.array([10.0, 12.0, 11.0])
volume = np.array([100.0, 100.0, 200.0])
print(fi.batch(close, volume))
```

Output:

```
[  nan  200. -200.]
```

### Node

```javascript
const ta = require('wickra');
const fi = new ta.ForceIndex(1);
console.log(fi.batch([10, 12, 11], [100, 100, 200]));
```

Output:

```
[ NaN, 200, -200 ]
```

## Interpretation

Elder reads the Force Index on two horizons. A short period (the classic `2`)
is a sensitive entry timer — it crosses zero often. A longer period (`13`)
tracks the conviction behind the prevailing trend: it staying above zero
confirms buyers are in control. Divergence between a `13`-period Force Index
and price flags an exhausting move.

## Common pitfalls

- **Comparing levels across instruments.** The force scales with raw volume,
  so a value of `200` means nothing without knowing the instrument.
- **Feeding it scalar prices.** It needs `close` *and* `volume`.

## References

Alexander Elder's Force Index, introduced in *Trading for a Living* (1993).

## See also

- [Indicator-Obv.md](Indicator-Obv.md) — cumulative signed volume, a coarser
  volume-conviction gauge.
- [Indicator-VolumePriceTrend.md](Indicator-VolumePriceTrend.md) — cumulative
  volume scaled by percentage move.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
