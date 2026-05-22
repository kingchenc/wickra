# Keltner Channels

> A pure composition of [EMA](../trend/Indicator-Ema.md) on typical price plus
> ATR-scaled envelopes. The middle line is the trend filter, the bands are
> the volatility cone.

## Quick reference

| Item                | Value                                                                              |
|---------------------|------------------------------------------------------------------------------------|
| Family              | Volatility                                                                         |
| Sub-category        | envelope (composed: EMA + ATR)                                                     |
| Input type          | `Candle` (uses `high`, `low`, `close`)                                             |
| Output type         | `KeltnerOutput { upper: f64, middle: f64, lower: f64 }`                            |
| Output range        | unbounded; `lower ≤ middle ≤ upper`                                                |
| Default parameters  | `ema_period = 20`, `atr_period = 10`, `multiplier = 2.0`                           |
| Warmup period       | `max(ema_period, atr_period)` (`20` for defaults) — exact first-emission index     |
| Interpretation      | trend-following envelope; tags signal momentum, not exhaustion                     |

## Formula

```
middle_t = EMA_{ema_period}( typical_price_t )           // tp = (H+L+C)/3
upper_t  = middle_t + multiplier * ATR_{atr_period}_t
lower_t  = middle_t - multiplier * ATR_{atr_period}_t
```

The middle line is an EMA of **typical price**, not of close
(`crates/wickra-core/src/indicators/keltner.rs:62`,
`candle.typical_price()`).

## Parameters

| Name         | Type    | Default | Constraint              | Source                                       |
|--------------|---------|---------|-------------------------|----------------------------------------------|
| `ema_period` | `usize` | `20`    | `> 0`                   | `Keltner::new` (`keltner.rs:33`)             |
| `atr_period` | `usize` | `10`    | `> 0`                   | `Keltner::new` (`keltner.rs:33`)             |
| `multiplier` | `f64`   | `2.0`   | finite and `> 0.0`      | `Keltner::new` (`keltner.rs:34-36`)          |

Python defaults from
`#[pyo3(signature = (ema_period=20, atr_period=10, multiplier=2.0))]` in
`bindings/python/src/lib.rs`. `Keltner::classic()` returns the same
configuration.

## Inputs / Outputs

```rust
impl Indicator for Keltner {
    type Input  = Candle;
    type Output = KeltnerOutput;
    fn update(&mut self, candle: Candle) -> Option<KeltnerOutput>;
}

pub struct KeltnerOutput { pub upper: f64, pub middle: f64, pub lower: f64 }
```

- **Python streaming.** Returns `(upper, middle, lower)` tuple or `None`.
- **Python batch.** `Keltner.batch(high, low, close)` returns a 2-D
  `np.ndarray` of shape `(n, 3)` with columns `[upper, middle, lower]`;
  warmup rows are `NaN` across all three columns.
- **Node streaming.** Returns a `{ upper, middle, lower }` object or
  `null`.
- **Node batch.** `keltner.batch(high, low, close)` returns a flat
  `Array<number>` of length `n * 3` interleaved per row:
  `[u0, m0, l0, u1, m1, l1, …]`.

## Warmup

`warmup_period()` reports `max(ema_period, atr_period)` — for the
default `(20, 10, 2.0)` that is `20` — and that figure is **exact**: the
first non-`None` output lands on candle `warmup_period()` (index
`warmup_period() - 1`).

`Keltner::update` feeds the EMA and ATR sub-indicators *unconditionally*
on every candle, then emits once both are ready. The two sub-indicators
warm up in parallel over the same candle window, so the slower of the
two (`max(ema_period, atr_period)`) governs the first emission. With the
classic `(20, 10, 2.0)` configuration the first valid `KeltnerOutput` is
the 20th candle (index `19`). This is pinned by the
`first_emission_matches_warmup_period` test in `keltner.rs`.

## Edge cases

- **Flat market.** A constant-OHLC series produces `upper == middle == lower`
  because ATR collapses to `0`. The pinned test
  `flat_market_collapses_bands` covers this.
- **Trending market.** When ATR rises, both bands widen symmetrically
  around the EMA centerline.
- **Reset.** `reset()` resets both the underlying EMA and ATR; the
  configured periods/multiplier are preserved.
- **NaN / infinity.** `Candle::new` rejects non-finite OHLC values up
  front; the indicator never receives them.
- **Invalid params.** `ema_period == 0`, `atr_period == 0`, or non-positive
  `multiplier` returns an error from `Keltner::new`.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, Keltner};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let candles = vec![
        Candle::new(10.0, 11.0, 9.0,  10.5, 1.0, 0)?,
        Candle::new(10.5, 12.0, 10.0, 11.5, 1.0, 0)?,
        Candle::new(11.5, 13.0, 11.0, 12.5, 1.0, 0)?,
        Candle::new(12.5, 14.0, 12.0, 13.5, 1.0, 0)?,
        Candle::new(13.5, 15.0, 13.0, 14.5, 1.0, 0)?,
    ];
    let mut k = Keltner::new(3, 3, 2.0)?;
    for (i, v) in k.batch(&candles).into_iter().enumerate() {
        println!("i={i} -> {:?}", v);
    }
    Ok(())
}
```

Output:

```
i=0 -> None
i=1 -> None
i=2 -> Some(KeltnerOutput { upper: 15.166666666666666, middle: 11.166666666666666, lower: 7.166666666666666 })
i=3 -> Some(KeltnerOutput { upper: 16.166666666666664, middle: 12.166666666666666, lower: 8.166666666666666 })
i=4 -> Some(KeltnerOutput { upper: 17.166666666666664, middle: 13.166666666666666, lower: 9.166666666666666 })
```

The first emission is at `i = 2` (the 3rd candle), exactly
`max(ema=3, atr=3) = 3` — the value `warmup_period()` reports. The EMA
and ATR sub-indicators are fed in parallel, so neither delays the
other.

### Python

```python
import numpy as np
import wickra as ta

k = ta.Keltner(3, 3, 2.0)
h = np.array([11.0, 12.0, 13.0, 14.0, 15.0])
l = np.array([ 9.0, 10.0, 11.0, 12.0, 13.0])
c = np.array([10.5, 11.5, 12.5, 13.5, 14.5])
print(k.batch(h, l, c))
```

Output:

```
[[        nan         nan         nan]
 [        nan         nan         nan]
 [        nan         nan         nan]
 [        nan         nan         nan]
 [17.16666667 13.16666667  9.16666667]]
```

### Node

```js
const w = require('wickra');

const k = new w.Keltner(3, 3, 2.0);
const flat = k.batch(
  [11, 12, 13, 14, 15],
  [ 9, 10, 11, 12, 13],
  [10.5, 11.5, 12.5, 13.5, 14.5],
);
console.log('length:', flat.length);
console.log('row 4 [upper, middle, lower]:', flat.slice(12, 15));
```

Output:

```
length: 15
row 4 [upper, middle, lower]: [ 17.166666666666664, 13.166666666666666, 9.166666666666666 ]
```

## Interpretation

- **Trend filter.** Persistent closes above the upper band signal
  trend continuation, much like Bollinger's "walking the band" pattern;
  Keltner is generally tighter than Bollinger on noisy series because
  ATR responds more smoothly than a rolling stddev.
- **Squeeze cross-over.** A common "squeeze" setup compares Bollinger
  bandwidth to Keltner channel width: when Bollinger fits *inside*
  Keltner, a volatility expansion is statistically more likely.
- **Pullback entries.** In a defined uptrend, pullbacks to the middle
  EMA line are a classic continuation entry; the lower band acts as
  the disaster stop.

## Common pitfalls

- **Typical price ≠ close.** The middle EMA runs on
  `(H + L + C) / 3`, not on close. A pre-computed "EMA of close"
  panel will not equal the Keltner middle line and trying to align
  them at floating-point precision will fail.

## References

- Chester W. Keltner, *How to Make Money in Commodities*, 1960. The
  original construction used a 10-day SMA of typical price with an
  envelope sized by the 10-day average range. The modern variant
  (EMA centerline + ATR envelope) is the form Wickra implements.
- Linda Bradford Raschke popularised the EMA + ATR rephrasing in the
  1990s; this is the version most TA libraries ship today.

## See also

- [EMA](../trend/Indicator-Ema.md) — the centerline component.
- [ATR](Indicator-Atr.md) — the envelope width component.
- [Bollinger Bands](Indicator-BollingerBands.md) — envelope using stddev
  rather than ATR; useful side-by-side comparison.
- [Donchian Channels](Indicator-Donchian.md) — envelope using rolling
  extrema with no smoothing.
