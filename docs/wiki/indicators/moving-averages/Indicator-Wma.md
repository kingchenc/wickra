# WMA

> Weighted Moving Average with linear weights `1, 2, …, period`, so the
> most recent bar carries the most weight.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Moving Averages |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded; tracks the input price scale |
| Default parameters | `period` is required (no default in either binding) |
| Warmup period | `period` |
| Interpretation | Front-weighted trend filter; faster than `Sma`, smoother than `Ema`. |

## Formula

```
weights = [1, 2, ..., n]                              // n = period
W       = n * (n + 1) / 2                             // sum of weights
WMA_t   = (1 / W) * Σ_{i=0}^{n-1} (n - i) * price_{t-i}
        = (1 / W) * (n * price_t + (n-1) * price_{t-1} + ... + 1 * price_{t-n+1})
```

Maintained in O(1) using the identity that, when sliding the window by
one, every retained element's weight drops by exactly one and the
newcomer enters at weight `n`:

```
new_weight_sum = old_weight_sum - old_value_sum + n * new_input
new_value_sum  = old_value_sum  - oldest_value  + new_input
```

This is the bookkeeping in the steady-state branch of `update`; during
warmup the full `Σ weight·value` is computed once when the window first
fills.

## Parameters

| Name     | Type    | Default | Valid range | Description |
|----------|---------|---------|-------------|-------------|
| `period` | `usize` | none    | `>= 1`      | Length of the rolling window. `period = 0` errors with `Error::PeriodZero`. `period = 1` is a pass-through. |

(The Python class `wickra.WMA(period)` does not set a `#[pyo3(signature)]`
default; pass the period explicitly.)

## Inputs / Outputs

From `crates/wickra-core/src/indicators/wma.rs`:

```rust
impl Indicator for Wma {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

Python returns `float | None` from `update` and a `numpy.ndarray`
(`float64`, `NaN` for warmup) from `batch`. Node returns `number | null`
and `Array<number>` (with `NaN` placeholders) respectively.

## Warmup

`Wma::new(period).warmup_period() == period`. Like `Sma`, the first
emission lands on the `period`-th `update()` call: the window needs
exactly `period` values for the weighted sum to be defined. There is no
seeding step beyond filling the window.

## Edge cases

- **Constant series.** For `[c; n]`, every element contributes `c · weight_i`
  and the result is `c · ΣW / ΣW = c`. The proptest
  `proptest_matches_naive` exercises this implicitly across many random
  inputs; the textbook `period = 4` test confirms `WMA(4)` of
  `[1, 2, 3, 4]` is exactly `(1·1 + 2·2 + 3·3 + 4·4) / 10 = 30 / 10 = 3.0`.
- **NaN / infinity inputs.** The first line of `update` is
  `if !input.is_finite() { return self.value(); }`. Non-finite inputs are
  silently dropped — they do not advance warmup, do not corrupt the
  rolling sums, and the previously emitted value (if any) is returned.
- **Reset.** `wma.reset()` clears the window and both rolling sums; the
  next `update` starts a new warmup countdown.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Wma};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut wma = Wma::new(4)?;
    let out: Vec<Option<f64>> = wma.batch(&[1.0, 2.0, 3.0, 4.0]);
    println!("{:?}", out);
    println!("warmup_period = {}", wma.warmup_period());
    Ok(())
}
```

Output:

```
[None, None, None, Some(3.0)]
warmup_period = 4
```

The fourth input emits `(1·1 + 2·2 + 3·3 + 4·4) / (1+2+3+4) = 30 / 10 = 3.0`.
This matches the `known_values_period_4` unit test in
`crates/wickra-core/src/indicators/wma.rs`.

### Python

```python
import numpy as np
import wickra as ta

wma = ta.WMA(4)
print(wma.batch(np.array([1.0, 2.0, 3.0, 4.0])))
print("warmup_period =", wma.warmup_period())
```

Output:

```
[nan nan nan  3.]
warmup_period = 4
```

### Node

```javascript
const ta = require('wickra');
const wma = new ta.WMA(4);
console.log(wma.batch([1, 2, 3, 4]));
console.log('warmupPeriod:', wma.warmupPeriod());
```

Output:

```
[ NaN, NaN, NaN, 3 ]
warmupPeriod: 4
```

## Interpretation

`Wma` sits between `Sma` and `Ema` on the lag/responsiveness spectrum:
because the most recent bar carries weight `n` (vs `1` for the oldest),
direction changes propagate faster than in `Sma`, but the smooth linear
decay produces less of the "exponential tail" overshoot you sometimes
see with `Ema`. The same two crossover signals (price-vs-WMA and
fast-WMA-vs-slow-WMA) apply.

The most important downstream use of `Wma` inside Wickra is `Hma`:
`Hma` is built entirely from three `Wma` instances (see
[Indicator-Hma.md](../moving-averages/Indicator-Hma.md)).

## Common pitfalls

- **Mistaking linear weights for exponential ones.** A `Wma(20)` is *not*
  an `Ema(20)`; the weights decay linearly `(20, 19, 18, …, 1)` rather
  than geometrically, so very old bars still contribute (weight 1) where
  in an EMA they would have decayed to near zero. If you want the
  exponential decay, use `Ema`.
- **Comparing `Wma(period)` to a "WMA" from a different library and
  finding the seed off.** Wickra's `Wma` has no separate seeding step —
  it simply returns `None` until the window is full and then returns the
  exact weighted mean from input `period` onward. Some libraries
  pre-seed with a partial-window value; that is a different convention
  and will produce different first-few-bar values.

## References

The linearly-weighted moving average is older than most named indicators
and has no single canonical citation; TA-Lib's `WMA` is the standard
reference implementation and matches Wickra's output bit-for-bit.

## See also

- [Indicator-Sma.md](../moving-averages/Indicator-Sma.md) — equal weights instead of linear.
- [Indicator-Ema.md](../moving-averages/Indicator-Ema.md) — exponential decay instead of linear.
- [Indicator-Hma.md](../moving-averages/Indicator-Hma.md) — Hull MA, built from three WMAs.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
