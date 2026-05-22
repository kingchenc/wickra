# ATR (Average True Range)

> Wilder's volatility benchmark: an exponentially-smoothed average of the
> per-bar true range that absorbs overnight gaps and is dimensioned in price
> units.

## Quick reference

| Item                | Value                                                                                |
|---------------------|--------------------------------------------------------------------------------------|
| Family              | Volatility                                                                           |
| Sub-category        | range-average                                                                        |
| Input type          | `Candle` (uses `high`, `low`, `close`)                                               |
| Output type         | `f64`                                                                                |
| Output range        | unbounded `≥ 0`                                                                      |
| Default parameters  | `period = 14` (Wilder)                                                               |
| Warmup period       | `period` (14 for defaults)                                                           |
| Interpretation      | dollar-denominated volatility scale; rises in chop and expansion                     |

## Formula

For each candle, the **true range** is
`TR_t = max(H_t - L_t, |H_t - C_{t-1}|, |L_t - C_{t-1}|)` when a previous
close is available, otherwise `TR_t = H_t - L_t` (see `Candle::true_range`
in `crates/wickra-core/src/ohlcv.rs`).

ATR is then Wilder-smoothed:

```
seed_ATR_period = (TR_1 + TR_2 + … + TR_period) / period
ATR_t           = ((period - 1) * ATR_{t-1} + TR_t) / period      for t > period
```

This is mathematically the same recursion as an EMA with `alpha = 1/period`
(Wilder smoothing), seeded with a simple mean of the first `period` true
ranges (`crates/wickra-core/src/indicators/atr.rs:58-69`).

## Parameters

| Name     | Type    | Default | Constraint | Source                              |
|----------|---------|---------|------------|-------------------------------------|
| `period` | `usize` | `14`    | `> 0`      | `Atr::new` (`atr.rs:26`)            |

Python default from `#[pyo3(signature = (period=14))]` in
`bindings/python/src/lib.rs`. `period == 0` returns `Error::PeriodZero`.

## Inputs / Outputs

```rust
impl Indicator for Atr {
    type Input  = Candle;
    type Output = f64;
    fn update(&mut self, candle: Candle) -> Option<f64>;
    fn warmup_period(&self) -> usize { self.period }
}
```

- **Rust input.** A full `Candle` struct; only `high`, `low`, and `close`
  are read (`prev_close` is cached internally between calls).
- **Python streaming.** Accepts either a 6-tuple
  `(open, high, low, close, volume, timestamp)` or a dict with keys
  `open`, `high`, `low`, `close`, `volume`, and optional `timestamp`.
- **Python batch.** `ATR.batch(high, low, close)` takes three equal-length
  `numpy.ndarray` columns and returns a 1-D `np.ndarray` with `NaN` for
  every warmup row.
- **Node streaming.** `atr.update(high, low, close)` returns `number | null`.
- **Node batch.** `atr.batch(high, low, close)` returns `Array<number>` of
  the same length, `NaN` during warmup.

## Warmup

`warmup_period() == period`. The first `period - 1` candles return `None`
(or `NaN`/`null` in batch); the `period`-th candle returns the seed value
`(TR_1 + … + TR_period) / period`. Each subsequent candle applies the
Wilder recursion.

Verified for `period = 3`: the first non-`None` output is at index `2`
(the 3rd candle).

## Edge cases

- **First candle.** `Candle::true_range(None)` falls back to `high - low`
  because there is no previous close yet. The first TR is the bar range.
- **Gaps.** With a previous close at `5.0` and a candle of `H=10, L=9`,
  `TR = max(1, 5, 4) = 5` — i.e. `|H - prev_close|` dominates. The
  pinned test `gap_up_uses_high_minus_prev_close` covers exactly this.
- **Constant input.** A series of identical candles (no gaps, fixed range)
  yields a constant ATR equal to the bar range, even before the seed is
  complete — the smoothing has nothing to smooth.
- **Non-negativity.** ATR is always `≥ 0`. The Rust test `never_negative`
  pins this property across a sinusoidal price series.
- **NaN / infinity.** `Candle::new` rejects non-finite `open`/`high`/
  `low`/`close`/`volume`; constructing the candle returns
  `Error::InvalidCandle` before it can ever reach ATR.
- **Reset.** `reset()` clears `prev_close`, the seed buffer, and the
  running average; the next call behaves as if the indicator were
  freshly constructed.

## Examples

### Rust

```rust
use wickra::{Atr, BatchExt, Candle, Indicator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let candles = vec![
        Candle::new(10.0, 11.0, 9.0,  10.5, 1.0, 0)?,
        Candle::new(10.5, 12.0, 10.0, 11.5, 1.0, 0)?,
        Candle::new(11.5, 13.0, 11.0, 12.5, 1.0, 0)?,
        Candle::new(12.5, 14.0, 12.0, 13.5, 1.0, 0)?,
        Candle::new(13.5, 15.0, 13.0, 14.5, 1.0, 0)?,
    ];
    let mut atr = Atr::new(3)?;
    println!("{:?}", atr.batch(&candles));
    Ok(())
}
```

Output:

```
[None, None, Some(2.0), Some(2.0), Some(2.0)]
```

Every bar has range `2.0` and no gap-driven TR component, so both the
seed `(2 + 2 + 2) / 3 = 2.0` and every subsequent Wilder update stay at
`2.0`.

### Python

```python
import numpy as np
import wickra as ta

atr = ta.ATR(3)
high  = np.array([11.0, 12.0, 13.0, 14.0, 15.0])
low   = np.array([ 9.0, 10.0, 11.0, 12.0, 13.0])
close = np.array([10.5, 11.5, 12.5, 13.5, 14.5])
print(atr.batch(high, low, close))
```

Output:

```
[nan nan  2.  2.  2.]
```

### Node

```js
const w = require('wickra');

const atr = new w.ATR(3);
console.log(atr.batch(
  [11, 12, 13, 14, 15],
  [ 9, 10, 11, 12, 13],
  [10.5, 11.5, 12.5, 13.5, 14.5],
));
```

Output:

```
[ NaN, NaN, 2, 2, 2 ]
```

Streaming form (`atr.update(high, low, close)`):

```js
const w = require('wickra');

const atr = new w.ATR(3);
console.log(atr.update(11, 9,  10.5));
console.log(atr.update(12, 10, 11.5));
console.log(atr.update(13, 11, 12.5));
console.log(atr.update(14, 12, 13.5));
```

Output:

```
null
null
2
2
```

## Interpretation

- **Stop sizing.** A common pattern is "place a stop `k * ATR` away from
  entry," with `k` typically in `[1.5, 3.0]` depending on the timeframe.
  ATR's units are price, so the stop distance is directly tradable.
- **Position sizing.** `risk_per_trade / ATR` gives a quantity that
  normalises risk across assets of very different price levels.
- **Regime detection.** Persistently rising ATR signals an expansion
  regime; persistently low ATR signals consolidation, often preceding
  expansion (the volatility-of-volatility argument).

## Common pitfalls

- **Wilder smoothing vs EMA.** Wilder's smoothing factor is `1/period`,
  not the EMA's `2/(period+1)`. They look similar but produce different
  numbers; a 14-period Wilder ATR is **not** the same as a 14-period
  EMA of true range. Wickra uses the Wilder recursion explicitly.
- **Off-by-one seeding.** ATR(14) emits its first value on the 14th
  candle, not the 15th — unlike RSI(14) which needs 15 candles for 14
  diffs. The difference is that ATR's seed uses `period` true ranges
  directly (and `TR_1` is well-defined even without a previous close),
  while RSI(14) needs 14 *differences* between consecutive closes.

## References

- J. Welles Wilder Jr., *New Concepts in Technical Trading Systems*,
  Trend Research, 1978. Chapter on the Average True Range and the
  Wilder smoothing constant.

## See also

- [Bollinger Bands](Indicator-BollingerBands.md) — stddev-based volatility
  envelope around an SMA.
- [Keltner Channels](Indicator-Keltner.md) — directly composes EMA + ATR.
- [Donchian Channels](Indicator-Donchian.md) — rolling high/low without
  any smoothing.
- [PSAR](Indicator-Psar.md) — uses ATR-like volatility tracking implicitly
  through its acceleration factor.
