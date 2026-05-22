# VWMA

> Volume-Weighted Moving Average — a rolling mean of closes where each bar
> is weighted by its own traded volume.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Moving Averages |
| Input type | `Candle` (uses `close` and `volume`) |
| Output type | `f64` |
| Output range | unbounded; tracks the input price scale |
| Default parameters | `period` is required (no default in either binding) |
| Warmup period | `period` |
| Interpretation | Trend line that leans toward high-conviction (high-volume) bars. |

## Formula

```
VWMA_t = Σ(close_i · volume_i) / Σ(volume_i)   over the last `period` bars
```

A heavy bar pulls the average toward its close; a thin bar barely moves
it. Both the numerator (`Σ price·volume`) and denominator (`Σ volume`)
are maintained as O(1) rolling sums, so `update` is O(1) regardless of
`period`.

If **every** bar in the window has zero volume the weighted mean is
undefined (`0 / 0`). VWMA then falls back to the plain unweighted mean of
the `period` closes, so the output is always finite and defined.

## Parameters

| Name     | Type    | Default | Valid range | Description |
|----------|---------|---------|-------------|-------------|
| `period` | `usize` | none    | `>= 1`      | Rolling window length in bars. `period = 0` errors with `Error::PeriodZero`. |

There is no Python `#[pyo3(signature = …)]` default for `VWMA`, so
`wickra.VWMA(period)` requires the period explicitly.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/vwma.rs`:

```rust
impl Indicator for Vwma {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`VWMA` is a **candle-input** indicator: it reads `close` and `volume` from
each `Candle`. In Python the streaming `update` accepts a 6-tuple or a
dict; the batch helper takes `close` and `volume` numpy arrays. Node and
WASM expose `update(close, volume)` and `batch(close, volume)`.

## Warmup

`Vwma::new(period).warmup_period() == period`. The first `period − 1`
candles fill the rolling window; the `period`-th `update()` produces the
first weighted mean.

## Edge cases

- **Constant closes.** Closes all equal to `c` give `VWMA = c` regardless
  of the volumes (`Σ c·v / Σ v = c`), and the zero-volume fallback also
  yields `c` (`constant_series_yields_the_constant` pins this).
- **Zero-volume window.** If every bar in the window has `volume = 0`,
  VWMA returns the unweighted mean of the `period` closes
  (`zero_volume_window_falls_back_to_unweighted_mean` pins this).
- **Candle validation.** `Candle::new` already rejects NaN/infinite fields
  and negative volume, so `update` never sees an invalid bar — there is no
  separate non-finite guard.
- **Reset.** `vwma.reset()` clears the window and all three rolling sums.

## Examples

### Rust

```rust
use wickra::{Candle, Indicator, Vwma};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vwma = Vwma::new(2)?;
    // (close, volume): (10, 1) then (20, 3).
    let a = Candle::new(10.0, 10.0, 10.0, 10.0, 1.0, 0)?;
    let b = Candle::new(20.0, 20.0, 20.0, 20.0, 3.0, 1)?;
    println!("{:?}", vwma.update(a));
    println!("{:?}", vwma.update(b));
    Ok(())
}
```

Output:

```
None
Some(17.5)
```

The window holds two bars: `(10·1 + 20·3) / (1 + 3) = 70 / 4 = 17.5`. The
heavier bar at `20` dominates, so the result sits well above the simple
mean of `15`. This matches the `reference_value` test in
`crates/wickra-core/src/indicators/vwma.rs`.

### Python

```python
import numpy as np
import wickra as ta

vwma = ta.VWMA(2)
close = np.array([10.0, 20.0, 30.0])
volume = np.array([1.0, 3.0, 1.0])
print(vwma.batch(close, volume))
print("warmup_period =", vwma.warmup_period())
```

Output:

```
[ nan 17.5 22.5]
warmup_period = 2
```

### Node

```javascript
const ta = require('wickra');
const vwma = new ta.VWMA(2);
console.log(vwma.batch([10, 20, 30], [1, 3, 1]));
console.log('warmupPeriod:', vwma.warmupPeriod());
```

Output:

```
[ NaN, 17.5, 22.5 ]
warmupPeriod: 2
```

## Interpretation

`Vwma` is a trend line that respects participation. Compared with an
equal-weighted `Sma` of the same period, it reacts faster to moves backed
by heavy volume and lags moves on thin volume. The classic read is the
`Vwma`-vs-`Sma` relationship: `Vwma` above `Sma` means recent strength was
volume-backed (more trustworthy); `Vwma` below `Sma` means the up-moves
came on light volume. It is a session-independent cousin of
[`Vwap`](../volume/Indicator-Vwap.md) — VWAP weights by volume since the
start of the stream, VWMA over a fixed rolling window.

## Common pitfalls

- **Feeding it scalar prices.** `VWMA` needs volume; it takes a `Candle`,
  not an `f64`. Use `Sma`/`Wma` for a pure price series.
- **Assuming a zero-volume window is an error.** It is not — VWMA falls
  back to the unweighted mean. If that fallback matters to you, screen the
  window's total volume yourself.

## References

The volume-weighted moving average is a standard volume-weighted rolling
mean; the rolling-sum formulation here matches the common pandas
implementation `(close*volume).rolling(n).sum() / volume.rolling(n).sum()`,
with an explicit zero-volume fallback added for robustness.

## See also

- [Indicator-Sma.md](../moving-averages/Indicator-Sma.md) — the equal-weighted counterpart.
- [Indicator-Vwap.md](../volume/Indicator-Vwap.md) — volume-weighted price
  since the start of the stream.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
