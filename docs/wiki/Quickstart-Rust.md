# Quickstart: Rust

A five-minute tour of the Wickra Rust crate. By the end you will have run a
batch SMA, fed an RSI tick by tick, and composed two indicators with `Chain`.

## Install

```bash
cargo add wickra
```

The default features pull in `parallel` (rayon-based `batch_parallel`); turn
them off with `cargo add wickra --no-default-features` if you want a leaner
build. The `wickra` crate is a thin façade that re-exports everything from
`wickra-core`; you can also depend on `wickra-core` directly if you want to
skip the façade.

The published crate is at version `0.2.1` on
[crates.io](https://crates.io/crates/wickra).

## The `Indicator` trait in 30 seconds

Every indicator implements the same trait:

```rust
pub trait Indicator {
    type Input;
    type Output;
    fn update(&mut self, input: Self::Input) -> Option<Self::Output>;
    fn reset(&mut self);
    fn warmup_period(&self) -> usize;
    fn is_ready(&self) -> bool;
    fn name(&self) -> &'static str;
}
```

`update` is O(1) in the input length. The companion trait `BatchExt` is a
blanket extension that adds a `batch(&[Self::Input])` method to every
indicator — its default implementation is literally a loop over `update`, so
batch and streaming results are bit-for-bit identical.

## Batch and streaming side by side

```rust
use wickra::{BatchExt, Indicator, Rsi, Sma};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Batch: SMA(3) over five prices.
    let mut sma = Sma::new(3)?;
    let out: Vec<Option<f64>> = sma.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    println!("{:?}", out);
    // -> [None, None, Some(2.0), Some(3.0), Some(4.0)]

    // 2. Streaming: feed Wilder's textbook example into RSI(14).
    let mut rsi = Rsi::new(14)?;
    let prices = [
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08,
        45.89, 46.03, 45.61, 46.28, 46.28, 46.00, 46.03, 46.41,
    ];
    for (tick, price) in prices.iter().enumerate() {
        if let Some(v) = rsi.update(*price) {
            println!("tick {:2}  close={:.2}  rsi={:.4}", tick + 1, price, v);
        }
    }
    Ok(())
}
```

The streaming loop prints:

```
tick 15  close=46.28  rsi=70.4641
tick 16  close=46.00  rsi=66.2496
tick 17  close=46.03  rsi=66.4809
tick 18  close=46.41  rsi=69.3469
```

The first value lands on tick 15 because `Rsi::new(14)?.warmup_period() == 15`
(14 diffs to seed Wilder's smoothing, so the 15th input emits the first RSI).
The `70.4641` value matches the textbook value pinned by the unit test
`classic_wilder_textbook_values` in `crates/wickra-core/src/indicators/rsi.rs`.

## Composing indicators with `Chain`

`Chain<A, B>` wires the output of `A` straight into the input of `B`, provided
both stages agree on `f64` as the bridging type. The chain itself is an
`Indicator`, so you can stack three stages with `.then(c)`, or four with
`.then(c).then(d)`.

```rust
use wickra::{Chain, Ema, Indicator, Rsi};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // RSI(7) computed on the output of EMA(14).
    let mut chain = Chain::new(Ema::new(14)?, Rsi::new(7)?);
    for i in 1..=22 {
        if let Some(v) = chain.update(f64::from(i)) {
            println!("chain emitted at input #{i}: {v}");
        }
    }
    println!("chain.warmup_period() = {}", chain.warmup_period());
    Ok(())
}
```

Output:

```
chain emitted at input #21: 100
chain emitted at input #22: 100
chain.warmup_period() = 22
```

`Ema::new(14)` needs 14 inputs to seed and `Rsi::new(7)` needs 8 more once
the EMA starts flowing, so the chain emits its first value at input 21. The
`warmup_period()` reported by `Chain` is a conservative `first + second` sum
(here `14 + 8 = 22`); see [Indicator Chaining](Indicator-Chaining.md) for the
exact contract.

## A deeper example

`examples/rust/src/bin/backtest.rs` (in the `wickra-examples` workspace
crate) computes a panel of indicators (RSI, EMA, Bollinger, MACD, ATR, ADX,
OBV) over an OHLCV CSV by way of `wickra-data`:

```bash
cargo run --release -p wickra-examples --bin backtest -- path/to/ohlcv.csv
```

For live-data work, `wickra-data` ships a streaming CSV reader, a
tick-to-candle aggregator, a candle resampler, and a Binance kline WebSocket
adapter under the `live-binance` feature. `examples/rust/src/bin/live_binance.rs`
is the canonical example for the latter.

## See also

- [Quickstart: Python](Quickstart-Python.md) — same engine, NumPy-flavoured.
- [Streaming vs Batch](Streaming-vs-Batch.md) — the `batch == repeated update`
  contract and the benchmark numbers it buys you.
- [Indicator Chaining](Indicator-Chaining.md) — three-stage chains and the
  stacked-warmup rule.
- Source: <https://github.com/kingchenc/wickra>
