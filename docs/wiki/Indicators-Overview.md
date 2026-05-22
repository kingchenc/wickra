# Indicators Overview

Wickra ships **71 indicators** organised into **eight families**. Each family
collects indicators that answer the same kind of question and groups at least
five of them, so the taxonomy here maps one-to-one onto the
`docs/wiki/indicators/<family>/` directory layout.

Every indicator is an O(1) state machine that consumes one input at a time
and produces either `Option<f64>` (Rust), `float | None` (Python), or
`number | null` (Node). Inputs are either a `f64` close price or an OHLCV
`Candle` (Rust) / dict-or-tuple (Python) / column arrays (Node). The full
trait surface and warmup-period semantics are covered in
[Quickstart: Rust](Quickstart-Rust.md) and [Warmup Periods](Warmup-Periods.md).

The "Output range" column is the value bounds an indicator emits once warm;
"unbounded" means it tracks the price scale of the input. The "Warmup" column
quotes `warmup_period()` as the indicator reports it — the **exact**
first-emission index: the first non-`None` output lands on input
`warmup_period()` (0-indexed `warmup_period() - 1`).

The eight families:

| # | Family | Count | What it answers |
|---|--------|-------|-----------------|
| 1 | [Moving Averages](#moving-averages) | 12 | Where is the smoothed trend line? |
| 2 | [Momentum Oscillators](#momentum-oscillators) | 13 | How fast is price changing; is it overbought? |
| 3 | [Trend & Directional](#trend--directional) | 9 | Is there a trend, and which way? |
| 4 | [Price Oscillators](#price-oscillators) | 5 | Difference-of-averages momentum around zero. |
| 5 | [Volatility & Bands](#volatility--bands) | 12 | How wide is the range; where are the envelopes? |
| 6 | [Trailing Stops](#trailing-stops) | 5 | Where is the stop-loss for this trend? |
| 7 | [Volume](#volume) | 9 | Is volume confirming the move? |
| 8 | [Price Statistics](#price-statistics) | 7 | Per-bar price transforms and rolling regressions. |

## Moving Averages

Smooth the price series to surface direction. All are single-input,
single-output (`f64 → f64`) except `Vwma`, which weights by volume.

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `Sma`   | Equal-weighted rolling mean over `period` closes. | `f64` | `f64` | unbounded (price scale) | `period` | `period` | [Indicator-Sma.md](indicators/moving-averages/Indicator-Sma.md) |
| `Ema`   | EMA with `α = 2 / (period + 1)`, SMA-seeded. | `f64` | `f64` | unbounded (price scale) | `period` | `period` | [Indicator-Ema.md](indicators/moving-averages/Indicator-Ema.md) |
| `Wma`   | Linear weights `1, 2, …, period`; newest bar matters most. | `f64` | `f64` | unbounded (price scale) | `period` | `period` | [Indicator-Wma.md](indicators/moving-averages/Indicator-Wma.md) |
| `Dema`  | Mulloy's `2·EMA − EMA(EMA)`; removes first-order EMA lag. | `f64` | `f64` | unbounded (price scale) | `period` | `2·period − 1` | [Indicator-Dema.md](indicators/moving-averages/Indicator-Dema.md) |
| `Tema`  | Mulloy's `3·EMA − 3·EMA(EMA) + EMA(EMA(EMA))`. | `f64` | `f64` | unbounded (price scale) | `period` | `3·period − 2` | [Indicator-Tema.md](indicators/moving-averages/Indicator-Tema.md) |
| `Hma`   | Hull's near-zero-lag `WMA(2·WMA(n/2) − WMA(n), √n)`. | `f64` | `f64` | unbounded (price scale) | `period` | `period + round(√period) − 1` | [Indicator-Hma.md](indicators/moving-averages/Indicator-Hma.md) |
| `Kama`  | Kaufman's adaptive average; efficiency ratio picks α per bar. | `f64` | `f64` | unbounded (price scale) | `(er_period=10, fast=2, slow=30)` | `er_period + 1` | [Indicator-Kama.md](indicators/moving-averages/Indicator-Kama.md) |
| `Smma`  | Wilder's RMA: SMA-seeded exponential average, `1/period` factor. | `f64` | `f64` | unbounded (price scale) | `period` | `period` | [Indicator-Smma.md](indicators/moving-averages/Indicator-Smma.md) |
| `Trima` | A `period`-window SMA applied twice; triangular weights. | `f64` | `f64` | unbounded (price scale) | `period` | `period` | [Indicator-Trima.md](indicators/moving-averages/Indicator-Trima.md) |
| `Zlema` | EMA of the de-lagged series `2·price − price[lag]`. | `f64` | `f64` | unbounded (price scale) | `period` | `lag + period` | [Indicator-Zlema.md](indicators/moving-averages/Indicator-Zlema.md) |
| `T3`    | Tillson's six-EMA cascade recombined with a volume factor `v`. | `f64` | `f64` | unbounded (price scale) | `(period, v=0.7)` (Python) | `6·period − 5` | [Indicator-T3.md](indicators/moving-averages/Indicator-T3.md) |
| `Vwma`  | Rolling mean of closes weighted by each bar's volume. | `Candle` | `f64` | unbounded (price scale) | `period` | `period` | [Indicator-Vwma.md](indicators/moving-averages/Indicator-Vwma.md) |

## Momentum Oscillators

Measure the *rate* of price change. Several are bounded by construction
(0–100 / ±100 oscillators), the rest are difference-driven.

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `Rsi`        | Wilder's RSI; smoothed `gain / (gain + loss) × 100`. | `f64` | `f64` | `[0, 100]` | `period = 14` (Python) | `period + 1` | [Indicator-Rsi.md](indicators/momentum-oscillators/Indicator-Rsi.md) |
| `Stochastic` | `%K = (close − low_n)/(high_n − low_n) × 100`, smoothed into `%D`. | `Candle` | `(k, d)` | each in `[0, 100]` | `(k_period=14, d_period=3)` (Python) | `k_period + d_period − 1` | [Indicator-Stochastic.md](indicators/momentum-oscillators/Indicator-Stochastic.md) |
| `Cci`        | `(typical − SMA(typical)) / (0.015 · mean_dev)`. | `Candle` | `f64` | unbounded (typically `±100`–`±200`) | `period = 20` (Python) | `period` | [Indicator-Cci.md](indicators/momentum-oscillators/Indicator-Cci.md) |
| `Roc`        | `(price − price_n) / price_n × 100`; raw percentage change. | `f64` | `f64` | unbounded around zero | `period` | `period + 1` | [Indicator-Roc.md](indicators/momentum-oscillators/Indicator-Roc.md) |
| `WilliamsR`  | `−100 × (high_n − close) / (high_n − low_n)`. | `Candle` | `f64` | `[−100, 0]` | `period = 14` (Python) | `period` | [Indicator-WilliamsR.md](indicators/momentum-oscillators/Indicator-WilliamsR.md) |
| `Mfi`        | "Volume-weighted RSI": Wilder smoothing of money-flow ratios. | `Candle` | `f64` | `[0, 100]` | `period = 14` (Python) | `period` | [Indicator-Mfi.md](indicators/momentum-oscillators/Indicator-Mfi.md) |
| `AwesomeOscillator` | `SMA(median, fast) − SMA(median, slow)`; zero-line crossover. | `Candle` | `f64` | unbounded around zero | `(fast=5, slow=34)` (Python) | `slow_period` | [Indicator-AwesomeOscillator.md](indicators/momentum-oscillators/Indicator-AwesomeOscillator.md) |
| `Mom`        | `price − price[period]`; raw price-difference momentum. | `f64` | `f64` | unbounded around zero | `period = 10` (Python) | `period + 1` | [Indicator-Mom.md](indicators/momentum-oscillators/Indicator-Mom.md) |
| `Cmo`        | Chande Momentum Oscillator; `100·(Σgain − Σloss)/(Σgain + Σloss)`. | `f64` | `f64` | `[−100, 100]` | `period = 14` (Python) | `period + 1` | [Indicator-Cmo.md](indicators/momentum-oscillators/Indicator-Cmo.md) |
| `Tsi`        | True Strength Index; double-EMA-smoothed momentum ratio. | `f64` | `f64` | ≈ `[−100, 100]` | `(long=25, short=13)` (Python) | `long + short` | [Indicator-Tsi.md](indicators/momentum-oscillators/Indicator-Tsi.md) |
| `Pmo`        | DecisionPoint Price Momentum Oscillator; doubly-smoothed ROC. | `f64` | `f64` | unbounded around zero | `(smoothing1=35, smoothing2=20)` (Python) | `2` | [Indicator-Pmo.md](indicators/momentum-oscillators/Indicator-Pmo.md) |
| `StochRsi`   | Stochastic Oscillator applied to the RSI series. | `f64` | `f64` | `[0, 100]` | `(rsi_period=14, stoch_period=14)` (Python) | `rsi_period + stoch_period` | [Indicator-StochRsi.md](indicators/momentum-oscillators/Indicator-StochRsi.md) |
| `UltimateOscillator` | Larry Williams' weighted three-timeframe buying-pressure oscillator. | `Candle` | `f64` | `[0, 100]` | `(short=7, mid=14, long=28)` (Python) | `max(short,mid,long) + 1` | [Indicator-UltimateOscillator.md](indicators/momentum-oscillators/Indicator-UltimateOscillator.md) |

## Trend & Directional

Answer whether a trend exists and which way it points — directional systems,
crossover packages and trend-versus-range filters.

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `MacdIndicator` | `EMA(fast) − EMA(slow)` plus a signal EMA and the histogram. | `f64` | `(macd, signal, histogram)` | unbounded around zero | `(fast=12, slow=26, signal=9)` (Python) | `slow + signal − 1` | [Indicator-MacdIndicator.md](indicators/trend-directional/Indicator-MacdIndicator.md) |
| `Adx`     | Wilder's directional system: `+DI`, `−DI` and the `ADX` strength index. | `Candle` | `(plus_di, minus_di, adx)` | each in `[0, 100]` | `period = 14` (Python) | `2·period` | [Indicator-Adx.md](indicators/trend-directional/Indicator-Adx.md) |
| `Aroon`   | Bars-since-high and bars-since-low scaled to `[0, 100]`. | `Candle` | `(up, down)` | each in `[0, 100]` | `period = 14` (Python) | `period + 1` | [Indicator-Aroon.md](indicators/trend-directional/Indicator-Aroon.md) |
| `Trix`    | Rate of change of a triple-smoothed EMA, `× 10000`. | `f64` | `f64` | unbounded around zero | `period = 15` (Python) | `3·period − 1` | [Indicator-Trix.md](indicators/trend-directional/Indicator-Trix.md) |
| `AroonOscillator` | `AroonUp − AroonDown`; the two Aroon lines as one gauge. | `Candle` | `f64` | `[−100, 100]` | `period = 14` (Python) | `period + 1` | [Indicator-AroonOscillator.md](indicators/trend-directional/Indicator-AroonOscillator.md) |
| `Vortex`  | Vortex Indicator `VI+` / `VI−`; crossings mark trend onset. | `Candle` | `(plus, minus)` | each `>= 0` | `period = 14` (Python) | `period + 1` | [Indicator-Vortex.md](indicators/trend-directional/Indicator-Vortex.md) |
| `MassIndex` | Dorsey's range-expansion sum of the EMA-of-range ratio. | `Candle` | `f64` | `> 0` | `(ema_period=9, sum_period=25)` (Python) | `2·ema_period + sum_period − 2` | [Indicator-MassIndex.md](indicators/trend-directional/Indicator-MassIndex.md) |
| `ChoppinessIndex` | Summed true range over the high-low span, log-scaled. | `Candle` | `f64` | `[0, 100]` | `period = 14` (Python) | `period` | [Indicator-ChoppinessIndex.md](indicators/trend-directional/Indicator-ChoppinessIndex.md) |
| `VerticalHorizontalFilter` | Net price move divided by total move over `period`. | `f64` | `f64` | `[0, 1]` | `period = 28` (Python) | `period + 1` | [Indicator-VerticalHorizontalFilter.md](indicators/trend-directional/Indicator-VerticalHorizontalFilter.md) |

## Price Oscillators

Difference-of-averages and intrabar oscillators that swing around a zero line.

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `Ppo`     | Percentage Price Oscillator; `100·(EMA_fast − EMA_slow)/EMA_slow`. | `f64` | `f64` | unbounded around zero (percent) | `(fast=12, slow=26)` (Python) | `slow` | [Indicator-Ppo.md](indicators/price-oscillators/Indicator-Ppo.md) |
| `Dpo`     | Detrended Price Oscillator; `price[t − period/2 − 1] − SMA(period)`. | `f64` | `f64` | unbounded around zero | `period = 20` (Python) | `max(period, period/2 + 2)` | [Indicator-Dpo.md](indicators/price-oscillators/Indicator-Dpo.md) |
| `Coppock` | Coppock Curve; `WMA(ROC(long) + ROC(short), wma_period)`. | `f64` | `f64` | unbounded around zero | `(roc_long=14, roc_short=11, wma_period=10)` (Python) | `max(roc_long, roc_short) + wma_period` | [Indicator-Coppock.md](indicators/price-oscillators/Indicator-Coppock.md) |
| `AcceleratorOscillator` | `AO − SMA(AO, signal)`; the acceleration of momentum. | `Candle` | `f64` | unbounded around zero | `(ao_fast=5, ao_slow=34, signal_period=5)` (Python) | `ao_slow + signal_period − 1` | [Indicator-AcceleratorOscillator.md](indicators/price-oscillators/Indicator-AcceleratorOscillator.md) |
| `BalanceOfPower` | `(close − open) / (high − low)`; intrabar buyer/seller control. | `Candle` | `f64` | `[−1, +1]` | (no parameters) | `1` | [Indicator-BalanceOfPower.md](indicators/price-oscillators/Indicator-BalanceOfPower.md) |

## Volatility & Bands

Indicators that measure dispersion / range and those that draw an envelope
around price.

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `Atr`     | Wilder-smoothed True Range; per-bar absolute volatility. | `Candle` | `f64` | `[0, ∞)` (price scale) | `period = 14` (Python) | `period` | [Indicator-Atr.md](indicators/volatility-bands/Indicator-Atr.md) |
| `BollingerBands` | SMA middle band with `±multiplier × population_stddev` bands. | `f64` | `(upper, middle, lower, stddev)` | unbounded (price scale) | `(period=20, multiplier=2.0)` (Python) | `period` | [Indicator-BollingerBands.md](indicators/volatility-bands/Indicator-BollingerBands.md) |
| `Keltner` | EMA middle band with `±multiplier × ATR` bands. | `Candle` | `(upper, middle, lower)` | unbounded (price scale) | `(ema_period=20, atr_period=10, multiplier=2.0)` (Python) | `max(ema_period, atr_period)` | [Indicator-Keltner.md](indicators/volatility-bands/Indicator-Keltner.md) |
| `Donchian` | Highest high and lowest low over `period` bars. | `Candle` | `(upper, middle, lower)` | unbounded (price scale) | `period = 20` (Python) | `period` | [Indicator-Donchian.md](indicators/volatility-bands/Indicator-Donchian.md) |
| `Natr`    | `100·ATR/close`; ATR as a percentage. | `Candle` | `f64` | `[0, ∞)` (percent) | `period = 14` (Python) | `period` | [Indicator-Natr.md](indicators/volatility-bands/Indicator-Natr.md) |
| `StdDev`  | Rolling population standard deviation of price. | `f64` | `f64` | `[0, ∞)` (price scale) | `period = 20` (Python) | `period` | [Indicator-StdDev.md](indicators/volatility-bands/Indicator-StdDev.md) |
| `UlcerIndex` | RMS of trailing-high drawdowns; downside-only risk. | `f64` | `f64` | `[0, ∞)` (percent) | `period = 14` (Python) | `2·period − 1` | [Indicator-UlcerIndex.md](indicators/volatility-bands/Indicator-UlcerIndex.md) |
| `HistoricalVolatility` | Annualised sample stddev of log returns. | `f64` | `f64` | `[0, ∞)` (annualised percent) | `(period=20, trading_periods=252)` (Python) | `period + 1` | [Indicator-HistoricalVolatility.md](indicators/volatility-bands/Indicator-HistoricalVolatility.md) |
| `BollingerBandwidth` | `(upper − lower) / middle` of the Bollinger Bands. | `f64` | `f64` | `[0, ∞)` | `(period=20, multiplier=2.0)` (Python) | `period` | [Indicator-BollingerBandwidth.md](indicators/volatility-bands/Indicator-BollingerBandwidth.md) |
| `PercentB` | `(price − lower) / (upper − lower)`; price position in the bands. | `f64` | `f64` | unbounded (`0`–`1` inside) | `(period=20, multiplier=2.0)` (Python) | `period` | [Indicator-PercentB.md](indicators/volatility-bands/Indicator-PercentB.md) |
| `TrueRange` | `max(H−L, |H−prevC|, |L−prevC|)`; raw single-bar volatility. | `Candle` | `f64` | `[0, ∞)` (price scale) | (no parameters) | `1` | [Indicator-TrueRange.md](indicators/volatility-bands/Indicator-TrueRange.md) |
| `ChaikinVolatility` | Rate of change of an EMA-smoothed high-low spread. | `Candle` | `f64` | unbounded around zero (percent) | `(ema_period=10, roc_period=10)` (Python) | `ema_period + roc_period` | [Indicator-ChaikinVolatility.md](indicators/volatility-bands/Indicator-ChaikinVolatility.md) |

## Trailing Stops

ATR-driven stop-loss trackers: per-bar levels that follow a trend and flip
when price closes through them.

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `Psar`    | Wilder's Parabolic Stop-and-Reverse; flips sides on a crossing. | `Candle` | `f64` | unbounded (price scale) | `(af_start=0.02, af_step=0.02, af_max=0.20)` (Python) | `2` | [Indicator-Psar.md](indicators/trailing-stops/Indicator-Psar.md) |
| `SuperTrend` | ATR-banded trailing stop with explicit flip logic. | `Candle` | `(value, direction)` | `value` price scale; `direction` `±1` | `(atr_period=10, multiplier=3.0)` (Python) | `atr_period` | [Indicator-SuperTrend.md](indicators/trailing-stops/Indicator-SuperTrend.md) |
| `ChandelierExit` | `highest_high − k·ATR` (long) and `lowest_low + k·ATR` (short). | `Candle` | `(long_stop, short_stop)` | unbounded (price scale) | `(period=22, multiplier=3.0)` (Python) | `period` | [Indicator-ChandelierExit.md](indicators/trailing-stops/Indicator-ChandelierExit.md) |
| `ChandeKrollStop` | Two-stage ATR stop: extreme-based, then smoothed. | `Candle` | `(stop_long, stop_short)` | unbounded (price scale) | `(atr_period=10, atr_multiplier=1.0, stop_period=9)` (Python) | `atr_period + stop_period − 1` | [Indicator-ChandeKrollStop.md](indicators/trailing-stops/Indicator-ChandeKrollStop.md) |
| `AtrTrailingStop` | A single line trailing the close by `k·ATR`, ratcheting. | `Candle` | `f64` | unbounded (price scale) | `(atr_period=14, multiplier=3.0)` (Python) | `atr_period` | [Indicator-AtrTrailingStop.md](indicators/trailing-stops/Indicator-AtrTrailingStop.md) |

## Volume

Price moves weighted or confirmed by traded volume. All take `Candle` input.

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `Obv`     | On-Balance Volume: cumulative signed volume. | `Candle` | `f64` | unbounded (drifts with volume) | (no parameters) | `1` | [Indicator-Obv.md](indicators/volume/Indicator-Obv.md) |
| `Vwap`    | Cumulative volume-weighted average price from the stream start. | `Candle` | `f64` | unbounded (price scale) | (no parameters) | `1` | [Indicator-Vwap.md](indicators/volume/Indicator-Vwap.md) |
| `RollingVwap` | VWAP over a sliding window instead of since-start. | `Candle` | `f64` | unbounded (price scale) | `period` | `period` | [Indicator-Vwap.md → RollingVwap](indicators/volume/Indicator-Vwap.md#rollingvwap-finite-window) |
| `Adl`     | Accumulation/Distribution Line; cumulative range-weighted volume. | `Candle` | `f64` | unbounded (drifts with volume) | (no parameters) | `1` | [Indicator-Adl.md](indicators/volume/Indicator-Adl.md) |
| `VolumePriceTrend` | Cumulative `volume · ROC`; volume weighted by percentage move. | `Candle` | `f64` | unbounded (drifts with volume) | (no parameters) | `1` | [Indicator-VolumePriceTrend.md](indicators/volume/Indicator-VolumePriceTrend.md) |
| `ChaikinMoneyFlow` | Summed money-flow volume over summed volume across `period` bars. | `Candle` | `f64` | `[−1, +1]` | `period = 20` (Python) | `period` | [Indicator-ChaikinMoneyFlow.md](indicators/volume/Indicator-ChaikinMoneyFlow.md) |
| `ChaikinOscillator` | `EMA(ADL, fast) − EMA(ADL, slow)`; the MACD of the ADL. | `Candle` | `f64` | unbounded around zero | `(fast=3, slow=10)` (Python) | `slow` | [Indicator-ChaikinOscillator.md](indicators/volume/Indicator-ChaikinOscillator.md) |
| `ForceIndex` | `EMA((close − prev_close) · volume, period)`. | `Candle` | `f64` | unbounded around zero | `period = 13` (Python) | `period + 1` | [Indicator-ForceIndex.md](indicators/volume/Indicator-ForceIndex.md) |
| `EaseOfMovement` | `SMA` of distance travelled per unit of volume. | `Candle` | `f64` | unbounded around zero | `(period=14, divisor=1e8)` (Python) | `period + 1` | [Indicator-EaseOfMovement.md](indicators/volume/Indicator-EaseOfMovement.md) |

## Price Statistics

Per-bar price transforms and rolling least-squares regressions.

| Indicator | One-liner | Input | Output | Range | Defaults | Warmup | Deep dive |
|-----------|-----------|-------|--------|-------|----------|--------|-----------|
| `TypicalPrice`  | `(high + low + close) / 3`. | `Candle` | `f64` | unbounded (price scale) | (no parameters) | `1` | [Indicator-TypicalPrice.md](indicators/price-statistics/Indicator-TypicalPrice.md) |
| `MedianPrice`   | `(high + low) / 2`. | `Candle` | `f64` | unbounded (price scale) | (no parameters) | `1` | [Indicator-MedianPrice.md](indicators/price-statistics/Indicator-MedianPrice.md) |
| `WeightedClose` | `(high + low + 2·close) / 4`. | `Candle` | `f64` | unbounded (price scale) | (no parameters) | `1` | [Indicator-WeightedClose.md](indicators/price-statistics/Indicator-WeightedClose.md) |
| `LinearRegression` | Endpoint of the rolling least-squares line. | `f64` | `f64` | unbounded (price scale) | `period = 14` (Python) | `period` | [Indicator-LinearRegression.md](indicators/price-statistics/Indicator-LinearRegression.md) |
| `LinRegSlope`   | Slope of the rolling least-squares line. | `f64` | `f64` | unbounded around zero | `period = 14` (Python) | `period` | [Indicator-LinRegSlope.md](indicators/price-statistics/Indicator-LinRegSlope.md) |
| `ZScore`        | `(price − SMA(n)) / population_stddev(n)`. | `f64` | `f64` | unbounded around zero | `period = 20` (Python) | `period` | [Indicator-ZScore.md](indicators/price-statistics/Indicator-ZScore.md) |
| `LinRegAngle`   | The rolling regression slope as a degree angle. | `f64` | `f64` | `(−90°, +90°)` | `period = 14` (Python) | `period` | [Indicator-LinRegAngle.md](indicators/price-statistics/Indicator-LinRegAngle.md) |

## Pick the right indicator for…

A short cheat-sheet of "I want X, which indicator?" answers, grounded in
what each indicator actually computes.

- **Fast trend filter, minimal lag.** `Hma` for smoothness + responsiveness,
  `Tema` for further lag reduction at the cost of noise, `Kama` for
  adaptiveness instead of fixed lag.
- **Slow trend filter.** `Sma` is the simplest; `Ema` responds slightly
  faster with the same smoothness budget.
- **Trend-following crossovers.** Two-line crossovers are the textbook entry;
  `MacdIndicator` packages the idea with a signal line and histogram.
- **Trend strength — is there a trend at all?** `Adx` (`> 25` trending,
  `< 20` ranging); `ChoppinessIndex` / `VerticalHorizontalFilter` answer the
  same question without a direction.
- **Overbought / oversold.** `Rsi` is the default; `Stochastic` for faster
  signals; `WilliamsR` for an inverted scale; `Mfi` for a volume-aware RSI.
- **Volatility level vs. momentum.** `Atr` / `TrueRange` for the level;
  `ChaikinVolatility` for whether ranges are expanding or contracting.
- **Breakout level.** `Donchian` upper/lower bands are the Turtle-style
  trigger.
- **Trailing stop.** `Psar`, `SuperTrend`, `ChandelierExit`,
  `ChandeKrollStop` and `AtrTrailingStop` are a whole family of them.
- **Volume confirmation.** `Obv` is the simplest; `ChaikinMoneyFlow` is a
  bounded balance; `Vwap` / `RollingVwap` give a volume-weighted reference.
- **Mean reversion.** `ZScore` flags statistically stretched prices;
  `BollingerBandwidth` / `PercentB` locate price within the bands.

## Source-of-truth files

Every claim above can be checked against the source in
[`crates/wickra-core/src/indicators/`](https://github.com/kingchenc/wickra/tree/main/crates/wickra-core/src/indicators)
— one file per indicator. The Rust unit tests inside each module are the
ground truth for sample values. Python defaults (the `period = 14` etc.) come
from the `#[pyo3(signature = …)]` attributes in
[`bindings/python/src/lib.rs`](https://github.com/kingchenc/wickra/blob/main/bindings/python/src/lib.rs);
indicators without a Python default require an explicit argument.

## See also

- [Warmup Periods](Warmup-Periods.md) — verified table of every indicator's
  `warmup_period()`.
- [Indicator Chaining](Indicator-Chaining.md) — combining indicators with
  `Chain` and the stacked-warmup rule.
- [Quickstart: Rust](Quickstart-Rust.md), [Quickstart: Python](Quickstart-Python.md),
  [Quickstart: Node](Quickstart-Node.md) — language-specific API surfaces.
- Source: <https://github.com/kingchenc/wickra>
