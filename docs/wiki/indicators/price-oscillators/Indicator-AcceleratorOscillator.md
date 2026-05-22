# AcceleratorOscillator

> Accelerator Oscillator (AC) — Bill Williams' measure of how fast
> momentum itself is changing.

## Quick reference

| Field | Value |
|-------|-------|
| Family | Price Oscillators |
| Input type | `Candle` (uses `high`, `low`) |
| Output type | `f64` |
| Output range | unbounded around zero |
| Default parameters | `ao_fast = 5`, `ao_slow = 34`, `signal_period = 5` (Python) |
| Warmup period | `ao_slow + signal_period − 1` |
| Interpretation | Acceleration of momentum; zero-line crossings lead the Awesome Oscillator. |

## Formula

```
AO = SMA(median, ao_fast) − SMA(median, ao_slow)   (the Awesome Oscillator)
AC = AO − SMA(AO, signal_period)
```

Where the [`AwesomeOscillator`](../momentum-oscillators/Indicator-AwesomeOscillator.md) measures
momentum, the Accelerator measures the *change* in momentum — it is the AO
minus a short moving average of itself. Because acceleration leads speed, the
`AC` tends to turn before the `AO` does. Bill Williams' classic configuration
is the `(5, 34)` AO with a `5`-period signal average.

## Parameters

- `ao_fast`, `ao_slow` — the underlying Awesome Oscillator periods (`5`, `34`).
- `signal_period` — the moving average of the AO subtracted from it (`5`).

`AcceleratorOscillator::classic()` returns the `(5, 34, 5)` configuration.

## Inputs / Outputs

From `crates/wickra-core/src/indicators/accelerator_oscillator.rs`:

```rust
impl Indicator for AcceleratorOscillator {
    type Input = Candle;
    type Output = f64;
    // update(&mut self, input: Candle) -> Option<f64>
}
```

It is a **candle-input** indicator — the inner Awesome Oscillator reads the
median price `(high + low) / 2`. Python's streaming `update` accepts a 6-tuple
or a dict; the batch helper takes `high`, `low` numpy arrays. Node and WASM
expose `update(high, low)` and the matching `batch`.

## Warmup

`AcceleratorOscillator::classic().warmup_period() == 38`. The AO first emits at
candle `ao_slow`; the signal average then needs `signal_period` AO values.

## Edge cases

- **Flat market.** A flat series gives `AO = 0`, so `AC = 0` throughout.
- **`ao_fast >= ao_slow`.** Rejected at construction.
- **Reset.** `ac.reset()` clears the AO and the signal average.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, AcceleratorOscillator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ac = AcceleratorOscillator::classic();
    let candles: Vec<Candle> = (0..60)
        .map(|i| Candle::new(10.0, 11.0, 9.0, 10.0, 1.0, i).unwrap())
        .collect();
    println!("{:?}", ac.batch(&candles).last().unwrap());
    Ok(())
}
```

Output:

```
Some(0.0)
```

A flat market produces a flat AO and therefore a zero Accelerator.

### Python

```python
import numpy as np
import wickra as ta

ac = ta.AcceleratorOscillator(5, 34, 5)
n = 60
print(ac.batch(np.full(n, 11.0), np.full(n, 9.0))[-1])
```

Output:

```
0.0
```

### Node

```javascript
const ta = require('wickra');
const ac = new ta.AcceleratorOscillator(5, 34, 5);
const out = ac.batch(Array(60).fill(11), Array(60).fill(9));
console.log(out[out.length - 1]);
```

Output:

```
0
```

## Interpretation

Trade the Accelerator like a momentum-acceleration gauge: bars rising above
the zero line mean momentum is building, bars falling below mean it is fading.
Because it leads the Awesome Oscillator, a colour change in the AC is an early
warning that the AO — and price momentum — is about to turn.

## Common pitfalls

- **Reading the level.** Only the sign and the slope matter; the magnitude
  scales with the instrument.
- **Feeding it scalar prices.** It needs the `high`/`low` bar.

## References

Bill Williams' Accelerator Oscillator, from *Trading Chaos*.

## See also

- [Indicator-AwesomeOscillator.md](../momentum-oscillators/Indicator-AwesomeOscillator.md) — the
  momentum oscillator the Accelerator is built on.
- [Indicators-Overview.md](../../Indicators-Overview.md) — the full taxonomy.
