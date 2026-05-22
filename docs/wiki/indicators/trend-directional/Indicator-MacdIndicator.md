# MacdIndicator

> Moving Average Convergence Divergence — the difference of two EMAs, with
> a third EMA on top as the signal line.

The Rust struct is `MacdIndicator` (since `Macd` would collide with the
output struct on case-insensitive file systems and several existing trait
imports). The Python and Node bindings expose the same engine under the
shorter, conventional name `MACD`.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Trend & Directional |
| Input type | `f64` (close) |
| Output type | `MacdOutput { macd, signal, histogram }` |
| Output range | unbounded (centred on 0) |
| Default parameters | `fast = 12`, `slow = 26`, `signal = 9` (`MacdIndicator::classic()`) |
| Warmup period | `slow + signal − 1` (34 for the classic configuration) |
| Interpretation | crossovers of `macd` and `signal`; zero-line crosses; histogram momentum |

## Formula

```
EMA_n(x) = exponential moving average of x over n periods
           (Wickra's EMA seeds from a simple average of the first n inputs)

macd_t   = EMA_fast(close)_t  −  EMA_slow(close)_t
signal_t = EMA_signal(macd)_t
hist_t   = macd_t − signal_t
```

The signal EMA does not start consuming inputs until `macd_t` becomes
defined (i.e. until both the fast and slow EMAs have seeded), which is
why the overall warmup is `slow + signal − 1` rather than
`max(slow, signal)`.

## Parameters

| Name | Type | Default (Python) | Valid range | Description |
|------|------|------------------|-------------|-------------|
| `fast` | `usize` | `12` | `>= 1` and `< slow` | Fast EMA period. |
| `slow` | `usize` | `26` | `>= 1` and `> fast` | Slow EMA period. |
| `signal` | `usize` | `9` | `>= 1` | EMA period applied to the raw MACD line. |

`MacdIndicator::new` returns `Error::PeriodZero` if any period is zero and
`Error::InvalidPeriod` if `fast >= slow`.

## Inputs / Outputs

From `impl Indicator for MacdIndicator`:

```rust
type Input  = f64;
type Output = MacdOutput;
fn update(&mut self, input: f64) -> Option<MacdOutput>;
```

`MacdOutput` carries three fields:

| Field | Description |
|-------|-------------|
| `macd` | `EMA(fast) − EMA(slow)` of the input series. |
| `signal` | `EMA(signal)` of `macd`. |
| `histogram` | `macd − signal`. |

Python's `MACD.batch(prices)` returns a `(n, 3)` `float64` array with
columns `[macd, signal, histogram]`; warmup rows are entirely `NaN`.

Node's `MACD.batch(prices)` returns a flat `number[]` of length `n * 3`
in the same interleaved order: index `i*3 + 0` is `macd`, `i*3 + 1` is
`signal`, `i*3 + 2` is `histogram`. The streaming `update(value)` returns
a `{ macd, signal, histogram }` object (or `null` during warmup).

## Warmup

`warmup_period()` returns `slow + signal − 1`. The slow EMA seeds at
input `slow`; from that point onward the signal EMA starts receiving
`macd` values, and needs `signal − 1` further inputs to seed itself.
For the classic `(12, 26, 9)` configuration this gives `26 + 9 − 1 = 34`
inputs before the first complete `MacdOutput` is emitted, as pinned by
the unit test `first_emission_matches_warmup_period`.

## Edge cases

- **Constant input.** Both EMAs converge to the constant value, so `macd`
  approaches `0`; with no movement in `macd`, the signal EMA also
  approaches `0`, and so does the histogram. The Rust test
  `constant_series_yields_zero_macd_eventually` pins this.
- **Non-finite input.** `update(NaN)` or `update(±∞)` returns the
  previously emitted `MacdOutput` without advancing any internal EMA.
- **Reset.** `reset()` resets all three EMAs and clears `last`. The next
  `warmup_period()` calls return `None` again.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, MacdIndicator};

let prices: Vec<f64> = (0..40).map(|i| 100.0 + i as f64 * (20.0 / 39.0)).collect();
let mut macd = MacdIndicator::classic();
let out = macd.batch(&prices);
let v = out[33].unwrap();
println!("row 33  macd={} signal={} hist={}", v.macd, v.signal, v.histogram);
let v = out[39].unwrap();
println!("row 39  macd={} signal={} hist={}", v.macd, v.signal, v.histogram);
```

Verified output:

```
row 33  macd=3.589743589743577 signal=3.5897435897435788 hist=-0.0000000000000017763568394002505
row 39  macd=3.589743589743591 signal=3.589743589743585 hist=0.000000000000006217248937900877
```

### Python

```python
import numpy as np
import wickra as ta

prices = np.linspace(100.0, 120.0, 40)
macd = ta.MACD(12, 26, 9)
out = macd.batch(prices)
print('shape :', out.shape)
print('warmup:', macd.warmup_period())
print('row 33:', out[33])
print('row 39:', out[39])
```

Verified output:

```
shape : (40, 3)
warmup: 34
row 33: [ 3.58974359e+00  3.58974359e+00 -1.77635684e-15]
row 39: [3.58974359e+00 3.58974359e+00 6.21724894e-15]
```

### Node

```javascript
const wickra = require('wickra');

const macd = new wickra.MACD(12, 26, 9);
const prices = Array.from({ length: 40 }, (_, i) => 100 + i * 20 / 39);
const flat = macd.batch(prices);
console.log('flat length:', flat.length);
console.log('row 33 macd  :', flat[33 * 3]);
console.log('row 33 signal:', flat[33 * 3 + 1]);
console.log('row 33 hist  :', flat[33 * 3 + 2]);
console.log('row 39 macd  :', flat[39 * 3]);
console.log('row 39 signal:', flat[39 * 3 + 1]);
console.log('row 39 hist  :', flat[39 * 3 + 2]);
```

Verified output:

```
flat length: 120
row 33 macd  : 3.589743589743577
row 33 signal: 3.5897435897435788
row 33 hist  : -1.7763568394002505e-15
row 39 macd  : 3.589743589743591
row 39 signal: 3.589743589743585
row 39 hist  : 6.217248937900877e-15
```

## Interpretation

- **Signal-line crossover.** `macd` crossing above `signal` is the canonical
  bullish signal; the symmetric crossover below is bearish. The
  `histogram` makes this explicit — it crosses zero on the same bar.
- **Zero-line crossover.** `macd` crossing above zero says the fast EMA
  has overtaken the slow EMA; a longer-term trend confirmation, weaker
  than the signal-line cross.
- **Histogram momentum.** Rising histogram bars (even while negative)
  indicate that bearish momentum is fading, and vice versa. Traders use
  this to anticipate signal-line crosses.

## Common pitfalls

- **The signal line lags the MACD line by `signal_period` bars.** A
  crossover signal therefore arrives one full EMA-cycle after the
  underlying momentum turn, which is why MACD is a *confirmation*
  indicator, not a leading one.
- **`fast >= slow` is rejected.** A common bug when reading
  parameters from a config file is swapping the two — the constructor
  returns `Error::InvalidPeriod` rather than silently producing an
  inverted MACD line.
- **Don't slice a single column out of a warmup row.** During the first
  `slow + signal − 1` inputs every field is `NaN` (Python) or absent
  (`None` in Rust / `null` in Node). Filter by checking `macd` for
  finiteness before reading `signal` or `histogram`.

## References

- Gerald Appel, *Technical Analysis: Power Tools for Active Investors*,
  Financial Times Prentice Hall, 2005 — the canonical modern treatment
  of the MACD line/signal-line/histogram trio Appel popularised in the
  late 1970s.

## See also

- [Indicator: Rsi](../momentum-oscillators/Indicator-Rsi.md) — bounded sibling oscillator, useful
  as a confirmation filter on top of MACD signals.
- [Indicator: Trix](../trend-directional/Indicator-Trix.md) — another EMA-based momentum
  oscillator (triple-smoothed rate of change).
- [Warmup Periods](../../Warmup-Periods.md) — table including the `slow +
  signal − 1` rule.
- [Quickstart: Python](../../Quickstart-Python.md) — MACD multi-column NaN
  pattern explained.
