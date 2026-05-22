# ChaikinMoneyFlow

> Chaikin Money Flow (CMF) — the ratio of money-flow volume to total
> volume over a rolling window, bounded to `[−1, +1]`.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volume |
| Input type | `Candle` (uses `high`, `low`, `close`, `volume`) |
| Output type | `f64` |
| Output range | `[−1, +1]` |
| Default parameters | `period = 20` (Python) |
| Warmup period | `period` |
| Interpretation | Window accumulation/distribution balance; sign and magnitude both matter. |

## Formula

```
MFM_t = ((close − low) − (high − close)) / (high − low)   (money-flow multiplier, −1..+1)
MFV_t = MFM_t · volume_t                                  (money-flow volume)
CMF_t = Σ(MFV, period) / Σ(volume, period)
```

CMF is the [`Adl`](../volume/Indicator-Adl.md) increment averaged the way RSI averages
gains: rather than a running total, it divides the *summed* money-flow volume
of the last `period` bars by the *summed* volume of those bars. The result is
volume-normalised, so it lives in `[−1, +1]` regardless of how heavily the
instrument trades. A bar with `high == low` carries no positional information
and contributes a money-flow volume of `0`.

## Parameters

`period` — the lookback window. The Python binding defaults it to `20`; the
Rust and Node constructors require it explicitly.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/cmf.rs`:

```rust
impl Indicator for ChaikinMoneyFlow {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`ChaikinMoneyFlow` is a **candle-input** indicator: it reads `high`, `low`,
`close` and `volume`. In Python the streaming `update` accepts a 6-tuple or a
dict; the batch helper takes `high`, `low`, `close`, `volume` numpy arrays.
Node and WASM expose `update(high, low, close, volume)` and the matching
`batch`.

## Warmup

`ChaikinMoneyFlow::new(20).warmup_period() == 20`. The first value lands once
the window holds a full `period` bars — on input index `period − 1`.

## Edge cases

- **Zero-range bar.** A bar with `high == low` contributes `MFV = 0`.
- **Empty-volume window.** If the whole window traded zero volume, the
  `0/0` ratio is defined as `0.0` (`zero_volume_window_yields_zero` pins this).
- **Saturated flow.** Every bar closing on its high gives `MFM = +1`, so CMF
  saturates at `+1` (`closes_at_high_yield_cmf_one` pins this).
- **Reset.** `cmf.reset()` clears the window and both running sums.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, ChaikinMoneyFlow};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmf = ChaikinMoneyFlow::new(2)?;
    let out = cmf.batch(&[
        Candle::new(8.0, 10.0, 8.0, 10.0, 100.0, 0)?,  // close at high -> MFV +100
        Candle::new(10.0, 12.0, 8.0, 10.0, 100.0, 1)?, // close mid-range -> MFV 0
    ]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[None, Some(0.5)]
```

Bar 1 closes at its high (`MFM = +1`, `MFV = +100`); bar 2 closes mid-range
(`MFM = 0`, `MFV = 0`). `CMF(2) = (100 + 0) / (100 + 100) = 0.5`. This matches
the `reference_values` test in `crates/wickra-core/src/indicators/cmf.rs`.

### Python

```python
import numpy as np
import wickra as ta

cmf = ta.ChaikinMoneyFlow(2)
high = np.array([10.0, 12.0])
low = np.array([8.0, 8.0])
close = np.array([10.0, 10.0])
volume = np.array([100.0, 100.0])
print(cmf.batch(high, low, close, volume))
```

Output:

```
[nan 0.5]
```

### Node

```javascript
const ta = require('wickra');
const cmf = new ta.ChaikinMoneyFlow(2);
console.log(cmf.batch([10, 12], [8, 8], [10, 10], [100, 100]));
```

Output:

```
[ NaN, 0.5 ]
```

## Interpretation

CMF reads as a balance: sustained positive values mean closes are clustering
near bar highs on real volume (accumulation), sustained negative values mean
the opposite (distribution). Crosses of the zero line are the textbook signal;
the `±0.05` band is often treated as a neutral zone. Because CMF is
volume-normalised it is comparable across instruments — unlike the raw
[`Adl`](../volume/Indicator-Adl.md), whose level is arbitrary.

## Common pitfalls

- **Confusing it with the ADL.** CMF is a *bounded ratio*; the ADL is an
  *unbounded running total*. They share the money-flow multiplier and nothing
  else.
- **Feeding it scalar prices.** It needs the full OHLCV bar.

## References

Marc Chaikin's Chaikin Money Flow; the money-flow-multiplier formulation here
matches the standard definition (StockCharts).

## See also

- [Indicator-Adl.md](../volume/Indicator-Adl.md) — the cumulative line CMF is built on.
- [Indicator-ChaikinOscillator.md](../volume/Indicator-ChaikinOscillator.md) — the
  EMA-difference oscillator on the ADL.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
