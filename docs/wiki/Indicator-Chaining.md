# Indicator Chaining

`Chain<A, B>` wires the output of one indicator straight into the input of
another. Both stages must agree on `f64` as the bridging type, which is the
case for the vast majority of price-in / value-out indicators. The chain
itself is an `Indicator`, so chains can be nested arbitrarily and used
anywhere a single indicator is accepted.

This page documents the public API of `Chain` in
`crates/wickra-core/src/traits.rs`, the worked EMA(14) → RSI(7) example
that the doctest pins, and the warmup-stacking rule.

## Construction

```rust
use wickra::{Chain, Ema, Rsi};

// Two stages.
let chain = Chain::new(Ema::new(14)?, Rsi::new(7)?);

// Three stages — note the `.then(third)` builder method.
let triple = Chain::new(Ema::new(14)?, Ema::new(5)?).then(Rsi::new(7)?);
```

The type of `triple` is `Chain<Chain<Ema, Ema>, Rsi>`. You can keep chaining
indefinitely; the `Chain<A, B>` produced at each step also implements
`Indicator<Input = f64, Output = ...>`, which is the constraint
`.then(third)` needs to satisfy.

Both `first()` and `second()` accessors return references to the underlying
stages if you need to inspect them; the chain owns its stages by value.

## Worked example: EMA(14) → RSI(7)

This is the canonical chain example from the doctest on `Chain` in
`crates/wickra-core/src/traits.rs`:

```rust
use wickra::{Chain, Ema, Indicator, Rsi};

let mut chain = Chain::new(Ema::new(14)?, Rsi::new(7)?);
for i in 1..=21 {
    chain.update(f64::from(i));
}
assert!(chain.is_ready());
```

The semantic shape of the chain is: smooth the input series with an
EMA(14), then compute an RSI(7) over the **smoothed** series — not the
raw inputs. `chain.update(price)` is the only thing your caller code ever
sees; the EMA-then-RSI plumbing is internal to the `Chain` value.

The chain emits its first non-`None` value at input **21**:

```rust
let mut chain = Chain::new(Ema::new(14)?, Rsi::new(7)?);
for i in 1..=22 {
    if let Some(v) = chain.update(f64::from(i)) {
        println!("chain emitted at input #{i}: {v}");
    }
}
println!("chain.warmup_period() = {}", chain.warmup_period());
```

Output:

```
chain emitted at input #21: 100
chain emitted at input #22: 100
chain.warmup_period() = 22
```

(The value is `100` because the input is the monotonic ramp `1, 2, ..., 22`;
an RSI on a strictly increasing series is `100` by construction. The point
is the *timing* of the first emission.)

## Why first emission is at input 21, and `warmup_period` is 22

`Chain::warmup_period` is implemented conservatively:

```rust
fn warmup_period(&self) -> usize {
    self.first.warmup_period() + self.second.warmup_period()
}
```

For `Chain::new(Ema::new(14)?, Rsi::new(7)?)` this expands to
`14 + 8 = 22` (RSI(7)'s warmup is `period + 1 = 8` — see
[Warmup Periods](Warmup-Periods.md) for the off-by-one detail).

In practice the chain emits one input earlier than that conservative sum
because the moment EMA(14) starts producing values is input 14, and RSI(7)
needs 8 EMA outputs to seed, so RSI(7) is ready on EMA output number 8,
which corresponds to input 14 + 7 = **21**. The conservative formula
`first.warmup + second.warmup` ignores this overlap; treat
`warmup_period()` as an upper bound and `is_ready()` as the source of truth
for "can I read a value yet":

```rust
if chain.is_ready() {
    if let Some(v) = chain.update(price) {
        // ...
    }
}
```

## State, reset, and `Send`

`Chain<A, B>` propagates `reset()` to both stages:

```rust
fn reset(&mut self) {
    self.first.reset();
    self.second.reset();
}
```

So calling `chain.reset()` returns the whole pipeline to the state of a
freshly constructed `Chain::new(A::new(...), B::new(...))`. Because both
stages are owned by value and the trait `Indicator` is auto-derive-friendly,
the chain inherits `Clone`, `Debug`, and (where each stage is) `Send`.

The `batch` extension comes through `BatchExt` automatically — there's
nothing chain-specific to call:

```rust
use wickra::{BatchExt, Chain, Ema, Rsi};

let mut chain = Chain::new(Ema::new(14)?, Rsi::new(7)?);
let out: Vec<Option<f64>> = chain.batch(&prices);
```

## Stages from different families

The bridging type is `f64`, so anything `Indicator<Input = f64, Output = f64>`
can serve as the first stage and anything `Indicator<Input = f64>` can serve
as the second (or third, or fourth). Indicators that consume `Candle` /
`Tick` (`Atr`, `Adx`, `Stochastic`, `Mfi`, `Vwap`, `Psar`, `Keltner`,
`Donchian`, `Aroon`, `AwesomeOscillator`, `Obv`) cannot sit at the second
stage of a chain because their `Input` is not `f64`. They can still be used
as standalone indicators alongside a chain — wire them in your own loop.

A multi-output indicator like `MacdIndicator` or `BollingerBands` can sit
last in a chain (its `Output` is a struct, but the chain only requires
`Input = f64` on the **second** stage). For example,
`Chain::new(Sma::new(20)?, MacdIndicator::classic())` is a valid type:
MACD-of-smoothed-prices.

## Python and Node

`Chain` is currently a Rust-only construct; the Python and Node bindings
expose individual indicators only. The straightforward equivalent in those
languages is a manual two-step loop:

```python
import wickra as ta

ema = ta.EMA(14)
rsi = ta.RSI(7)

for price in prices:
    smoothed = ema.update(price)
    if smoothed is not None:
        chained = rsi.update(smoothed)
        if chained is not None:
            ...
```

This is exactly what `Chain::update` does in Rust, transcribed to the
binding's `update` method. No information is lost.

## See also

- [Quickstart: Rust](Quickstart-Rust.md) — the `Chain` example in context.
- [Warmup Periods](Warmup-Periods.md) — the underlying stage formulas, and
  the RSI `period + 1` off-by-one.
- [Streaming vs Batch](Streaming-vs-Batch.md) — `Chain` works with both
  paths automatically, via `BatchExt`.
- Source: <https://github.com/kingchenc/wickra>
