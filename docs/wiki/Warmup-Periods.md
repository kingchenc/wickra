# Warmup Periods

Every Wickra indicator returns `None` (Rust), `None` (Python), or `null`
(Node) for its first few inputs while it gathers enough data to produce a
defined value. The number of inputs an indicator needs before it emits its
first non-empty value is its **warmup period**, surfaced everywhere as
`warmup_period()` / `warmupPeriod()`.

After the first emission, the indicator never goes back to a "no value yet"
state — it has rolled its state forward and will produce a steady value on
every subsequent `update()`. Calling `reset()` returns to the warming-up
state, equivalent to a freshly constructed instance.

## How to read the formula column

The formulas below are taken verbatim from the `warmup_period()` methods in
`crates/wickra-core/src/indicators/<name>.rs`. The "Inputs at first
emission" column says, in 1-indexed terms, which `update()` call returns the
first `Some`/non-`NaN` value. They are the same number; "first emission
index" in 0-indexed terms is `warmup_period − 1`.

## Single-output indicators

| Indicator       | Constructor                                  | Formula                          | `warmup_period()` for shown args | Inputs at first emission |
|-----------------|----------------------------------------------|----------------------------------|----------------------------------|--------------------------|
| `Sma`           | `Sma::new(14)`                               | `period`                         | 14                               | 14th                     |
| `Ema`           | `Ema::new(14)`                               | `period`                         | 14                               | 14th                     |
| `Wma`           | `Wma::new(14)`                               | `period`                         | 14                               | 14th                     |
| `Dema`          | `Dema::new(14)`                              | `2 * period - 1`                 | 27                               | 27th                     |
| `Tema`          | `Tema::new(14)`                              | `3 * period - 2`                 | 40                               | 40th                     |
| `Hma`           | `Hma::new(14)`                               | `period + round(sqrt(period)).max(1) - 1` | 17                      | 17th                     |
| `Kama`          | `Kama::new(10, 2, 30)`                       | `er_period + 1`                  | 11                               | 11th                     |
| `Rsi`           | `Rsi::new(14)`                               | `period + 1`                     | 15                               | 15th                     |
| `Cci`           | `Cci::new(20)`                               | `period`                         | 20                               | 20th                     |
| `Roc`           | `Roc::new(12)`                               | `period + 1`                     | 13                               | 13th                     |
| `WilliamsR`     | `WilliamsR::new(14)`                         | `period`                         | 14                               | 14th                     |
| `Mfi`           | `Mfi::new(14)`                               | `period`                         | 14                               | 14th                     |
| `Trix`          | `Trix::new(15)`                              | `3 * period - 1`                 | 44                               | 44th                     |
| `AwesomeOscillator` | `AwesomeOscillator::new(5, 34)`          | `slow_period`                    | 34                               | 34th                     |
| `Atr`           | `Atr::new(14)`                               | `period`                         | 14                               | 14th                     |
| `Psar`          | `Psar::new(0.02, 0.02, 0.20)`                | constant `2`                     | 2                                | 2nd                      |
| `Obv`           | `Obv::new()`                                 | constant `1`                     | 1                                | 1st                      |
| `Vwap`          | `Vwap::new()`                                | constant `1`                     | 1                                | 1st                      |
| `RollingVwap`   | `RollingVwap::new(20)`                       | `period`                         | 20                               | 20th                     |
| `Smma`          | `Smma::new(14)`                              | `period`                         | 14                               | 14th                     |
| `Trima`         | `Trima::new(20)`                             | `period`                         | 20                               | 20th                     |
| `Zlema`         | `Zlema::new(14)`                             | `lag + period` (`lag = (period − 1) / 2`) | 20                      | 20th                     |
| `T3`            | `T3::new(5, 0.7)`                            | `6 * period - 5`                 | 25                               | 25th                     |
| `Vwma`          | `Vwma::new(20)`                              | `period`                         | 20                               | 20th                     |
| `Mom`           | `Mom::new(10)`                               | `period + 1`                     | 11                               | 11th                     |
| `Cmo`           | `Cmo::new(14)`                               | `period + 1`                     | 15                               | 15th                     |
| `Tsi`           | `Tsi::new(25, 13)`                           | `long + short`                   | 38                               | 38th                     |
| `Pmo`           | `Pmo::new(35, 20)`                           | constant `2`                     | 2                                | 2nd                      |
| `StochRsi`      | `StochRsi::new(14, 14)`                      | `rsi_period + stoch_period`      | 28                               | 28th                     |
| `UltimateOscillator` | `UltimateOscillator::new(7, 14, 28)`    | `max(short, mid, long) + 1`      | 29                               | 29th                     |
| `Ppo`           | `Ppo::new(12, 26)`                           | `slow`                           | 26                               | 26th                     |
| `Dpo`           | `Dpo::new(20)`                               | `max(period, period / 2 + 2)`    | 20                               | 20th                     |
| `Coppock`       | `Coppock::new(14, 11, 10)`                   | `max(roc_long, roc_short) + wma_period` | 24                        | 24th                     |
| `AroonOscillator` | `AroonOscillator::new(14)`                 | `period + 1`                     | 15                               | 15th                     |
| `MassIndex`     | `MassIndex::new(9, 25)`                      | `2 * ema_period + sum_period - 2` | 41                              | 41st                     |
| `Natr`          | `Natr::new(14)`                              | `period`                         | 14                               | 14th                     |
| `StdDev`        | `StdDev::new(20)`                            | `period`                         | 20                               | 20th                     |
| `UlcerIndex`    | `UlcerIndex::new(14)`                        | `2 * period - 1`                 | 27                               | 27th                     |
| `HistoricalVolatility` | `HistoricalVolatility::new(20, 252)`  | `period + 1`                     | 21                               | 21st                     |
| `BollingerBandwidth` | `BollingerBandwidth::new(20, 2.0)`      | `period`                         | 20                               | 20th                     |
| `PercentB`      | `PercentB::new(20, 2.0)`                     | `period`                         | 20                               | 20th                     |
| `AtrTrailingStop` | `AtrTrailingStop::new(14, 3.0)`            | `atr_period`                     | 14                               | 14th                     |
| `Adl`           | `Adl::new()`                                 | constant `1`                     | 1                                | 1st                      |
| `VolumePriceTrend` | `VolumePriceTrend::new()`                 | constant `1`                     | 1                                | 1st                      |
| `ChaikinMoneyFlow` | `ChaikinMoneyFlow::new(20)`               | `period`                         | 20                               | 20th                     |
| `ChaikinOscillator` | `ChaikinOscillator::new(3, 10)`          | `slow`                           | 10                               | 10th                     |
| `ForceIndex`    | `ForceIndex::new(13)`                        | `period + 1`                     | 14                               | 14th                     |
| `EaseOfMovement` | `EaseOfMovement::new(14)`                   | `period + 1`                     | 15                               | 15th                     |
| `TypicalPrice`  | `TypicalPrice::new()`                        | constant `1`                     | 1                                | 1st                      |
| `MedianPrice`   | `MedianPrice::new()`                         | constant `1`                     | 1                                | 1st                      |
| `WeightedClose` | `WeightedClose::new()`                       | constant `1`                     | 1                                | 1st                      |
| `LinearRegression` | `LinearRegression::new(14)`               | `period`                         | 14                               | 14th                     |
| `LinRegSlope`   | `LinRegSlope::new(14)`                       | `period`                         | 14                               | 14th                     |
| `AcceleratorOscillator` | `AcceleratorOscillator::classic()`   | `ao_slow + signal_period - 1`    | 38                               | 38th                     |
| `BalanceOfPower` | `BalanceOfPower::new()`                     | constant `1`                     | 1                                | 1st                      |
| `ChoppinessIndex` | `ChoppinessIndex::new(14)`                 | `period`                         | 14                               | 14th                     |
| `VerticalHorizontalFilter` | `VerticalHorizontalFilter::new(28)` | `period + 1`                  | 29                               | 29th                     |
| `TrueRange`     | `TrueRange::new()`                           | constant `1`                     | 1                                | 1st                      |
| `ChaikinVolatility` | `ChaikinVolatility::new(10, 10)`         | `ema_period + roc_period`        | 20                               | 20th                     |
| `ZScore`        | `ZScore::new(20)`                            | `period`                         | 20                               | 20th                     |
| `LinRegAngle`   | `LinRegAngle::new(14)`                       | `period`                         | 14                               | 14th                     |

## Multi-output indicators

These indicators emit several values at once (a struct in Rust, a tuple in
Python, an object in Node) and every column / field transitions from "not
ready" to "ready" together — there are no rows that have a `signal` but no
`macd`, for example.

| Indicator         | Constructor                          | Formula                                  | `warmup_period()` for shown args | Inputs at first emission | Outputs                                                |
|-------------------|--------------------------------------|------------------------------------------|----------------------------------|--------------------------|--------------------------------------------------------|
| `MacdIndicator`   | `MacdIndicator::new(12, 26, 9)`      | `slow + signal - 1`                      | 34                               | 34th                     | `macd`, `signal`, `histogram`                          |
| `BollingerBands`  | `BollingerBands::new(20, 2.0)`       | `period`                                 | 20                               | 20th                     | `upper`, `middle`, `lower`, `stddev`                   |
| `Stochastic`      | `Stochastic::new(14, 3)`             | `k_period + d_period - 1`                | 16                               | 16th                     | `k`, `d`                                               |
| `Adx`             | `Adx::new(14)`                       | `2 * period`                             | 28                               | 28th                     | `plus_di`, `minus_di`, `adx`                           |
| `Aroon`           | `Aroon::new(14)`                     | `period + 1`                             | 15                               | 15th                     | `up`, `down`                                           |
| `Keltner`         | `Keltner::new(20, 10, 2.0)`          | `ema_period.max(atr_period)`             | 20                               | 20th                     | `upper`, `middle`, `lower`                             |
| `Donchian`        | `Donchian::new(20)`                  | `period`                                 | 20                               | 20th                     | `upper`, `middle`, `lower`                             |
| `Vortex`          | `Vortex::new(14)`                    | `period + 1`                             | 15                               | 15th                     | `plus`, `minus`                                        |
| `SuperTrend`      | `SuperTrend::new(10, 3.0)`           | `atr_period`                             | 10                               | 10th                     | `value`, `direction`                                   |
| `ChandelierExit`  | `ChandelierExit::new(22, 3.0)`       | `period`                                 | 22                               | 22nd                     | `long_stop`, `short_stop`                              |
| `ChandeKrollStop` | `ChandeKrollStop::new(10, 1.0, 9)`   | `atr_period + stop_period - 1`           | 18                               | 18th                     | `stop_long`, `stop_short`                              |

## "Off-by-one" cases worth memorising

A few indicators look like they should warm up at `period` but in fact need
`period + 1` inputs. The reason is always the same — they consume *diffs*
or *previous-close* differences, not the prices themselves, and the very
first input has nothing to diff against.

- **`Rsi::new(period)` warmup is `period + 1`.** RSI is based on Wilder's
  smoothing over per-tick gains and losses. With 14 prices you only have 13
  diffs; you need 15 prices to compute 14 diffs and seed `avg_gain` /
  `avg_loss`. The Rust unit test that pins this is
  `warmup_period_is_period_plus_one`:
  ```rust
  let rsi = Rsi::new(14).unwrap();
  assert_eq!(rsi.warmup_period(), 15);
  ```
- **`Roc::new(period)` warmup is `period + 1`.** ROC compares the current
  price to the price `period` bars ago; that comparison only makes sense
  starting at input `period + 1`.
- **`Aroon::new(period)` warmup is `period + 1`.** Aroon scans a `period + 1`-bar
  window to find the bars-since-high and bars-since-low.
- **`Kama::new(er_period, ...)` warmup is `er_period + 1`.** Kaufman's
  efficiency ratio needs `er_period` differences, which costs one extra
  bar.

## Cross-checking from your own code

The cleanest way to verify any of these from your application code is the
indicator's own `warmup_period()`:

```rust
use wickra::{Indicator, MacdIndicator};
let macd = MacdIndicator::classic();   // (12, 26, 9)
assert_eq!(macd.warmup_period(), 34);
```

```python
import wickra as ta
assert ta.MACD(12, 26, 9).warmup_period() == 34
```

```javascript
const wickra = require('wickra');
const sma = new wickra.SMA(20);
console.log(sma.warmupPeriod());   // -> 20
```

(Since `wickra@0.1.5`, `warmupPeriod()` is exposed on every Node and
WASM class — single- and multi-output — alongside `update()`, `reset()`
and `isReady()`. Consult `bindings/node/index.d.ts` for the
authoritative TypeScript surface.)

## See also

- [Streaming vs Batch](Streaming-vs-Batch.md) — the `is_ready()` gate, and
  why a `len(prices) > warmup_period` check is the wrong abstraction.
- [Indicator Chaining](Indicator-Chaining.md) — how warmups stack inside a
  `Chain`.
- Source: <https://github.com/kingchenc/wickra>
