# RSI

> Relative Strength Index — Wilder's bounded momentum oscillator that maps
> the ratio of average gains to average losses onto the `[0, 100]` range.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum Oscillators |
| Input type | `f64` (close) |
| Output type | `f64` |
| Output range | `[0, 100]` |
| Default parameters | `period = 14` (Python) |
| Warmup period | `period + 1` (15 for `period = 14`) |
| Interpretation | overbought above 70, oversold below 30 (Wilder's thresholds) |

## Formula

```
diff_t   = close_t − close_{t-1}
gain_t   = max(diff_t, 0)
loss_t   = max(−diff_t, 0)

Seed (Wilder, at t = period):
  avg_gain_p = (gain_1 + … + gain_p) / p
  avg_loss_p = (loss_1 + … + loss_p) / p

Recursive smoothing (t > period), with α = 1 / period:
  avg_gain_t = (avg_gain_{t-1} · (period − 1) + gain_t) / period
  avg_loss_t = (avg_loss_{t-1} · (period − 1) + loss_t) / period

RS_t  = avg_gain_t / avg_loss_t
RSI_t = 100 − 100 / (1 + RS_t)
```

When `avg_loss_t == 0` and `avg_gain_t > 0`, RSI is `100` directly; when both
are zero (a perfectly flat series) the implementation returns the standard
`50` convention.

## Parameters

| Name | Type | Default (Python) | Valid range | Description |
|------|------|------------------|-------------|-------------|
| `period` | `usize` | `14` | `>= 1` | Wilder smoothing length. `Rsi::new(0)` returns `Error::PeriodZero`. |

## Inputs / Outputs

From `impl Indicator for Rsi` in `crates/wickra-core/src/indicators/rsi.rs`:

```rust
type Input  = f64;
type Output = f64;
fn update(&mut self, input: f64) -> Option<f64>;
```

The output is a scalar in `[0, 100]`. In Python `batch(prices)` returns a
1-D `np.ndarray` of `float64`, with `NaN` in the warmup positions. In Node
`batch(prices)` returns a flat `number[]`, also `NaN` during warmup.

## Warmup

`warmup_period()` returns `period + 1`. The reason is that RSI consumes
*diffs*, not prices: with `period` prices you only have `period − 1` diffs,
so you need exactly one extra price before Wilder's seed average is well
defined. The Rust test `warmup_period_is_period_plus_one` pins this:

```rust
let rsi = Rsi::new(14).unwrap();
assert_eq!(rsi.warmup_period(), 15);
```

In streaming terms, the first `period` calls to `update()` return `None`;
the `(period + 1)`-th call returns the first `Some(value)`.

## Edge cases

- **Flat input.** When every input price is identical, every `gain` and
  every `loss` is zero, so `avg_loss == avg_gain == 0`. The implementation
  returns `50.0` by convention (see `Rsi::rsi_from_avgs`). The unit test
  `flat_series_yields_rsi_50` pins this behaviour.
- **Pure uptrend / pure downtrend.** `avg_loss == 0` with `avg_gain > 0`
  short-circuits to `100`; the mirror case returns `0`. Tests
  `pure_uptrend_yields_rsi_100` and `pure_downtrend_yields_rsi_0` cover
  this.
- **Non-finite input.** `update()` returns the previously emitted value
  (or `None` if no value has been emitted yet) when the input is `NaN` or
  infinite — the internal state is *not* advanced.
- **Reset.** `reset()` returns the indicator to the freshly-constructed
  state: `prev_close`, both seed buffers, both averages, and `last_value`
  are cleared.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Rsi};

let prices = [
    44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42,
    45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.00,
    46.03, 46.41, 46.22, 45.64,
];
let mut rsi = Rsi::new(14)?;
let out = rsi.batch(&prices);
println!("first = {}", out[14].unwrap());
println!("last  = {}", out[19].unwrap());
# Ok::<(), wickra::Error>(())
```

Verified output:

```
first = 70.46413502109705
last  = 57.91502067008556
```

### Python

```python
import numpy as np
import wickra as ta

prices = np.array([
    44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42,
    45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.00,
    46.03, 46.41, 46.22, 45.64,
], dtype=float)
rsi = ta.RSI(14)
v = rsi.batch(prices)
print("warmup:", rsi.warmup_period())
print("first :", float(v[14]))
print("last  :", float(v[-1]))
```

Verified output:

```
warmup: 15
first : 70.46413502109705
last  : 57.91502067008556
```

### Node

```javascript
const wickra = require('wickra');

const rsi = new wickra.RSI(14);
const prices = [
  44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42,
  45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.00,
  46.03, 46.41, 46.22, 45.64,
];
const v = rsi.batch(prices);
console.log('warmup:', rsi.warmupPeriod());
console.log('first :', v[14]);
console.log('last  :', v[19]);
```

Verified output:

```
warmup: 15
first : 70.46413502109705
last  : 57.91502067008556
```

## Interpretation

- **Overbought / oversold zones.** Wilder's classic thresholds are `70`
  (overbought) and `30` (oversold). Many crypto and FX desks tighten them
  to `80 / 20` for trending markets and loosen to `60 / 40` for
  range-bound markets.
- **Midline cross.** A move through `50` is sometimes used as a directional
  signal; above 50 means average gains exceed average losses over the
  smoothing window.
- **Divergence.** A higher price high paired with a lower RSI high (bearish
  divergence) is a classic Wilder signal; the symmetric pattern at lows is
  bullish.

## Common pitfalls

- **RSI on flat input is `50`, not undefined.** The implementation returns
  `50.0` when both averages are zero. Do not interpret this as a neutral
  signal — it is a placeholder that means "the indicator has no opinion
  yet". Pair RSI with a volatility filter (e.g. ATR) if your strategy is
  sensitive to ranging markets.
- **`period + 1` warmup, not `period`.** A common bug is sizing the result
  array against `period` and indexing into the warmup region. The first
  `Some` arrives at the *(period + 1)*-th `update`; in batch form, indices
  `0..period` are `None`/`NaN`. See [Warmup Periods](../../Warmup-Periods.md).
- **Non-finite inputs are absorbed silently.** `update(f64::NAN)` does not
  advance the state and returns the previous value. If you depend on a 1:1
  input-to-output mapping, pre-validate your data before feeding it in.

## References

- J. Welles Wilder, *New Concepts in Technical Trading Systems*, Trend
  Research, 1978. The original publication that defines both RSI and the
  Wilder smoothing scheme used internally.

## See also

- [Indicator: MacdIndicator](../trend-directional/Indicator-MacdIndicator.md) — also momentum,
  but trend-following and unbounded.
- [Indicator: Stochastic](../momentum-oscillators/Indicator-Stochastic.md) — sibling bounded
  oscillator, faster and noisier than RSI.
- [Warmup Periods](../../Warmup-Periods.md) — the canonical `period + 1`
  off-by-one explained.
- [Quickstart: Python](../../Quickstart-Python.md) — full RSI batch / streaming
  walk-through.
