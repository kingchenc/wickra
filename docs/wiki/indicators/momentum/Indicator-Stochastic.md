# Stochastic

> The fast Stochastic Oscillator — `%K` measures where the current close
> sits inside the high/low range of the last `k_period` bars, and `%D` is
> a short SMA on top of `%K`.

Wickra ships a single **fast** variant (`%K` is the raw oscillator value,
`%D` is its SMA). The "slow stochastic" wraps an additional SMA on `%K`;
that variant is not built in — if you need it, smooth `%K` yourself via
a `Chain` with `Sma::new(slow_period)`.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Momentum |
| Sub-category | bounded oscillator |
| Input type | `Candle` |
| Output type | `StochasticOutput { k, d }` |
| Output range | `k, d ∈ [0, 100]` |
| Default parameters | `k_period = 14`, `d_period = 3` (`Stochastic::classic()`) |
| Warmup period | `k_period + d_period − 1` (16 for the classic configuration) |
| Interpretation | overbought above 80, oversold below 20; %K / %D crossovers |

## Formula

For each new candle at time `t`, let `HH` and `LL` be the highest high
and lowest low over the last `k_period` candles:

```
HH_t = max(high_{t-k_period+1}, …, high_t)
LL_t = min(low_{t-k_period+1},  …, low_t)

%K_t =  100 · (close_t − LL_t) / (HH_t − LL_t)        when HH ≠ LL
%K_t =  50                                            when HH == LL (flat range)

%D_t =  SMA_{d_period}(%K)_t
```

The implementation maintains `HH` and `LL` with two monotonic deques so
each update is amortized O(1).

## Parameters

| Name | Type | Default (Python) | Valid range | Description |
|------|------|------------------|-------------|-------------|
| `k_period` | `usize` | `14` | `>= 1` | Lookback window for the `%K` extrema. |
| `d_period` | `usize` | `3` | `>= 1` | SMA period for `%D` over the `%K` stream. |

Either period being zero returns `Error::PeriodZero`.

## Inputs / Outputs

From `impl Indicator for Stochastic`:

```rust
type Input  = Candle;
type Output = StochasticOutput;
fn update(&mut self, candle: Candle) -> Option<StochasticOutput>;
```

`StochasticOutput`:

| Field | Description |
|-------|-------------|
| `k` | Raw `%K` (where `close` sits inside the window's H–L range). |
| `d` | `SMA(d_period)` of the `%K` series — the slower "signal" line. |

Python's `Stochastic.batch(high, low, close)` returns a `(n, 2)` array
with columns `[k, d]`; warmup rows are `[NaN, NaN]`.

Node's `Stochastic.batch(high, low, close)` returns a flat `number[]`
of length `n * 2`, interleaved as `[k_0, d_0, k_1, d_1, …]`. There is
no streaming `update()` on the Node binding — only `batch` is exposed.

## Warmup

`warmup_period()` returns `k_period + d_period − 1`. The `%K` series itself
becomes available at input `k_period`; the `%D` SMA then needs `d_period`
of those `%K` values to seed, producing its first output at input
`k_period + d_period − 1`. For the classic `(14, 3)` configuration this is
`16` — verified above.

## Edge cases

- **Flat range (`HH == LL`).** The implementation returns `%K = 50` by
  convention (mirroring RSI's flat-input behaviour). The unit test
  `flat_range_yields_k_50` pins this; with a constant input both `%K` and
  `%D` collapse to `50`.
- **Close at the window high.** `%K = 100` exactly; close at the window
  low gives `%K = 0` exactly (tests `close_at_high_yields_k_100` and
  `close_at_low_yields_k_0`).
- **Reset.** `reset()` clears the candle buffer, both monotonic deques,
  the SMA, and `last_k` — the indicator returns to a freshly-constructed
  state.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, Stochastic};

let candles: Vec<Candle> = (0..20)
    .map(|i| {
        let m = 10.0 + (i as f64 * 0.5).sin() * 2.0;
        Candle::new(m, m + 1.0, m - 1.0, m, 1.0, 0).unwrap()
    })
    .collect();
let mut s = Stochastic::new(14, 3)?;
let out = s.batch(&candles);
let v = out[15].unwrap();
println!("row 15  k={} d={}", v.k, v.d);
let v = out[19].unwrap();
println!("row 19  k={} d={}", v.k, v.d);
# Ok::<(), wickra::Error>(())
```

Verified output:

```
row 15  k=81.19360374383255 d=69.94559370965067
row 19  k=47.26766986190959 d=62.55762656278284
```

### Python

```python
import numpy as np
import wickra as ta

n = 20
i = np.arange(n, dtype=float)
m = 10.0 + np.sin(i * 0.5) * 2.0
high = m + 1.0
low  = m - 1.0
close = m
stoch = ta.Stochastic(14, 3)
out = stoch.batch(high, low, close)
print('shape :', out.shape)
print('warmup:', stoch.warmup_period())
print('row 15:', out[15])
print('row 19:', out[19])
```

Verified output:

```
shape : (20, 2)
warmup: 16
row 15: [81.19360374 69.94559371]
row 19: [47.26766986 62.55762656]
```

### Node

```javascript
const wickra = require('wickra');

const n = 20;
const high = [], low = [], close = [];
for (let i = 0; i < n; i++) {
  const m = 10.0 + Math.sin(i * 0.5) * 2.0;
  high.push(m + 1.0);
  low.push(m - 1.0);
  close.push(m);
}
const s = new wickra.Stochastic(14, 3);
const out = s.batch(high, low, close);
console.log('len    :', out.length);
console.log('row 15 :', { k: out[15 * 2], d: out[15 * 2 + 1] });
console.log('row 19 :', { k: out[19 * 2], d: out[19 * 2 + 1] });
```

Verified output:

```
len    : 40
row 15 : { k: 81.19360374383255, d: 69.94559370965067 }
row 19 : { k: 47.26766986190959, d: 62.55762656278284 }
```

## Interpretation

- **Overbought / oversold zones.** The canonical Lane thresholds are
  `80` and `20`. Crossings back from outside these bands are typically
  used as reversal-confirmation signals, not entries on their own.
- **`%K` / `%D` crossover.** `%K` crossing above `%D` from below is a
  short-horizon bullish signal; the mirror cross is bearish.
- **Divergence.** A price making a new high but `%K` failing to confirm
  is a classic bearish divergence — same logic as RSI divergence but on
  a faster, range-based oscillator.

## Common pitfalls

- **`%K` on a flat candle window is `50`, not undefined.** During a
  quiet drift where `HH == LL`, the convention used here is `50.0` and
  `%D` therefore also converges to `50.0`. Do not interpret a sequence
  of `50`s as a real oversold/overbought cycle — it is the silent-market
  fallback path.
- **Wickra exposes only the fast variant.** "Slow stochastic" is `%K =
  SMA(raw_%K, slow_k)` with `%D = SMA(%K, d_period)` on top. The
  built-in `Stochastic` skips the first SMA; to reproduce the slow
  variant, drive the raw `%K` (taken from `stoch.update(candle).k`)
  through your own `Sma`.

## References

- George C. Lane, *Investment Educators* seminars and articles
  (late 1950s, popularised through the 1980s) — the original
  formulation of `%K` and `%D` as a fast oscillator.

## See also

- [Indicator: Rsi](Indicator-Rsi.md) — sister bounded oscillator, slower
  and smoother than `%K`.
- [Indicator: WilliamsR](Indicator-WilliamsR.md) — the negated mirror of
  fast `%K`, plotted on `[−100, 0]`.
- [Warmup Periods](../../Warmup-Periods.md) — `k_period + d_period − 1` rule
  in context.
