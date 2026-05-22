# CMO

> Chande Momentum Oscillator — a bounded `[−100, 100]` momentum gauge from
> the unsmoothed sum of gains versus losses.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum |
| Sub-category | Bounded oscillators (−100 … 100) |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | `[−100, 100]` |
| Default parameters | `period = 14` (Python) |
| Warmup period | `period + 1` |
| Interpretation | `+100` pure gains, `−100` pure losses, `0` balanced. |

## Formula

Over the last `period` price *changes*, sum the gains and the losses
separately:

```
gain_t = max(price_t − price_{t−1}, 0)
loss_t = max(price_{t−1} − price_t, 0)
CMO    = 100 · (Σ gain − Σ loss) / (Σ gain + Σ loss)
```

Unlike RSI — which Wilder-smooths the gain/loss averages — CMO sums them
raw, with equal weight on every change in the window. That makes it
faster and wider-swinging than RSI at the same period.

## Parameters

| Name     | Type    | Default       | Valid range | Description |
|----------|---------|---------------|-------------|-------------|
| `period` | `usize` | `14` (Python) | `>= 1`      | Number of price changes summed. `period = 0` errors with `Error::PeriodZero`. |

The Python binding defaults `period` to `14` via `#[pyo3(signature = (period=14))]`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/cmo.rs`:

```rust
impl Indicator for Cmo {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`Cmo::new(period).warmup_period() == period + 1`. The first price change
needs two inputs, and the gain/loss window must hold `period` changes, so
the first non-`None` output lands on input `period + 1`.

## Edge cases

- **Pure trend.** A window of only gains returns `+100`; only losses,
  `−100` (`pure_uptrend_saturates_at_plus_100` /
  `pure_downtrend_saturates_at_minus_100` pin this).
- **Constant series.** A flat series has no gains and no losses; the
  `0 / 0` is guarded and the output is `0.0`
  (`constant_series_yields_zero` pins this).
- **NaN / infinity inputs.** Non-finite inputs are silently dropped; state
  is left untouched.
- **Reset.** `cmo.reset()` clears the previous price, the gain/loss window
  and both running sums.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Cmo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmo = Cmo::new(3)?;
    let out: Vec<Option<f64>> = cmo.batch(&[10.0, 11.0, 10.0, 12.0]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[None, None, None, Some(50.0)]
```

The three changes are `+1, −1, +2`: `Σ gain = 3`, `Σ loss = 1`, so
`CMO = 100·(3 − 1)/(3 + 1) = 50`. This matches the `reference_value` test
in `crates/wickra-core/src/indicators/cmo.rs`.

### Python

```python
import numpy as np
import wickra as ta

cmo = ta.CMO(3)
print(cmo.batch(np.array([10.0, 11.0, 10.0, 12.0])))
```

Output:

```
[nan nan nan 50.]
```

### Node

```javascript
const ta = require('wickra');
const cmo = new ta.CMO(3);
console.log(cmo.batch([10, 11, 10, 12]));
```

Output:

```
[ NaN, NaN, NaN, 50 ]
```

## Interpretation

`Cmo` is read like other bounded oscillators: readings near `+50` and
above flag overbought conditions, near `−50` and below oversold, and the
zero line marks the gain/loss balance point. Because it is unsmoothed it
reacts a bar or two sooner than RSI but is noisier — pair it with a slower
filter, or use it for divergence rather than raw threshold triggers.

## Common pitfalls

- **Expecting the `[0, 100]` RSI scale.** `Cmo` is centred on zero and
  spans `[−100, 100]`; an RSI of `30` corresponds to a `Cmo` near `−40`.
- **Treating it as a smoothed average.** `Cmo` sums raw changes — it is
  deliberately not Wilder-smoothed.

## References

Tushar Chande, *The New Technical Trader* (1994). The unsmoothed
gain/loss sum here matches the original definition and TA-Lib's `CMO`.

## See also

- [Indicator-Rsi.md](Indicator-Rsi.md) — the Wilder-smoothed relative.
- [Indicator-Mom.md](Indicator-Mom.md) — raw price-difference momentum.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
