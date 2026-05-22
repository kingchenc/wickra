# AwesomeOscillator

> Bill Williams' Awesome Oscillator — the difference of two simple moving
> averages computed on the bar's median price `(high + low) / 2`.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum Oscillators |
| Input type | `Candle` |
| Output type | `f64` |
| Output range | unbounded (centred on 0; in price-difference units) |
| Default parameters | `fast = 5`, `slow = 34` (`AwesomeOscillator::classic()`, Python default) |
| Warmup period | `slow_period` (34 for the classic configuration) |
| Interpretation | zero-line cross; "saucer" and "twin-peaks" Bill Williams patterns |

## Formula

For each new candle, compute the median price:

```
median_t  =  (high_t + low_t) / 2
```

Then AO is the difference of two SMAs of that series:

```
AO_t  =  SMA_fast(median)_t  −  SMA_slow(median)_t
```

There is no smoothing on top — the output is in the same units as the
input prices (a number, not a percent).

## Parameters

| Name | Type | Default (Python) | Valid range | Description |
|------|------|------------------|-------------|-------------|
| `fast` | `usize` | `5` | `>= 1` and `< slow` | Fast SMA period over median price. |
| `slow` | `usize` | `34` | `>= 1` and `> fast` | Slow SMA period over median price. |

`AwesomeOscillator::new` returns `Error::PeriodZero` if either period is
zero and `Error::InvalidPeriod` if `fast >= slow`.

## Inputs / Outputs

From `impl Indicator for AwesomeOscillator`:

```rust
type Input  = Candle;
type Output = f64;
fn update(&mut self, candle: Candle) -> Option<f64>;
```

The `close` and `volume` fields on the input candle are ignored — only
`high` and `low` matter, via `Candle::median_price()`.

Python's `AwesomeOscillator.batch(high, low)` returns a 1-D `float64`
`np.ndarray`. Node's `AwesomeOscillator.batch(high, low)` returns a
flat `number[]`. Both produce `NaN` during warmup; only Python exposes
a streaming `update(candle)` method.

## Warmup

`warmup_period()` returns `slow_period`. The slow SMA is the slower of
the two SMAs, and because both consume the same median-price stream the
first time both have valid output is exactly the `slow_period`-th input.
For the classic `(5, 34)` configuration this is `34` — verified above.

## Edge cases

- **Constant input.** Both SMAs converge to the constant median price,
  so `AO == 0` (test `constant_series_yields_zero`).
- **Reset.** `reset()` resets both SMAs; the next `slow_period` updates
  return `None`.

## Examples

### Rust

```rust
use wickra::{AwesomeOscillator, BatchExt, Candle, Indicator};

let candles: Vec<Candle> = (0..40)
    .map(|i| {
        let m = 100.0 + i as f64;
        Candle::new(m, m + 1.0, m - 1.0, m, 1.0, 0).unwrap()
    })
    .collect();
let mut ao = AwesomeOscillator::classic();
let out = ao.batch(&candles);
println!("row 33 = {}", out[33].unwrap());
println!("row 39 = {}", out[39].unwrap());
```

Verified output:

```
row 33 = 14.5
row 39 = 14.5
```

(`SMA(5) − SMA(34)` on a unit-slope ramp converges to a constant offset
that depends only on the difference between the two windows' centres,
which is why both rows print the same number.)

### Python

```python
import numpy as np
import wickra as ta

n = 40
i = np.arange(n, dtype=float)
m = 100.0 + i
high = m + 1.0
low  = m - 1.0
ao = ta.AwesomeOscillator(5, 34)
out = ao.batch(high, low)
print('warmup:', ao.warmup_period())
print('row 33:', out[33])
print('row 39:', out[39])
```

Verified output:

```
warmup: 34
row 33: 14.5
row 39: 14.5
```

### Node

```javascript
const wickra = require('wickra');

const n = 40;
const high = [], low = [];
for (let i = 0; i < n; i++) {
  const m = 100 + i;
  high.push(m + 1);
  low.push(m - 1);
}
const ao = new wickra.AwesomeOscillator(5, 34);
const out = ao.batch(high, low);
console.log('row 33:', out[33]);
console.log('row 39:', out[39]);
```

Verified output:

```
row 33: 14.5
row 39: 14.5
```

## Interpretation

- **Zero-line cross.** AO crossing zero from below is a bullish
  momentum signal — the fast SMA of median price has overtaken the
  slow SMA. The mirror cross is bearish.
- **Saucer.** A short sequence of bars where AO turns from negative to
  positive momentum without crossing zero (two declining-magnitude
  bars on the same side of zero followed by a turn) is Bill Williams'
  "saucer" pattern.
- **Twin peaks.** Two AO peaks on the same side of the zero line, with
  the second peak lower (or shallower) than the first while price
  pushes further, is Williams' divergence-style "twin peaks" pattern.

## Common pitfalls

- **Median-price input, not close.** AO ignores `close` entirely. If
  your data source reports an "average" price or only closes, you must
  reconstruct `high` and `low` or pick a different oscillator (e.g.
  MACD on closes).
- **Output magnitude depends on the asset.** Because AO is in raw
  price units, an AO of `14.5` on a price ramp through `100..140`
  means something completely different than `14.5` on a price stream
  near `0.00012`. Always interpret AO relative to a per-asset baseline
  or normalise by ATR.

## References

- Bill Williams, *Trading Chaos: Applying Expert Techniques to
  Maximize Your Profits*, Wiley, 1995 — introduces the Awesome
  Oscillator alongside the rest of the Profitunity tool set.

## See also

- [Indicator: MacdIndicator](../trend-directional/Indicator-MacdIndicator.md) — sister
  oscillator on closes (with an extra signal line on top).
- [Indicator: Trix](../trend-directional/Indicator-Trix.md) — momentum oscillator on a
  triple-smoothed series.
- [Warmup Periods](../../Warmup-Periods.md) — bare `slow_period`.
