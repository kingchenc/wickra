# PercentB

> Bollinger %b — where price sits within the Bollinger Bands, scaled so
> `0` is the lower band and `1` is the upper band.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volatility |
| Sub-category | Envelopes (derived) |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded (`0` = lower band, `1` = upper band) |
| Default parameters | `(period = 20, multiplier = 2.0)` (Python) |
| Warmup period | `period` |
| Interpretation | Price position in the band; `> 1` / `< 0` = band overshoot. |

## Formula

```
%b = (price − lower) / (upper − lower)
```

where `upper` and `lower` come from
[`BollingerBands`](Indicator-BollingerBands.md). `%b = 1` is price exactly
on the upper band, `%b = 0` on the lower band, `%b = 0.5` on the middle
band. The value is **deliberately not clamped**: a close above the upper
band gives `%b > 1`, a close below the lower band gives `%b < 0` — so %b
shows band overshoots directly.

## Parameters

| Name         | Type    | Default        | Valid range | Description |
|--------------|---------|----------------|-------------|-------------|
| `period`     | `usize` | `20` (Python)  | `>= 1`      | Bollinger Bands period. `0` errors with `Error::PeriodZero`. |
| `multiplier` | `f64`   | `2.0` (Python) | `> 0`       | Band standard-deviation multiplier. `<= 0` errors with `Error::NonPositiveMultiplier`. |

The Python binding defaults the pair to `(20, 2.0)`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/percent_b.rs`:

```rust
impl Indicator for PercentB {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`warmup_period() == period` — identical to the underlying `BollingerBands`.

## Edge cases

- **Constant series.** Flat prices collapse the bands onto the middle;
  with zero band width the price is exactly mid-band and %b is reported
  as `0.5` (`constant_series_yields_midpoint` pins this).
- **Band overshoot.** %b is not clamped — values outside `[0, 1]` are
  expected and meaningful.
- **NaN / infinity inputs.** Passed straight to the underlying
  `BollingerBands`, which drops them.
- **Reset.** `pb.reset()` clears the underlying bands.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, PercentB};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut pb = PercentB::new(5, 2.0)?;
    // A flat series: price is exactly mid-band, so %b is 0.5.
    let out = pb.batch(&[100.0; 20]);
    println!("{:?}", out[10]);
    Ok(())
}
```

Output:

```
Some(0.5)
```

### Python

```python
import numpy as np
import wickra as ta

pb = ta.PercentB(20, 2.0)
prices = np.full(40, 100.0)  # flat series -> mid-band
print(pb.batch(prices)[-1])  # 0.5
```

Output:

```
0.5
```

### Node

```javascript
const ta = require('wickra');
const pb = new ta.PercentB(20, 2.0);
const prices = Array.from({ length: 60 }, (_, i) => 100 + Math.sin(i * 0.3) * 6);
console.log('warmupPeriod:', pb.warmupPeriod());
```

## Interpretation

`PercentB` turns "is price near a band?" into a single number. The
canonical reads: `%b > 1` is a close above the upper band (strong, often
overbought); `%b < 0` is a close below the lower band (weak, often
oversold); `%b` crossing `0.5` is price crossing the middle SMA. Because
it is normalised, %b is the right input when you want to *compare* band
position across instruments, or feed band position into another rule —
for example "buy when %b crosses back above 0 from below".

## Common pitfalls

- **Expecting `[0, 1]` bounds.** %b is intentionally unclamped; values
  outside `[0, 1]` are the band-overshoot signal, not an error.
- **Confusing it with bandwidth.** %b is price *position*;
  [`BollingerBandwidth`](Indicator-BollingerBandwidth.md) is band *width*.

## References

John Bollinger, *Bollinger on Bollinger Bands* (2001). %b is one of
Bollinger's two derived indicators (with bandwidth).

## See also

- [Indicator-BollingerBands.md](Indicator-BollingerBands.md) — the bands
  this locates price within.
- [Indicator-BollingerBandwidth.md](Indicator-BollingerBandwidth.md) — the
  companion derived indicator: band *width*.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
