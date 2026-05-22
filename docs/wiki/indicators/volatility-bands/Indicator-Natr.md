# NATR

> Normalized Average True Range — ATR expressed as a percentage of price, so
> volatility is comparable across instruments.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volatility & Bands |
| Input type | `Candle` (uses `high`, `low`, `close`) |
| Output type | `f64` |
| Output range | `[0, ∞)` (percent) |
| Default parameters | `period = 14` (Python) |
| Warmup period | `period` |
| Interpretation | Average true range as a percent of the close. |

## Formula

```
NATR = 100 · ATR(period) / close
```

[`Atr`](../volatility-bands/Indicator-Atr.md) measures volatility in raw price units — a `2.0`
ATR is large on a $10 stock and tiny on a $5000 index. Dividing by the
current close converts it to a percentage, so a NATR of `2.0` always
means "the average true range is 2 % of price". That makes NATR readings
comparable across a portfolio, and stop or position-size rules expressed
as a NATR multiple behave consistently regardless of price level.

## Parameters

| Name     | Type    | Default       | Valid range | Description |
|----------|---------|---------------|-------------|-------------|
| `period` | `usize` | `14` (Python) | `>= 1`      | Wilder smoothing period of the underlying ATR. `0` errors with `Error::PeriodZero`. |

The Python binding defaults `period` to `14`.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/natr.rs`:

```rust
impl Indicator for Natr {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

`NATR` is a **candle-input** indicator: it reads `high`, `low` and
`close`. In Python the streaming `update` accepts a 6-tuple or a dict; the
batch helper takes `high`, `low`, `close` numpy arrays. Node and WASM
expose `update(high, low, close)` and `batch(high, low, close)`.

## Warmup

`Natr::new(period).warmup_period() == period` — identical to the
underlying `Atr`, which is Wilder-seeded over `period` true ranges.

## Edge cases

- **Flat market.** A market with no range has `ATR = 0`, so `NATR = 0`
  (`flat_market_yields_zero` pins this).
- **Zero close.** NATR is undefined against a `0.0` close; the indicator
  reports `0.0` for that bar.
- **Identity.** NATR equals `100 · ATR / close` bar for bar
  (`natr_is_atr_over_close_as_percent` pins this).
- **Reset.** `natr.reset()` clears the underlying ATR.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, Natr};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut natr = Natr::new(14)?;
    let candles: Vec<Candle> = (0..40)
        .map(|i| {
            let p = 100.0 + f64::from(i);
            Candle::new(p, p + 2.0, p - 2.0, p, 10.0, i64::from(i)).unwrap()
        })
        .collect();
    let out = natr.batch(&candles);
    println!("warmup_period = {}", natr.warmup_period());
    println!("last = {:?}", out.last().unwrap());
    Ok(())
}
```

### Python

```python
import numpy as np
import wickra as ta

natr = ta.NATR(14)
high = np.arange(102.0, 142.0)
low = high - 4.0
close = high - 2.0
print(natr.batch(high, low, close)[-1])
```

### Node

```javascript
const ta = require('wickra');
const natr = new ta.NATR(14);
const high = Array.from({ length: 40 }, (_, i) => 102 + i);
const low = high.map((h) => h - 4);
const close = high.map((h) => h - 2);
console.log(natr.batch(high, low, close).at(-1));
```

## Interpretation

`Natr` is the tool of choice whenever an ATR-based rule must work across
instruments or across long stretches of time where the price level
drifts. A volatility filter like "skip entries when NATR > 5" or a stop
at "entry − 3 × NATR %" stays meaningful on any symbol. Use raw
[`Atr`](../volatility-bands/Indicator-Atr.md) only when you specifically want the answer in
price units (e.g. to place a stop a fixed number of points away).

## Common pitfalls

- **Feeding it scalar prices.** It needs `high`/`low`/`close`.
- **Confusing it with ATR.** NATR is a percentage; an ATR-multiple stop
  and a NATR-multiple stop are different distances.

## References

NATR is the percentage-normalised ATR as implemented by TA-Lib (`NATR`);
the underlying ATR is Wilder's from *New Concepts in Technical Trading
Systems* (1978).

## See also

- [Indicator-Atr.md](../volatility-bands/Indicator-Atr.md) — the price-unit original.
- [Indicator-HistoricalVolatility.md](../volatility-bands/Indicator-HistoricalVolatility.md) —
  return-based annualised volatility.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
