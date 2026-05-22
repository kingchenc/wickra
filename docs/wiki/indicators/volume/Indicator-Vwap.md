# VWAP (Volume-Weighted Average Price)

> The volume-weighted mean of typical price; the institutional benchmark for
> "fair" intraday execution. Wickra ships both the unbounded cumulative
> session VWAP and a finite-window `RollingVwap`.

## Quick reference

| Item                | Value                                                          |
|---------------------|----------------------------------------------------------------|
| Family              | Volume                                                         |
| Sub-category        | cumulative (`Vwap`) / rolling (`RollingVwap`)                  |
| Input type          | `Candle` (uses `high`, `low`, `close`, `volume`)               |
| Output type         | `f64`                                                          |
| Output range        | unbounded (price-units)                                        |
| Default parameters  | none for `Vwap`; `period` required for `RollingVwap`           |
| Warmup period       | `1` for `Vwap`, `period` for `RollingVwap`                     |
| Interpretation      | intraday fair-price benchmark for execution                    |

## Formula

Both variants use the typical price `tp_t = (H_t + L_t + C_t) / 3`
(see `Candle::typical_price` in `crates/wickra-core/src/ohlcv.rs:104-108`).

Cumulative VWAP:

```
VWAP_t = ( Σ_{i=1..t} tp_i * v_i ) / ( Σ_{i=1..t} v_i )
```

Rolling VWAP over the last `period` candles:

```
RollingVWAP_t = ( Σ_{i=t-period+1..t} tp_i * v_i ) / ( Σ_{i=t-period+1..t} v_i )
```

Both forms gate their output: when the relevant volume sum is `0.0`, no
value is emitted (`vwap.rs:50, 121`).

---

## `Vwap` (cumulative)

The session VWAP. State grows forever; call `reset()` at session
boundaries (e.g. the start of the trading day) to restart accumulation.

### Parameters

`Vwap::new()` takes no parameters. Python: `wickra.VWAP()`. Node:
`new w.VWAP()`.

### Inputs / Outputs

```rust
impl Indicator for Vwap {
    type Input  = Candle;
    type Output = f64;
    fn update(&mut self, candle: Candle) -> Option<f64>;
    fn warmup_period(&self) -> usize { 1 }
}
```

- **Rust input.** A full `Candle`; the indicator multiplies
  `typical_price() * volume` and accumulates.
- **Python batch.** `VWAP.batch(high, low, close, volume)` returns a 1-D
  `np.ndarray` with `NaN` for any prefix where the cumulative volume is
  still `0`.
- **Node batch.** `vwap.batch(high, low, close, volume)` returns
  `Array<number>` with `NaN` for the same prefix.

### Warmup

`warmup_period() == 1`. Provided the first candle has positive volume,
the indicator emits on tick 1. If the first `k` candles all have
`volume == 0`, no output is emitted until the first candle with
non-zero volume — `RollingVwap`'s warmup gating is independent of
this volume-gating logic and applies on top of it.

### Edge cases

- **Zero-volume bar.** A candle with `volume == 0` does not advance the
  running sums in any visible way and (if it is the *first* such bar
  the indicator has seen) keeps the output at `None`. The implementation
  short-circuits with `if self.sum_v == 0.0 { return None; }`
  (`vwap.rs:50`).
- **Constant input.** Identical candles produce a flat VWAP equal to
  their typical price.
- **Session boundaries.** There is no automatic reset; the caller is
  responsible for invoking `reset()` at the start of each new session.
- **NaN / infinity.** `Candle::new` rejects non-finite OHLCV values
  before they can reach the indicator.
- **Reset.** `reset()` zeroes both running sums and unsets the `has_emitted`
  flag.

### Examples

#### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, Vwap};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let candles = vec![
        Candle::new(10.0, 10.0, 10.0, 10.0, 1.0, 0)?, // tp = 10
        Candle::new(20.0, 20.0, 20.0, 20.0, 3.0, 0)?, // tp = 20
        Candle::new(30.0, 30.0, 30.0, 30.0, 1.0, 0)?, // tp = 30
        Candle::new(40.0, 40.0, 40.0, 40.0, 2.0, 0)?, // tp = 40
    ];
    let mut v = Vwap::new();
    println!("{:?}", v.batch(&candles));
    Ok(())
}
```

Output:

```
[Some(10.0), Some(17.5), Some(20.0), Some(25.714285714285715)]
```

Hand check at `t = 2`: `(10*1 + 20*3) / (1+3) = 70/4 = 17.5`.
At `t = 4`: `(10*1 + 20*3 + 30*1 + 40*2) / (1+3+1+2) = 180/7 ≈ 25.7142857`.

#### Python

```python
import numpy as np
import wickra as ta

vw = ta.VWAP()
h = np.array([10.0, 20.0, 30.0, 40.0])
l = np.array([10.0, 20.0, 30.0, 40.0])
c = np.array([10.0, 20.0, 30.0, 40.0])
v = np.array([ 1.0,  3.0,  1.0,  2.0])
print(vw.batch(h, l, c, v))
```

Output:

```
[10.         17.5        20.         25.71428571]
```

#### Node

```js
const w = require('wickra');

const vw = new w.VWAP();
console.log(vw.batch(
  [10, 20, 30, 40],
  [10, 20, 30, 40],
  [10, 20, 30, 40],
  [ 1,  3,  1,  2],
));
```

Output:

```
[ 10, 17.5, 20, 25.714285714285715 ]
```

---

## `RollingVwap` (finite window)

A rolling-window variant for streaming bots that want a finite-memory
fair-price benchmark instead of an unbounded session aggregate.

### Parameters

| Name     | Type    | Default      | Constraint | Source                                       |
|----------|---------|--------------|------------|----------------------------------------------|
| `period` | `usize` | (no default) | `> 0`      | `RollingVwap::new` (`vwap.rs:89`)            |

`period == 0` returns `Error::PeriodZero`. `RollingVwap` is exposed in
Rust only — Python's `VWAP` / Node's `VWAP` correspond to the cumulative
form.

### Inputs / Outputs

```rust
impl Indicator for RollingVwap {
    type Input  = Candle;
    type Output = f64;
    fn update(&mut self, candle: Candle) -> Option<f64>;
    fn warmup_period(&self) -> usize { self.period }
}
```

The window stores `(typical_price * volume, volume)` pairs and runs
incremental `sum_pv` / `sum_v` aggregates, so each `update` is O(1).

### Warmup

`warmup_period() == period`. The first `period - 1` candles return
`None`; the `period`-th candle emits the first value provided the rolling
volume sum is positive. If the entire window has `volume == 0`, the
indicator stays at `None`.

### Edge cases

- **Window slides.** Once `window.len() == period`, the oldest
  `(pv, v)` pair is subtracted from the running sums before the new
  pair is added.
- **Zero-volume window.** If every candle in the window has zero
  volume, `sum_v == 0` and the indicator suppresses output until a
  positive-volume candle is in scope.
- **Reset.** `reset()` clears the window and both running sums.
- **`is_ready()`.** Returns `true` only when the window is full **and**
  `sum_v > 0` (`vwap.rs:138`).

### Examples

#### Rust

```rust
use wickra::{BatchExt, Candle, Indicator, RollingVwap};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let candles = vec![
        Candle::new(10.0, 10.0, 10.0, 10.0, 1.0, 0)?,
        Candle::new(20.0, 20.0, 20.0, 20.0, 3.0, 0)?,
        Candle::new(30.0, 30.0, 30.0, 30.0, 1.0, 0)?,
        Candle::new(40.0, 40.0, 40.0, 40.0, 2.0, 0)?,
    ];
    let mut rv = RollingVwap::new(3)?;
    println!("{:?}", rv.batch(&candles));
    Ok(())
}
```

Output:

```
[None, None, Some(20.0), Some(28.333333333333332)]
```

Hand check at `t = 3` with window `[10@1, 20@3, 30@1]`:
`(10 + 60 + 30) / (1+3+1) = 100/5 = 20.0`.
At `t = 4` with window `[20@3, 30@1, 40@2]`:
`(60 + 30 + 80) / (3+1+2) = 170/6 ≈ 28.333`.

(`RollingVwap` is currently exposed only in the Rust API; the Python
`VWAP` and Node `VWAP` classes correspond to the cumulative form.)

## Interpretation

- **Execution benchmark.** "Beat VWAP" is the canonical buy-side
  execution mandate: an aggressive algo that ends up paying *below*
  VWAP on the day is considered to have earned alpha relative to a
  passive participation strategy.
- **Mean reversion.** Intraday strategies often fade extensions away
  from VWAP, treating the VWAP line as a magnet.
- **Trend filter.** Some systems trade only longs above VWAP and only
  shorts below it; the line acts as a session-aware bias toggle.

## Common pitfalls

- **Forgetting to reset.** Call `reset()` at session start (or on each
  new trading day) — otherwise you average yesterday's tape into
  today's signal and the line drifts permanently behind current
  price action.
- **Zero-volume warmup.** Several common data sources include
  pre-session candles with `volume = 0` for "no print this minute".
  Cumulative VWAP returns `None` until at least one positive-volume
  candle has been seen; downstream code should treat `None` /
  `NaN` / `null` as "not yet ready," not as "VWAP is zero."
- **Typical price vs close.** Wickra uses typical price
  `(H + L + C) / 3`, not close. A naive implementation that uses
  close will produce noticeably different numbers on bars with wide
  intraday ranges.

## References

- The VWAP construct emerged in institutional execution literature in
  the late 1980s and early 1990s; it has no single attributed
  inventor. The textbook reference for its role as an execution
  benchmark is Bertsimas & Lo, "Optimal control of execution costs,"
  *Journal of Financial Markets*, 1998.

## See also

- [OBV](Indicator-Obv.md) — cumulative signed-volume measure that pairs
  well with VWAP as a divergence flag.
- [MFI](../momentum/Indicator-Mfi.md) — money-flow oscillator that also blends
  typical price with volume.
- [Bollinger Bands](../volatility/Indicator-BollingerBands.md) — non-volume volatility
  envelope, often layered alongside VWAP on intraday charts.
