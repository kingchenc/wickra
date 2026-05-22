# UlcerIndex

> Ulcer Index — Peter Martin's downside-only risk measure: the
> root-mean-square of recent drawdowns.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volatility & Bands |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | `[0, ∞)` (percent) |
| Default parameters | `period = 14` (Python) |
| Warmup period | `2·period − 1` |
| Interpretation | Depth and duration of drawdowns; `0` means no drawdown at all. |

## Formula

```
max_t       = highest price over the trailing `period` bars
drawdown_t  = 100 · (price_t − max_t) / max_t
UlcerIndex  = √( mean( drawdown² over period ) )
```

Standard deviation treats an up-move and a down-move as equally
"volatile". The Ulcer Index measures only the **pain of being underwater**:
for each bar it takes the percentage drop from the trailing high, squares
it, and reports the root-mean-square. A market that only rises has no
drawdown and an Ulcer Index of `0`; the deeper and longer the drawdowns,
the higher the reading. It is the volatility term in the Martin ratio
(Ulcer Performance Index).

## Parameters

| Name     | Type    | Default       | Valid range | Description |
|----------|---------|---------------|-------------|-------------|
| `period` | `usize` | `14` (Python) | `>= 1`      | Look-back for both the trailing high and the RMS window. `0` errors with `Error::PeriodZero`. |

The Python binding defaults `period` to `14`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/ulcer_index.rs`:

```rust
impl Indicator for UlcerIndex {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`UlcerIndex::new(period).warmup_period() == 2·period − 1`. The first
`period` prices fill the trailing-maximum window; the per-bar squared
drawdown then needs another `period − 1` bars to fill the RMS window.

## Edge cases

- **Pure up-trend.** Price never trades below its own running high, so
  every drawdown — and the Ulcer Index — is `0`
  (`pure_uptrend_yields_zero` pins this).
- **Constant series.** A flat series has no drawdown; the output is `0.0`
  (`constant_series_yields_zero` pins this).
- **Non-negative.** The Ulcer Index is an RMS of real numbers and is
  never negative (`output_is_non_negative` pins this).
- **NaN / infinity inputs.** Non-finite inputs are silently dropped.
- **Reset.** `ui.reset()` clears both rolling windows and the sum.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, UlcerIndex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ui = UlcerIndex::new(2)?;
    let out: Vec<Option<f64>> = ui.batch(&[10.0, 8.0, 12.0, 9.0]);
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
[None, None, Some(14.142135623730951), Some(17.67766952966369)]
```

`UlcerIndex(2)` warms up after `3` bars. At bar 3 the squared drawdowns in
the window are `[400, 0]`, so the index is `√(400/2) = √200`. At bar 4
they are `[0, 625]`, giving `√(625/2) = √312.5`. This matches the
`reference_values` test in
`crates/wickra-core/src/indicators/ulcer_index.rs`.

### Python

```python
import numpy as np
import wickra as ta

ui = ta.UlcerIndex(2)
print(ui.batch(np.array([10.0, 8.0, 12.0, 9.0])))
```

Output:

```
[       nan        nan 14.1421356 17.6776695]
```

### Node

```javascript
const ta = require('wickra');
const ui = new ta.UlcerIndex(2);
console.log(ui.batch([10, 8, 12, 9]));
```

Output:

```
[ NaN, NaN, 14.142135623730951, 17.67766952966369 ]
```

## Interpretation

`UlcerIndex` answers "how uncomfortable has holding this been?" — a high
reading means deep or prolonged drawdowns, a low reading means a smooth
ride up. It is most useful for *comparing* instruments or strategies on a
downside-risk basis, and as the denominator of the Ulcer Performance
Index (`(return − risk-free) / UlcerIndex`), a Sharpe-ratio analogue that
penalises only downside volatility.

## Common pitfalls

- **Reading it as two-sided volatility.** The Ulcer Index ignores upside
  entirely — a wildly choppy *up*-trend can still score near `0`. Use
  [`StdDev`](../volatility-bands/Indicator-StdDev.md) for two-sided dispersion.
- **Forgetting the doubled warmup.** Warmup is `2·period − 1`, not
  `period`.

## References

Peter Martin and Byron McCann, *The Investor's Guide to Fidelity Funds*
(1989); the index is also documented at StockCharts. The trailing-high
drawdown RMS here follows that definition.

## See also

- [Indicator-StdDev.md](../volatility-bands/Indicator-StdDev.md) — two-sided dispersion.
- [Indicator-Atr.md](../volatility-bands/Indicator-Atr.md) — per-bar range volatility.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
