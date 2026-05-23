# Bollinger Bands

> An SMA centerline wrapped in symmetric standard-deviation envelopes; the
> classical reading is that price persistently outside a band signals a
> volatility-driven trend, not a reversal.

## Quick reference

| Item                | Value                                                                          |
|---------------------|--------------------------------------------------------------------------------|
| Family | Volatility & Bands |
| Input type          | `f64` (typically the close price)                                              |
| Output type         | `BollingerOutput { upper: f64, middle: f64, lower: f64, stddev: f64 }`         |
| Output range        | unbounded; `lower ≤ middle ≤ upper`, `stddev ≥ 0`                              |
| Default parameters  | `period = 20`, `multiplier = 2.0`                                              |
| Warmup period       | `period` (20 for defaults)                                                     |
| Interpretation      | width tracks recent volatility; price tags band on momentum                    |

## Formula

Each step uses the trailing window of the last `period` inputs:

```
mean   = (1/n) * Σ x_i
var    = (1/n) * Σ (x_i - mean)^2     (population variance, denominator = n)
stddev = sqrt(var)
upper  = mean + multiplier * stddev
middle = mean
lower  = mean - multiplier * stddev
```

Wickra computes `var` from the streaming sums `Σ x` and `Σ x²` as
`Σx²/n - (Σx/n)²` and clamps to `0.0` to absorb catastrophic cancellation on
near-constant inputs (`crates/wickra-core/src/indicators/bollinger.rs`). On
long-running streams the running `Σ x` and `Σ x²` are reseeded from the live
window every `16 · period` updates — amortised O(1), bounds the cancellation
drift to roughly `16 · period · ULP · max(|x|²)` (sub-picodollar on
real-world price scales).

## Parameters

| Name         | Type    | Default | Constraint           | Source                                                   |
|--------------|---------|---------|----------------------|----------------------------------------------------------|
| `period`     | `usize` | `20`    | `> 0`                | `BollingerBands::new` (`bollinger.rs:43`)                |
| `multiplier` | `f64`   | `2.0`   | finite and `> 0.0`   | `BollingerBands::new` (`bollinger.rs:47`)                |

Python defaults come from `#[pyo3(signature = (period=20, multiplier=2.0))]`
in `bindings/python/src/lib.rs`. Invalid inputs raise `ValueError` in Python
and return `Error::PeriodZero` / `Error::NonPositiveMultiplier` in Rust.

## Inputs / Outputs

Rust signature:

```rust
impl Indicator for BollingerBands {
    type Input  = f64;
    type Output = BollingerOutput;
    fn update(&mut self, input: f64) -> Option<BollingerOutput>;
    fn warmup_period(&self) -> usize { self.period }
}
```

`BollingerOutput` fields: `upper`, `middle`, `lower`, `stddev`.

- **Python streaming** (`update`) returns the 4-tuple `(upper, middle, lower, stddev)`
  or `None` during warmup.
- **Python batch** (`batch`) returns a 2-D `numpy.ndarray` of shape `(n, 4)` with
  columns `[upper, middle, lower, stddev]`; warmup rows are entirely `NaN`.
- **Node streaming** (`update`) returns a `{ upper, middle, lower, stddev }`
  object or `null` during warmup.
- **Node batch** (`batch`) returns a flat `Array<number>` of length `n * 4`
  interleaved per row: `[u0, m0, l0, s0, u1, m1, l1, s1, …]`. Warmup rows
  are four consecutive `NaN`s.

## Warmup

`warmup_period() == period`. The first `period - 1` inputs return `None`; the
`period`-th input emits the first `BollingerOutput`. Verified for `period = 5`:
the first non-`None` value appears on the 5th input (index 4).

## Edge cases

- **Constant input.** With a flat series the population stddev collapses to
  exactly `0.0`, so `upper == middle == lower == mean`. The library guards
  against tiny negative floating-point values from catastrophic cancellation
  by clamping the variance with `.max(0.0)`.
- **Flat range / squeeze.** Real markets never give exactly `0.0`, but very
  low-volatility windows produce visibly narrow bands; the upper and lower
  bands collapse onto the middle band (the "Bollinger squeeze").
- **NaN / infinity input.** The implementation skips non-finite inputs:
  `if !input.is_finite() { return self.current(); }`. The window is not
  advanced and the previous `BollingerOutput` (or `None`) is returned.
- **Multiplier validation.** `multiplier <= 0` or non-finite returns
  `Error::NonPositiveMultiplier`. `period == 0` returns `Error::PeriodZero`.
- **Reset.** `reset()` clears the window and both running sums, returning the
  indicator to a freshly-constructed state.

## Examples

### Rust

```rust
use wickra::{BatchExt, BollingerBands, Indicator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bb = BollingerBands::new(5, 2.0)?;
    let out = bb.batch(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
    for (i, v) in out.into_iter().enumerate() {
        println!("i={i} -> {:?}", v);
    }
    Ok(())
}
```

Output:

```
i=0 -> None
i=1 -> None
i=2 -> None
i=3 -> None
i=4 -> Some(BollingerOutput { upper: 5.759591794226543, middle: 3.8, lower: 1.8404082057734565, stddev: 0.9797958971132716 })
i=5 -> Some(BollingerOutput { upper: 5.379795897113269, middle: 4.4, lower: 3.420204102886732, stddev: 0.48989794855663404 })
i=6 -> Some(BollingerOutput { upper: 7.190890230020663, middle: 5.0, lower: 2.809109769979336, stddev: 1.095445115010332 })
i=7 -> Some(BollingerOutput { upper: 9.577708763999665, middle: 6.0, lower: 2.422291236000335, stddev: 1.7888543819998326 })
```

The first emission at `i=4` uses the window `[2, 4, 4, 4, 5]` with mean
`3.8` and population stddev `sqrt(0.96) ≈ 0.9797959`.

### Python

```python
import numpy as np
import wickra as ta

bb = ta.BollingerBands(5, 2.0)
prices = np.array([2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0], dtype=float)
out = bb.batch(prices)
print("shape:", out.shape)
print("row 4:", out[4])
print("row 7:", out[7])
```

Output:

```
shape: (8, 4)
row 4: [5.75959179 3.8        1.84040821 0.9797959 ]
row 7: [9.57770876 6.         2.42229124 1.78885438]
```

Streaming variant returns a 4-tuple `(upper, middle, lower, stddev)` per
tick or `None` during warmup:

```python
import wickra as ta

bb = ta.BollingerBands(5, 2.0)
for p in [2.0, 4.0, 4.0, 4.0, 5.0, 9.0]:
    print(p, "->", bb.update(p))
```

Output:

```
2.0 -> None
4.0 -> None
4.0 -> None
4.0 -> None
5.0 -> (5.759591794226543, 3.8, 1.8404082057734565, 0.9797958971132716)
9.0 -> (9.078143885933063, 5.2, 1.321856114066938, 1.939071942966531)
```

### Node

```js
const w = require('wickra');

const bb = new w.BollingerBands(5, 2.0);
const flat = bb.batch([2, 4, 4, 4, 5, 5, 7, 9]);
console.log('length:', flat.length);
console.log('row 4 [upper, middle, lower, stddev]:', flat.slice(16, 20));
console.log('row 7 [upper, middle, lower, stddev]:', flat.slice(28, 32));
```

Output:

```
length: 32
row 4 [upper, middle, lower, stddev]: [ 5.759591794226543, 3.8, 1.8404082057734565, 0.9797958971132716 ]
row 7 [upper, middle, lower, stddev]: [ 9.577708763999665, 6, 2.422291236000335, 1.7888543819998326 ]
```

Streaming returns the named object `{ upper, middle, lower, stddev }`:

```js
const w = require('wickra');

const bb = new w.BollingerBands(5, 2.0);
[2, 4, 4, 4, 5].forEach(p => console.log(p, '->', bb.update(p)));
```

Output:

```
2 -> null
4 -> null
4 -> null
4 -> null
5 -> {
  upper: 5.759591794226543,
  middle: 3.8,
  lower: 1.8404082057734565,
  stddev: 0.9797958971132716
}
```

## Interpretation

- **Bandwidth as volatility.** `(upper - lower) / middle` is the Bollinger
  bandwidth; a multi-month low in bandwidth is the classic "squeeze" that
  often precedes an expansion move.
- **Tags vs breakouts.** A single touch of the upper band is not a sell
  signal in Bollinger's own framework; persistent closes outside the band
  ("walking the band") signal trend continuation, not exhaustion.
- **%b position.** `(price - lower) / (upper - lower)` normalises position
  inside the channel and is useful as a feature for cross-asset comparison.

## Common pitfalls

- **Stddev convention.** Wickra uses **population** standard deviation
  (denominator `n`, not `n - 1`). This matches Bollinger's original
  formulation and every reference implementation (TA-Lib, pandas-ta);
  switching to the sample variant would mis-align bands by a factor of
  `sqrt(n / (n - 1))` and break parity with other tools.
- **Partial rows.** In the Python 2-D batch result, do not slice an
  individual column out and use it for analysis without checking for
  `NaN` — every warmup row is `NaN` across all four columns. Filter with
  `mask = ~np.isnan(out[:, 0])` before reading any single column.
- **Flat batch length in Node.** The Node `batch` returns `n * 4` numbers
  interleaved per row, not four parallel arrays. Reshape with
  `Array.from({ length: n }, (_, i) => flat.slice(i * 4, i * 4 + 4))`
  if you want per-row records.

## References

- John Bollinger, *Bollinger on Bollinger Bands*, McGraw-Hill, 2001 (the
  original publication of the indicator dates to the early 1980s).
- Wilder's *New Concepts in Technical Trading Systems* (1978) for the
  surrounding family of volatility envelopes.

## See also

- [Keltner Channels](../volatility-bands/Indicator-Keltner.md) — same envelope shape but band
  width is driven by ATR instead of stddev.
- [Donchian Channels](../volatility-bands/Indicator-Donchian.md) — rolling high/low envelope
  with no smoothing.
- [ATR](../volatility-bands/Indicator-Atr.md) — the volatility scale most commonly used to
  size Bollinger-style stops.
