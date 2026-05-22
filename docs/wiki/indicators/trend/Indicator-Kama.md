# KAMA

> Kaufman's Adaptive Moving Average — picks its own smoothing constant
> on every bar from a fast/slow EMA pair, weighted by an efficiency
> ratio that measures how trending the recent price action has been.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Trend |
| Sub-category | Adaptive & hybrid |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded; tracks the input price scale |
| Default parameters | Python: `(er_period=10, fast=2, slow=30)`; Rust: `Kama::classic()` returns the same triple |
| Warmup period (`warmup_period()`) | `er_period + 1` — see below; the *first* emission lands at this index, but on a fresh KAMA that emission equals the seed (the input itself) |
| Interpretation | Fast in trending markets, slow in choppy markets — by construction. |

## Formula

For each new input `price_t` (with `n = er_period`):

```
direction_t  = | price_t - price_{t-n} |
volatility_t = Σ_{i=1}^{n} | price_{t-i+1} - price_{t-i} |
ER_t         = direction_t / volatility_t       // 0 = pure chop, 1 = pure trend; 0 if volatility = 0

fast_sc      = 2 / (fast + 1)                   // fast EMA smoothing constant
slow_sc      = 2 / (slow + 1)                   // slow EMA smoothing constant
SC_t         = (ER_t * (fast_sc - slow_sc) + slow_sc) ^ 2

KAMA_t       = KAMA_{t-1} + SC_t * (price_t - KAMA_{t-1})
```

The squared `SC_t` is Kaufman's choice (he found that squaring widens
the dynamic range between "act like a fast EMA" and "act like a slow
EMA"). On the very first emission `KAMA_{t-1}` is seeded with the
oldest price in the window (`window.front()`), which is the convention
in the source.

## Parameters

| Name        | Type    | Default (Python `KAMA(...)`) | Valid range | Description |
|-------------|---------|-------------------------------|-------------|-------------|
| `er_period` | `usize` | `10` | `>= 1` | Lookback for the efficiency ratio. Larger → smoother ER, slower adaptation. |
| `fast`      | `usize` | `2`  | `>= 1`, strictly `< slow` | Fast EMA period; sets the lower bound on responsiveness. |
| `slow`      | `usize` | `30` | `>= 1`, strictly `> fast` | Slow EMA period; sets the upper bound on smoothness. |

Any of `er_period`, `fast`, `slow` being `0` errors with
`Error::PeriodZero`; `fast >= slow` errors with `Error::InvalidPeriod`.
The Python defaults come from
`#[pyo3(signature = (er_period=10, fast=2, slow=30))]` in
`bindings/python/src/lib.rs`; the Rust convenience constructor
`Kama::classic()` returns the same triple.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/kama.rs`:

```rust
impl Indicator for Kama {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

Python returns `float | None` (streaming) / `numpy.ndarray` (batch,
`NaN` for warmup). Node returns `number | null` (streaming) /
`Array<number>` with `NaN` (batch). `warmup_period()` is exposed in
Rust and Python but **not** on the Node `KAMA` class (consult
`bindings/node/index.d.ts` for the surface).

## Warmup

`Kama::new(er_period, fast, slow).warmup_period() == er_period + 1`.
The "off-by-one" is because the efficiency ratio compares `price_t` to
`price_{t-er_period}` and sums `er_period` consecutive absolute diffs;
that requires `er_period + 1` prices in the window. For
`Kama::classic()` (`er_period = 10`) the first emission lands on input
11, matching the table in [Warmup Periods](../../Warmup-Periods.md).

The implementation uses a `VecDeque` of capacity `er_period + 1`. Once
full, every subsequent `update` pops the front and pushes the new
input — `update` is O(`er_period`) in principle (the volatility sum is
re-computed) but O(1) in `period`/`fast`/`slow` since the EMA-style
recursion has no window.

Note: on the *first* emission, `prev = window.front()` (the oldest
price), and the output is
`prev + SC · (input − prev)`. On a perfectly trending series
(`ER ≈ 1`, `SC ≈ fast_sc² ≈ 0.444`) this means the first KAMA value
is materially below the latest price; on `[1, 2, …, 20]` for instance,
KAMA's first emission at input 11 is `5.444…`, not `11`. See the
example output below.

## Edge cases

- **Constant series.** Feeding `[100.0; n]` produces `Some(100.0)`:
  both `direction` and `volatility` are zero, the source branches
  `if volatility == 0.0 { 0.0 } else { ... }` so `ER = 0`,
  `SC = slow_sc² ≈ 0.00416`, and
  `100 + 0.00416 · (100 − 100) = 100`. The unit test
  `constant_series_yields_constant_kama` pins this with
  `Kama::classic()`.
- **NaN / infinity inputs.** The first line of `update` is
  `if !input.is_finite() { return self.state; }`. Non-finite inputs are
  silently dropped; the window is not advanced, the previously emitted
  value is preserved.
- **Reset.** `kama.reset()` clears both the window and the smoothed
  state. The next `update` starts a fresh `er_period + 1` warmup
  countdown.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, Kama};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut kama = Kama::classic();   // (10, 2, 30)
    let prices: Vec<f64> = (1..=20).map(f64::from).collect();
    let out: Vec<Option<f64>> = kama.batch(&prices);
    println!("warmup_period = {}", kama.warmup_period());
    println!("{:?}", out);
    Ok(())
}
```

Output:

```
warmup_period = 11
[None, None, None, None, None, None, None, None, None, None, Some(5.444444444444443), Some(8.358024691358022), Some(10.421124828532234), Some(12.011736015851241), Some(13.339853342139579), Some(14.522140745633099), Some(15.62341152535172), Some(16.679673069639843), Some(17.710929483133246), Some(18.728294157296247)]
```

`Kama::classic().periods()` returns `(10, 0.6666666666666666, 0.06451612903225806)`
— the second and third numbers are `fast_sc = 2/3` and `slow_sc = 2/31`,
not the integer `fast`/`slow` periods themselves. On the linear ramp
`1, 2, …, 20` the efficiency ratio is `1.0` (every step moves direction
the same as volatility), so `SC = fast_sc² ≈ 0.4444`. The first
emission `5.444…` is `1 + 0.4444 · (11 − 1)` and each subsequent value
follows the same recursion.

### Python

```python
import numpy as np
import wickra as ta

kama = ta.KAMA()    # defaults: er_period=10, fast=2, slow=30
out = kama.batch(np.arange(1.0, 21.0))
print("warmup_period =", kama.warmup_period())
print(out)
```

Output:

```
warmup_period = 11
[        nan         nan         nan         nan         nan         nan
         nan         nan         nan         nan  5.44444444  8.35802469
 10.42112483 12.01173602 13.33985334 14.52214075 15.62341153 16.67967307
 17.71092948 18.72829416]
```

### Node

```javascript
const ta = require('D:/Coding/Wickra/bindings/node');
const kama = new ta.KAMA(10, 2, 30);    // no default constructor; pass the triple
const prices = Array.from({ length: 20 }, (_, i) => i + 1);
console.log(kama.batch(prices));
```

Output:

```
[
                 NaN,                NaN,
                 NaN,                NaN,
                 NaN,                NaN,
                 NaN,                NaN,
                 NaN,                NaN,
   5.444444444444443,  8.358024691358022,
  10.421124828532234, 12.011736015851241,
  13.339853342139579, 14.522140745633099,
   15.62341152535172, 16.679673069639843,
  17.710929483133246, 18.728294157296247
]
```

(The Node `KAMA` class does not expose `warmupPeriod()`; use the Rust
or Python binding if you need that getter from your application.)

## Interpretation

KAMA's defining property is that it **changes its own behaviour with the
market**. In a clean trend the efficiency ratio approaches `1`, `SC`
approaches `fast_sc²`, and KAMA behaves like a fast EMA — it tracks
price closely. In a choppy sideways market the efficiency ratio
collapses toward `0`, `SC` approaches `slow_sc²`, and KAMA effectively
freezes — its line goes nearly flat regardless of how violently price
oscillates around it. This is by design: Kaufman's argument is that you
should not chase noise.

The two usable signals are slope (positive = uptrend; flat = ranging;
negative = downtrend) and price-vs-KAMA crossover. Because KAMA can
sit nearly flat for long stretches in a range, "price crossed KAMA"
generates fewer false signals than the same test against an EMA of
similar period.

Prefer `Kama` over a static EMA/SMA when the market regime varies
materially (trending → ranging → trending). Prefer a fixed `Ema` /
`Hma` when you want a predictable smoothing profile that is independent
of price action.

## Common pitfalls

- **Assuming `warmup_period()` is when the line is "good".** The first
  emission lands at input 11 (for the default `er_period = 10`), but
  the seed `KAMA_{t-1} = window.front()` is the *oldest* price in the
  window, so the very first emitted value is biased toward the
  10-bars-ago price. On a strong trend this means the first 3–5
  emissions are noticeably below (or above, depending on direction) the
  current price. If that matters, drop the first `er_period` post-warmup
  emissions, not just the warmup itself.
- **Tuning `fast` and `slow` independently of `er_period`.** Kaufman's
  derivation assumes `slow >> fast` so that the per-bar SC has room to
  move. Picking, say, `(10, 5, 6)` gives `fast_sc ≈ 0.333` and
  `slow_sc ≈ 0.286`, so SC barely changes regardless of the efficiency
  ratio — KAMA degenerates into "an EMA somewhere around period 6".
  Keep `slow` at least `5×` `fast` if you want the adaptive behaviour
  to actually matter.

## References

Perry J. Kaufman, *Smarter Trading*, McGraw-Hill, 1995 (book-length
introduction); reprinted in Kaufman's *Trading Systems and Methods*
across multiple editions, where the squared-SC choice is justified
empirically.

## See also

- [Indicator-Ema.md](Indicator-Ema.md) — the two endpoints (`fast` and
  `slow`) KAMA interpolates between.
- [Indicator-Hma.md](Indicator-Hma.md) — the other "smart" trend filter in
  Wickra.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
