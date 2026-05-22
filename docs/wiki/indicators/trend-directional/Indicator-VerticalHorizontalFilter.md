# VerticalHorizontalFilter

> Vertical Horizontal Filter (VHF) — net distance covered divided by total
> distance walked; a trend-versus-range gauge.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Trend & Directional |
| Input type | `f64` (close price) |
| Output type | `f64` |
| Output range | `[0, 1]` |
| Default parameters | `period = 28` (Python) |
| Warmup period | `period + 1` |
| Interpretation | Near `1` = trending, near `0` = choppy. |

## Formula

```
VHF = (highest_close(n) − lowest_close(n)) / Σ|close − close_prev|(n)
```

The numerator is the *net* distance price covered over the window; the
denominator is the *total* distance it walked. Their ratio lives in `[0, 1]`:
a clean trend walks almost only in its net direction, so `VHF` approaches `1`;
a choppy market doubles back constantly, inflating the denominator and pushing
`VHF` toward `0`. It answers the same question as the
[`ChoppinessIndex`](../trend-directional/Indicator-ChoppinessIndex.md) on an inverted scale.

## Parameters

`period` — the lookback window. The Python binding defaults it to `28`; the
Rust and Node constructors require it explicitly.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/vertical_horizontal_filter.rs`:

```rust
impl Indicator for VerticalHorizontalFilter {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

`VerticalHorizontalFilter` is a **scalar** indicator: it consumes one `f64`
close per step. Because `Input = f64` it can sit inside a
[`Chain`](../../Indicator-Chaining.md).

## Warmup

`VerticalHorizontalFilter::new(28).warmup_period() == 29`. The high/low window
fills at `period` closes, but the `period`-th difference needs one extra input
because the first close has nothing to diff against.

## Edge cases

- **Flat series.** A window that walked nowhere has a zero denominator; `VHF`
  is defined as `0`.
- **Pure trend.** A series rising by a fixed step reads `(period − 1) / period`.
- **Choppy series.** An oscillating series reads near `0`.
- **Reset.** `vhf.reset()` clears the close and difference windows.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, VerticalHorizontalFilter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vhf = VerticalHorizontalFilter::new(5)?;
    // Closes 1..6: each diff is 1 (Σ = 5), the 5-close span is 4 -> 4/5.
    let out = vhf.batch(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[None, None, None, None, None, Some(0.8)]
```

### Python

```python
import numpy as np
import wickra as ta

vhf = ta.VerticalHorizontalFilter(5)
print(vhf.batch(np.array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0])))
```

Output:

```
[ nan  nan  nan  nan  nan 0.8]
```

### Node

```javascript
const ta = require('wickra');
const vhf = new ta.VerticalHorizontalFilter(5);
console.log(vhf.batch([1, 2, 3, 4, 5, 6]));
```

Output:

```
[ NaN, NaN, NaN, NaN, NaN, 0.8 ]
```

## Interpretation

Use the VHF as a regime filter: a high, rising VHF says a trend is in force —
favour trend-following entries; a low VHF says price is ranging — favour
mean-reversion. A VHF turning down from a high level is an early hint the
trend is losing its grip.

## Common pitfalls

- **Expecting a direction.** Like the Choppiness Index it is non-directional —
  pair it with a trend indicator.
- **Reading a single bar.** It is a regime gauge; read its level and slope.

## References

Adam White's Vertical Horizontal Filter; the net-over-total formulation here
is the standard one.

## See also

- [Indicator-ChoppinessIndex.md](../trend-directional/Indicator-ChoppinessIndex.md) — the same
  trending-vs-ranging question on an inverted `[0, 100]` scale.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
