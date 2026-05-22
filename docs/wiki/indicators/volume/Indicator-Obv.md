# OBV (On-Balance Volume)

> A cumulative signed-volume series: each candle adds its volume on an up
> close, subtracts on a down close, and leaves the running total unchanged
> on a flat close. The shape of the OBV curve, not its absolute level, is
> what carries information.

## Quick reference

| Item                | Value                                                        |
|---------------------|--------------------------------------------------------------|
| Family              | Volume                                                       |
| Sub-category        | cumulative                                                   |
| Input type          | `Candle` (uses `close` and `volume`)                         |
| Output type         | `f64`                                                        |
| Output range        | unbounded (signed, integer-of-volume in spirit)              |
| Default parameters  | none                                                         |
| Warmup period       | `1`                                                          |
| Interpretation      | divergence vs price signals accumulation / distribution      |

## Formula

For each candle `t > 0` (after the seed):

```
if close_t > close_{t-1}:  OBV_t = OBV_{t-1} + volume_t
if close_t < close_{t-1}:  OBV_t = OBV_{t-1} - volume_t
if close_t == close_{t-1}: OBV_t = OBV_{t-1}
```

The first candle initialises the running total to `0.0` and emits
that value (`crates/wickra-core/src/indicators/obv.rs:42-55`).

## Parameters

`Obv::new()` takes no parameters. Python: `wickra.OBV()`. Node:
`new w.OBV()`.

## Inputs / Outputs

```rust
impl Indicator for Obv {
    type Input  = Candle;
    type Output = f64;
    fn update(&mut self, candle: Candle) -> Option<f64>;
    fn warmup_period(&self) -> usize { 1 }
}
```

- **Python streaming.** Accepts a 6-tuple or dict candle; returns
  `float | None`.
- **Python batch.** `OBV.batch(close, volume)` takes two equal-length
  1-D `numpy.ndarray` columns and returns a 1-D `np.ndarray`. The
  first value is `0.0`, never `NaN`.
- **Node streaming.** Not exposed; the Node binding ships only
  `batch` for `OBV`.
- **Node batch.** `obv.batch(close, volume)` returns `Array<number>`
  of the same length.

## Warmup

`warmup_period() == 1`. The very first candle emits `0.0` by
convention (the "baseline" — there is no prior close to compare
against, so the indicator starts the running total at zero). Every
subsequent candle emits the updated cumulative total.

## Edge cases

- **First bar.** Always emits `0.0` (pinned test
  `first_candle_baseline_zero`). This is the canonical OBV convention
  used by Granville's original formulation.
- **Equal closes.** A candle with `close_t == close_{t-1}` does not
  change the running total — the volume is discarded. (`obv.rs:46-50`).
- **Down close.** Subtracts the bar's volume, so OBV can go strongly
  negative on a sustained downtrend; that is expected and meaningful.
- **Zero volume.** A zero-volume bar adds or subtracts `0`, so OBV
  is unchanged regardless of close direction.
- **NaN / infinity.** `Candle::new` rejects non-finite OHLCV values
  before they reach OBV.
- **Reset.** `reset()` zeroes the running total and clears the
  `has_emitted` / `prev_close` state.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, Obv};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let candles = vec![
        Candle::new(10.0, 10.0, 10.0, 10.0, 100.0, 0)?, // baseline -> 0
        Candle::new(10.0, 11.0, 10.0, 11.0,  20.0, 0)?, // up   -> +20
        Candle::new(11.0, 11.0, 10.5, 10.5,  30.0, 0)?, // down -> -30
        Candle::new(10.5, 10.5, 10.5, 10.5,  40.0, 0)?, // flat ->   0
        Candle::new(10.5, 12.0, 10.5, 12.0,  10.0, 0)?, // up   -> +10
    ];
    let mut obv = Obv::new();
    println!("{:?}", obv.batch(&candles));
    Ok(())
}
```

Output:

```
[Some(0.0), Some(20.0), Some(-10.0), Some(-10.0), Some(0.0)]
```

Hand check: baseline `0`, then `0 + 20 = 20`, then `20 - 30 = -10`,
then `-10` (flat close discards the 40), then `-10 + 10 = 0`.

### Python

```python
import numpy as np
import wickra as ta

obv = ta.OBV()
c = np.array([10.0, 11.0, 10.5, 10.5, 12.0])
v = np.array([100.0, 20.0, 30.0, 40.0, 10.0])
print(obv.batch(c, v))
```

Output:

```
[  0.  20. -10. -10.   0.]
```

### Node

```js
const w = require('wickra');

const obv = new w.OBV();
console.log(obv.batch(
  [10, 11, 10.5, 10.5, 12],
  [100, 20, 30, 40, 10],
));
```

Output:

```
[ 0, 20, -10, -10, 0 ]
```

## Interpretation

- **Divergence is the signal.** OBV's absolute level depends entirely
  on where the series started and is therefore meaningless on its
  own. The interpretable signal is the *shape* of OBV relative to
  price: a new price high without a new OBV high (bearish divergence)
  suggests the rally is not being confirmed by accumulating buy
  volume, and vice versa.
- **Trend confirmation.** A rising OBV that tracks a rising price is
  confirmation of the trend; a flattening OBV under a still-rising
  price is the canonical warning of distribution.
- **Smoothing.** Many traders apply an SMA or EMA to OBV (e.g. 20-period
  SMA) and treat crossings of that smoothed line as buy/sell triggers.

## Common pitfalls

- **Absolute value is arbitrary.** Comparing OBV values across
  different start times or different instruments is meaningless —
  only slopes, divergences, and crossings of derived smoothers carry
  signal.
- **Flat closes discard volume.** A candle that closes exactly at the
  previous close contributes nothing to OBV no matter how heavy its
  volume. Some practitioners prefer A/D-style alternatives (e.g.
  Chaikin Money Flow) that distribute the volume according to where
  in the bar's range the close landed, precisely to avoid this
  discontinuity.

## References

- Joseph Granville, *Granville's New Strategy of Daily Stock Market
  Timing for Maximum Profit*, Prentice-Hall, 1976. The OBV
  construction was first popularised in Granville's earlier 1963
  work and refined in his subsequent books.

## See also

- [VWAP](Indicator-Vwap.md) — volume-weighted price benchmark; OBV and
  VWAP are the two canonical volume-aware indicators in the panel.
- [MFI](../momentum/Indicator-Mfi.md) — money-flow index, an oscillator blending
  typical price with volume.
- [SMA](../trend/Indicator-Sma.md) / [EMA](../trend/Indicator-Ema.md) — the smoothers
  most commonly layered on top of OBV to define trade triggers.
