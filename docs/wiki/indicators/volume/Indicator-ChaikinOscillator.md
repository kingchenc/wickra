# ChaikinOscillator

> Chaikin Oscillator — the MACD of the Accumulation/Distribution Line:
> a fast EMA of the ADL minus a slow EMA of the ADL.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Volume |
| Input type | `Candle` (uses `high`, `low`, `close`, `volume`) |
| Output type | `f64` |
| Output range | unbounded around zero |
| Default parameters | `fast = 3`, `slow = 10` (Python) |
| Warmup period | `slow` |
| Interpretation | Momentum of accumulation/distribution; zero-line crossings are the signal. |

## Formula

```
ChaikinOsc_t = EMA(ADL, fast)_t − EMA(ADL, slow)_t
```

The [`Adl`](../volume/Indicator-Adl.md) is an unbounded line that drifts with cumulative
volume — useful for its slope but awkward to trade directly. The Chaikin
Oscillator applies the MACD construction to it: difference a fast and a slow
EMA of the ADL to get a zero-centred momentum reading. Positive values mean
short-term accumulation is outrunning the longer trend; negative values mean
distribution leads.

## Parameters

- `fast` — period of the fast EMA on the ADL (classic `3`).
- `slow` — period of the slow EMA on the ADL (classic `10`).

`fast` must be strictly less than `slow`. `ChaikinOscillator::classic()`
returns the `(3, 10)` configuration.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/chaikin_oscillator.rs`:

```rust
impl Indicator for ChaikinOscillator {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

It is a **candle-input** indicator (the ADL inside it needs `high`, `low`,
`close`, `volume`). Python's streaming `update` accepts a 6-tuple or a dict;
the batch helper takes `high`, `low`, `close`, `volume` numpy arrays. Node and
WASM expose `update(high, low, close, volume)` and the matching `batch`.

## Warmup

`ChaikinOscillator::classic().warmup_period() == 10`. The ADL emits a value
from the very first candle, so both EMAs are fed every bar and the slow EMA
gates the first output — the warmup is exactly `slow`.

## Edge cases

- **Flat market.** A flat candle has zero money-flow volume, so the ADL never
  moves and both EMAs of the constant-zero series stay at zero — the
  oscillator sits at `0.0` (`flat_market_yields_zero` pins this).
- **`fast >= slow`.** Rejected at construction with an error.
- **Reset.** `osc.reset()` clears the ADL and both EMAs.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, ChaikinOscillator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut osc = ChaikinOscillator::classic();   // EMA(ADL, 3) − EMA(ADL, 10)
    // A flat market: the ADL never moves, so the oscillator sits at zero.
    let candles: Vec<Candle> = (0..20)
        .map(|i| Candle::new(10.0, 10.0, 10.0, 10.0, 100.0, i).unwrap())
        .collect();
    let out = osc.batch(&candles);
    println!("{:?}", out.last().unwrap());
    Ok(())
}
```

Output:

```
Some(0.0)
```

A flat series produces a flat ADL and therefore a zero oscillator. This
matches the `flat_market_yields_zero` test in
`crates/wickra-core/src/indicators/chaikin_oscillator.rs`.

### Python

```python
import numpy as np
import wickra as ta

osc = ta.ChaikinOscillator(3, 10)
n = 20
flat = np.full(n, 10.0)
print(osc.batch(flat, flat, flat, np.full(n, 100.0))[-1])
```

Output:

```
0.0
```

### Node

```javascript
const ta = require('wickra');
const osc = new ta.ChaikinOscillator(3, 10);
const flat = Array(20).fill(10);
const vol = Array(20).fill(100);
const out = osc.batch(flat, flat, flat, vol);
console.log(out[out.length - 1]);
```

Output:

```
0
```

## Interpretation

Trade the Chaikin Oscillator like any MACD-style line: a cross above zero is a
bullish accumulation signal, a cross below is bearish. Divergence between the
oscillator and price is the higher-conviction setup — for example, price
making a new high while the oscillator does not is the same warning the raw
ADL gives, but packaged as a bounded, zero-centred series.

## Common pitfalls

- **Treating the level as meaningful.** Only the sign and the slope carry
  information; the magnitude scales with the instrument's volume.
- **Feeding it scalar prices.** It needs the full OHLCV bar.

## References

Marc Chaikin's Chaikin Oscillator — the MACD construction applied to his
Accumulation/Distribution Line (StockCharts).

## See also

- [Indicator-Adl.md](../volume/Indicator-Adl.md) — the cumulative line this oscillates.
- [Indicator-ChaikinMoneyFlow.md](../volume/Indicator-ChaikinMoneyFlow.md) — a bounded
  ratio built from the same money-flow volume.
- [Indicator-MacdIndicator.md](../trend-directional/Indicator-MacdIndicator.md) — the
  same fast/slow EMA-difference construction on price.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
