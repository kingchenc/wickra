# StochRSI

> Stochastic RSI — the Stochastic Oscillator formula applied to the RSI
> series, sharpening RSI's overbought/oversold turns.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum |
| Sub-category | Bounded oscillators (0 … 100) |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | `[0, 100]` |
| Default parameters | `(rsi_period = 14, stoch_period = 14)` (Python) |
| Warmup period | `rsi_period + stoch_period` |
| Interpretation | Where RSI sits in its own recent range; near `0`/`100` = extremes. |

## Formula

```
RSI_t    = Rsi(rsi_period) of price
StochRSI = 100 · (RSI_t − min(RSI, stoch_period)) / (max(RSI, …) − min(RSI, …))
```

RSI rarely visits its `0`/`100` extremes — it spends most of its life
bunched around the middle. StochRSI re-normalises it: it asks where the
*current* RSI sits within its own high/low range over the last
`stoch_period` bars. The result swings the full `[0, 100]` width far more
often than raw RSI, so reversals are easier to spot.

## Parameters

| Name           | Type    | Default       | Valid range | Description |
|----------------|---------|---------------|-------------|-------------|
| `rsi_period`   | `usize` | `14` (Python) | `>= 1`      | Period of the underlying RSI. `0` errors with `Error::PeriodZero`. |
| `stoch_period` | `usize` | `14` (Python) | `>= 1`      | Lookback for the high/low range of RSI. `0` errors with `Error::PeriodZero`. |

The Python binding defaults the pair to `(14, 14)` via
`#[pyo3(signature = (rsi_period=14, stoch_period=14))]`. Node and WASM
take both explicitly. The `periods` property returns
`(rsi_period, stoch_period)`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/stoch_rsi.rs`:

```rust
impl Indicator for StochRsi {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`StochRsi::new(rsi_period, stoch_period).warmup_period()
== rsi_period + stoch_period`. The inner RSI emits its first value on
input `rsi_period + 1`; the stochastic window then needs `stoch_period`
RSI values, so the first non-`None` output lands on input
`rsi_period + stoch_period`.

## Edge cases

- **Flat RSI window.** When every RSI value in the window is equal — for
  example a constant price (RSI pinned at `50`) or a pure trend (RSI
  pinned at `100`) — the range is zero and StochRSI reports the neutral
  `50.0` (`flat_rsi_window_yields_50` and `pure_uptrend_yields_50` pin
  this).
- **Bounds.** The output is always within `[0, 100]`
  (`output_stays_within_0_100` pins this).
- **NaN / infinity inputs.** Non-finite inputs are silently dropped; the
  RSI and the window are not advanced.
- **Reset.** `stoch_rsi.reset()` clears the inner RSI and the window.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, StochRsi};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sr = StochRsi::new(14, 14)?;
    let prices: Vec<f64> = (1..=60)
        .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 10.0)
        .collect();
    let out = sr.batch(&prices);
    println!("warmup_period = {}", sr.warmup_period());
    println!("ready values: {}", out.iter().flatten().count());
    Ok(())
}
```

Output:

```
warmup_period = 28
ready values: 33
```

The first 27 inputs return `None`; from input 28 onward every output is a
defined `[0, 100]` value.

### Python

```python
import numpy as np
import wickra as ta

sr = ta.StochRSI()  # (rsi_period=14, stoch_period=14)
prices = np.full(40, 100.0)  # constant series
print(sr.batch(prices)[-1])  # flat RSI window -> neutral 50
```

Output:

```
50.0
```

### Node

```javascript
const ta = require('wickra');
const sr = new ta.StochRSI(14, 14);
const prices = Array.from({ length: 60 }, (_, i) => 100 + Math.sin(i * 0.3) * 10);
console.log('warmupPeriod:', sr.warmupPeriod());
```

## Interpretation

`StochRsi` is read like any `[0, 100]` oscillator, but with tighter
thresholds because it saturates so readily: above `80` is overbought,
below `20` oversold, and the `50` line is the midpoint. Because it is two
oscillators deep, it is *fast and noisy* — excellent for spotting
short-term turns, poor as a standalone trend filter. Many traders smooth
it further (an SMA of StochRSI) and trade the crossover.

## Common pitfalls

- **Using it as a trend filter.** `StochRsi` whipsaws; confirm with a
  slower indicator before acting on a raw threshold cross.
- **Forgetting the stacked warmup.** Warmup is `rsi_period + stoch_period`
  — for the default `(14, 14)` that is 28 bars.
- **Expecting raw-RSI values.** `StochRsi` is a *position within range*,
  not RSI itself; the two are not interchangeable.

## References

Tushar Chande and Stanley Kroll, *The New Technical Trader* (1994). The
implementation is the standard Stochastic-of-RSI; the flat-window
convention (`50`) matches this library's [`Stochastic`](Indicator-Stochastic.md).

## See also

- [Indicator-Rsi.md](Indicator-Rsi.md) — the underlying oscillator.
- [Indicator-Stochastic.md](Indicator-Stochastic.md) — the same formula on
  price instead of RSI.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
