# BollingerBandwidth

> Bollinger Bandwidth — the width of the Bollinger Bands relative to the
> middle band: a normalised volatility reading.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volatility |
| Sub-category | Envelopes (derived) |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | `[0, ∞)` |
| Default parameters | `(period = 20, multiplier = 2.0)` (Python) |
| Warmup period | `period` |
| Interpretation | Band width as a fraction of price; lows flag a "squeeze". |

## Formula

```
Bandwidth = (upper − lower) / middle
```

where `upper`, `middle` and `lower` come from
[`BollingerBands`](Indicator-BollingerBands.md). Since the bands are
`middle ± multiplier · stddev`, the bandwidth simplifies to
`2 · multiplier · stddev / middle` — volatility normalised by price level.
Its extremes name two classic patterns: the **squeeze** (bandwidth at a
multi-month low — a coiled, quiet market that often precedes a sharp
move) and the **bulge** (bandwidth at an extreme high — an exhausted,
over-extended move).

## Parameters

| Name         | Type    | Default        | Valid range | Description |
|--------------|---------|----------------|-------------|-------------|
| `period`     | `usize` | `20` (Python)  | `>= 1`      | Bollinger Bands period. `0` errors with `Error::PeriodZero`. |
| `multiplier` | `f64`   | `2.0` (Python) | `> 0`       | Band standard-deviation multiplier. `<= 0` errors with `Error::NonPositiveMultiplier`. |

The Python binding defaults the pair to `(20, 2.0)`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/bollinger_bandwidth.rs`:

```rust
impl Indicator for BollingerBandwidth {
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

- **Constant series.** Flat prices collapse the bands onto the middle, so
  the width — and bandwidth — is `0.0` (`constant_series_yields_zero`
  pins this).
- **Zero middle band.** Bandwidth is undefined against a `0.0` middle
  band; the indicator reports `0.0` for that bar.
- **Non-negative.** Bandwidth is `(upper − lower) / middle` with
  `upper >= lower` and a positive middle band, so it is never negative
  (`output_is_non_negative` pins this).
- **Reset.** `bbw.reset()` clears the underlying bands.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, BollingerBandwidth};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bbw = BollingerBandwidth::new(20, 2.0)?;
    // A flat stretch then a volatile stretch: bandwidth rises.
    let mut prices: Vec<f64> = vec![100.0; 30];
    prices.extend((0..30).map(|i| 100.0 + (f64::from(i)).sin() * 10.0));
    let out = bbw.batch(&prices);
    println!("flat-window bandwidth: {:?}", out[25]);
    Ok(())
}
```

Output:

```
flat-window bandwidth: Some(0.0)
```

While prices are flat the bands sit on top of each other, so bandwidth is
`0`; once volatility arrives it climbs.

### Python

```python
import numpy as np
import wickra as ta

bbw = ta.BollingerBandwidth(20, 2.0)
prices = np.full(40, 100.0)  # flat series
print(bbw.batch(prices)[-1])  # 0.0
```

Output:

```
0.0
```

### Node

```javascript
const ta = require('wickra');
const bbw = new ta.BollingerBandwidth(20, 2.0);
const prices = Array.from({ length: 60 }, (_, i) => 100 + Math.sin(i * 0.3) * 6);
console.log('warmupPeriod:', bbw.warmupPeriod());
```

## Interpretation

`BollingerBandwidth` is the standard way to quantify the Bollinger
"squeeze". Volatility is mean-reverting and cyclical: extended periods of
low bandwidth tend to be followed by expansion, and vice versa. Traders
watch for bandwidth dropping to a multi-month low (the squeeze) as a
heads-up that a directional move is loading — then take the direction
from price breaking the band, or from a separate trend indicator.

## Common pitfalls

- **Treating the squeeze as directional.** Low bandwidth says a move is
  *coming*, not which way. Confirm direction separately.
- **Comparing raw bandwidth across instruments without context.** It is
  normalised by price, which helps, but "low" is relative to each
  instrument's own history — compare against its own range.

## References

John Bollinger, *Bollinger on Bollinger Bands* (2001). Bandwidth is one
of Bollinger's two derived indicators (with %b).

## See also

- [Indicator-BollingerBands.md](Indicator-BollingerBands.md) — the bands
  this measures.
- [Indicator-PercentB.md](Indicator-PercentB.md) — the companion derived
  indicator: price *position* within the bands.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
