# EMA

> Exponential Moving Average with smoothing factor `α = 2 / (period + 1)`,
> seeded from the SMA of the first `period` inputs (the TA-Lib convention).

## Quick reference

| Field | Value |
|-------|-------|
| Family | Moving Averages |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | unbounded; tracks the input price scale |
| Default parameters | `period` is required; or `Ema::with_alpha(α)` for a custom smoothing factor |
| Warmup period | `period` |
| Interpretation | Smoother, less laggy than `Sma` of the same length. |

## Formula

For `t >= period` (after warmup):

```
α      = 2 / (period + 1)
seed   = (1 / period) * Σ_{i=0}^{period-1} price_i        // SMA of first `period` inputs
EMA_t  = α * price_t + (1 - α) * EMA_{t-1}                // recursive update
```

The first emitted value (at input `period`) is the seed itself, identical
to `Sma::new(period)` on the same prefix. From input `period + 1` onward
the recursive formula takes over. (`Ema::with_alpha(α)` skips the seed and
uses the very first input as the initial state, so `warmup_period() == 1`
in that mode — see the `with_alpha` method for details.)

Wilder's smoothing (used by `Rsi`/`Atr`/`Adx`) uses `α = 1/period`
instead; that is a different smoothing constant and a different
indicator family.

## Parameters

| Name     | Type     | Default | Valid range | Description |
|----------|----------|---------|-------------|-------------|
| `period` | `usize`  | none    | `>= 1`      | Window length used to derive `α`. `period = 0` errors with `Error::PeriodZero`. |
| `α` (alternative constructor `Ema::with_alpha`) | `f64` | none | `(0.0, 1.0]` and finite | Custom smoothing factor; bypasses the period-derived α. Reported `period` is 1, `warmup_period() == 1`. Invalid `α` errors with `Error::InvalidPeriod`. |

(The Python class `wickra.EMA(period)` does not set a `#[pyo3(signature)]`
default; the period must be passed explicitly. `with_alpha` is Rust-only.)

## Inputs / Outputs

From `crates/wickra-core/src/indicators/ema.rs`:

```rust
impl Indicator for Ema {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

Python streams as `float | None`, batches as a 1-D `numpy.ndarray`
(`NaN` for warmup). Node streams as `number | null`, batches as
`Array<number>` with `NaN` placeholders.

## Warmup

`Ema::new(period).warmup_period() == period`. The first non-empty value
is the SMA of the first `period` inputs (the "seed"); from there each
new input contributes `α * input + (1 − α) * previous`. The unit test
`warmup_returns_none_until_seed` and the test
`first_value_equals_sma_seed` pin this contract.

This is the same warmup count as `Sma::new(period)` because the seed
itself is an SMA — `Ema` is "no slower to start emitting than `Sma`, just
more reactive afterwards".

## Edge cases

- **Constant series.** Feeding `[42.0; n]` returns `Some(42.0)` from input
  `period` onward; the seed is `42.0`, and `α · 42 + (1 − α) · 42 = 42`.
  The unit test `constant_series_converges_to_constant` pins this.
- **NaN / infinity inputs.** The first line of `update` is
  `if !input.is_finite() { return self.state; }`. Non-finite inputs are
  silently dropped: they do not advance warmup, do not corrupt the
  state, and the previously emitted value (if any) is returned. The unit
  test `ignores_non_finite_input` pins this.
- **Reset.** `ema.reset()` clears both the smoothed state and the warmup
  buffer; the next `update` starts a new warmup countdown.

## Examples

### Rust

```rust
use wickra::{BatchExt, Ema, Indicator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ema = Ema::new(3)?;
    let out: Vec<Option<f64>> = ema.batch(&[1.0, 2.0, 3.0, 10.0]);
    println!("{:?}", out);
    println!("alpha = {}", ema.alpha());
    Ok(())
}
```

Output:

```
[None, None, Some(2.0), Some(6.0)]
alpha = 0.5
```

`period = 3` gives `α = 2 / 4 = 0.5`. The seed at input 3 is the SMA of
`[1, 2, 3] = 2.0`; the next step is `0.5 · 10 + 0.5 · 2 = 6.0`. This
matches the `step_after_seed_uses_alpha_formula` unit test.

### Python

```python
import wickra as ta

ema = ta.EMA(3)
for x in [1.0, 2.0, 3.0, 10.0]:
    print(x, '->', ema.update(x))
print('alpha:', ema.alpha)
print('warmup_period:', ema.warmup_period())
```

Output:

```
1.0 -> None
2.0 -> None
3.0 -> 2.0
10.0 -> 6.0
alpha: 0.5
warmup_period: 3
```

### Node

```javascript
const ta = require('wickra');
const ema = new ta.EMA(3);
for (const x of [1, 2, 3, 10]) {
  console.log(x, '->', ema.update(x));
}
```

Output:

```
1 -> null
2 -> null
3 -> 2
10 -> 6
```

## Interpretation

`Ema` is the "default" smoothed trend filter for most practitioners. The
two main signals are price-vs-EMA and EMA-fast-vs-EMA-slow crossovers
(the latter is the basis of `MacdIndicator`). Compared with `Sma` at the
same period, `Ema` reacts faster to direction changes at the cost of
slightly noisier output — useful when you care about the inflection
point, not the long-run level.

Prefer `Ema` over `Sma` when you want a single-line trend filter with
moderate lag. Prefer `Dema` / `Tema` when the EMA lag is too much for
your timeframe. Prefer `Hma` when you want lag reduction *and* a built-in
noise filter (a triple WMA chain rather than a triple EMA chain).

## Common pitfalls

- **Confusing `α = 2/(n+1)` with Wilder's `α = 1/n`.** Wickra's `Ema`
  uses the TA-Lib convention `α = 2/(n+1)`. The same numerical period
  passed to `Rsi(14)` or `Atr(14)` uses `α = 1/14 ≈ 0.0714`, not
  `α = 2/15 ≈ 0.1333`. They are different smoothing schemes; comparing
  an EMA(14) line directly to the RSI/ATR's internal smoothing will not
  match. If you want a Wilder-style EMA, build it on top of `Ema` with
  the custom factor: `Ema::with_alpha(1.0 / 14.0)`.
- **Assuming the first emitted EMA is "the EMA".** The first value is
  the SMA seed, not a recursively-smoothed EMA. The series only starts
  behaving like an EMA from input `period + 1` onward. For short series,
  this means the first emission tracks `Sma::new(period)` exactly — that
  is the intended behaviour, not a bug.

## References

The TA-Lib seeding convention used here ("EMA is seeded with an SMA")
is documented in the TA-Lib source and replicated by virtually every
commercial charting platform. The recursive form
`EMA_t = α · price + (1 − α) · EMA_{t-1}` is the standard exponential
smoothing identity attributed to Robert Brown (1956).

## See also

- [Indicator-Sma.md](../moving-averages/Indicator-Sma.md) — equal weights, identical seed.
- [Indicator-Dema.md](../moving-averages/Indicator-Dema.md) — `2·EMA − EMA(EMA)`.
- [Indicator-Tema.md](../moving-averages/Indicator-Tema.md) — `3·EMA − 3·EMA(EMA) + EMA(EMA(EMA))`.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
