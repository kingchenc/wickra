# Donchian Channels

> The unsmoothed price-extreme envelope: highest high and lowest low over a
> rolling window, with the mid-band defined as their average. Breakouts of
> the Donchian channel are the foundation of the Turtle trading rules.

## Quick reference

| Item                | Value                                                              |
|---------------------|--------------------------------------------------------------------|
| Family | Volatility & Bands |
| Input type          | `Candle` (uses `high` and `low`)                                   |
| Output type         | `DonchianOutput { upper: f64, middle: f64, lower: f64 }`           |
| Output range        | unbounded; `lower ≤ middle ≤ upper`                                |
| Default parameters  | `period = 20`                                                      |
| Warmup period       | `period` (20 for defaults)                                         |
| Interpretation      | breakout boundary; channel touches are tradable events             |

## Formula

For a lookback of `period` candles:

```
upper_t  = max( high_t, high_{t-1}, …, high_{t-period+1} )
lower_t  = min(  low_t,  low_{t-1}, …,  low_{t-period+1} )
middle_t = (upper_t + lower_t) / 2
```

`crates/wickra-core/src/indicators/donchian.rs:58-72` computes both
extrema by folding over the in-window candles each tick; this is O(n)
per update in the period size and O(1) in the data length.

## Parameters

| Name     | Type    | Default | Constraint | Source                                  |
|----------|---------|---------|------------|-----------------------------------------|
| `period` | `usize` | `20`    | `> 0`      | `Donchian::new` (`donchian.rs:30`)      |

Python default from `#[pyo3(signature = (period=20))]` in
`bindings/python/src/lib.rs`. `period == 0` returns `Error::PeriodZero`.

## Inputs / Outputs

```rust
impl Indicator for Donchian {
    type Input  = Candle;
    type Output = DonchianOutput;
    fn update(&mut self, candle: Candle) -> Option<DonchianOutput>;
}

pub struct DonchianOutput { pub upper: f64, pub middle: f64, pub lower: f64 }
```

- **Python streaming.** Returns `(upper, middle, lower)` tuple or `None`.
- **Python batch.** `Donchian.batch(high, low)` returns a 2-D
  `np.ndarray` of shape `(n, 3)` with columns `[upper, middle, lower]`;
  warmup rows are `NaN` across all three columns. (`close` is not
  required.)
- **Node streaming.** Not exposed — the Node binding ships only the
  `batch` form for `Donchian`.
- **Node batch.** `donchian.batch(high, low)` returns a flat
  `Array<number>` of length `n * 3` interleaved per row:
  `[u0, m0, l0, u1, m1, l1, …]`.

## Warmup

`warmup_period() == period`. The first `period - 1` candles return
`None`; the `period`-th candle emits the first envelope. Verified for
`period = 3`: the first non-`None` output is at index `2` (the 3rd
candle).

## Edge cases

- **Flat market (HH == LL).** When every candle in the window has
  identical highs and identical lows, `upper == lower` (and therefore
  `middle == upper == lower`). The pinned test
  `flat_market_yields_equal_bands` covers this.
- **Single extreme candle.** A lone wick at the edge of the window sets
  the boundary until it scrolls out. Donchian therefore reacts in
  step-functions, not smoothly — a new all-time high inside the window
  immediately moves the upper band; a single bar later, that high
  remains the boundary unless an even higher print occurs.
- **NaN / infinity.** `Candle::new` rejects non-finite OHLC values
  before they can reach Donchian.
- **Reset.** `reset()` clears the candle window; the configured
  `period` is preserved.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Donchian, Indicator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let candles = vec![
        Candle::new(10.0, 11.0, 9.0,  10.5, 1.0, 0)?,
        Candle::new(10.5, 12.0, 10.0, 11.5, 1.0, 0)?,
        Candle::new(11.5, 13.0, 11.0, 12.5, 1.0, 0)?,
        Candle::new(12.5, 14.0, 12.0, 13.5, 1.0, 0)?,
        Candle::new(13.5, 15.0, 13.0, 14.5, 1.0, 0)?,
    ];
    let mut d = Donchian::new(3)?;
    for (i, v) in d.batch(&candles).into_iter().enumerate() {
        println!("i={i} -> {:?}", v);
    }
    Ok(())
}
```

Output:

```
i=0 -> None
i=1 -> None
i=2 -> Some(DonchianOutput { upper: 13.0, middle: 11.0, lower: 9.0 })
i=3 -> Some(DonchianOutput { upper: 14.0, middle: 12.0, lower: 10.0 })
i=4 -> Some(DonchianOutput { upper: 15.0, middle: 13.0, lower: 11.0 })
```

At `i = 2` the window contains highs `[11, 12, 13]` and lows `[9, 10, 11]`,
so `upper = 13`, `lower = 9`, `middle = 11`.

### Python

```python
import numpy as np
import wickra as ta

d = ta.Donchian(3)
h = np.array([11.0, 12.0, 13.0, 14.0, 15.0])
l = np.array([ 9.0, 10.0, 11.0, 12.0, 13.0])
print(d.batch(h, l))
```

Output:

```
[[nan nan nan]
 [nan nan nan]
 [13. 11.  9.]
 [14. 12. 10.]
 [15. 13. 11.]]
```

### Node

```js
const w = require('wickra');

const d = new w.Donchian(3);
const flat = d.batch(
  [11, 12, 13, 14, 15],
  [ 9, 10, 11, 12, 13],
);
console.log('length:', flat.length);
console.log('row 2 [upper, middle, lower]:', flat.slice(6, 9));
console.log('row 4 [upper, middle, lower]:', flat.slice(12, 15));
```

Output:

```
length: 15
row 2 [upper, middle, lower]: [ 13, 11, 9 ]
row 4 [upper, middle, lower]: [ 15, 13, 11 ]
```

## Interpretation

- **Breakouts.** The original Turtle Trading rules (Dennis / Eckhardt,
  early 1980s) buy on a 20-day Donchian upper-band breach and sell on
  a 10-day lower-band breach. The modern descendant is the "channel
  breakout" family of trend-following systems.
- **Mean reversion.** A small minority of systems take the bands as
  fade levels; this works on range-bound assets and fails dramatically
  in trends — the inverse of breakout systems.
- **Volatility proxy.** Channel width `upper - lower` is a simple
  volatility proxy that requires no smoothing and no parameter tuning
  beyond the lookback length.

## Common pitfalls

- **Stale extreme.** A single shock high from `period` candles ago
  keeps the upper band elevated even when current prices have fallen
  back to normal. Watch for the "channel drop" event when that high
  scrolls out of the window — the upper band will step down sharply
  in a single bar.
- **No close required.** Donchian only uses high/low. Feeding it a
  close-only series (with high = low = close) collapses it into an
  envelope of close extremes, which is a much noisier signal than
  the canonical high/low form. The Python `batch` accepts only
  `(high, low)` for exactly this reason.
- **Flat range collapse.** On a truly flat instrument the channel
  collapses to a line (`upper == middle == lower`); downstream code
  that divides by `upper - lower` (e.g. computing channel position)
  must handle this division-by-zero case explicitly.

## References

- Richard Donchian published the 4-week channel rule in the early
  1960s as part of his broader trend-following work.
- Curtis Faith, *Way of the Turtle*, McGraw-Hill, 2007, documents
  the 20/10-day Donchian variant that defined the Turtle program.

## See also

- [Bollinger Bands](../volatility-bands/Indicator-BollingerBands.md) — envelope shaped by
  stddev rather than rolling extrema.
- [Keltner Channels](../volatility-bands/Indicator-Keltner.md) — envelope shaped by ATR
  around an EMA centerline.
- [PSAR](../trailing-stops/Indicator-Psar.md) — alternative trailing-stop construction
  for breakout systems.
