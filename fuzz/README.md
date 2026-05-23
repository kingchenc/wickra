# Fuzzing Wickra

[`cargo-fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html) harnesses for
the parsing and stateful entry points of Wickra. Fuzzing requires a nightly
Rust toolchain.

## Setup

```bash
cargo install cargo-fuzz
rustup toolchain install nightly
```

## Targets

| Target | What it exercises |
| --- | --- |
| `csv_reader` | `CandleReader` over arbitrary bytes — headers, cells, BOM, binary noise. |
| `binance_envelope` | `RawWsEnvelope` deserialization from arbitrary strings. |
| `indicator_update` | Every scalar-input indicator (SMA / EMA / WMA / RSI / DEMA / TEMA / HMA / ROC / TRIX / SMMA / TRIMA / ZLEMA / KAMA / T3 / MOM / CMO / TSI / PMO / StochRSI / DPO / PPO / Coppock / StdDev / UlcerIndex / HistoricalVolatility / LinearRegression / LinRegSlope / LinRegAngle / VHF / ZScore / MACD / Bollinger) streamed + batched over arbitrary `f64` sequences (NaN, ±inf, jumps). |
| `indicator_update_candle` | Every candle-input indicator (ATR, NATR, TrueRange, ChaikinVolatility, Keltner, Donchian, PSAR, SuperTrend, ChandelierExit, ChandeKrollStop, ATRTrailingStop, ADX, Aroon, AroonOscillator, Vortex, MassIndex, ChoppinessIndex, CCI, WilliamsR, AwesomeOscillator, AcceleratorOscillator, UltimateOscillator, BalanceOfPower, OBV, MFI, VWAP, RollingVWAP, VWMA, ADL, VPT, CMF, ChaikinOscillator, ForceIndex, EaseOfMovement, TypicalPrice, MedianPrice, WeightedClose, Stochastic) streamed + batched over fuzz-derived OHLCV candles. |
| `tick_aggregator` | `TickAggregator` over arbitrary `(price, volume, timestamp)` triples. |

## Run

```bash
# From the repository root:
cargo +nightly fuzz run csv_reader
cargo +nightly fuzz run binance_envelope
cargo +nightly fuzz run indicator_update
cargo +nightly fuzz run indicator_update_candle
cargo +nightly fuzz run tick_aggregator
```

Each run continues until a crash is found or it is interrupted. A short
time-boxed smoke run is useful in CI:

```bash
cargo +nightly fuzz run csv_reader -- -max_total_time=60
```

The expectation for every target is that it never panics: malformed or
adversarial input must surface as an `Err`, never a crash.
