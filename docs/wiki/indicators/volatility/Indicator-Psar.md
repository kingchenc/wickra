# PSAR (Parabolic SAR)

> Wilder's parabolic Stop-And-Reverse: a state-machine trailing stop that
> accelerates toward price as a trend extends and flips sides on a
> penetration of the SAR line.

## Quick reference

| Item                | Value                                                                              |
|---------------------|------------------------------------------------------------------------------------|
| Family              | Volatility                                                                         |
| Sub-category        | trailing-stop (state machine)                                                      |
| Input type          | `Candle` (uses `high`, `low`)                                                      |
| Output type         | `f64`                                                                              |
| Output range        | unbounded; bracketed by the prior two highs/lows                                   |
| Default parameters  | `af_start = 0.02`, `af_step = 0.02`, `af_max = 0.20` (Wilder)                      |
| Warmup period       | `2` (state machine seeds on the 2nd candle)                                        |
| Interpretation      | trailing stop that "flips" sides on penetration; never tied to a fixed bar count   |

## Formula

PSAR is a two-state machine — `Up` (long bias) and `Down` (short bias).
Each bar updates three pieces of state:

```
EP_t  = extreme price reached so far in the current trend (max high in Up,
        min low in Down)
AF_t  = acceleration factor, bumped by af_step each time EP makes a new
        extreme, capped at af_max
SAR_t = stop-and-reverse level
```

The transition is:

```
SAR_t = SAR_{t-1} + AF_{t-1} * (EP_{t-1} - SAR_{t-1})

# Wilder rule: SAR cannot penetrate today's or yesterday's range
if Up:    SAR_t = min(SAR_t, low_{t-1}, low_t)
if Down:  SAR_t = max(SAR_t, high_{t-1}, high_t)

# Reversal test
if Up and low_t <= SAR_t:   flip to Down, SAR_t = EP_{t-1}, reset AF
if Down and high_t >= SAR_t: flip to Up,   SAR_t = EP_{t-1}, reset AF
```

The exact step-by-step is `crates/wickra-core/src/indicators/psar.rs:75-141`.

## Parameters

| Name       | Type  | Default | Constraint                                | Source                                |
|------------|-------|---------|-------------------------------------------|---------------------------------------|
| `af_start` | `f64` | `0.02`  | finite, `> 0`, `≤ af_max`                  | `Psar::new` (`psar.rs:39-50`)         |
| `af_step`  | `f64` | `0.02`  | finite, `> 0`                              | `Psar::new` (`psar.rs:39-50`)         |
| `af_max`   | `f64` | `0.20`  | finite, `> 0`                              | `Psar::new` (`psar.rs:39-50`)         |

Python defaults from
`#[pyo3(signature = (af_start=0.02, af_step=0.02, af_max=0.20))]` in
`bindings/python/src/lib.rs`. `Psar::classic()` returns the same triple.

Validation errors:
- non-finite or non-positive AF parameter → `Error::NonPositiveMultiplier`
- `af_start > af_max` → `Error::InvalidPeriod { message: "af_start must be <= af_max" }`

## Inputs / Outputs

```rust
impl Indicator for Psar {
    type Input  = Candle;
    type Output = f64;
    fn update(&mut self, candle: Candle) -> Option<f64>;
    fn warmup_period(&self) -> usize { 2 }
}
```

- **Python streaming.** Returns `float | None`.
- **Python batch.** `PSAR.batch(high, low, close)` returns a 1-D
  `np.ndarray`; the first row is `NaN` (warmup) and every subsequent
  row holds the SAR level for that bar.
- **Node streaming.** Not exposed in the Node binding.
- **Node batch.** `psar.batch(high, low, close)` returns
  `Array<number>` with `NaN` for the first row.

## Warmup

`warmup_period() == 2`. The very first candle seeds internal state
(`prev_high`, `prev_low`, `sar = low`, `ep = high`, `trend = Up`,
`af = af_start`) and returns `None`. The second candle produces the
first SAR value.

The seed trend is **always** `Up` (`psar.rs:83`); the indicator will
reverse to `Down` on the first qualifying penetration. There is no
look-ahead at the second candle's close — the seed is purely structural.

## Edge cases

- **First bar.** Always returns `None`; downstream code must tolerate
  the first row being absent without crashing.
- **Pure uptrend.** With monotonically rising highs and lows, the SAR
  remains below the lows and accelerates toward price as the EP makes
  successive new highs. The pinned test `pure_uptrend_sar_below_lows`
  asserts `SAR ≤ low` on every emitted bar of a 40-bar ramp.
- **Pure downtrend.** Symmetrically, with monotonically falling highs,
  the SAR sits above the highs after the trend establishes.
  `pure_downtrend_sar_above_highs` covers this.
- **Reversal mechanics.** When the trend flips, `SAR` is set to the
  previous EP (not the calculated parabola value), AF is reset to
  `af_start`, and the new EP is the current bar's high (Down→Up) or
  low (Up→Down).
- **Choppy regime.** Frequent reversals cause many AF resets; SAR
  becomes a poor stop in mean-reverting regimes and whipsaws.
- **NaN / infinity.** `Candle::new` rejects non-finite OHLC values.
  `Psar::new` rejects non-finite AF parameters.
- **Reset.** `reset()` clears the initialised flag and resets `af` to
  `af_start`, `sar` to `0.0`, `ep` to `0.0`; the next `update` re-seeds.

## Examples

### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, Psar};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let candles: Vec<Candle> = (0..8)
        .map(|i| {
            let base = 100.0 + f64::from(i);
            Candle::new(base, base + 0.5, base - 0.5, base + 0.25, 1.0, 0).unwrap()
        })
        .collect();
    let mut p = Psar::classic(); // (0.02, 0.02, 0.20)
    for (i, v) in p.batch(&candles).into_iter().enumerate() {
        println!("i={i} -> {:?}", v);
    }
    Ok(())
}
```

Output:

```
i=0 -> None
i=1 -> Some(99.5)
i=2 -> Some(99.58)
i=3 -> Some(99.7552)
i=4 -> Some(100.054784)
i=5 -> Some(100.4993056)
i=6 -> Some(101.099388928)
i=7 -> Some(101.85547447808)
```

The SAR starts at `99.5` (the first candle's low) and accelerates
upward toward price as the EP makes new highs on every bar.

### Python

```python
import numpy as np
import wickra as ta

p = ta.PSAR()  # defaults (0.02, 0.02, 0.20)
h  = np.array([100.5, 101.5, 102.5, 103.5, 104.5, 105.5, 106.5, 107.5])
l  = np.array([ 99.5, 100.5, 101.5, 102.5, 103.5, 104.5, 105.5, 106.5])
cl = np.array([100.25, 101.25, 102.25, 103.25, 104.25, 105.25, 106.25, 107.25])
print(p.batch(h, l, cl))
```

Output:

```
[         nan  99.5         99.58        99.7552     100.054784
 100.4993056  101.09938893 101.85547448]
```

### Node

```js
const w = require('wickra');

const p = new w.PSAR(0.02, 0.02, 0.20);
console.log(p.batch(
  [100.5, 101.5, 102.5, 103.5, 104.5, 105.5, 106.5, 107.5],
  [ 99.5, 100.5, 101.5, 102.5, 103.5, 104.5, 105.5, 106.5],
  [100.25, 101.25, 102.25, 103.25, 104.25, 105.25, 106.25, 107.25],
));
```

Output:

```
[
  NaN,
  99.5,
  99.58,
  99.7552,
  100.054784,
  100.4993056,
  101.099388928,
  101.85547447808
]
```

## Interpretation

- **Stop & reverse.** PSAR is a *trailing stop*, not a signal generator
  in isolation: a long is exited (and a short is initiated) the bar
  that price penetrates the SAR line.
- **Acceleration.** The further a trend extends without making new
  extremes, the slower the SAR rises (or falls). When EP makes a new
  extreme, AF bumps by `af_step` and the SAR closes the distance to
  price more aggressively.
- **Whipsaw risk.** In sideways markets PSAR flips repeatedly; pair it
  with a trend filter (ADX, slope of EMA) to skip trades when the
  underlying isn't actually trending.

## Common pitfalls

- **The first bar always returns `None`.** Code that pre-allocates a
  vector and does `out[i] = psar.update(c).unwrap()` will panic on
  the very first input. Use `if let Some(...)` or skip the first
  row explicitly.
- **Initial trend is hard-coded to `Up`.** The seed bar always sets
  `trend = Up`, regardless of whether the data is in a downtrend.
  Expect a near-immediate reversal to `Down` if you feed PSAR a
  decisively bearish series — the first emitted SAR may look
  "wrong" because it is the prior EP from the artificial `Up`
  seed, not from a real bullish run.
- **Acceleration cap matters.** `af_max = 0.20` is Wilder's choice;
  raising it produces an extremely tight stop near tops/bottoms but
  exits good trends prematurely. Lowering it produces a forgiving
  stop that gives back more open profit. Always re-validate strategy
  PnL when you change `af_max`.

## References

- J. Welles Wilder Jr., *New Concepts in Technical Trading Systems*,
  Trend Research, 1978. Chapter on the Parabolic SAR introduces the
  state-machine recursion and the default `(0.02, 0.02, 0.20)`
  parameters.

## See also

- [ATR](Indicator-Atr.md) — sister indicator from the same Wilder text.
- [Donchian Channels](Indicator-Donchian.md) — alternative breakout-style
  trailing stop based on rolling extrema.
- [Keltner Channels](Indicator-Keltner.md) — envelope you can use as a
  smoother stop boundary than PSAR in choppy regimes.
