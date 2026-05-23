# HistoricalVolatility

> Historical Volatility — the annualised standard deviation of log returns,
> the realised volatility used to price options and size risk.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volatility & Bands |
| Input type | `f64` (single close) |
| Output type | `f64` |
| Output range | `[0, ∞)` (annualised percent) |
| Default parameters | `(period = 20, trading_periods = 252)` (Python) |
| Warmup period | `period + 1` |
| Interpretation | Annualised volatility of returns, in percent. |

## Formula

```
r_t = ln(price_t / price_{t−1})
HV  = stddev_sample(r over period) · √trading_periods · 100
```

The log returns over the window are measured with the **sample** standard
deviation (divisor `n − 1`, Bessel's correction — the unbiased volatility
estimator), then annualised by `√trading_periods` and expressed as a
percentage. `trading_periods` is the number of bars in a year for the
data's frequency: `252` for daily bars, `52` for weekly, `12` for
monthly.

## Parameters

| Name              | Type    | Default        | Valid range | Description |
|-------------------|---------|----------------|-------------|-------------|
| `period`          | `usize` | `20` (Python)  | `>= 2`      | Number of log returns in the window. `0` errors with `Error::PeriodZero`; `1` with `Error::InvalidPeriod` (the sample stddev needs two returns). |
| `trading_periods` | `usize` | `252` (Python) | `>= 1`      | Annualisation factor. `0` errors with `Error::PeriodZero`. |

The Python binding defaults the pair to `(20, 252)`. The `periods`
property returns `(period, trading_periods)`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/historical_volatility.rs`:

```rust
impl Indicator for HistoricalVolatility {
    type Input = f64;
    type Output = f64;
    // update(&mut self, input: f64) -> Option<f64>
}
```

A single `f64` close in, an `Option<f64>` out. Python maps this to
`float | None` / `numpy.ndarray` (NaN warmup); Node to `number | null` /
`Array<number>` (NaN warmup).

## Warmup

`warmup_period() == period + 1`. The first log return needs a previous
price, and the window must then hold `period` returns — so the first
non-`None` output lands on input `period + 1`.

## Edge cases

- **Constant series.** A flat price series has all log returns equal to
  `0`, so volatility is `0.0` (`constant_series_yields_zero` pins this).
- **Geometric series.** A constant growth factor produces a *constant*
  log return; its standard deviation — and so HV — is `0`
  (`geometric_series_yields_zero` pins this).
- **Non-positive prices.** A log return is undefined when either price is
  `<= 0`. Such ticks are **skipped**: the previous valid value is returned,
  the indicator's state (previous price, window, sums) is left untouched, and
  the next real positive tick re-anchors against the previous *valid* price.
  Previous releases silently treated bad ticks as a `0.0` log-return, which
  underreported realised volatility on broken data feeds — that behaviour has
  changed.
- **Non-negative.** Volatility is a standard deviation and is never
  negative (`output_is_non_negative` pins this).
- **NaN / infinity inputs.** Non-finite inputs are silently dropped.
- **Reset.** `hv.reset()` clears the previous price, the window and the
  running sums.

## Examples

### Rust

```rust
use wickra::{BatchExt, Indicator, HistoricalVolatility};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 20-bar window, 252 trading days per year.
    let mut hv = HistoricalVolatility::new(20, 252)?;
    let prices: Vec<f64> = (0..40).map(|i| 100.0 * 1.01_f64.powi(i)).collect();
    let out = hv.batch(&prices);
    println!("warmup_period = {}", hv.warmup_period());
    // A perfectly geometric series has constant returns -> zero volatility.
    println!("last = {:?}", out.last().unwrap());
    Ok(())
}
```

Output:

```
warmup_period = 21
last = Some(0.0)
```

### Python

```python
import numpy as np
import wickra as ta

hv = ta.HistoricalVolatility()  # (period=20, trading_periods=252)
prices = np.full(40, 100.0)  # flat series
print(hv.batch(prices)[-1])  # no return variation -> 0
```

Output:

```
0.0
```

### Node

```javascript
const ta = require('wickra');
// 52 trading periods per year for weekly bars.
const hv = new ta.HistoricalVolatility(20, 52);
const prices = Array.from({ length: 60 }, (_, i) => 100 + Math.sin(i * 0.3) * 5);
console.log('warmupPeriod:', hv.warmupPeriod());
```

## Interpretation

`HistoricalVolatility` is the realised-volatility number quoted in
options and risk work — "this stock has been running at 30 % annualised
vol". Compare it against an option's *implied* volatility to judge whether
options are cheap or rich, feed it into position-sizing (smaller size as
HV rises), or track its own trend: volatility clusters, so a rising HV
tends to keep rising.

Always match `trading_periods` to your bar frequency — annualising daily
bars with `252`, weekly with `52`, monthly with `12`. Using the wrong
factor rescales every reading.

## Common pitfalls

- **Mismatched `trading_periods`.** Annualising weekly data with `252`
  inflates HV by `√(252/52) ≈ 2.2×`.
- **Confusing it with `StdDev`.** `StdDev` is the population dispersion of
  *prices*; `HistoricalVolatility` is the sample (`n − 1`) dispersion of
  *log returns*, annualised.

## References

Historical (realised) volatility is the standard `√252`-annualised
standard deviation of log returns; the unbiased `n − 1` estimator is the
conventional choice for volatility estimation.

## See also

- [Indicator-StdDev.md](../volatility-bands/Indicator-StdDev.md) — population dispersion of
  raw prices.
- [Indicator-Natr.md](../volatility-bands/Indicator-Natr.md) — range-based volatility as a
  percentage.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
