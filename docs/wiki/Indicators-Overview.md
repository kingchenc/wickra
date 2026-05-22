# Indicators Overview

Wickra ships 58 indicators, organised in source under the four classical
families — trend, momentum, volatility, volume — that map directly to the
directory structure of `crates/wickra-core/src/indicators/`. The same family
labels are used here, plus a second-level grouping that reflects how the
indicators actually behave (which output range they live in, what data they
need, what question they answer).

Every indicator is an O(1) state machine that consumes one input at a time
and produces either `Option<f64>` (Rust), `float | None` (Python), or
`number | null` (Node). Inputs are either a `f64` close price or an OHLCV
`Candle` (Rust) / dict-or-tuple (Python) / column arrays (Node). The full
trait surface and warmup-period semantics are covered in
[Quickstart: Rust](Quickstart-Rust.md) and [Warmup Periods](Warmup-Periods.md).

The "Output range" column below is the value bounds an indicator emits once
warm. "unbounded" means it tracks the price scale of the input. The
"Warmup" column quotes `warmup_period()` as the indicator reports it; this
is the **exact** first-emission index for every indicator — the first
non-`None` output lands on input `warmup_period()` (index
`warmup_period() - 1`).

## Trend

Trend indicators smooth the price series to surface direction. They are
all single-input, single-output (`f64 → f64`).

### Simple averages

Pure linear weighting. Mostly used as fast baselines or as comparison
benchmarks against fancier averages.

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `Sma`     | Equal-weighted rolling mean over `period` closes. | `f64` | `f64` | unbounded (price scale) | `period` (no default in core; Python defaults vary by binding) | `period` | [Indicator-Sma.md](indicators/trend/Indicator-Sma.md) |
| `Wma`     | Linear weights `1, 2, …, period` so the newest bar matters most. | `f64` | `f64` | unbounded (price scale) | `period` | `period` | [Indicator-Wma.md](indicators/trend/Indicator-Wma.md) |
| `Trima`   | A `period`-window SMA applied twice; triangular weights centred on the middle bar. | `f64` | `f64` | unbounded (price scale) | `period` | `period` | [Indicator-Trima.md](indicators/trend/Indicator-Trima.md) |
| `Vwma`    | Rolling mean of closes weighted by each bar's volume. | `Candle` | `f64` | unbounded (price scale) | `period` | `period` | [Indicator-Vwma.md](indicators/trend/Indicator-Vwma.md) |

### Exponential family

Recursive smoothing with one or more chained EMAs. Lag reduction grows as
you stack more EMAs, but so does responsiveness to noise.

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `Ema`  | EMA with `α = 2 / (period + 1)`, seeded from the SMA of the first `period` inputs. | `f64` | `f64` | unbounded (price scale) | `period` | `period` | [Indicator-Ema.md](indicators/trend/Indicator-Ema.md) |
| `Dema` | Mulloy's `2·EMA − EMA(EMA)`; removes first-order EMA lag. | `f64` | `f64` | unbounded (price scale) | `period` | `2·period − 1` | [Indicator-Dema.md](indicators/trend/Indicator-Dema.md) |
| `Tema` | Mulloy's `3·EMA − 3·EMA(EMA) + EMA(EMA(EMA))`; removes more lag than DEMA. | `f64` | `f64` | unbounded (price scale) | `period` | `3·period − 2` | [Indicator-Tema.md](indicators/trend/Indicator-Tema.md) |
| `Smma` | Wilder's RMA: an SMA-seeded exponential average with the slow `1/period` factor. | `f64` | `f64` | unbounded (price scale) | `period` | `period` | [Indicator-Smma.md](indicators/trend/Indicator-Smma.md) |
| `Zlema` | EMA of the de-lagged series `2·price − price[lag]`; near-zero group delay. | `f64` | `f64` | unbounded (price scale) | `period` | `lag + period` | [Indicator-Zlema.md](indicators/trend/Indicator-Zlema.md) |
| `T3` | Tillson's six-EMA cascade recombined with a volume factor `v`. | `f64` | `f64` | unbounded (price scale) | `(period, v=0.7)` (Python) | `6·period − 5` | [Indicator-T3.md](indicators/trend/Indicator-T3.md) |

`Trix` is also built from a triple-smoothed EMA, but it is a *momentum
oscillator* — it emits the rate of change of that EMA, not a price-scale
trend line — so it is listed under [Momentum](#momentum), matching the
`indicators/momentum/` source layout.

### Adaptive & hybrid

These two adjust their effective smoothing on the fly. They are the
"smart" trend filters; both also live in Trend by directory placement.

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `Hma`  | Hull's `WMA(2·WMA(n/2) − WMA(n), √n)`; near-zero lag with a built-in noise filter. | `f64` | `f64` | unbounded (price scale) | `period` | `period + round(√period) − 1` (see notes) | [Indicator-Hma.md](indicators/trend/Indicator-Hma.md) |
| `Kama` | Kaufman's adaptive average: efficiency ratio picks an α between a fast and slow EMA per bar. | `f64` | `f64` | unbounded (price scale) | `(er_period=10, fast=2, slow=30)` | `er_period + 1` (see notes) | [Indicator-Kama.md](indicators/trend/Indicator-Kama.md) |

## Momentum

Momentum indicators measure the *rate* of price change, not the level.
Several are bounded by construction (0–100 oscillators); others are
unbounded; one (`Adx`) is directional and bundles three values.

### Bounded oscillators (0 – 100)

These all share the "overbought above 70/80, oversold below 30/20"
mental model, though the exact thresholds differ in the literature.

| Indicator    | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|--------------|-----------|-------|--------|-------|----------|--------|-----------|
| `Rsi`        | Wilder's RSI; smoothed `gain / (gain + loss) × 100`. | `f64` | `f64` | `[0, 100]` | `period = 14` (Python) | `period + 1` | [Indicator-Rsi.md](indicators/momentum/Indicator-Rsi.md) |
| `Stochastic` | `%K = (close − low_n)/(high_n − low_n) × 100`, smoothed into `%D`. | `Candle` | `(k, d)` | each in `[0, 100]` | `(k_period=14, d_period=3)` (Python) | `k_period + d_period − 1` | [Indicator-Stochastic.md](indicators/momentum/Indicator-Stochastic.md) |
| `Mfi`        | "Volume-weighted RSI": Wilder smoothing of money-flow ratios. | `Candle` | `f64` | `[0, 100]` | `period = 14` (Python) | `period` | [Indicator-Mfi.md](indicators/momentum/Indicator-Mfi.md) |
| `Aroon`      | Bars-since-high and bars-since-low scaled to `[0, 100]`. | `Candle` | `(up, down)` | each in `[0, 100]` | `period = 14` (Python) | `period + 1` | [Indicator-Aroon.md](indicators/momentum/Indicator-Aroon.md) |
| `StochRsi`   | Stochastic Oscillator applied to the RSI series; sharpens RSI extremes. | `f64` | `f64` | `[0, 100]` | `(rsi_period=14, stoch_period=14)` (Python) | `rsi_period + stoch_period` | [Indicator-StochRsi.md](indicators/momentum/Indicator-StochRsi.md) |
| `UltimateOscillator` | Larry Williams' weighted three-timeframe buying-pressure oscillator. | `Candle` | `f64` | `[0, 100]` | `(short=7, mid=14, long=28)` (Python) | `max(short,mid,long) + 1` | [Indicator-UltimateOscillator.md](indicators/momentum/Indicator-UltimateOscillator.md) |

### Unbounded oscillators

Centered on zero or driven by raw price differences; no fixed cap.

| Indicator           | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|---------------------|-----------|-------|--------|-------|----------|--------|-----------|
| `MacdIndicator`     | `EMA(fast) − EMA(slow)` plus a signal-line EMA and the difference histogram. | `f64` | `(macd, signal, histogram)` | unbounded around zero | `(fast=12, slow=26, signal=9)` (Python) | `slow + signal − 1` | [Indicator-MacdIndicator.md](indicators/momentum/Indicator-MacdIndicator.md) |
| `Cci`               | `(typical − SMA(typical)) / (0.015 · mean_dev)`; unbounded but typically `±100`. | `Candle` | `f64` | unbounded (typically `±100` to `±200`) | `period = 20` (Python) | `period` | [Indicator-Cci.md](indicators/momentum/Indicator-Cci.md) |
| `Roc`               | `(price − price_n) / price_n × 100`; raw percentage change over `period` bars. | `f64` | `f64` | unbounded around zero | `period` | `period + 1` | [Indicator-Roc.md](indicators/momentum/Indicator-Roc.md) |
| `AwesomeOscillator` | `SMA(median, fast) − SMA(median, slow)`; Bill Williams' zero-line crossover oscillator. | `Candle` | `f64` | unbounded around zero | `(fast=5, slow=34)` (Python) | `slow_period` | [Indicator-AwesomeOscillator.md](indicators/momentum/Indicator-AwesomeOscillator.md) |
| `WilliamsR`         | `−100 × (high_n − close) / (high_n − low_n)`; same family as Stochastic but inverted to `[−100, 0]`. | `Candle` | `f64` | `[−100, 0]` | `period = 14` (Python) | `period` | [Indicator-WilliamsR.md](indicators/momentum/Indicator-WilliamsR.md) |
| `Trix`              | `(EMA(EMA(EMA(price))).pct_change × 10000)`; oscillator built from a triple-smoothed EMA. | `f64` | `f64` | unbounded around zero | `period = 15` (Python) | `3·period − 1` | [Indicator-Trix.md](indicators/momentum/Indicator-Trix.md) |
| `Mom`               | `price − price[period]`; raw price-difference momentum. | `f64` | `f64` | unbounded around zero | `period = 10` (Python) | `period + 1` | [Indicator-Mom.md](indicators/momentum/Indicator-Mom.md) |
| `Cmo`               | Chande Momentum Oscillator; `100·(Σgain − Σloss)/(Σgain + Σloss)` over `period` changes. | `f64` | `f64` | `[−100, 100]` | `period = 14` (Python) | `period + 1` | [Indicator-Cmo.md](indicators/momentum/Indicator-Cmo.md) |
| `Tsi`               | True Strength Index; ratio of double-EMA-smoothed momentum to its absolute value. | `f64` | `f64` | ≈ `[−100, 100]` around zero | `(long=25, short=13)` (Python) | `long + short` | [Indicator-Tsi.md](indicators/momentum/Indicator-Tsi.md) |
| `Pmo`               | DecisionPoint Price Momentum Oscillator; doubly-smoothed rate of change. | `f64` | `f64` | unbounded around zero | `(smoothing1=35, smoothing2=20)` (Python) | `2` | [Indicator-Pmo.md](indicators/momentum/Indicator-Pmo.md) |
| `Ppo`               | Percentage Price Oscillator; `100·(EMA_fast − EMA_slow)/EMA_slow`. | `f64` | `f64` | unbounded around zero (percent) | `(fast=12, slow=26)` (Python) | `slow` | [Indicator-Ppo.md](indicators/momentum/Indicator-Ppo.md) |
| `Dpo`               | Detrended Price Oscillator; `price[t − period/2 − 1] − SMA(period)`. | `f64` | `f64` | unbounded around zero | `period = 20` (Python) | `max(period, period/2 + 2)` | [Indicator-Dpo.md](indicators/momentum/Indicator-Dpo.md) |
| `Coppock`           | Coppock Curve; `WMA(ROC(long) + ROC(short), wma_period)`. | `f64` | `f64` | unbounded around zero | `(roc_long=14, roc_short=11, wma_period=10)` (Python) | `max(roc_long, roc_short) + wma_period` | [Indicator-Coppock.md](indicators/momentum/Indicator-Coppock.md) |

### Directional

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `Adx`     | Wilder's directional system: `+DI`, `−DI` (each `[0, 100]`) and `ADX` trend-strength index. | `Candle` | `(plus_di, minus_di, adx)` | each in `[0, 100]` | `period = 14` (Python) | `2·period` | [Indicator-Adx.md](indicators/momentum/Indicator-Adx.md) |
| `AroonOscillator` | `AroonUp − AroonDown`; the two Aroon lines as one trend gauge. | `Candle` | `f64` | `[−100, 100]` | `period = 14` (Python) | `period + 1` | [Indicator-AroonOscillator.md](indicators/momentum/Indicator-AroonOscillator.md) |
| `Vortex`  | Vortex Indicator `VI+` / `VI−`; crossings mark trend onset. | `Candle` | `(plus, minus)` | each `>= 0` | `period = 14` (Python) | `period + 1` | [Indicator-Vortex.md](indicators/momentum/Indicator-Vortex.md) |
| `MassIndex` | Dorsey's range-expansion sum of the EMA-of-range ratio. | `Candle` | `f64` | `> 0` (around `sum_period`) | `(ema_period=9, sum_period=25)` (Python) | `2·ema_period + sum_period − 2` | [Indicator-MassIndex.md](indicators/momentum/Indicator-MassIndex.md) |

## Volatility

Volatility indicators sit in three functional groups: those that draw an
envelope around price, those that report a scalar dispersion/range, and a
set of trailing stops — ATR-driven stop-loss trackers rather than width
measures — that live in the volatility module by source convention.

### Envelopes

| Indicator         | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-------------------|-----------|-------|--------|-------|----------|--------|-----------|
| `BollingerBands`  | SMA middle band with `±multiplier × population_stddev` upper/lower bands. | `f64` | `(upper, middle, lower, stddev)` | unbounded (price scale) | `(period=20, multiplier=2.0)` (Python) | `period` | [Indicator-BollingerBands.md](indicators/volatility/Indicator-BollingerBands.md) |
| `Keltner`         | EMA middle band with `±multiplier × ATR` upper/lower bands. | `Candle` | `(upper, middle, lower)` | unbounded (price scale) | `(ema_period=20, atr_period=10, multiplier=2.0)` (Python) | `max(ema_period, atr_period)` | [Indicator-Keltner.md](indicators/volatility/Indicator-Keltner.md) |
| `Donchian`        | Highest high and lowest low over `period` bars; middle = mean of the two. | `Candle` | `(upper, middle, lower)` | unbounded (price scale) | `period = 20` (Python) | `period` | [Indicator-Donchian.md](indicators/volatility/Indicator-Donchian.md) |
| `BollingerBandwidth` | `(upper − lower) / middle` of the Bollinger Bands; the "squeeze" gauge. | `f64` | `f64` | `[0, ∞)` | `(period=20, multiplier=2.0)` (Python) | `period` | [Indicator-BollingerBandwidth.md](indicators/volatility/Indicator-BollingerBandwidth.md) |
| `PercentB`        | `(price − lower) / (upper − lower)`; price position within the bands. | `f64` | `f64` | unbounded (`0`–`1` inside the bands) | `(period=20, multiplier=2.0)` (Python) | `period` | [Indicator-PercentB.md](indicators/volatility/Indicator-PercentB.md) |

### Range-average

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `Atr`     | Wilder-smoothed True Range; per-bar absolute volatility. | `Candle` | `f64` | `[0, ∞)` (price scale) | `period = 14` (Python) | `period` | [Indicator-Atr.md](indicators/volatility/Indicator-Atr.md) |
| `Natr`    | `100·ATR/close`; ATR as a percentage, comparable across instruments. | `Candle` | `f64` | `[0, ∞)` (percent) | `period = 14` (Python) | `period` | [Indicator-Natr.md](indicators/volatility/Indicator-Natr.md) |
| `StdDev`  | Rolling population standard deviation of price. | `f64` | `f64` | `[0, ∞)` (price scale) | `period = 20` (Python) | `period` | [Indicator-StdDev.md](indicators/volatility/Indicator-StdDev.md) |
| `UlcerIndex` | RMS of trailing-high drawdowns; downside-only risk. | `f64` | `f64` | `[0, ∞)` (percent) | `period = 14` (Python) | `2·period − 1` | [Indicator-UlcerIndex.md](indicators/volatility/Indicator-UlcerIndex.md) |
| `HistoricalVolatility` | Annualised sample stddev of log returns. | `f64` | `f64` | `[0, ∞)` (annualised percent) | `(period=20, trading_periods=252)` (Python) | `period + 1` | [Indicator-HistoricalVolatility.md](indicators/volatility/Indicator-HistoricalVolatility.md) |

### Trailing stop

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `Psar`    | Wilder's Parabolic Stop-and-Reverse; per-bar stop level that flips sides on price crossing. | `Candle` | `f64` | unbounded (price scale) | `(af_start=0.02, af_step=0.02, af_max=0.20)` (Python) | `2` | [Indicator-Psar.md](indicators/volatility/Indicator-Psar.md) |
| `SuperTrend` | ATR-banded trailing stop that flips on a close through the band; reports the line and the trend direction. | `Candle` | `(value, direction)` | `value` price scale; `direction` `±1` | `(atr_period=10, multiplier=3.0)` (Python) | `atr_period` | [Indicator-SuperTrend.md](indicators/volatility/Indicator-SuperTrend.md) |
| `ChandelierExit` | `highest_high − k·ATR` (long stop) and `lowest_low + k·ATR` (short stop). | `Candle` | `(long_stop, short_stop)` | unbounded (price scale) | `(period=22, multiplier=3.0)` (Python) | `period` | [Indicator-ChandelierExit.md](indicators/volatility/Indicator-ChandelierExit.md) |
| `ChandeKrollStop` | Two-stage ATR stop: an extreme-based stop, then smoothed over a shorter window. | `Candle` | `(stop_long, stop_short)` | unbounded (price scale) | `(atr_period=10, atr_multiplier=1.0, stop_period=9)` (Python) | `atr_period + stop_period − 1` | [Indicator-ChandeKrollStop.md](indicators/volatility/Indicator-ChandeKrollStop.md) |
| `AtrTrailingStop` | A single line trailing the close by `k·ATR`, ratcheting toward the trend and flipping on a cross. | `Candle` | `f64` | unbounded (price scale) | `(atr_period=14, multiplier=3.0)` (Python) | `atr_period` | [Indicator-AtrTrailingStop.md](indicators/volatility/Indicator-AtrTrailingStop.md) |

## Volume

Volume indicators all take `Candle` input because they need `close` and
`volume` together (some also need `high`/`low`).

### Cumulative

| Indicator     | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|---------------|-----------|-------|--------|-------|----------|--------|-----------|
| `Obv`         | On-Balance Volume: cumulative signed volume driven by close-vs-prior-close sign. | `Candle` | `f64` | unbounded (drifts with cumulative volume) | (no parameters) | `1` | [Indicator-Obv.md](indicators/volume/Indicator-Obv.md) |
| `Vwap`        | Cumulative volume-weighted average price from the start of the stream (intraday reset is your responsibility). | `Candle` | `f64` | unbounded (price scale) | (no parameters) | `1` | [Indicator-Vwap.md](indicators/volume/Indicator-Vwap.md) |
| `Adl`         | Accumulation/Distribution Line; cumulative range-weighted volume. | `Candle` | `f64` | unbounded (drifts with volume) | (no parameters) | `1` | [Indicator-Adl.md](indicators/volume/Indicator-Adl.md) |
| `VolumePriceTrend` | Cumulative `volume · ROC`; volume flow weighted by percentage move. | `Candle` | `f64` | unbounded (drifts with volume) | (no parameters) | `1` | [Indicator-VolumePriceTrend.md](indicators/volume/Indicator-VolumePriceTrend.md) |

### Rolling

| Indicator     | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|---------------|-----------|-------|--------|-------|----------|--------|-----------|
| `RollingVwap` | VWAP over a sliding window instead of since-start; useful for session-independent VWAP. | `Candle` | `f64` | unbounded (price scale) | `period` | `period` | [Indicator-Vwap.md → RollingVwap](indicators/volume/Indicator-Vwap.md#rollingvwap-finite-window) |

### Oscillators

Volume-flow oscillators: bounded or zero-centred readings derived from where
price closes within each bar and how much volume backed the move.

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `ChaikinMoneyFlow`  | Summed money-flow volume divided by summed volume over `period` bars. | `Candle` | `f64` | `[−1, +1]` | `period = 20` (Python) | `period` | [Indicator-ChaikinMoneyFlow.md](indicators/volume/Indicator-ChaikinMoneyFlow.md) |
| `ChaikinOscillator` | `EMA(ADL, fast) − EMA(ADL, slow)`; the MACD of the ADL. | `Candle` | `f64` | unbounded around zero | `(fast=3, slow=10)` (Python) | `slow` | [Indicator-ChaikinOscillator.md](indicators/volume/Indicator-ChaikinOscillator.md) |
| `ForceIndex`        | `EMA((close − prev_close) · volume, period)`; the conviction behind a move. | `Candle` | `f64` | unbounded around zero | `period = 13` (Python) | `period + 1` | [Indicator-ForceIndex.md](indicators/volume/Indicator-ForceIndex.md) |
| `EaseOfMovement`    | `SMA` of distance travelled per unit of volume. | `Candle` | `f64` | unbounded around zero | `(period=14, divisor=1e8)` (Python) | `period + 1` | [Indicator-EaseOfMovement.md](indicators/volume/Indicator-EaseOfMovement.md) |

## Pick the right indicator for…

A short cheat-sheet of "I want X, which indicator?" answers, grounded in
what each indicator actually computes.

- **Fast trend filter, minimal lag, single line.** `Hma` for smoothness +
  responsiveness, `Tema` for further lag reduction at the cost of more
  noise. If you want adaptiveness instead of fixed lag, `Kama`.
- **Slow trend filter, smooth as glass.** `Sma` is the simplest; `Ema`
  responds slightly faster with the same smoothness budget. For long
  trend filters either is appropriate; the difference is mostly aesthetic.
- **Trend-following crossovers.** Two-line crossovers (`Ema(fast)` vs
  `Ema(slow)`, or any of the trend pairs) are the textbook entry signal;
  `MacdIndicator` packages the same idea with a signal line and histogram.
- **Trend strength (is there a trend at all?).** `Adx` is the canonical
  answer: `adx > 25` is "trending", `adx < 20` is "ranging". `Aroon` is
  a softer alternative when you want directional confirmation.
- **Overbought / oversold reversal candidate.** `Rsi` is the default;
  `Stochastic` for faster signals; `WilliamsR` for the same logic with an
  inverted scale; `Mfi` if you have volume and want a volume-aware RSI.
- **Volatility expansion / contraction.** `BollingerBands` width
  (`upper − lower`) for relative volatility; `Atr` for absolute per-bar
  volatility in price units; `Keltner` to compare price against an
  ATR-scaled envelope.
- **Breakout level.** `Donchian` upper/lower bands are the textbook
  Turtle-style breakout trigger.
- **Trailing stop.** `Psar` gives you a per-bar stop level that flips
  sides as the trend reverses. `Atr · k` (compute `Atr` yourself, multiply
  by your preferred `k`) is the common alternative.
- **Volume confirmation.** `Obv` is the simplest; `Mfi` adds price into
  the equation; `Vwap` / `RollingVwap` give you the volume-weighted
  reference price.
- **Bill Williams setups.** `AwesomeOscillator` for the zero-line cross /
  twin-peaks pattern from his suite.
- **Rate-of-change scalar.** `Roc` is the unsmoothed percentage change;
  `Trix` is the same idea but on a triple-smoothed EMA.

## Source-of-truth files

Every claim above can be checked against the source in
[`crates/wickra-core/src/indicators/`](https://github.com/kingchenc/wickra/tree/main/crates/wickra-core/src/indicators)
— one file per indicator. The Rust unit tests inside each module are the
ground truth for sample values. Python defaults (the `period = 14` etc. in
tables above) come from the `#[pyo3(signature = …)]` attributes in
[`bindings/python/src/lib.rs`](https://github.com/kingchenc/wickra/blob/main/bindings/python/src/lib.rs);
indicators not listed with a Python default require an explicit `period`
argument.

## See also

- [Warmup Periods](Warmup-Periods.md) — full verified table of every
  indicator's `warmup_period()`.
- [Indicator Chaining](Indicator-Chaining.md) — combining indicators with
  `Chain` and the stacked-warmup rule.
- [Quickstart: Rust](Quickstart-Rust.md), [Quickstart: Python](Quickstart-Python.md),
  [Quickstart: Node](Quickstart-Node.md) — language-specific API surfaces.
- Source: <https://github.com/kingchenc/wickra>
