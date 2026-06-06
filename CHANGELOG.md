# Changelog

All notable changes to Wickra are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
- **Modified MA Stop** — Modified MA Stop — SMMA-ratcheted trailing stop with directional flip (`MODIFIED_MA_STOP`).
- **Time-Based Stop** — Time-Based Stop — bar-count timer that fires after a fixed holding period (`TIME_BASED_STOP`).
- **NRTR** — NRTR (Nick Rypock Trailing Reverse) — percentage trailing-reverse stop (`NRTR`).
- **ATR Ratchet** — ATR Ratchet — Kaufman per-bar tightening volatility trailing stop (`ATR_RATCHET`).
- **Elder SafeZone** — Elder SafeZone Stop — average noise-penetration trailing stop with directional flip (`ELDER_SAFE_ZONE`).
- **Kase DevStop** — Kase DevStop volatility trailing stop using standard-deviation of two-bar true range (`KASE_DEV_STOP`).

## [0.6.1] - 2026-06-07
- **Projection Oscillator** — Widner projection oscillator: close position inside the projection bands, scaled 0..100 (`ProjectionOscillator`).
- **Projection Bands** — Widner projection bands: forward-projected high/low regression envelope (`ProjectionBands`).
- **Median Channel** — robust median +/- multiplier*MAD envelope (`MedianChannel`).
- **Bomar Bands** — adaptive percentage bands containing a target coverage fraction of recent closes (`BomarBands`).
- **Quartile Bands** — rolling 25th/50th/75th-percentile (Q1/median/Q3) envelope (`QuartileBands`).

## [0.6.0] - 2026-06-06
- **Volatility Cone** — volatility cone: current realized volatility within its historical min/median/max envelope (`VolatilityCone`).
- **VolatilityRatio** — Schwager's volatility ratio: true range over the EMA of prior true ranges (`VolatilityRatio`).
- **BipowerVariation** — jump-robust realized bipower variation (pi/2 sum of adjacent absolute log-return products) (`BipowerVariation`).
- **VolatilityOfVolatility** — vol-of-vol: sample stddev of a rolling realized-volatility series (`VolatilityOfVolatility`).
- **Garch11** — GARCH(1,1) conditional volatility with a long-run-variance anchor (`Garch11`).
- **EwmaVolatility** — RiskMetrics exponentially-weighted volatility of log returns (lambda decay) (`EwmaVolatility`).

## [0.5.9] - 2026-06-06

### Added

- Internal Rust cross-library benchmark harness (`crates/wickra-bench`, not
  published) comparing Wickra against `kand`, `ta-rs` and `yata` on an identical
  candle series in both streaming and batch modes; wired into the nightly
  `cross-library-bench` workflow.
- `tulipy` runners and expanded per-tick streaming coverage (SMA, EMA, RSI,
  MACD, Bollinger) in the Python `compare_libraries` benchmark.

### Changed

- Faster streaming and batch updates for SMA, Bollinger Bands, RSI, EMA and ATR
  (flat ring buffers replacing `VecDeque`, hoisted reciprocals in the Wilder
  smoothing, leaner hot state) — indicator outputs are unchanged.
- Rewrote the README benchmark section into honest, tiered tables (Rust core vs
  the other Rust crates, and Python vs the Python ecosystem) that show where
  Wickra wins and where it loses, not only the favourable comparisons.

## [0.5.8] - 2026-06-04
- **TSF Oscillator** — the percentage gap of the close to the one-bar-ahead time-series forecast, a close-relative companion to CFO (`TsfOscillator`).
- **MACD Histogram** — the standalone macd-minus-signal bar of MACD as a scalar series (`MacdHistogram`).
- **PPO Histogram** — the Percentage Price Oscillator with its signal EMA and the resulting zero-centered histogram (`PpoHistogram`).

## [0.5.7] - 2026-06-04
- **Qstick** — Qstick (Chande), the SMA of the candle body (close − open) as a net buying/selling pressure gauge (`QSTICK`).
- **TTM Trend** — TTM Trend (John Carter), +1/−1 by whether the close sits above the SMA of recent median prices (`TTM_TREND`).
- **Trend Strength Index** — trend strength index, the signed r² of a linear regression of price against time (`TREND_STRENGTH_INDEX`).
- **Polarized Fractal Efficiency** — polarized fractal efficiency (Hannula), directional trend efficiency over a fractal lookback (`POLARIZED_FRACTAL_EFFICIENCY`).
- **Wave PM** — Wave PM (Kase), a variance-normalised peak-momentum statistic (`WAVE_PM`).
- **Gator Oscillator** — Gator Oscillator (Bill Williams), the Alligator convergence/divergence histogram (`GATOR_OSCILLATOR`).
- **Kase Permission Stochastic** — Kase Permission Stochastic, a double-smoothed stochastic used as a trade-permission filter (`KASE_PERMISSION_STOCHASTIC`).

## [0.5.6] - 2026-06-04
- **QQE** — quantitative qualitative estimation, a smoothed RSI with an ATR-of-RSI trailing line (`QQE`).
- **Intraday Momentum Index** — intraday momentum index (Chande), RSI on the open-to-close body (`IMI`).
- **Elder Ray** — Elder Ray bull power and bear power around an EMA of close (`ElderRay`).
- **Derivative Oscillator** — derivative oscillator (Constance Brown), a double-smoothed RSI histogram (`DerivativeOscillator`).
- **RMI** — relative momentum index (RMI), RSI over a multi-bar momentum lookback (`RMI`).
- **Stochastic CCI** — stochastic CCI, a stochastic oscillator over the CCI (`StochasticCCI`).
- **Dynamic Momentum Index** — dynamic momentum index (Chande), a volatility-adaptive RSI (`DynamicMomentumIndex`).
- **RSX** — RSX, a Jurik-style three-stage smoothed RSI (`RSX`).
- **Fisher RSI** — Fisher RSI, the Fisher transform of a normalised RSI (`FisherRSI`).
- **Disparity Index** — disparity index, the percent gap between price and its moving average (`DisparityIndex`).

## [0.5.5] - 2026-06-04
- **GD** — generalized DEMA (GD), Tillson's volume-factor double EMA and the building block of T3 (`GD`).
- **GMA** — geometric moving average (GMA), the rolling geometric mean of prices (`GMA`).
- **Holt-Winters** — Holt's linear (double exponential) smoothing with level and trend components (`HoltWinters`).
- **Adaptive Laguerre** — Ehlers adaptive Laguerre filter with median-error-adaptive gamma (`AdaptiveLaguerre`).
- **Median MA** — median moving average, the rolling median of prices (`MedianMA`).
- **EHMA** — exponential Hull moving average (EHMA), the Hull construction built from EMAs (`EHMA`).
- **SWMA** — sine-weighted moving average (SWMA), a symmetric half-cycle sine window (`SWMA`).

## [0.5.4] - 2026-06-04
- **Roll Measure** — effective spread implied by the negative serial covariance of trade-price changes (Roll 1984) (`RollMeasure`).
- **Amihud Illiquidity** — average absolute log return per unit of traded value (price-impact liquidity proxy, Amihud 2002) (`AmihudIlliquidity`).
- **VPIN** — volume-synchronised probability of informed trading (volume-bucketed order-flow toxicity) (`Vpin`).
- **Order Flow Imbalance** — rolling sum of best-level order-flow events (Cont-Kukanov-Stoikov OFI) (`OrderFlowImbalance`).
- **Expectancy** — expected return per unit of average loss (R-multiple) over a rolling window of returns (`Expectancy`).
- **Win Rate** — fraction of strictly-positive returns over a rolling window (`WinRate`).
- **Regime Label** — volatility-quantile regime classification: −1 calm / 0 normal / +1 stressed, by where the rolling volatility sits in its own recent distribution (`RegimeLabel`).
- **Jump Indicator** — flags return outliers beyond `threshold ×` trailing return volatility (−1 down / 0 / +1 up) (`JumpIndicator`).
- **Trend Label** — discrete trend state from the sign of the rolling least-squares slope (−1 / 0 / +1) (`TrendLabel`).
- **High-Low Range** — bar high-low range as a fraction of close (scale-free per-bar volatility) (`HighLowRange`).
- **Wick Ratio** — signed upper-vs-lower shadow imbalance as a fraction of the range (`WickRatio`).
- **Body Size Percent** — absolute candle body as a fraction of the bar range (`BodySizePct`).
- **Close vs Open** — signed body as a fraction of the open price, `(close − open) / open` (`CloseVsOpen`).
- **Spread AR(1) Coefficient** — first-order autoregression coefficient of the spread `a − b` (direct cointegration / mean-reversion strength) (`SpreadAr1Coefficient`).
- **Rolling Quantile** — interpolated q-th quantile over a trailing window (type-7 / NumPy default) (`RollingQuantile`).
- **Rolling Percentile Rank** — percentile rank of the latest value within its trailing window (`RollingPercentileRank`).
- **Rolling IQR** — interquartile range (Q3 − Q1) over a trailing window (robust dispersion) (`RollingIqr`).
- **Realized Volatility** — square root of the summed squared log returns (raw, un-annualised quadratic variation) (`RealizedVolatility`).
- **Log Return** — logarithmic return over a fixed lag, `ln(price_t / price_{t−period})` (`LogReturn`).

## [0.5.3] - 2026-06-04
- **Fibonacci Time Zones** — vertical markers at Fibonacci bar-distances (1/2/3/5/8/...) from the latest swing pivot (`FIB_TIME_ZONES`).
- **Fibonacci Channel** — a sloped base trendline plus parallel lines at Fibonacci multiples of the channel width (`FIB_CHANNEL`).
- **Fibonacci Arcs** — semicircular retracement levels centred on the swing end, normalised by leg bar-width (`FIB_ARCS`).
- **Fibonacci Fan** — three trendlines fanning from a swing start through its 38.2/50/61.8% retracement levels (`FIB_FAN`).
- **Fibonacci Confluence** — densest cluster of retracement levels across recent swing legs (price + strength) (`FIB_CONFLUENCE`).
- **Golden Pocket** — the 0.618-0.65 optimal-trade-entry band of the most recent swing leg (`GOLDEN_POCKET`).
- **Auto-Fibonacci** — retracement anchored on the dominant (largest-magnitude) leg among recent swings (`AUTO_FIB`).
- **Fibonacci Projection** — measured-move target zone from the last three pivots (A-B-C), projecting A->B from C (`FIB_PROJECTION`).
- **Fibonacci Extension** — projects the latest swing leg to the canonical extension ratios (127.2/141.4/161.8/200/261.8%) (`FIB_EXTENSION`).
- **Fibonacci Retracement** — seven retracement levels (0/23.6/38.2/50/61.8/78.6/100%) of the most recent confirmed swing leg (`FIB_RETRACEMENT`).

## [0.5.2] - 2026-06-03

### Added
- **Three Drives** — three symmetric drives with extension legs; bullish +1, bearish -1 (`THREE_DRIVES`).
- **Cypher** — five-point harmonic whose D retraces XC by 0.786; bullish +1, bearish -1 (`CYPHER`).
- **Shark** — five-point harmonic with an expansion leg and 0.886-1.13 D; bullish +1, bearish -1 (`SHARK`).
- **Crab** — five-point harmonic with the deepest (1.618 XA) D completion; bullish +1, bearish -1 (`CRAB`).
- **Bat** — five-point harmonic with a shallow B and 0.886 D completion; bullish +1, bearish -1 (`BAT`).
- **Butterfly** — five-point harmonic with an extended (1.27-1.618 XA) D; bullish +1, bearish -1 (`BUTTERFLY`).
- **Gartley** — five-point harmonic with a 0.786 D completion; bullish +1, bearish -1 (`GARTLEY`).
- **AB=CD** — four-point AB=CD harmonic: BC retraces AB, CD mirrors AB; bullish +1, bearish -1 (`ABCD`).
- **Cup and Handle** — rounded base with a shallow handle near the rim; bullish +1, inverse -1 (`CUP_AND_HANDLE`).
- **Rectangle / Range** — flat support and resistance; mean-reversion signal off the just-touched boundary; support +1, resistance -1 (`RECTANGLE_RANGE`).
- **Flag / Pennant** — shallow consolidation against a sharp pole; continuation in the pole direction; bull +1, bear -1 (`FLAG_PENNANT`).
- **Wedge (rising/falling)** — both trendlines slope the same way but converge; rising wedge -1, falling wedge +1 (`WEDGE`).
- **Triangle (asc/desc/sym)** — converging trendlines; ascending +1, descending -1, symmetrical follows the last swing (`TRIANGLE`).
- **Head and Shoulders** — central head flanked by two matching shoulders over a flat neckline; top -1, inverse +1 (`HEAD_AND_SHOULDERS`).
- **Triple Top / Bottom** — three matching peaks / troughs; a stronger reversal than the double; bearish -1, bullish +1 (`TRIPLE_TOP_BOTTOM`).
- **Double Top / Bottom** — twin-peak / twin-trough reversal confirmed on the second matching swing extreme; bearish -1, bullish +1 (`DOUBLE_TOP_BOTTOM`).

## [0.5.1] - 2026-06-03

### Added — Seasonality & Session family (12 indicators)

- **Volume-by-Time Profile** — mean traded volume bucketed by intraday time (`VOLUME_BY_TIME_PROFILE`).
- **Intraday Volatility Profile** — return standard deviation bucketed by intraday time (`INTRADAY_VOLATILITY_PROFILE`).
- **Day-of-Week Profile** — mean bar return bucketed by weekday (`DAY_OF_WEEK_PROFILE`).
- **Time-of-Day Return Profile** — mean bar return bucketed by intraday time (`TIME_OF_DAY_RETURN_PROFILE`).
- **Seasonal Z-Score** — z-score of the current return versus the same hour-of-day history (`SEASONAL_Z_SCORE`).
- **Turn-of-Month** — mean daily return inside the turn-of-month window (`TURN_OF_MONTH`).
- **Overnight/Intraday Return** — decomposition of session return into overnight and intraday legs (`OVERNIGHT_INTRADAY_RETURN`).
- **Overnight Gap** — close-to-open return across the session boundary (`OVERNIGHT_GAP`).
- **Average Daily Range** — mean high-low range of the last N completed sessions (`AVERAGE_DAILY_RANGE`).
- **Session Range** — per-session (Asia/EU/US) high-low range (`SESSION_RANGE`).
- **Session High/Low** — running high and low of the current session (`SESSION_HIGH_LOW`).
- **Session VWAP** — session-anchored volume-weighted average price (`SESSION_VWAP`).

## [0.5.0] - 2026-06-03

### Added
- **TICK Index** — instantaneous net advancing-minus-declining issues (`TICK_INDEX`).
- **Absolute Breadth Index** — absolute value of net advancing-minus-declining issues (`ABSOLUTE_BREADTH_INDEX`).
- **Cumulative Volume Index** — running total of volume-normalised net advancing volume (`CUMULATIVE_VOLUME_INDEX`).
- **Bullish Percent Index** — percentage of the universe on a point-and-figure buy signal (`BULLISH_PERCENT_INDEX`).
- **Up/Down Volume Ratio** — advancing volume divided by declining volume (`UP_DOWN_VOLUME_RATIO`).
- **Percent Above Moving Average** — percentage of the universe trading above its reference moving average (`PERCENT_ABOVE_MA`).
- **High-Low Index** — moving average of the record-high percentage (`HIGH_LOW_INDEX`).
- **New Highs - New Lows** — net count of new period highs minus new period lows (`NEW_HIGHS_NEW_LOWS`).
- **Breadth Thrust** — moving average of the advancing-issues share (Zweig) (`BREADTH_THRUST`).
- **TRIN / Arms Index** — advance-decline ratio divided by the up-down volume ratio (`TRIN`).
- **McClellan Summation Index** — running cumulative total of the McClellan Oscillator (`MCCLELLAN_SUMMATION_INDEX`).
- **McClellan Oscillator** — spread between a 19- and 39-period EMA of ratio-adjusted net advances (`MCCLELLAN_OSCILLATOR`).
- **Advance/Decline Volume Line** — cumulative net advancing-minus-declining volume across the universe (`AD_VOLUME_LINE`).
- **Advance/Decline Ratio** — advancing issues divided by declining issues across the universe (`ADVANCE_DECLINE_RATIO`).

### Changed
- **Relicensed** from PolyForm Noncommercial 1.0.0 to dual **MIT OR Apache-2.0**. Wickra is now OSI-approved, permissive open source; commercial use is permitted under either license. See [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).

## [0.4.7] - 2026-06-03

### Added
- **Spread Bollinger Bands** — Bollinger bands on the spread of two series for pairs mean-reversion (`SPREAD_BOLLINGER_BANDS`).
- **Kalman Hedge Ratio** — Kalman-filter dynamic hedge ratio and spread between two series (`KALMAN_HEDGE_RATIO`).
- **Granger Causality** — Granger causality F-statistic measuring whether one series predicts another (`GRANGER_CAUSALITY`).
- **Variance Ratio** — Lo-MacKinlay variance-ratio test on the spread of two series (`VARIANCE_RATIO`).
- **Beta-Neutral Spread** — beta-neutral spread: the rolling OLS regression residual of two series (`BETA_NEUTRAL_SPREAD`).
- **Distance SSD** — Gatev sum-of-squared-deviations distance between two normalised series (`DISTANCE_SSD`).
- **Spread Hurst** — Hurst exponent of the spread of two series for regime detection (`SPREAD_HURST`).
- **OU Half-Life** — Ornstein-Uhlenbeck half-life of mean reversion for the spread of two series (`OU_HALF_LIFE`).
- **Rolling Covariance** — rolling covariance of the period-over-period returns of two series (`ROLLING_COVARIANCE`).
- **Rolling Correlation** — rolling Pearson correlation of the period-over-period returns of two series (`ROLLING_CORRELATION`).

- **Market Breadth family** — a new indicator family built on a new
  `CrossSection` input type that carries the per-symbol state of an entire
  universe in one tick (each `Member` holds a signed `change`, a `volume`, and
  `new_high` / `new_low` flags). `CrossSection::new` validates the universe
  (non-empty, finite changes, finite non-negative volumes); `new_unchecked`
  skips validation for hot paths.
  - `AdvanceDecline` (`ADVANCE_DECLINE`) — the Advance/Decline Line, the running
    cumulative sum of net advancing-minus-declining issues across the universe.

## [0.4.6] - 2026-06-03

### Added

- **TA-Lib parity — Directional Movement components** — the ADX building blocks,
  previously available only bundled inside `Adx`, as standalone single-output
  indicators:
  - `PlusDm` (`PLUS_DM`) — Wilder-smoothed plus directional movement.
  - `MinusDm` (`MINUS_DM`) — Wilder-smoothed minus directional movement.
  - `PlusDi` (`PLUS_DI`) — plus directional indicator, `100 · smoothed(+DM) / ATR`.
  - `MinusDi` (`MINUS_DI`) — minus directional indicator, `100 · smoothed(-DM) / ATR`.
  - `Dx` (`DX`) — directional movement index, `100 · |+DI − −DI| / (+DI + −DI)`.
- **TA-Lib parity — price transforms** — window and per-bar price aggregates:
  - `MidPrice` (`MIDPRICE`) — `(highest high + lowest low) / 2` over a window.
  - `MidPoint` (`MIDPOINT`) — `(max + min) / 2` of a scalar series over a window.
  - `AvgPrice` (`AVGPRICE`) — per-bar `(open + high + low + close) / 4`.
- **TA-Lib parity — rate-of-change variants** — the ratio forms of `Roc`:
  - `Rocp` (`ROCP`) — `(close − close[period]) / close[period]` (fraction).
  - `Rocr` (`ROCR`) — `close / close[period]` (ratio).
  - `Rocr100` (`ROCR100`) — `close / close[period] · 100`.
- **TA-Lib parity — linear-regression outputs** — the remaining OLS endpoints:
  - `LinRegIntercept` (`LINEARREG_INTERCEPT`) — the OLS intercept `a`.
  - `Tsf` (`TSF`) — time series forecast, `a + b·period` (one bar ahead).
- **TA-Lib parity — `MacdFix` (`MACDFIX`)** — MACD with fast/slow fixed at 12/26
  and only the signal period configurable; output is the usual `{macd, signal,
  histogram}` triple.
- **TA-Lib parity — `SarExt` (`SAREXT`)** — Parabolic SAR with a start value,
  reversal offset, independent long/short acceleration, and a signed output
  (positive in long phases, negative in short phases).
- **TA-Lib parity — `MacdExt` (`MACDEXT`)** — MACD with an independently
  selectable moving-average type (new `MaType` enum: SMA/EMA/WMA/DEMA/TEMA/TRIMA)
  for each of the fast, slow and signal lines.
- **TA-Lib parity — `HtPhasor` (`HT_PHASOR`)** — the in-phase and quadrature
  components of the Hilbert-transform analytic signal, as a `{inphase,
  quadrature}` pair.
- **TA-Lib parity — `HtDcPhase` (`HT_DCPHASE`)** — the phase angle (in degrees)
  of the Hilbert-transform dominant cycle.
- **TA-Lib parity — `HtTrendMode` (`HT_TRENDMODE`)** — Ehlers' trend (`1`) vs
  cycle (`0`) classification from the Hilbert-transform dominant cycle.

## [0.4.5] - 2026-06-02

### Added

- **Anchored RSI** — a cumulative Relative Strength Index whose averaging begins at a runtime-chosen anchor bar (`set_anchor`), the momentum counterpart to Anchored VWAP. Every up- and down-move since the anchor is weighted equally, so it reports the RSI of the entire move since the anchor point. Scalar input, Momentum Oscillators family; available in Rust, Python, Node and WASM.
- **Volume Profile** — the full per-bin volume distribution over a rolling window, exposing the raw histogram (price bounds plus per-bin volume) that Value Area reduces to POC/VAH/VAL. Market Profile family; candle input, available in Rust, Python, Node and WASM.
- **TPO Profile** — the Time-Price-Opportunity (market-profile letter) distribution: a volume-agnostic count of how many periods traded at each price level over a rolling window. Market Profile family; candle input, available in Rust, Python, Node and WASM.
- **Alt-Chart Bars** — a new `BarBuilder` trait and family of price-driven chart constructors that emit a variable number of completed bars per candle (so they are deliberately not `Indicator`s): **Renko** (fixed box-size bricks with the 2-box reversal rule), **Kagi** (reversal-amount line segments), and **Point & Figure** (box-size X/O columns with an N-box reversal). Available in Rust, Python, Node and WASM.

## [0.4.4] - 2026-06-02

### Added
- **TA-Lib candlestick patterns (part 1).** New candlestick pattern detectors
  matching TA-Lib `CDL*`, emitting the family's signed `+1 / 0 / −1` convention
  over OHLCV candles in Rust, Python, Node and WASM:
  - **Two Crows** — a three-bar bearish reversal (`CDL2CROWS`): a long white
    candle, a black candle whose body gaps up, then a black candle that opens
    inside the second's body and closes inside the first's.
  - **Upside Gap Two Crows** — a three-bar bearish reversal
    (`CDLUPSIDEGAP2CROWS`): two black candles gap up over a long white candle,
    the second engulfing the first crow yet still closing above the white body,
    leaving the upside gap open.
  - **Identical Three Crows** — a three-bar bearish reversal
    (`CDLIDENTICAL3CROWS`): three red candles with steadily lower closes, each
    opening at the prior candle's close so the bodies stack in an identical
    staircase.
  - **Three Line Strike** — a four-bar pattern (`CDL3LINESTRIKE`): a
    three-candle advance or decline struck by a fourth opposite-colour candle
    that engulfs the entire run; bullish `+1`, bearish `−1`.
  - **Three Stars in the South** — a rare three-bar bullish reversal
    (`CDL3STARSINSOUTH`): three shrinking red candles each carving a higher low
    and contracting toward a tiny black marubozu as selling exhausts.
  - **Abandoned Baby** — a strong three-bar reversal (`CDLABANDONEDBABY`): a doji
    isolated by price gaps on both sides; bullish `+1` after a decline, bearish
    `−1` after an advance.
  - **Advance Block** — a three-bar bearish warning (`CDLADVANCEBLOCK`): three
    green candles to higher closes whose bodies shrink as their upper shadows
    lengthen, signalling the advance is stalling.
  - **Belt-hold** — a single-bar reversal that opens at one extreme of its range and runs the other way; bullish +1, bearish -1 (`CDLBELTHOLD`).
  - **Breakaway** — a 5-bar reversal that gaps with the trend, drifts two more bars, then snaps back into the bar1/bar2 body gap; bullish +1, bearish -1 (`CDLBREAKAWAY`).
  - **Counterattack** — a 2-bar reversal where an opposite-coloured second bar closes level with the first (the counterattack line); bullish +1, bearish -1 (`CDLCOUNTERATTACK`).
  - **Doji Star** — a long body followed by a doji gapping away in the trend direction; bullish +1, bearish -1 (`CDLDOJISTAR`).
  - **Dragonfly Doji** — a doji opening and closing at the high with a long lower shadow, a bullish reversal; +1 (`CDLDRAGONFLYDOJI`).
  - **Gravestone Doji** — a doji opening and closing at the low with a long upper shadow, a bearish reversal; -1 (`CDLGRAVESTONEDOJI`).
  - **Long-Legged Doji** — a doji with long shadows on both sides, an indecision signal; +1 detection (`CDLLONGLEGGEDDOJI`).
  - **Rickshaw Man** — a long-legged doji with the body centred in the range, an indecision signal; +1 detection (`CDLRICKSHAWMAN`).
  - **Evening Doji Star** — a bearish top reversal: long white bar, a doji gapping up, then a black bar closing deep into the first body; -1 (`CDLEVENINGDOJISTAR`).
  - **Morning Doji Star** — a bullish bottom reversal: long black bar, a doji gapping down, then a white bar closing deep into the first body; +1 (`CDLMORNINGDOJISTAR`).
  - **Gap Side-by-Side White** — two similar white candles opening side by side after a gap, a continuation; gap up +1, gap down -1 (`CDLGAPSIDESIDEWHITE`).
  - **High-Wave** — a small body with very long shadows on both sides, an extreme indecision signal; +1 detection (`CDLHIGHWAVE`).
  - **Hikkake** — an inside bar followed by a failed breakout, a trap; bullish +1, bearish -1 (`CDLHIKKAKE`).
  - **Modified Hikkake** — a close-confirmed Hikkake: an inside bar then a failed breakout closing back inside; bullish +1, bearish -1 (`CDLHIKKAKEMOD`).
  - **Homing Pigeon** — two black candles, the second a small body inside the first, a bullish reversal; +1 (`CDLHOMINGPIGEON`).
  - **On-Neck** — a long black candle then a white candle closing at its low (the neckline), a bearish continuation; -1 (`CDLONNECK`).
  - **In-Neck** — a long black candle then a white candle closing just into its body, a bearish continuation; -1 (`CDLINNECK`).
  - **Thrusting** — a long black candle then a white candle closing well into but below the midpoint of its body, a bearish continuation; -1 (`CDLTHRUSTING`).
  - **Separating Lines** — opposite-coloured candles sharing the same open, the second an opening marubozu resuming the trend; bullish +1, bearish -1 (`CDLSEPARATINGLINES`).
  - **Kicking** — two opposite-coloured marubozu separated by a gap; bullish +1, bearish -1 (`CDLKICKING`).
  - **Kicking by Length** — a kicking pattern signalled by the colour of the longer marubozu; +1 / -1 (`CDLKICKINGBYLENGTH`).
  - **Ladder Bottom** — three descending black candles, a fourth with an upper shadow, then a white candle gapping up, a bullish reversal; +1 (`CDLLADDERBOTTOM`).
  - **Mat Hold** — a long white candle, a holding three-bar pullback, then a new-high white candle, a bullish continuation; +1 (`CDLMATHOLD`).
  - **Matching Low** — a 2-bar bullish reversal where two black candles in a decline share the same close, signalling selling pressure is exhausting; bullish +1 (`CDLMATCHINGLOW`).
  - **Long Line** — a single long-bodied candle with short shadows; bullish +1 (white) or bearish -1 (black) by colour (`CDLLONGLINE`).
  - **Short Line** — a single short-bodied candle with short shadows; bullish +1 (white) or bearish -1 (black) by colour (`CDLSHORTLINE`).
  - **Rising Three Methods** — a 5-bar bullish continuation: a long white candle, three small pullback bars holding within its range, then a white breakout to new highs; bullish +1 (`CDLRISEFALL3METHODS`).
  - **Falling Three Methods** — the bearish mirror of rising three methods: a long black candle, three small bars holding within its range, then a black breakdown to new lows; bearish -1 (`CDLRISEFALL3METHODS`).
  - **Upside Gap Three Methods** — a 3-bar bullish continuation: two white candles gap up, then a black candle opens within the second body and closes within the first; bullish +1 (`CDLXSIDEGAP3METHODS`).
  - **Downside Gap Three Methods** — the bearish mirror of upside gap three methods: two black candles gap down, then a white candle opens within the second body and closes within the first; bearish -1 (`CDLXSIDEGAP3METHODS`).
  - **Stalled Pattern** — a 3-bar bearish reversal warning: two long white candles then a small white candle riding the shoulder, signalling the rally is stalling; bearish -1 (`CDLSTALLEDPATTERN`).
  - **Stick Sandwich** — a 3-bar bullish reversal: two black candles closing at the same level sandwich a white candle, marking a support floor; bullish +1 (`CDLSTICKSANDWICH`).
  - **Takuri** — a single-bar bullish reversal, a strict Dragonfly Doji with a negligible upper shadow and very long lower shadow; bullish +1 (`CDLTAKURI`).
  - **Closing Marubozu** — a single long-bodied candle with no shadow on the close end; bullish +1 (white, closes at the high) or bearish -1 (black, closes at the low) (`CDLCLOSINGMARUBOZU`).
  - **Opening Marubozu** — a single long-bodied candle with no shadow on the open end; bullish +1 (white, opens at the low) or bearish -1 (black, opens at the high). No direct TA-Lib equivalent — completes the pair with the closing marubozu.
  - **Tasuki Gap** — a 3-bar continuation: two same-coloured candles gap in the trend direction, then an opposite candle opens within the second body and closes back into the gap without filling it; upside +1, downside -1 (`CDLTASUKIGAP`).
  - **Unique Three River** — a 3-bar bullish reversal: a long black candle, a black candle probing a new low with its body inside the first, then a small white candle held below it; bullish +1 (`CDLUNIQUE3RIVER`).
  - **Concealing Baby Swallow** — a rare 4-bar bullish capitulation: two black marubozu, a black candle gapping down with an upper shadow into the second, then a large black candle engulfing it entirely; bullish +1 (`CDLCONCEALBABYSWALL`).
- **Derivatives family — funding & open interest (part 1).** A new family of
  indicators that consume a perpetual / futures tick (`DerivativesTick`,
  bundling funding rate, mark / index / futures price, open interest,
  positioning, taker flow and liquidations) rather than OHLCV, exposed in Rust,
  Python, Node and WASM:
  - **Funding Rate** — the current perpetual funding rate.
  - **Funding Rate Mean** — the rolling mean funding rate over a window.
  - **Funding Rate Z-Score** — the latest funding rate in standard deviations
    from its rolling mean.
  - **Funding Basis** — the perpetual's relative premium to spot,
    `(markPrice − indexPrice) / indexPrice`.
  - **Open-Interest Delta** — the tick-over-tick change in open interest.
- **Derivatives family — open interest, flow & liquidations (part 2).** More
  indicators over the same `DerivativesTick` feed:
  - **OI / Price Divergence** — relative open-interest change minus relative
    price change over a window, the positioning-vs-price gap.
  - **OI-Weighted Price** — the cumulative mark price weighted by open interest.
  - **Long/Short Ratio** — aggregate long size over short size.
  - **Taker Buy/Sell Ratio** — taker buy volume over taker sell volume.
  - **Liquidation Features** — a multi-output breakdown of long/short
    liquidation notional into net, total and a bounded imbalance.
- **Derivatives family — basis & term structure (part 3).** The final
  perpetual-vs-futures basis indicators over the `DerivativesTick` feed:
  - **Term-Structure Basis** — the dated future's relative premium to spot,
    `(futuresPrice − indexPrice) / indexPrice`.
  - **Calendar Spread** — the dated future's relative premium to the perpetual,
    `(futuresPrice − markPrice) / markPrice`.

## [0.4.3] - 2026-06-01

### Added
- **Microstructure family — price impact & depth (part 3).** Indicators over a
  trade paired with the prevailing mid (`TradeQuote`) and over the order-book
  depth profile, exposed in Rust, Python, Node and WASM:
  - **Effective Spread** — `2 · D · (tradePrice − mid) / mid · 10_000` bps, the
    realised round-trip cost of a single trade against the mid.
  - **Realized Spread** — `2 · D · (tradePrice − mid_{t+horizon}) / mid_t ·
    10_000` bps, the share of the effective spread a liquidity provider keeps
    once the mid has moved over a configurable horizon.
  - **Kyle's Lambda** — the rolling OLS slope of mid changes on signed volume
    (`cov(Δmid, q) / var(q)`), the canonical price-impact / market-depth proxy.
  - **Depth Slope** — the mean per-side OLS slope of cumulative resting size
    against distance from the mid, measuring how fast the book thickens away
    from the touch.
- **Microstructure family — footprint (part 4).** **Footprint** decomposes the
  volume traded in a bar across price buckets (`round(price / tick_size)`),
  splitting each bucket into buy-initiated (ask) and sell-initiated (bid)
  volume. A multi-output, variable-length indicator: every `update` returns the
  full footprint accumulated since the last `reset`, exposed in Rust, Python
  (`(k, 3)` arrays), Node (`{ price, bidVol, askVol }` rows) and WASM.

## [0.4.2] - 2026-06-01

### Added
- **Microstructure family — order book (part 1).** A new family of indicators
  that consume an order-book depth snapshot (`OrderBook` of sorted, uncrossed
  bid/ask `Level`s) rather than OHLCV, exposed in Rust, Python, Node and WASM:
  - **Order-Book Imbalance** — `OrderBookImbalanceTop1`, `OrderBookImbalanceTopN`
    (configurable depth) and `OrderBookImbalanceFull` measure signed depth
    pressure `(bidDepth − askDepth) / (bidDepth + askDepth)` over the top level,
    the top-N levels, or the full book.
  - **Microprice** — the size-weighted fair value
    `(bidPx·askSz + askPx·bidSz) / (bidSz + askSz)`, tilting the mid toward the
    side more likely to be hit.
  - **Quoted Spread** — the top-of-book spread in basis points of the mid.
- **Microstructure family — trade flow (part 2).** Indicators over a trade tape
  (`Trade` with an aggressor `Side`), exposed in Rust, Python, Node and WASM:
  - **Signed Volume** — per-trade size signed by aggressor side (`+size` buy,
    `−size` sell).
  - **Cumulative Volume Delta** — the running total of signed volume; reset to
    re-anchor per session.
  - **Trade Imbalance** — the rolling `(buyVol − sellVol)/(buyVol + sellVol)`
    over a configurable window of trades.

  New public value types `Level`, `OrderBook`, `Side`, `Trade` and `TradeQuote`
  back this and the upcoming trade-flow and price-impact indicators. Python and
  Node accept a batch over a list of snapshots; WASM exposes per-snapshot
  `update`.
- **Signed Doji encoding.** `Doji` gains an opt-in `.signed()` mode
  (`Doji(signed=True)` in Python, `new Doji(true)` in Node and WASM) that
  classifies a detected Doji by the position of its body within the bar range —
  a dragonfly (long lower shadow) emits `+1.0` (bullish), a gravestone (long
  upper shadow) emits `−1.0` (bearish), and a long-legged / standard Doji emits
  `0.0` (neutral). The default construction is unchanged — a direction-less
  `+1.0` / `0.0` detection flag — so existing callers are unaffected. This
  completes the uniform `+1` bull / `−1` bear / `0` none sign convention across
  every candlestick pattern, making the family a drop-in machine-learning
  feature where bullish and bearish instances share a single dimension.

### Fixed
- **README banner now self-updates.** The top README banner points at the org
  profile image that `.github/banner.yml` regenerates from the indicator count,
  and `sync-about.yml` bumps a `?v=<count>` cache-buster so GitHub's Camo proxy
  refetches it immediately. Also fixes the webpage indicator-count sync, which
  silently crashed on a removed `public/hero.svg` and left the marketing site's
  count (and its OG banner) stale.

### Security
- **CI dependency installs are pinned by hash.** The Node binding now installs
  with `npm ci` (strict `package-lock.json`), and the Python CI/bench tooling is
  installed from hash-locked `--require-hashes` requirements under
  `.github/requirements/` (OpenSSF Scorecard PinnedDependencies). The `ci-dev`
  tooling is locked twice — for Python 3.9 and for 3.10+ — because numpy ships no
  single release with wheels for both cp39 and cp313. A new
  `scripts/update-lockfiles.sh` regenerates every workspace lockfile (Rust, Node
  and the hash-pinned Python requirements) via `uv`, and Dependabot keeps the
  pinned requirements current.

## [0.4.1] - 2026-06-01

### Added
- **Cross-asset pairwise indicators.** A new two-series family of
  `Indicator<Input = (f64, f64)>` implementations that relate two distinct
  assets rather than a single OHLCV stream. Each is exposed in Rust, Python,
  Node, and WASM:
  - **Pairwise Beta** (`PairwiseBeta`) — rolling OLS slope of one asset's
    **log-returns** on another's. Unlike `Beta`, which regresses the raw inputs
    it is fed, `PairwiseBeta` differences consecutive prices into log-returns
    internally — the conventional way to measure cross-asset beta, where a beta
    on price levels would be dominated by the shared trend.
  - **Pair Spread Z-Score** (`PairSpreadZScore`) — the standardised log-spread
    `ln(a) − β·ln(b)` of a pair, where `β` is a rolling-OLS hedge ratio and the
    spread is z-scored over its own look-back. The canonical mean-reversion /
    statistical-arbitrage entry signal, with independent `beta_period` and
    `z_period` windows.
  - **Lead–Lag Cross-Correlation** (`LeadLagCrossCorrelation`) — the integer
    offset `k ∈ [−max_lag, max_lag]` that maximises `|corr(a[t], b[t+k])|`,
    answering which of two assets leads the other and by how many bars. Emits
    `{ lag, correlation }`; a positive lag means `a` leads `b`.
  - **Cointegration** (`Cointegration`) — the Engle–Granger two-step screen for
    pairs trading: a rolling OLS hedge ratio `β`, the spread (residual)
    `a − (α + β·b)`, and an augmented Dickey–Fuller `t`-statistic on the spread
    (configurable `adf_lags`). A strongly negative statistic flags a
    mean-reverting, tradeable spread. Emits `{ hedge_ratio, spread, adf_stat }`.
  - **Relative Strength A-vs-B** (`RelativeStrengthAB`) — the comparative
    relative strength of two assets: the ratio line `a / b` together with its
    moving average and its RSI, the classic asset-vs-asset / asset-vs-index
    rotation screen. Emits `{ ratio, ratio_ma, ratio_rsi }`.

## [0.4.0] - 2026-06-01

### Added
- **Build-provenance attestations for release artifacts.** The release workflow
  now emits signed SLSA build-provenance attestations for the published crates
  and Python wheels/sdist (`actions/attest-build-provenance`); npm packages
  carry inline Sigstore provenance from `npm publish --provenance`. Every
  published artifact is cryptographically traceable to this repository's release
  workflow run.

### Security
- **CodeQL static analysis and OpenSSF Scorecard run in CI.** CodeQL (Rust,
  Python, JavaScript) and the OpenSSF Scorecard workflow now run on every push;
  results appear under Security → Code scanning and a public Scorecard badge is
  shown in the README.
- **CI workflows hardened against script injection.** Untrusted event contexts
  (PR branch names, `workflow_dispatch` inputs) are passed through the step
  environment instead of being interpolated directly into shell commands.

### Changed
- **Node binding: invalid indicator periods now throw instead of being silently
  clamped.** The scalar-indicator constructors previously clamped `period = 0`
  to `1`; every Node constructor now propagates the core's validation error
  (e.g. `period must be greater than zero`), matching the Python and WASM
  bindings and the Rust core. Constructing with a valid period is unaffected.
- **Binding package READMEs are now per-ecosystem.** The Python, Node.js, and
  WebAssembly READMEs were byte-identical 314-line copies of the workspace
  README and had drifted out of sync (stale indicator count, Python snippets
  shown on the Node and WASM package pages). Each is now a focused landing page
  with the correct install command, a language-correct quick-start snippet, and
  links to the canonical documentation — removing the manual three-way sync
  burden. No code or API changes.
- **CONTRIBUTING now states the correct MSRV (1.86 workspace / 1.88
  `bindings/node`)** and documents that these are the dependency-forced floors,
  kept minimal on purpose. The previous text claimed 1.75 / 1.77, which the
  `msrv` CI job has enforced against since the criterion and napi-build bumps.

## [0.3.1] - 2026-05-30

### Fixed
- **Release pipeline — CycloneDX SBOM generation.** `cargo-cyclonedx` has no
  `-p`/`--package` selector; it walks the whole workspace in a single pass.
  The `release.yml` SBOM step invoked it as `cargo cyclonedx … -p <crate>` and
  aborted with `error: unexpected argument '-p' found`, which failed the
  crates.io publish job *after* the crates were already published and skipped
  the GitHub Release attach-assets job (no release page, no SBOM artefacts).
  The step now runs a single workspace pass and collects the three crates.io
  crate SBOMs. No library changes relative to 0.3.0 — this patch republishes
  the same code with a working release pipeline.

## [0.3.0] - 2026-05-30

### Added
- **Family 15 — Risk / Performance metrics (17 new indicators).** Implemented
  pragmatically as standard `Indicator`s rather than a separate
  `wickra-metrics` crate; the input is a scalar `f64` per bar (period return,
  equity sample, or trade P&L depending on the metric).
  - **Scalar `Indicator<f64>` — 14 metrics:** Sharpe Ratio, Sortino Ratio,
    Calmar Ratio, Omega Ratio, Max Drawdown (rolling), Average Drawdown,
    Drawdown Duration (time-under-water), Pain Index, Value at Risk
    (historical, linear-interpolated percentile), Conditional Value at Risk
    (Expected Shortfall), Profit Factor, Gain/Loss Ratio, Recovery Factor,
    Kelly Criterion.
  - **Two-series `Indicator<(f64, f64)>` — 3 metrics on `(asset_return,
    benchmark_return)` pairs:** Treynor Ratio, Information Ratio,
    Jensen's Alpha (CAPM).
- **Candlestick patterns family (15 indicators).** A new "Candlestick
  Patterns" family covers the standard 1- to 3-bar reversal and
  continuation shapes: `Doji`, `Hammer`, `InvertedHammer`, `HangingMan`,
  `ShootingStar`, `Engulfing`, `Harami`, `MorningEveningStar`,
  `ThreeSoldiersOrCrows`, `PiercingDarkCloud`, `Marubozu`, `Tweezer`,
  `SpinningTop`, `ThreeInside` and `ThreeOutside`. Every detector takes a
  `Candle` and emits a signed `f64` (`+1.0` bullish, `-1.0` bearish, `0.0`
  no pattern; `Doji` is direction-less and emits `+1.0`/`0.0`). The MVP is
  a pattern-shape check only — no trend filter is applied. Available
  across Rust, Python, Node and WASM bindings. Harmonic and chart
  patterns remain out of scope and will follow once the pattern-detection
  framework (pivot detector + multi-bar state machines) lands.
- **Market Profile family** (3 new indicators, opens family #9 across the
  catalogue):
  - `ValueArea(period, bin_count, value_area_pct)` — rolling
    bin-approximation volume profile over the last `period` candles.
    Outputs `{poc, vah, val}`: Point of Control is the bin with the highest
    cumulative volume; the Value Area expands symmetrically from POC and
    always absorbs the higher-volume neighbour next, until the configured
    percentage of total volume (default 70%) is enclosed. Each candle's
    volume is spread uniformly across its `[low, high]` range; single-print
    bars (`low == high`) drop their entire volume into one bin.
  - `InitialBalance(period)` — first-N-bar session high / low, frozen
    once `period` bars have been ingested. Outputs `{high, low}`. Default
    `period = 12` (one-hour IB on 5-minute bars for US equities). Callers
    MUST invoke `reset()` at every session boundary, otherwise the IB
    locks and stays fixed for the lifetime of the instance.
  - `OpeningRange(period)` — same lock-after-N-bars semantics as IB but
    with a smaller default window (`period = 6`, 30 min on 5-minute
    bars) and a third output `breakout_distance` = `close - or_mid`,
    signed (positive above the range, negative below).
- Histogram-output Market Profile variants (Volume Profile / VPVR /
  Composite Profile) and tick-data-only variants (TPO / Single Print /
  Cumulative Delta / Order Flow Delta / Volume-Weighted Open) are
  deliberately out of scope of this PR: the former need a new
  histogram-output API layer, the latter need tick / L2 data which
  `wickra-data` does not yet expose.
- **Family 12 — Statistik / Regression (13 indicators).** A complete
  statistical toolkit for analysing rolling price distributions and
  cross-series relationships. Every indicator ships in the Rust core
  plus all three bindings (Python, Node, WASM), with full streaming +
  batch parity, fuzz coverage, and benches against the BTCUSDT
  dataset:
  - **Variance** — rolling population variance (`StdDev` squared).
  - **CoefficientOfVariation** — `StdDev / Mean`, dimensionless dispersion.
  - **Skewness** — rolling third standardised moment (Pearson skewness).
  - **Kurtosis** — rolling excess kurtosis (fourth moment minus `3`).
  - **StandardError** — standard error of estimate for the rolling OLS
    fit, with `n − 2` residual degrees of freedom.
  - **DetrendedStdDev** — population standard deviation of OLS
    residuals (the StdDev that remains after subtracting the linear
    trend).
  - **RSquared** — coefficient of determination of the rolling OLS
    fit; the trend-quality filter.
  - **MedianAbsoluteDeviation** — robust dispersion measure that
    survives outliers (median of absolute deviations from the median).
  - **Autocorrelation** — rolling lag-`k` Pearson autocorrelation;
    detects periodicity and tests for white-noise behaviour.
  - **HurstExponent** — R/S-analysis estimator of trend-persistence
    vs. mean-reversion regime (`0.5` is random walk).
  - **PearsonCorrelation** — rolling correlation between two
    synchronised series; takes `(x, y)` pairs.
  - **Beta** — rolling OLS slope of an asset on a benchmark; the CAPM
    sensitivity coefficient.
  - **SpearmanCorrelation** — rolling rank correlation (monotone,
    outlier-robust analogue of Pearson).

  Indicator count: 71 → 84.
- **Family 13 — Ichimoku & alternative charts.** Two new indicators:
  - `Ichimoku` (Ichimoku Kinko Hyo) — the full five-line cloud system
    (Tenkan-sen, Kijun-sen, Senkou Span A/B, Chikou Span) with the
    classic `(9, 26, 52, 26)` defaults and configurable periods. Forward
    displacement is handled in a streaming ring buffer so the
    currently-visible Senkou A/B at bar *n* are the values computed
    from bar *n − displacement*.
  - `HeikinAshi` — the candle smoothing transform that recursively
    averages OHLC into a four-component output (`ha_open`, `ha_high`,
    `ha_low`, `ha_close`). Seeds `ha_open` from the first bar's
    `(open + close) / 2`.

  Exposed in all four bindings (Rust, Python, Node, WASM). Renko,
  Kagi, and Point & Figure from the family ideas list are deferred:
  they are custom bar generators rather than indicators and belong in
  `wickra-data`.
- **Family 10 — Ehlers / Cycle (DSP) indicators.** 16 new
  streaming-first indicators implementing John Ehlers'
  digital-signal-processing school of cycle analytics — a strong
  differentiation feature versus TA-Lib and pandas-ta, which only
  ship fragments of this catalogue:
  - **MAMA / FAMA** (MESA Adaptive Moving Average + Following
    Adaptive Moving Average) — phase-rate-adaptive smoothing pair
    from the 2001 MESA paper, exposed both jointly via `Mama` (multi-
    output) and as a scalar `Fama` wrapper.
  - **Fisher Transform** and **Inverse Fisher Transform** — Gaussian
    normalisation of price (Ehlers 2002) and its tanh-based bounded
    counterpart for oscillators.
  - **SuperSmoother**, **Roofing Filter**, **Decycler** and **Decycler
    Oscillator** — 2-pole Butterworth lowpass, bandpass and
    high-pass complement building blocks from *Cycle Analytics for
    Traders* (2013).
  - **Hilbert Dominant Cycle**, **Sine Wave** and **Adaptive Cycle**
    — Hilbert-transform-based period estimation from *Rocket Science
    for Traders* (2001).
  - **Center of Gravity**, **Cybernetic Cycle Component**,
    **Instantaneous Trendline**, **Ehlers Stochastic** and
    **Empirical Mode Decomposition** — EasyLanguage classics from
    Ehlers' published catalogue.
  - All sixteen are exposed across Rust, Python, Node.js and WASM
    bindings, fuzz-tested, benchmarked against real BTCUSDT
    1-minute data, and pass `batch == streaming` equivalence.
  - Indicator count rises from 71 to **87** across **nine** families.
- **DeMark family (family 11) — 12 new indicators.** TD Setup (9-bar
  buy/sell setup counter with parameterised lookback and target), TD
  Sequential (Setup + Countdown phase machine emitting setup count,
  countdown count and active countdown direction), TD DeMarker
  (bounded [0, 1] range oscillator built from high/low expansions),
  TD REI (Range Expansion Index — bounded ±100 oscillator with the
  classic 5-bar default), TD Pressure (volume-weighted buying /
  selling pressure normalised to ±100), TD Combo (aggressive
  countdown variant with extra monotone-low / monotone-close
  strictness conditions on top of the classic countdown rule), TD
  Countdown (standalone 13-bar countdown phase machine emitting
  only the signed countdown count and direction — smaller streaming
  payload than the full TD Sequential), TD Lines (TDST horizontal
  support / resistance levels derived from the highs and lows of
  the most-recently-completed setup), TD Range Projection (next-bar
  high / low projection from the current bar's OHLC via DeMark's
  open-vs-close-weighted pivot), TD Differential (2-bar
  buying-pressure-vs-selling-pressure reversal pattern emitting
  +1 / -1 / 0), TD Open (gap-and-fade reversal pattern emitting
  +1 / -1 / 0 when the open prints outside the prior bar's range
  but the subsequent action recovers back into it), and TD Risk
  Level (protective stop levels derived from the lowest-low / highest-
  high setup bar's true range). All twelve are exposed through the
  Rust, Python, Node, and WASM bindings with `batch == streaming`
  equivalence tests, candle-stream fuzz coverage, and benchmark
  entries on the BTCUSDT 1-minute dataset.
- **Family 08 — Pivots & Support/Resistance.** Seven new indicators land
  the previously empty pivot family: Classic (Floor-Trader) Pivot Points
  with three resistance and support tiers, Fibonacci Pivots spaced by
  0.382 / 0.618 / 1.000 of the prior range, Camarilla Pivots
  (Nick Stott's four-tier `(H − L) · 1.1 / {12, 6, 4, 2}` levels),
  Woodie Pivots with the close-weighted `PP = (H + L + 2·C) / 4`,
  DeMark Pivots whose conditional `X` depends on whether the bar closed
  up, down or flat, Williams Fractals as a five-bar swing detector and
  ZigZag as a percent-threshold swing tracker. Every level/swing is
  exposed across Rust, Python, Node and WASM with the standard
  `update` / `batch` / `reset` / `is_ready` / `warmup_period` surface
  and matching streaming-vs-batch and reference-value tests. The fuzz
  candle target now covers all seven.
- **Family 09 — Trailing Stops, seven new indicators.** Rounds out the
  trailing-stop family from 5 to 12: `HiLoActivator` (Crabel's
  SMA-of-high / SMA-of-low trail), `VoltyStop` (Cynthia Kase's
  extreme-anchor ATR stop), `YoyoExit` (long-only ATR trail with a
  re-entry trigger), `DonchianStop` (the original Turtle exit, lowest
  low / highest high), `PercentageTrailingStop` (fixed-percent trail),
  `StepTrailingStop` (round-number grid trail) and `RenkoTrailingStop`
  (block-anchored Renko-style trail). All wired into the four bindings
  (Rust, Python, Node, WASM), the streaming + batch fuzz targets, and
  the bench harness.
- **Klinger Volume Oscillator (KVO).** Stephen J. Klinger's trend-aware
  volume-force oscillator: `EMA(vf, fast) − EMA(vf, slow)` over a daily
  volume force scaled by cumulative-measurement ratio. Classic
  `(fast, slow) = (34, 55)` exposed via `Kvo::classic()`.
- **Volume Oscillator (VO).** Percent difference between a fast and a
  slow SMA of bar volume: `100 · (SMA(vol, fast) − SMA(vol, slow)) /
  SMA(vol, slow)`. Default `(14, 28)`.
- **Negative Volume Index (NVI).** Paul Dysart's cumulative index that
  only updates on volume-contraction bars (`volume_t < volume_{t−1}`),
  absorbing the percent close change on those quiet days. Fosback
  baseline `1000.0`, configurable via `Nvi::with_baseline`.
- **Positive Volume Index (PVI).** The complementary index that
  updates on volume-expansion bars (`volume_t > volume_{t−1}`).
- **Williams Accumulation/Distribution.** Larry Williams' volume-less
  cumulative flow that anchors to the previous close (true high/low) and
  classifies each bar as accumulation, distribution, or neutral by the
  sign of the close-to-close change.
- **Anchored VWAP.** A cumulative VWAP whose accumulation begins at a
  user-chosen anchor bar rather than the session open. Re-anchor at
  runtime via `AnchoredVwap::set_anchor` for click-to-anchor trader
  workflows.
- **Demand Index (Sibbet).** James Sibbet's smoothed buying-vs-selling
  pressure ratio in the streaming-friendly textbook form
  `EMA(volume · close-return · (1 + range/close), period)`.
- **Time Segmented Volume (TSV).** Don Worden's rolling sum of signed
  volume weighted by the close-to-close move: a window-sum measure of
  net accumulation/distribution.
- **Volume Zone Oscillator (VZO).** Walid Khalil's normalised
  volume-flow oscillator bounded in `[−100, 100]`, defined as
  `100 · EMA(signed_volume) / EMA(volume)`.
- **Market Facilitation Index (Bill Williams).** Per-bar
  `(high − low) / volume` — how much price movement the market produces
  per unit of volume.
- **ADXR (Average Directional Movement Index Rating)** in the Trend &
  Directional family. Wilder's directional-strength smoother: the
  average of the current `ADX` and the `ADX` from `period - 1` bars
  ago. Warmup is `3 * period - 1` (e.g. 41 for the default `period =
  14`). Shipped across all four bindings (Rust core, Python, Node,
  WASM) plus fuzz/test/bench coverage.
- **Random Walk Index (RWI)** in the Trend & Directional family. Mike
  Poulos' trend-vs.-random-walk gauge: for each lookback `i ∈ [2,
  period]` the ratio of actual displacement to the random-walk
  expectation `ATR_i * sqrt(i)` is taken; the per-bar output is the
  maximum across lookbacks for both the high (`RWI_High`) and low
  (`RWI_Low`) directions. Multi-output `(high, low)` across all four
  bindings; warmup `= period`.
- **Trend Intensity Index (TII)** in the Trend & Directional family.
  M.H. Pee's `[0, 100]` oscillator: the share of the most recent
  `dev_period` SMA-deviations that are positive, scaled to
  `[0, 100]`. Saturates at 100 on a pure uptrend, at 0 on a pure
  downtrend, and returns the neutral 50 on a perfectly flat market.
  Canonical Python defaults `(sma_period=60, dev_period=30)`; warmup
  `= sma_period + dev_period − 1`.
- **Wave Trend Oscillator (LazyBear)** in the Trend & Directional
  family. Two-line mean-reverting momentum gauge built from the
  typical price and three cascaded EMAs:
  `esa = EMA(ap, channel)`, `d = EMA(|ap − esa|, channel)`,
  `ci = (ap − esa) / (0.015 · d)`, `wt1 = EMA(ci, average)`,
  `wt2 = SMA(wt1, signal)`. `WaveTrend::classic()` exposes the
  LazyBear defaults `(channel = 10, average = 21, signal = 4)`;
  warmup `= 2 · channel + average + signal − 3` (42 for the classic
  defaults). Includes a sub-ULP flat-tolerance guard on `ci` so a
  perfectly flat market reports `(0, 0)` instead of the
  mathematically indeterminate `−1 / 0.015 = −66.67`. Multi-output
  `(wt1, wt2)` across all four bindings.
- **Family 05 — Bands & Channels (11 new indicators).** Eleven additional
  price-envelope overlays organised into the new "Bands & Channels"
  family, exposed across all four bindings (Rust, Python, Node, WASM):
  - `MaEnvelope` — SMA centerline with fixed-percent envelope (the oldest
    band overlay still in use).
  - `AccelerationBands` — Price Headley's momentum-biased bands that widen
    with the bar's relative range `(H − L) / (H + L)`.
  - `StarcBands` — Stoller Average Range Channel: SMA(close) ± k·ATR
    (Keltner's SMA-centerline sibling).
  - `AtrBands` — Close-anchored envelope of width `k · ATR`, the standard
    volatility-targeting stop/target band.
  - `HurstChannel` — SMA centerline wrapped by the rolling high-low range
    (Brian Millard / Hurst-cycle channel).
  - `LinRegChannel` — Linear-regression endpoint ± k·σ of the residuals,
    measuring dispersion about the *trend* rather than the mean.
  - `StandardErrorBands` — Linear regression with the OLS standard error
    (denominator `n − 2`) for prediction-interval bands.
  - `DoubleBollinger` — Kathy Lien's `±1σ` plus `±2σ` zone-partition setup.
  - `TtmSqueeze` — John Carter's BB-inside-KC squeeze flag paired with a
    detrended-close momentum reading.
  - `FractalChaosBands` — Bill Williams 5-bar fractal high/low envelope.
  - `VwapStdDevBands` — Cumulative VWAP with volume-weighted standard
    deviation bands.
  Indicator count rises from 71 to 82 across nine families; the README
  family table and the wiki overview/sidebar/warmup pages were updated to
  match.
- **Yang-Zhang Volatility.** Yang & Zhang (2000) gold-standard OHLC
  estimator: a convex blend of overnight (close-to-open), open-to-close
  and Rogers-Satchell variances. The blending factor
  `k = 0.34 / (1.34 + (n+1)/(n-1))` is the one that minimises
  estimator variance under driftless GBM with overnight gaps. The
  overnight and open-to-close pieces use sample variance (Bessel's
  correction, divisor `n−1`), so the indicator needs `period + 1` bars
  to emit. Output annualised to a percent. Defaults: `period = 20`,
  `trading_periods = 252`. The recommended OHLC estimator for equities,
  futures, and any asset with material close-to-open gaps.
- **Rogers-Satchell Volatility.** Drift-free OHLC realised-volatility
  estimator from Rogers, Satchell & Yoon (1994). Per-bar sample is
  `ln(H/C)·ln(H/O) + ln(L/C)·ln(L/O)`; every term is non-negative by
  construction (high >= open, close; low <= open, close), so the
  rolling mean is exact, not biased, under arbitrary drift. The
  algebraic drift-cancellation is what differentiates it from
  Garman-Klass. Output annualised to a percent. Defaults:
  `period = 20`, `trading_periods = 252`.
- **Garman-Klass Volatility.** Garman & Klass (1980) OHLC realised
  volatility estimator: per-bar sample is
  `0.5·(ln H/L)² − (2·ln2 − 1)·(ln C/O)²`, then take the annualised
  square root of the rolling mean. Roughly 7.4× more statistically
  efficient than close-to-close stddev under driftless GBM. Output
  annualised to a percent. Defaults: `period = 20`,
  `trading_periods = 252`.
- **Parkinson Volatility.** Michael Parkinson's (1980) high-low realised
  volatility estimator: `sigma² = (1 / (4n·ln2)) · Σ (ln(H/L))²`. Output
  annualised to a percent in the same style as `HistoricalVolatility`
  (pass `trading_periods = 1` for the raw per-bar `sigma·100` figure).
  Roughly 5× more statistically efficient than close-to-close stddev
  under a driftless-GBM assumption. Defaults: `period = 20`,
  `trading_periods = 252`.
- **RVIVolatility (Relative Volatility Index).** Donald Dorsey's
  RSI-shaped volatility gauge: partition the rolling standard
  deviation of close into "up" (close rose) and "down" (close fell)
  samples, Wilder-smooth each side, and compute
  `100 · AvgUp / (AvgUp + AvgDown)`. Bounded on `[0, 100]`; saturates
  at `100` in pure uptrends, `0` in pure downtrends, and falls back to
  `50` on a completely flat series (same undefined-RS convention as
  `RSI`). Single `period` parameter (default `10`) drives both the
  stddev window and the Wilder smoothing. Named `RVIVolatility` rather
  than plain `RVI` to disambiguate from Relative Vigor Index, which
  ships in Family 02 under the shorter `RVI` name.
- **Family 03 — MACD & Price Oscillators.** `Stc` (Schaff Trend Cycle,
  Doug Schaff): doubly-`Stochastic`-smoothed MACD producing a bounded
  `[0, 100]` reading that reacts faster than `MACD` itself. Four
  parameters `(fast = 23, slow = 50, schaff_period = 10, factor = 0.5)`.
  Output is clamped to `[0, 100]` to absorb floating-point rounding.
  Exposed in all four bindings.
- **Family 03 — MACD & Price Oscillators.** `ElderImpulse` (Alexander
  Elder's Impulse System): tri-state momentum gauge combining `EMA`
  trend slope with `MACD` histogram slope. Returns `+1` (green/buy)
  when both rise, `−1` (red/sell) when both fall, `0` (blue/neutral)
  on disagreement. Four parameters
  `(ema_period, macd_fast, macd_slow, macd_signal)`; defaults
  `(13, 12, 26, 9)` track *Come Into My Trading Room*. Exposed in all
  four bindings.
- **Family 03 — MACD & Price Oscillators.** `ZeroLagMacd`: classic
  MACD topology with `ZLEMA` substituted for `EMA` everywhere — faster
  reaction to trend changes at the cost of slightly noisier readings.
  Multi-output `ZeroLagMacdOutput { macd, signal, histogram }`. Three
  parameters `(fast = 12, slow = 26, signal = 9)`; `fast` must be
  strictly less than `slow`. Exposed in all four bindings.
- **Family 03 — MACD & Price Oscillators.** `CFO` (Chande Forecast
  Oscillator): `100 · (close − LinReg(close, period)) / close`. Positive
  when the close overshoots the linear forecast, negative when it
  undershoots. Holds the previous value if the close is zero. Default
  period 14. Exposed in all four bindings.
- **Family 03 — MACD & Price Oscillators.** `AwesomeOscillatorHistogram`:
  `AO − SMA(AO, sma_period)`. A configurable variant of the existing
  `AcceleratorOscillator` (which fixes `(fast, slow, sma) = (5, 34, 5)`).
  Three parameters; defaults match Bill Williams' Accelerator. Exposed
  in all four bindings.
- **Family 03 — MACD & Price Oscillators.** `APO` (Absolute Price
  Oscillator): `EMA(close, fast) − EMA(close, slow)`. Like MACD's line
  without the signal EMA. Default `(fast = 12, slow = 26)`. `fast` must
  be strictly less than `slow`. Exposed in all four bindings.
- **Family 02 — Momentum Oscillators.** `Inertia` (Dorsey): a
  `LinearRegression` smoothing of the `RVI` series — preserves trend
  direction while damping the underlying ratio. Candle input, two
  parameters `(rvi_period, linreg_period)` (defaults 14 / 20). Exposed
  in all four bindings.
- **Family 02 — Momentum Oscillators.** `ConnorsRsi`: Larry Connors'
  3-component aggregate — `RSI(close)`, `RSI(streak)`, and the
  percentile rank of the 1-bar return over the recent `period_rank`
  returns. Bounded in `[0, 100]`. Three parameters
  `(period_rsi, period_streak, period_rank)` (defaults 3 / 2 / 100).
  Exposed in all four bindings.
- **Family 02 — Momentum Oscillators.** `LaguerreRsi` (Ehlers):
  four-stage Laguerre polynomial filter wrapped in an RSI-style up/down
  accumulator. Single parameter `gamma` in `[0, 1]` (default 0.5) trades
  lag for smoothness. State is seeded to the first input so a constant
  series stays at the neutral 50. Output clamped to `[0, 100]`. Exposed
  in all four bindings.
- **Family 02 — Momentum Oscillators.** `SMI` (Stochastic Momentum
  Index, Blau): doubly-`EMA`-smoothed bounded oscillator measuring the
  close's displacement from the centre of the recent high-low range,
  scaled by the smoothed range. Candle input, three parameters
  `(period, d_period, d2_period)` (defaults 5 / 3 / 3). Exposed in all
  four bindings.
- **Family 02 — Momentum Oscillators.** `KST` (Know Sure Thing, Pring):
  weighted sum of four `SMA`-smoothed `ROC` series with Pring's fixed
  weights `1, 2, 3, 4`, plus an `SMA` signal line. Nine parameters
  (four ROC periods, four SMA periods, signal period); `Kst::classic()`
  uses Pring's recommended defaults. Multi-output indicator emitting
  `KstOutput { kst, signal }`. Exposed in all four bindings.
- **Family 02 — Momentum Oscillators.** `PGO` (Pretty Good Oscillator,
  Mark Johnson): `(close − SMA(close, period)) / EMA(TR, period)`.
  Candle input, single parameter `period` (default 14). Roughly counts
  how many ATR-equivalents the close is from its mean. Exposed in all
  four bindings.
- **Family 02 — Momentum Oscillators.** `RVI` (Relative Vigor Index,
  Dorsey): per-bar ratio `SMA(close - open, period) / SMA(high - low,
  period)`. Candle input, single parameter `period` (default 10).
  Positive on average-bullish windows, negative on average-bearish.
  Holds previous value if the entire window has zero range. Exposed in
  all four bindings.
- **Family 01 — Moving Averages.** `ALMA` (Arnaud Legoux Moving Average):
  Gaussian-weighted moving average with configurable centre (`offset` in
  `[0, 1]`) and kernel width (`sigma > 0`). Community-standard defaults
  `(period = 9, offset = 0.85, sigma = 6.0)` available via `Alma::classic()`.
  Exposed in all four bindings (Rust, Python, Node, WASM).
- **Family 01 — Moving Averages.** `EVWMA` (Elastic Volume-Weighted
  Moving Average, Fries 2001): an "elastic" recurrence whose smoothing
  weight is the bar's volume relative to the running window-volume.
  Candle input (uses close + volume), single parameter `period`
  (default 20). Holds its previous value if the entire window has zero
  volume. Exposed in all four bindings.
- **Family 01 — Moving Averages.** `Alligator` (Bill Williams): three
  SMMA lines (Jaw / Teeth / Lips) of the median price `(high + low) / 2`
  with default periods 13 / 8 / 5. Multi-output indicator emitting
  `AlligatorOutput { jaw, teeth, lips }`. Visual chart shift is left to
  the consumer. Exposed in all four bindings.
- **Family 01 — Moving Averages.** `JMA` (Jurik Moving Average):
  three-stage filter reconstruction of Mark Jurik's adaptive MA.
  Three parameters: `period` (14), `phase` in `[-100, 100]` (0), `power`
  in `1..=4` (2). State is seeded to the first input so a constant series
  is reproduced exactly. Exposed in all four bindings.
- **Family 01 — Moving Averages.** `VIDYA` (Variable Index Dynamic
  Average, Chande 1992): EMA whose smoothing factor is scaled by the
  absolute Chande Momentum Oscillator. Two parameters `period` and
  `cmo_period` (defaults 14 / 9). Exposed in all four bindings.
- **Family 01 — Moving Averages.** `FRAMA` (Fractal Adaptive Moving
  Average, Ehlers 2005): adapts its smoothing constant to the fractal
  dimension of the recent window — fast in trends, slow in chop. Single
  parameter `period` (must be even, default 16). Exposed in all four
  bindings.
- **Family 01 — Moving Averages.** `McGinleyDynamic`: John McGinley's
  self-adjusting MA. Single parameter `period`; the recurrence
  `MD + (price - MD) / (0.6 * period * (price / MD)^4)` speeds up when price
  falls below the indicator and damps when price runs above. Seeded with the
  simple average of the first `period` inputs. Exposed in all four bindings.

## [0.2.7] - 2026-05-24

### Added
- **Windows ARM64 is back.** npm Support unblocked the
  `wickra-win32-arm64-msvc` sub-package name (same path
  `wickra-win32-x64-msvc` took through 0.1.4) and transferred write
  access to @kingchenc. 0.2.7 ships the binding for
  `aarch64-pc-windows-msvc` alongside the existing five platforms:
  the `napi.triples.additional` entry, the `optionalDependencies`
  pin, the `bindings/node/npm/win32-arm64-msvc/` sub-package and the
  `windows-11-arm` row of the release.yml node-build matrix are all
  restored from 8aa74cb. `npm install wickra` on Windows ARM64 now
  resolves to a native build instead of failing the loader's
  optional-dep lookup. PyPI's `win_arm64` wheel was unaffected and
  carries through as before.

### Changed
- **Benchmark CPU renamed.** The "Reproduced on" line in every
  README listed an AMD Ryzen 9 7950X3D; the canonical machine is
  actually a Ryzen 9 9950X. Speedup ratios in the tables are
  unchanged (they're relative across libraries on the same machine),
  only the labelling is corrected. The performance-regression issue
  template's CPU example was updated for consistency.

## [0.2.6] - 2026-05-24

### Fixed
- **docs.rs build.** Rust 1.92 removed the `doc_auto_cfg` feature gate
  and folded it back into `doc_cfg` (rust-lang/rust#138907). docs.rs
  builds against the latest nightly and sets `--cfg docsrs`, so every
  published 0.2.x failed with E0557 on the
  `#![cfg_attr(docsrs, feature(doc_auto_cfg))]` line at the top of
  `wickra`, `wickra-core`, and `wickra-data`. GitHub CI didn't see
  this — stable rustc never enables the `docsrs` cfg. The three
  library crates now gate on `doc_cfg` (same intent, same rendered
  output on docs.rs, builds again on nightly).

### Changed
- **README — Wickra is now the top row of every comparison table.**
  The "Why Wickra exists" library matrix and the per-indicator
  benchmark tables previously placed Wickra at the bottom; a reader
  landing on the README is here to compare *against* Wickra, so the
  pivot row belongs at the top with a ★ marker. Same column data,
  same winner annotations — only row order changed. Mirrored across
  the umbrella README and every binding README so crates.io / PyPI /
  npm landing pages stay in sync.

## [0.2.5] - 2026-05-24

### Added
- `BinanceConfig` plus `BinanceKlineStream::connect_with_config(symbols, interval, config)`
  in `wickra-data`'s `live::binance` module. `connect()` keeps its previous
  signature and now forwards to the new entry-point with the defaults, so the
  public API is backwards-compatible. The config lets callers point the
  stream at Binance Testnet (`wss://testnet.binance.vision`) or tune the
  read timeout, reconnect attempt count, initial / capped backoff and frame
  size limits without rewriting the connector.
- README **Disclaimer** section clarifying that Wickra is an indicator
  toolkit (not a trading system) and that any production-trading use is at
  the caller's own risk. The legal terms in [LICENSE](LICENSE) are
  unchanged.

### Changed
- `BinanceKlineStream::next_event` now writes the Pong reply to a server
  `Ping` on a best-effort basis. A failed write means the connection is
  already dead, so the existing timeout / read-error reconnect arm one
  loop iteration later picks it up — the previous explicit reconnect on
  Pong-write failure is gone. Observable behaviour is unchanged for every
  healthy connection.

## [0.2.1] - 2026-05-23

### Changed
- **MSRV bumped.** Workspace minimum supported Rust version is now **1.86**
  (was 1.75) and the Node binding (`wickra-node`) is now **1.88** (was 1.77).
  The bumps are driven by transitive-dependency floors that were lifted in
  recent updates: `criterion 0.8.2` (the bench dev-dep) requires Rust 1.86,
  and `napi-build >= 2.3.2` requires Rust 1.88. Pinning those deps to the
  older versions would have frozen us out of future security fixes from
  those upstreams, so lifting the MSRV is the cleaner path for a young 0.x
  library. Downstream consumers on older Rust toolchains can stay on
  Wickra 0.2.0.
- Bumped the bench dev-dep `criterion` from 0.5 to 0.8 and migrated
  `bindings/wickra/benches/indicators.rs` from the deprecated
  `criterion::black_box` re-export to the stable `std::hint::black_box`.
- Bumped `tokio-tungstenite` from 0.24 to 0.29. `WebSocketConfig` became
  `#[non_exhaustive]` upstream, so the struct-literal construction in
  `crates/wickra-data/src/live/binance.rs` is rewritten to the
  builder-style `WebSocketConfig::default().max_message_size(..).max_frame_size(..)`.
  Same caps, same semantics, same default carry-over.
- Bumped every committed CI/release GitHub Action to its latest pinned
  SHA: `actions/checkout` 4 → 6, `actions/setup-node` 4 → 6,
  `actions/setup-python` 5 → 6, `actions/upload-artifact` 4 → 7,
  `actions/download-artifact` 4 → 8, `softprops/action-gh-release` 2 → 3,
  `codecov/codecov-action` 5 → 6, `taiki-e/install-action` patch.

### Fixed
- `tick_aggregator` gap-fill no longer allocates an unbounded number of
  placeholder candles. The new `MAX_GAP_FILL_CANDLES = 1_000_000` cap
  surfaces an adversarial timestamp jump (e.g. a clock-glitch tick years
  in the future) as `Error::Malformed` instead of an OOM panic. Found by
  the new `tick_aggregator` fuzz target.
- `HistoricalVolatility::geometric_series_yields_zero` now uses an `1e-6`
  tolerance instead of `1e-9`. The mathematical result on a perfectly
  geometric price series is exactly zero, but the underlying
  `1.01_f64.powi(i)` + log-return + std-dev cascade accumulates
  platform-sensitive FP drift on the order of 1e-7 on x86_64 Linux and
  macOS. The widened tolerance stays four decimal places below any
  realistic annualised volatility value while absorbing the drift across
  every supported platform.
- Replaced every `(high + low) / 2.0` test-helper and three real call
  sites (`Ohlcv::median_price`, `Donchian.middle`, `EaseOfMovement.mid`,
  `SuperTrend.hl2`) with `f64::midpoint(high, low)`. The change satisfies
  clippy 1.95's new `manual_midpoint` lint without affecting values
  (`f64::midpoint` matches the naive average to better than 1 ULP for the
  inputs used here).
- Replaced `i.is_multiple_of(2)` (unstable on Rust 1.85) with `i % 2 == 0`
  in the SMA / Bollinger long-stream-drift tests so the workspace MSRV
  job builds cleanly on Rust 1.86.
- The `Compile examples` CI step now invokes
  `cargo build -p wickra-examples --bins` instead of the now-deleted
  `cargo build -p wickra --example backtest` / `-p wickra-data --example
  live_binance` (the Z5 reorganisation moved every runnable example into
  the dedicated `wickra-examples` crate, but the CI step had not been
  updated).
- The `Fuzz (smoke)` CI job installs `cargo-fuzz` from a prebuilt binary
  via `taiki-e/install-action` instead of `cargo install cargo-fuzz`.
  The source install resolved against `rustix 0.36.5`, which uses
  internal `#[rustc_*]` attributes the current nightly compiler rejects.
- The fuzz targets now build with an explicit
  `--target x86_64-unknown-linux-gnu`; cargo-fuzz was defaulting to
  `x86_64-unknown-linux-musl`, which is not installed on the standard
  GitHub-hosted Ubuntu runner.

### Removed
- **`wickra-win32-arm64-msvc` is temporarily omitted from this release.**
  The npm spam-detection filter blocks the first publish of this brand-new
  package name (same situation that affected `wickra-win32-x64-msvc`
  through 0.1.4 until npm Support unblocked it). A support ticket is open;
  once the new name is unblocked the
  `aarch64-pc-windows-msvc` triple will be restored in
  `bindings/node/package.json` (`napi.triples.additional` +
  `optionalDependencies`), in the `release.yml` `node-build` matrix, and
  as a fresh `bindings/node/npm/win32-arm64-msvc/` template. Until then,
  `npm install wickra@0.2.1` on Windows ARM64 will surface the loader's
  standard `Cannot find module 'wickra-win32-arm64-msvc'` error; every
  other platform (Linux x64 / Linux ARM64 / macOS x64 / macOS ARM64 /
  Windows x64) ships normally. The PyPI wheel for Windows ARM64 is
  unaffected and still published.

## [0.2.0] - 2026-05-23

### Fixed
- `HistoricalVolatility::update` no longer substitutes a `0.0` log-return on
  non-positive prices (audit finding R13). Negative or zero prices are
  semantically invalid for a log-return calculation; silently treating them as
  "no movement" underreported realised volatility. They are now skipped — the
  previous valid value is returned and the indicator's state (`prev_price`,
  window, sums) is left untouched — matching how every other indicator handles
  invalid inputs.
- `Tick::new` now returns the new `Error::InvalidTick` variant for negative
  volume instead of `Error::InvalidCandle` (audit finding R14). A tick is not
  a candle, and downstream tick-stream pipelines should be able to match on a
  semantically-correct error. The Python binding's `map_err` was extended to
  forward the new variant as a `ValueError`; the Node and WASM bindings format
  via `Error::to_string()` and pick the new variant up automatically.
- `Psar::is_ready` now matches the convention shared by every other indicator:
  `is_ready() == true` iff a real value has been produced (audit finding R6).
  The previous implementation returned `self.initialised`, which flipped to
  `true` after the seed candle even though the seed candle itself returns
  `None`. A streaming consumer that wrote
  `if ind.is_ready() { use(ind.update(c)?) }` would hit an unexpected `None`
  on the first post-seed update. The fix introduces a `has_emitted` gate set
  when the first `Some` value is returned.
- `Psar::reset` now restores the compute fields (`prev_high`, `prev_low`,
  `sar`, `ep`) to `f64::NAN` sentinels instead of `0.0` (audit Opus-Bonus 1).
  The fields are gated by `initialised` today, so the `0.0` sentinel never
  leaked into output — but a future refactor that read them pre-init would
  have silently treated `0.0` as a real price. A `debug_assert!` at the read
  site makes the invariant explicit.

### Changed
- `Sma` and `BollingerBands` now reseed their incremental `sum` (and `sum_sq`
  for Bollinger) from the live window every `16 · period` finite updates,
  capping floating-point drift on long-running streams (audit findings R7 and
  L2-Rust). Previously the incremental single-subtract `sum -= old` could
  accumulate catastrophic-cancellation error on streams with alternating
  large/small magnitudes; the misleading `sma.rs` comment that claimed the
  drift was already bounded "by recomputing the sum after each pop" is
  replaced with an accurate description of the new reseed strategy. Amortised
  cost stays at O(1) (`O(period)` work amortised over `O(period)` updates),
  values are bit-identical on inputs that did not drift to begin with, and
  two new `long_stream_drift_stays_bounded` tests stress the recompute by
  alternating `1e9` / `1.0` (SMA) and `1e6` / `1.0` (Bollinger) for several
  recompute cycles and verify the reported values track a fresh from-scratch
  computation over the live window.
- `LinearRegression`, `LinRegSlope` and `LinRegAngle` (via composition over
  `LinRegSlope`) now run their rolling ordinary-least-squares fit
  **incrementally** in O(1) per update (audit finding R2). Previously every
  tick refit the line from scratch in O(period). The OLS denominators (`Σx`
  and `Σxx`) depend only on `period`, so they were already precomputed; this
  release adds running `Σy` and `Σxy` accumulators and slides them in closed
  form via the identity
  `new_Σxy = old_Σxy − old_Σy + popped_y₀` (then `Σxy += (n − 1) · new_value`
  and `Σy += new_value`). New per-bar equivalence tests compare the O(1)
  output against a fresh O(n) refit on noisy ramps, step functions, and
  constants — values agree to within 1e-9.
- Fuzz suite expanded from 2 indicators to the full catalogue (audit finding
  R9). The existing `indicator_update` target now exercises every scalar-input
  indicator (~33 classes including MACD and Bollinger Bands); a new
  `indicator_update_candle` target exercises every candle-input indicator (~37
  classes, including ATR, ADX, Stochastic, PSAR, Keltner, SuperTrend,
  ChandelierExit, AwesomeOscillator, OBV, MFI, VWAP, RollingVWAP, and the rest
  of the volume / volatility / trailing-stop / price-statistics families). Each
  iteration sweeps every indicator through both the streaming `update` loop
  and a full `batch` call so any state-mutation bug surfaces on either path.
  CI gains a `fuzz-smoke` job that runs each of the five targets for 30 s on
  every push and pull-request.
- `UlcerIndex::update` now tracks the trailing maximum with a monotonically-
  decreasing deque of `(index, price)` pairs instead of scanning the whole
  trailing window on every tick. The indicator now honours the `Indicator`
  trait's O(1)-per-tick contract; values and warmup semantics are unchanged
  (verified by a new adversarial-input test that compares the deque output
  bar-by-bar against a naive O(n) trailing-max scan on strictly increasing,
  strictly decreasing, constant, and sawtooth inputs). The doc comment on
  `warmup_period()` is also corrected: the two windows overlap by one bar, so
  the formula is `2 * period - 1`.

### Added
- `RollingVWAP` is now exposed in Python, Node and WASM under that name
  (previously the rolling-window VWAP existed only in the Rust core, even
  though the README's volume-family table already advertised
  `VWAP (cumulative + rolling)`). All four bindings now ship the same
  cumulative `VWAP` plus the finite-window `RollingVWAP(period)`. The wiki page
  `Indicator-Vwap.md` adds Python, Node and WASM examples and drops the
  "Rust-only" caveat.
- WASM binding now exposes the streaming `update()` method on every candle-input
  indicator: `Adx`, `WilliamsR`, `Cci`, `Mfi`, `Psar`, `Keltner`, `Donchian`,
  `Vwap`, `AwesomeOscillator`, `Aroon`, `Stochastic`, and `Obv`. Multi-output
  indicators (`Adx`, `Keltner`, `Donchian`, `Aroon`, `Stochastic`) return a
  named JS object (`{ plusDi, minusDi, adx }`, `{ upper, middle, lower }`,
  `{ up, down }`, `{ k, d }`) once warm, or `null` during warmup — matching the
  existing `SuperTrend` convention. Each class also gains `reset()`, `isReady()`
  and `warmupPeriod()`, bringing the WASM surface to full parity with Python
  and Node so browser-side streaming code no longer has to replay `batch()`
  on every tick. `WasmKama` gains the previously missing `warmupPeriod()`.
- New `wasm-bindgen` integration test exercises `update == batch` plus the full
  lifecycle (`reset` / `isReady` / `warmupPeriod`) for all twelve newly wired
  classes against a deterministic 40-bar synthetic OHLCV stream.

### Security
- Upgrade `pyo3` (0.22 → 0.28) and `numpy` (0.22 → 0.28) in the Python binding.
  Fixes [RUSTSEC-2025-0020](https://rustsec.org/advisories/RUSTSEC-2025-0020) —
  a buffer overflow in `PyString::from_object` that affected the published
  Python wheels. The `cargo-deny` ignore entry that previously suppressed the
  advisory has been removed; `cargo deny check` is now clean without
  suppression. Migrated `into_pyarray_bound` to `into_pyarray`,
  `downcast::<PyDict>` to `cast::<PyDict>`, and opted every `#[pyclass]` out of
  the deprecated automatic `FromPyObject` derive via `skip_from_py_object`.

### Added
- 46 new technical indicators, taking the library from 25 to 71 and
  reorganising the catalogue into **eight families**, each with at least five
  members. Every indicator is implemented once in the Rust core and wired
  through the Python, Node and WASM bindings, with reference-value tests and a
  dedicated wiki page:
  - Moving Averages: `Smma`, `Trima`, `Zlema`, `T3`, `Vwma`.
  - Momentum Oscillators: `Mom`, `Cmo`, `Tsi`, `Pmo`, `StochRsi`,
    `UltimateOscillator`.
  - Trend & Directional: `AroonOscillator`, `Vortex`, `MassIndex`,
    `ChoppinessIndex`, `VerticalHorizontalFilter`.
  - Price Oscillators: `Ppo`, `Dpo`, `Coppock`, `AcceleratorOscillator`,
    `BalanceOfPower`.
  - Volatility & Bands: `Natr`, `StdDev`, `UlcerIndex`,
    `HistoricalVolatility`, `BollingerBandwidth`, `PercentB`, `TrueRange`,
    `ChaikinVolatility`.
  - Trailing Stops: `SuperTrend`, `ChandelierExit`, `ChandeKrollStop`,
    `AtrTrailingStop`.
  - Volume: `Adl`, `VolumePriceTrend`, `ChaikinMoneyFlow`,
    `ChaikinOscillator`, `ForceIndex`, `EaseOfMovement`.
  - Price Statistics: `TypicalPrice`, `MedianPrice`, `WeightedClose`,
    `LinearRegression`, `LinRegSlope`, `ZScore`, `LinRegAngle`.
- `TickAggregator::with_gap_fill` — opt-in mode that emits a flat placeholder
  candle for every empty bucket between two ticks, keeping the candle series
  evenly spaced for downstream indicators.
- CSV reader: a leading UTF-8 byte-order mark is stripped, fields are trimmed,
  and the header is validated against the required OHLCV columns.
- CI: an `msrv` job that builds and tests the workspace on Rust 1.75 and the
  node binding on Rust 1.77.
- Community health files: `CONTRIBUTING.md`, `SECURITY.md`,
  `CODE_OF_CONDUCT.md`, issue / pull-request templates, `CODEOWNERS`, and a
  Dependabot configuration.
- Seven example OHLCV datasets under `examples/data/`, one per timeframe
  (1m / 5m / 15m / 1h / 12h / 1d / 1month), holding real BTCUSDT spot klines,
  alongside the `fetch_btcusdt` example that regenerates them from the
  Binance REST API.
- `Timeframe::minutes`, `Timeframe::hours` and `Timeframe::days` convenience
  constructors, each building on seconds with a checked-multiplication
  overflow guard.

### Changed
- The indicator wiki is reorganised into eight family folders under
  `docs/wiki/indicators/` (`moving-averages/`, `momentum-oscillators/`,
  `trend-directional/`, `price-oscillators/`, `volatility-bands/`,
  `trailing-stops/`, `volume/`, `price-statistics/`); `Indicators-Overview.md`,
  `Home.md` and the README indicator table follow the same eight families.
- `TickAggregator::push` returns `Result<Vec<Candle>>` (was
  `Result<Option<Candle>>`) so a single tick can yield a closed bar plus gap
  fillers.
- `Resampler::push` returns `Result<Option<Candle>>`: a candle in a bucket
  earlier than the open bar is now rejected as out of order.
- Aggregated candles are finalised through the validating `Candle::new`, so a
  volume that overflows to a non-finite value is surfaced as an error instead
  of producing a poisoned candle.
- All GitHub Actions are pinned to commit SHAs; the four publish jobs run in a
  protected `release` environment.
- The indicator benchmarks (`crates/wickra/benches/indicators.rs`) now run
  against the checked-in real BTCUSDT 1-minute dataset instead of a synthetic
  price series.
- Every language's examples now live under a uniform `examples/<lang>/`
  tree: Rust moved into a new `examples/rust/` workspace member crate
  (`wickra-examples`, run via `cargo run -p wickra-examples --bin <name>`),
  Node into `examples/node/` with its own `package.json` linking `wickra` via
  `file:../../bindings/node`, and the WASM browser demos into
  `examples/wasm/`. The bundled BTCUSDT datasets move alongside them at
  `examples/data/`. Six new examples close the cross-language parity matrix:
  streaming demos for Python and Rust; multi-timeframe and parallel-assets
  demos for both Rust and Node.
- Cross-language data-generator parity: `examples/python/fetch_btcusdt.py`
  (stdlib only: `urllib` + `json` + `csv`) and `examples/node/fetch_btcusdt.js`
  (Node 18+ built-in `fetch`) mirror the Rust `fetch_btcusdt` binary —
  byte-for-byte identical CSV output on the same Binance snapshot.
- Four additional WebAssembly browser demos under `examples/wasm/`
  alongside the original `index.html`: `backtest.html` (fetch + basket of
  indicators), `live_trading.html` (browser-native `WebSocket` to
  Binance), `multi_timeframe.html` (in-page resample) and
  `parallel_assets.html` + `parallel_worker.js` (module-Worker pool with
  serial-vs-parallel speedup). The cross-language matrix is now closed
  for every cell where the pattern makes sense.
- Three new wiki pages: `TA-Lib-Migration.md` (full mapping table from
  `talib.X(...)` calls to Wickra), `Cookbook.md` (seven concrete
  strategy recipes — RSI mean reversion, MACD crossover, Bollinger
  breakout, ADX-gated trend, multi-timeframe confirmation, SuperTrend,
  chained indicators) and `FAQ.md`. All three linked from `Home.md`.

### Fixed
- `Timeframe::floor` no longer overflows for timestamps near `i64::MIN`.
- The aggregator rejects same-bucket ticks that arrive out of order instead of
  silently overwriting the bar's close with a stale price.
- The Binance live stream reconnects with exponential backoff, skips non-kline
  frames, applies a read timeout and message-size limits, and tracks a closed
  flag.
- Example scripts: `live_trading.py` skips non-kline frames and validates the
  symbol/interval; `backtest.py` and `multi_timeframe.py` report clear errors
  for malformed CSV input.

## [0.1.4] - 2026-05-21

### Added
- GitHub Release runs now attach every built artefact (wheels, sdist, native
  Node binaries, npm-pack tarballs, cargo `.crate` files) to the tag's
  release page.

## [0.1.3] - 2026-05-21

### Fixed
- npm package ships the napi-generated loader and is built with `--platform`
  so the per-platform binary is resolved correctly.

## [0.1.2] - 2026-05-21

### Fixed
- Release pipeline: per-platform idempotent npm publishing with a spam-filter
  retry, and committed `npm/<platform>/` package templates.

## [0.1.1] - 2026-05-21

### Fixed
- Node publish step and coordinated version bump across all bindings.

## [0.1.0] - 2026-05-21

### Added
- Initial release: a streaming-first technical-analysis library with 25
  indicators (SMA, EMA, WMA, DEMA, TEMA, HMA, KAMA, RSI, MACD, ROC, Stochastic,
  CCI, Williams %R, ADX, MFI, TRIX, Aroon, Awesome Oscillator, Bollinger Bands,
  ATR, Keltner Channels, Donchian Channels, Parabolic SAR, OBV, VWAP).
- Rust core (`wickra-core`), umbrella crate (`wickra`), and a data layer
  (`wickra-data`) with a CSV reader, tick aggregator, resampler, and an
  optional Binance live feed.
- Bindings for Python, Node.js, and WebAssembly.

[Unreleased]: https://github.com/wickra-lib/wickra/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/wickra-lib/wickra/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/wickra-lib/wickra/compare/v0.5.9...v0.6.0
[0.5.9]: https://github.com/wickra-lib/wickra/compare/v0.5.8...v0.5.9
[0.5.8]: https://github.com/wickra-lib/wickra/compare/v0.5.7...v0.5.8
[0.5.7]: https://github.com/wickra-lib/wickra/compare/v0.5.6...v0.5.7
[0.5.6]: https://github.com/wickra-lib/wickra/compare/v0.5.5...v0.5.6
[0.5.5]: https://github.com/wickra-lib/wickra/compare/v0.5.4...v0.5.5
[0.5.4]: https://github.com/wickra-lib/wickra/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/wickra-lib/wickra/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/wickra-lib/wickra/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/wickra-lib/wickra/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/wickra-lib/wickra/compare/v0.4.7...v0.5.0
[0.4.7]: https://github.com/wickra-lib/wickra/compare/v0.4.6...v0.4.7
[0.4.6]: https://github.com/wickra-lib/wickra/compare/v0.4.5...v0.4.6
[0.4.5]: https://github.com/wickra-lib/wickra/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/wickra-lib/wickra/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/wickra-lib/wickra/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/wickra-lib/wickra/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/wickra-lib/wickra/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/wickra-lib/wickra/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/wickra-lib/wickra/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/wickra-lib/wickra/compare/v0.2.7...v0.3.0
[0.2.7]: https://github.com/wickra-lib/wickra/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/wickra-lib/wickra/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/wickra-lib/wickra/compare/v0.2.1...v0.2.5
[0.2.1]: https://github.com/wickra-lib/wickra/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/wickra-lib/wickra/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/wickra-lib/wickra/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/wickra-lib/wickra/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/wickra-lib/wickra/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wickra-lib/wickra/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wickra-lib/wickra/releases/tag/v0.1.0
