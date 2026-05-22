# TSI

> True Strength Index — a double-smoothed momentum oscillator that strips
> noise while keeping a clean, zero-centred read on trend pressure.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum |
| Sub-category | Unbounded oscillators |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | roughly `[−100, 100]`, centred on zero |
| Default parameters | `(long = 25, short = 13)` (Python) |
| Warmup period | `long + short` |
| Interpretation | Positive = net upward pressure, negative = net downward. |

## Formula

```
momentum_t = price_t − price_{t−1}
TSI = 100 · EMA_short(EMA_long(momentum)) / EMA_short(EMA_long(|momentum|))
```

The 1-bar momentum and its absolute value are each smoothed twice — first
with an EMA of length `long`, then with an EMA of length `short`. The
ratio of the two double-smoothed series normalises the result: when every
recent move is up, numerator and denominator are equal and TSI saturates
at `+100`; when every move is down, at `−100`.

## Parameters

| Name    | Type    | Default       | Valid range | Description |
|---------|---------|---------------|-------------|-------------|
| `long`  | `usize` | `25` (Python) | `>= 1`      | First (slow) smoothing length. `0` errors with `Error::PeriodZero`. |
| `short` | `usize` | `13` (Python) | `>= 1`      | Second (fast) smoothing length. `0` errors with `Error::PeriodZero`. |

The Python binding defaults the pair to `(25, 13)` via
`#[pyo3(signature = (long=25, short=13))]`. Node and WASM take both
explicitly. The `periods` property returns `(long, short)`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/tsi.rs`:

```rust
impl Indicator for Tsi {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`Tsi::new(long, short).warmup_period() == long + short`. The momentum
series starts on input 2; the SMA-seeded `long` EMA seeds at input
`long + 1`, and the `short` EMA stacked on top seeds `short − 1` inputs
later, so the first non-`None` output lands on input `long + short`.

## Edge cases

- **Pure trend.** A monotone rising series saturates at `+100`, a falling
  one at `−100` — `|momentum|` equals `momentum` (or its negative), so the
  ratio is `±1` (`pure_uptrend_saturates_at_plus_100` /
  `pure_downtrend_saturates_at_minus_100` pin this).
- **Constant series.** Every momentum is `0`; the `0 / 0` is guarded and
  the output is `0.0` (`constant_series_yields_zero` pins this).
- **NaN / infinity inputs.** Non-finite inputs are silently dropped; the
  smoothing chains are not advanced.
- **Reset.** `tsi.reset()` clears the previous price and all four EMAs.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Tsi};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prices: Vec<f64> = (1..=40).map(f64::from).collect();
    let mut tsi = Tsi::new(5, 3)?;
    let out = tsi.batch(&prices);
    println!("warmup_period = {}", tsi.warmup_period());
    println!("last = {:?}", out.last().unwrap());
    Ok(())
}
```

Output:

```
warmup_period = 8
last = Some(100.0)
```

A pure ramp has a constant `+1` momentum, so the double-smoothed ratio is
exactly `1` and TSI saturates at `+100`. This matches the
`pure_uptrend_saturates_at_plus_100` test in
`crates/wickra-core/src/indicators/tsi.rs`.

### Python

```python
import numpy as np
import wickra as ta

tsi = ta.TSI()  # (long=25, short=13)
prices = np.linspace(100.0, 80.0, 60)  # steady downtrend
out = tsi.batch(prices)
print("last =", out[-1])
```

Output:

```
last = -100.0
```

### Node

```javascript
const ta = require('wickra');
const tsi = new ta.TSI(25, 13);
const prices = Array.from({ length: 60 }, (_, i) => 100 + i);
console.log('last:', tsi.batch(prices).at(-1));
```

## Interpretation

`Tsi` is a low-noise momentum oscillator. The standard signals are the
zero-line cross (momentum changing sign), overbought/oversold extremes
near `±25` for the default settings, and a signal-line cross — many
traders overlay an EMA of TSI and trade the crossover. The double
smoothing makes divergences unusually clean compared with raw momentum.

## Common pitfalls

- **Reading it as a `[0, 100]` oscillator.** TSI is centred on zero and
  signed; `+25` is "strong up", not "mid-range".
- **Under-budgeting warmup.** Warmup is `long + short` — for the default
  `(25, 13)` that is 38 bars.

## References

William Blau, "True Strength Index", *Technical Analysis of Stocks &
Commodities* (1991), and *Momentum, Direction, and Divergence* (1995).
The double-EMA-of-momentum definition here follows Blau's original.

## See also

- [Indicator-Mom.md](Indicator-Mom.md) — the raw momentum TSI smooths.
- [Indicator-MacdIndicator.md](Indicator-MacdIndicator.md) — another
  EMA-difference momentum oscillator with a signal line.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
