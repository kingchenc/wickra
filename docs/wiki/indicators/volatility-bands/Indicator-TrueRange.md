# TrueRange

> True Range — the single-bar volatility measure that ATR is the average
> of, exposed raw.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volatility & Bands |
| Input type | `Candle` (uses `high`, `low`, `close`) |
| Output type | `f64` |
| Output range | `[0, ∞)` (price scale) |
| Default parameters | none (no parameters) |
| Warmup period | `1` |
| Interpretation | Per-bar volatility including overnight gaps. |

## Formula

```
TR = max( high − low, |high − close_prev|, |low − close_prev| )
```

True Range is the greatest of the bar's own range and the two gaps to the
previous close, so it captures volatility that opens *between* bars — an
overnight gap — not only the range printed within a bar. The first bar has no
previous close and falls back to `high − low`. Where [`Atr`](../volatility-bands/Indicator-Atr.md)
is the Wilder-smoothed average of this series, `TrueRange` exposes it raw, one
value per bar.

## Parameters

`TrueRange` takes **no parameters** — `TrueRange::new()` in Rust,
`wickra.TrueRange()` in Python, `new ta.TrueRange()` in Node.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/true_range.rs`:

```rust
impl Indicator for TrueRange {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`TrueRange` is a **candle-input** indicator that reads `high`, `low` and
`close` (the close drives the gap terms). Python's streaming `update` accepts
a 6-tuple or a dict; the batch helper takes `high`, `low`, `close` numpy
arrays. Node and WASM expose `update(high, low, close)` and the matching
`batch`.

## Warmup

`TrueRange::new().warmup_period() == 1`. It emits a value from the very first
candle — that bar simply has no previous close and uses `high − low`.

## Edge cases

- **First bar.** No previous close: `TR = high − low`.
- **Gap.** A bar that opens far from the prior close has a `TR` larger than
  its own `high − low`.
- **Non-negative.** `TR` is always `>= 0`.
- **Reset.** `tr.reset()` drops the previous close; the next bar restarts.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, TrueRange};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tr = TrueRange::new();
    let out = tr.batch(&[
        Candle::new(11.0, 12.0, 8.0, 11.0, 1.0, 0)?,  // no prev close -> 12 - 8
        Candle::new(9.5, 10.0, 9.0, 9.5, 1.0, 1)?,    // prev close 11 -> max(1, 1, 2)
    ]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[Some(4.0), Some(2.0)]
```

### Python

```python
import numpy as np
import wickra as ta

tr = ta.TrueRange()
print(tr.batch(
    np.array([12.0, 10.0]), np.array([8.0, 9.0]), np.array([11.0, 9.5])
))
```

Output:

```
[4. 2.]
```

### Node

```javascript
const ta = require('wickra');
const tr = new ta.TrueRange();
console.log(tr.batch([12, 10], [8, 9], [11, 9.5]));
```

Output:

```
[ 4, 2 ]
```

## Interpretation

Read `TrueRange` as raw per-bar volatility. It spikes on wide-range or gapping
bars and shrinks in quiet stretches. Smoothing it with a moving average gives
[`Atr`](../volatility-bands/Indicator-Atr.md); using it directly is useful for volatility-scaled
position sizing or for spotting single outlier bars an average would hide.

## Common pitfalls

- **Confusing it with `high − low`.** On a gap bar the True Range is larger —
  that is the whole point.
- **Feeding it scalar prices.** It needs the full `high`/`low`/`close` bar.

## References

J. Welles Wilder Jr.'s True Range, from *New Concepts in Technical Trading
Systems* (1978).

## See also

- [Indicator-Atr.md](../volatility-bands/Indicator-Atr.md) — the Wilder-smoothed average of the
  True Range.
- [Indicator-ChaikinVolatility.md](../volatility-bands/Indicator-ChaikinVolatility.md) — a
  rate-of-change volatility measure.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
