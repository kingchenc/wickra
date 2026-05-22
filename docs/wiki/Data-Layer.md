# Data Layer (`wickra-data`)

`wickra-data` is a separate crate that feeds candles into Wickra's indicators.
It is not part of `wickra-core` — depend on it explicitly:

```toml
[dependencies]
wickra = "0.1"
wickra-data = "0.1"
```

It provides four pieces:

- a streaming OHLCV **CSV reader**,
- a **tick-to-candle aggregator**,
- a **candle resampler** for multi-timeframe analysis,
- an optional **Binance Spot WebSocket** kline feed (feature `live-binance`).

## CSV reader

`CandleReader` streams OHLCV rows out of a CSV file into validated `Candle`
values.

```rust
use wickra_data::csv::CandleReader;

let mut reader = CandleReader::open("ohlcv.csv")?;
let candles = reader.read_all()?;          // Vec<Candle>

// Or stream row by row without buffering the whole file:
let mut reader = CandleReader::open("ohlcv.csv")?;
for candle in reader.candles() {
    let candle = candle?;
    // feed `candle` into an indicator...
}
```

The reader is defensive about real-world files:

- The first line **must** be a header naming the columns
  `timestamp,open,high,low,close,volume`. A missing column, or a file with no
  header at all, is rejected with a clear `Error::Malformed` instead of
  silently consuming the first data row.
- A leading UTF-8 byte-order mark (Excel exports one) is stripped.
- Whitespace around values is trimmed.
- Each row is validated through `Candle::new`, so an inconsistent OHLC row
  (e.g. `high < low`) surfaces as an error.

## Tick aggregator

`TickAggregator` rolls a stream of trade `Tick`s up into `Candle`s of an
arbitrary timeframe. The timeframe's bucket size is in the same unit as the
tick timestamps (milliseconds for Binance, seconds for daily bars, …).

```rust
use wickra_data::aggregator::{TickAggregator, Timeframe};
use wickra_core::Tick;

let mut agg = TickAggregator::new(Timeframe::one_minute_ms());

for tick in trade_feed {
    // push returns every candle that closed because of this tick —
    // empty while the bar grows, one candle when a bar boundary is crossed.
    for closed in agg.push(tick)? {
        // feed `closed` into an indicator...
    }
}

// Capture the final, still-open bar at the end of the stream.
if let Some(last) = agg.flush()? {
    // ...
}
```

Out-of-order ticks — across or within a bucket — are rejected with
`Error::Malformed` rather than silently corrupting a bar.

### Gap filling

By default a tick that jumps across one or more empty buckets simply opens
the next non-empty bar, leaving a time hole in the output. Enable
`with_gap_fill` to emit a flat placeholder candle
(`open == high == low == close`, `volume == 0`) for every skipped bucket, so
downstream indicators see an unbroken, evenly spaced series:

```rust
let mut agg = TickAggregator::new(Timeframe::one_minute_ms()).with_gap_fill(true);
```

## Resampler

`Resampler` rolls an existing candle stream up to a coarser timeframe — for
example 1-minute bars into 5-minute bars, without touching the original tick
stream.

```rust
use wickra_data::aggregator::Timeframe;
use wickra_data::resample::{resample_all, Resampler};

// One-shot over an iterator:
let five_min = resample_all(Timeframe::millis(5 * 60_000), one_min_candles)?;

// Or incrementally:
let mut r = Resampler::new(Timeframe::millis(60 * 60_000)); // 1-hour bars
for candle in one_min_candles {
    if let Some(closed) = r.push(candle?)? {
        // a coarser bar just closed
    }
}
let last = r.flush()?;
```

The output timeframe's bucket must be a multiple of the input timeframe's
bucket — picking sensible aggregations (1m → 5m → 1h) is the caller's
responsibility. A candle that arrives in a bucket earlier than the open bar
is rejected as out of order.

## Binance live feed

With the `live-binance` feature enabled, `BinanceKlineStream` connects to the
Binance Spot WebSocket and yields closed klines as candles.

```toml
wickra-data = { version = "0.1", features = ["live-binance"] }
```

```rust
use wickra::{Indicator, Rsi};
use wickra_data::live::binance::{BinanceKlineStream, Interval};

let mut stream =
    BinanceKlineStream::connect(&["BTCUSDT".into()], Interval::OneMinute).await?;
let mut rsi = Rsi::new(14)?;

while let Some(event) = stream.next_event().await? {
    if event.is_closed {
        if let Some(v) = rsi.update(event.candle.close) {
            println!("RSI = {v:.2}");
        }
    }
}
```

The stream is resilient: it reconnects with exponential backoff after a
dropped connection, skips non-kline frames (subscription acks, heartbeats),
applies a read timeout and message-size limits, and tracks a closed flag so a
deliberately closed stream is not reused.

A runnable example lives at `crates/wickra-data/examples/live_binance.rs`:

```bash
cargo run -p wickra-data --example live_binance --features live-binance
```

## See also

- [Quickstart: Rust](Quickstart-Rust.md) — the core indicator API.
- [Indicators Overview](Indicators-Overview.md) — every indicator and its
  parameters.
- Source: <https://github.com/kingchenc/wickra>
