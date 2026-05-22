# WilliamsR

> Williams %R — Larry Williams' negated mirror of fast Stochastic %K,
> plotted on `[−100, 0]` instead of `[0, 100]`.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum Oscillators |
| Input type | `Candle` |
| Output type | `f64` |
| Output range | `[−100, 0]` |
| Default parameters | `period = 14` (Python) |
| Warmup period | `period` (14 for `period = 14`) |
| Interpretation | overbought above `−20`, oversold below `−80` |

## Formula

For each new candle, let `HH` and `LL` be the highest high and lowest
low over the last `period` candles:

```
HH_t = max(high_{t-period+1}, …, high_t)
LL_t = min(low_{t-period+1},  …, low_t)

%R_t = −100 · (HH_t − close_t) / (HH_t − LL_t)        when HH ≠ LL
%R_t = −50                                            when HH == LL (flat range)
```

This is the negation of fast Stochastic `%K` measured from the *top* of
the window: when the close sits at the window high, `%R = 0`; when it
sits at the window low, `%R = −100`.

## Parameters

| Name | Type | Default (Python) | Valid range | Description |
|------|------|------------------|-------------|-------------|
| `period` | `usize` | `14` | `>= 1` | Lookback window for the `HH` / `LL` extrema. |

`WilliamsR::new(0)` returns `Error::PeriodZero`.

## Inputs / Outputs

From `impl Indicator for WilliamsR`:

```rust
type Input  = Candle;
type Output = f64;
fn update(&mut self, candle: Candle) -> Option<f64>;
```

Python's `WilliamsR.batch(high, low, close)` returns a 1-D `float64`
`np.ndarray` (warmup → `NaN`). Node's `WilliamsR.batch(high, low, close)`
returns a flat `number[]` (warmup → `NaN`); only `batch` is exposed on
the Node binding.

## Warmup

`warmup_period()` returns `period`. Williams %R works on a rolling
range, not a rolling diff, so once `period` candles have arrived the
indicator is ready — there is no off-by-one. The first `period − 1`
calls to `update()` return `None`; the `period`-th call returns the
first `Some(value)`.

## Edge cases

- **Close at the window high.** `%R == 0` exactly. The unit test
  `close_at_high_yields_zero` pins this case (with H, L = 8, 10, 12 and
  closes ending at 12, the result is `0`). Note that floating-point
  zero can print as `-0` when scaled by `-100`; both compare equal to
  `0`.
- **Close at the window low.** `%R == −100` exactly (test
  `close_at_low_yields_minus_100`).
- **Flat range.** When `HH == LL`, the implementation returns `−50` as
  the neutral convention.
- **Reset.** `reset()` clears the candle buffer; the next `period`
  updates return `None`.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, WilliamsR};

let candles = vec![
    Candle::new(9.0, 10.0, 8.0, 9.0, 1.0, 0).unwrap(),
    Candle::new(10.0, 11.0, 9.0, 10.0, 1.0, 0).unwrap(),
    Candle::new(12.0, 12.0, 10.0, 12.0, 1.0, 0).unwrap(),  // close == HH
];
let mut w = WilliamsR::new(3)?;
let out = w.batch(&candles);
println!("Williams %R(3) at idx 2 = {}", out[2].unwrap());
# Ok::<(), wickra::Error>(())
```

Verified output:

```
Williams %R(3) at idx 2 = -0
```

(`-0.0` is bit-equal to `0.0` in IEEE-754; the negative sign is just a
side effect of multiplying `+0.0` by `-100.0`.)

### Python

```python
import numpy as np
import wickra as ta

high  = np.array([10.0, 11.0, 12.0])
low   = np.array([8.0,  9.0,  10.0])
close = np.array([9.0,  10.0, 12.0])
w = ta.WilliamsR(3)
out = w.batch(high, low, close)
print('warmup:', w.warmup_period())
print('row 2 :', out[2])
```

Verified output:

```
warmup: 3
row 2 : -0.0
```

### Node

```javascript
const wickra = require('wickra');

const high  = [10.0, 11.0, 12.0];
const low   = [8.0,  9.0,  10.0];
const close = [9.0,  10.0, 12.0];
const w = new wickra.WilliamsR(3);
const out = w.batch(high, low, close);
console.log('row 2:', out[2]);
```

Verified output:

```
row 2: -0
```

## Interpretation

- **Larry Williams' thresholds.** `%R > −20` is overbought; `%R < −80`
  is oversold. Because the scale runs from `−100` (oversold) to `0`
  (overbought), the inequalities feel inverted to anyone used to
  Stochastic — but the *positions* of the bands are identical.
- **Failure swings.** A `%R` value that pokes into overbought, retreats,
  then fails to reach overbought on the next rally is the classic
  Williams "failure swing" — interpreted as bearish exhaustion.
- **Use alongside trend.** %R is a pure range oscillator; in a strong
  trend it can stay pinned at `0` or `−100` for many bars. Pair with
  ADX or a moving-average filter before reading it as a reversal cue.

## Common pitfalls

- **Sign inversion.** Williams %R lives in `[−100, 0]`, not `[0, 100]`.
  Code that assumes "higher value = more bullish" will work; code that
  assumes a positive range will silently mis-classify every value.
- **Mirror of fast %K, not slow.** Williams %R has no built-in
  smoothing; it tracks raw `%K` (with a sign flip and a shift). If you
  need a smoothed version, drive `%R` through your own `Sma` or `Ema`
  via a `Chain`.

## References

- Larry Williams, *How I Made One Million Dollars … Last Year …
  Trading Commodities*, Windsor Books, 1973 — the original %R
  publication.

## See also

- [Indicator: Stochastic](../momentum-oscillators/Indicator-Stochastic.md) — the positive-axis
  sibling; `%R` and `%K` are linked by `%R = %K − 100`.
- [Indicator: Rsi](../momentum-oscillators/Indicator-Rsi.md) — slower bounded oscillator,
  better behaved in trending markets.
- [Warmup Periods](../../Warmup-Periods.md) — bare `period` (no off-by-one).
