# Streaming vs Batch

Wickra has one engine, not two. Every indicator is a state machine driven by
a single method, `Indicator::update`, and the batch API is a thin loop over
that method. This page is the concept doc for why that matters, and what
contracts you can rely on when you mix the two in real code.

## The `update` contract

`Indicator::update` is the only state transition. From `crates/wickra-core/src/traits.rs`:

```rust
pub trait Indicator {
    type Input;
    type Output;

    /// Feed one new data point into the indicator and return the freshly computed
    /// output, or `None` if the indicator is still warming up.
    fn update(&mut self, input: Self::Input) -> Option<Self::Output>;

    fn reset(&mut self);
    fn warmup_period(&self) -> usize;
    fn is_ready(&self) -> bool;
    fn name(&self) -> &'static str;
}
```

Three properties hold by contract:

1. **O(1) in the input length.** `update` may touch some pre-existing
   buffered state, but it must never recompute over the entire history. The
   `wickra-core` crate is `#![forbid(unsafe_code)]`, and the standard
   indicator implementations all carry rolling sums, single recursive
   accumulators, or fixed-size `VecDeque` windows.
2. **`None` during warmup, `Some` thereafter.** An indicator returns `None`
   while it doesn't yet have enough data to produce a defined value. After
   the first `Some`, it never goes back to `None` (short of a `reset()`).
3. **`reset()` restores construction-time state.** The state-machine is
   fully encapsulated, so resetting and replaying produces bit-identical
   results to a fresh instance.

## The `BatchExt` blanket implementation

The batch API is a blanket extension on top of every `Indicator`. The whole
implementation is six lines:

```rust
pub trait BatchExt: Indicator {
    fn batch(&mut self, inputs: &[Self::Input]) -> Vec<Option<Self::Output>>
    where Self::Input: Clone,
    {
        let mut out = Vec::with_capacity(inputs.len());
        for x in inputs {
            out.push(self.update(x.clone()));
        }
        out
    }
}

impl<T: Indicator> BatchExt for T {}
```

Two consequences:

- **`batch == repeated update`, exactly.** There is no separate "vectorised"
  code path that might disagree numerically with the streaming one. A unit
  test pinning this invariant — `batch_equals_streaming` — lives in nearly
  every `crates/wickra-core/src/indicators/<name>.rs` file. You can rely on
  the batch results in your backtest matching the streaming results that
  your live bot will see.
- **Implementing one trait is enough.** Adding a new indicator means
  implementing `Indicator` in Rust; every binding plus every batch helper
  comes along for free.

You can verify the equivalence yourself in Python:

```python
import numpy as np
import wickra as ta

np.random.seed(0)
prices = np.cumsum(np.random.randn(100)) + 100.0

# Batch path.
batch_out = ta.RSI(14).batch(prices)

# Streaming path: same inputs, fresh indicator, fed one at a time.
rsi = ta.RSI(14)
stream_out = np.array(
    [np.nan if (v := rsi.update(p)) is None else v for p in prices]
)

b_nan = np.isnan(batch_out)
s_nan = np.isnan(stream_out)
assert np.array_equal(b_nan, s_nan)
assert np.array_equal(batch_out[~b_nan], stream_out[~s_nan])
```

This passes; the last three values of both arrays are
`[69.64533252, 70.00767057, 71.18111330]`.

## Why batch-only libraries fall behind live

Suppose a strategy looks at RSI(14) on each new minute-bar of a market. A
classical batch-only library (TA-Lib, pandas-ta, finta, ...) gives you a
single function `rsi(prices)` that recomputes the indicator over the entire
input array. To use it inside a streaming loop, you concatenate each new
tick onto your history and call `rsi(history)` again. That's
`O(n)` work for every new bar, and the gap widens linearly as `n` grows.

Wickra's `update` is the opposite: each new bar is O(1) because the
recursive smoothing state is already inside the indicator. You never carry
history just to recompute it.

The numbers below are reproduced from the project README, where
`python -m benchmarks.compare_libraries` is the source script.

### Batch — single full pass over a 20 000-bar series

| Indicator           | Wickra              | finta                       | talipp                        |
|---------------------|---------------------|-----------------------------|-------------------------------|
| SMA(20)             | **95.6 µs**         | 343.5 µs (3.6× slower)      | 7 640.6 µs (79.9× slower)     |
| EMA(20)             | **64.6 µs**         | 223.1 µs (3.5× slower)      | 12 160.9 µs (188.2× slower)   |
| RSI(14)             | **126.2 µs**        | 1 107.1 µs (8.8× slower)    | 15 792.2 µs (125.1× slower)   |
| MACD(12, 26, 9)     | **119.0 µs**        | 531.8 µs (4.5× slower)      | 49 788.1 µs (418.2× slower)   |
| Bollinger(20, 2.0)  | **105.3 µs**        | 812.0 µs (7.7× slower)      | 130 938.3 µs (1 243.7× slower)|
| ATR(14)             | **123.5 µs**        | 5 144.8 µs (41.7× slower)   | 28 816.0 µs (233.4× slower)   |

### Streaming — per-tick latency after seeding with 5 000 historical bars

| Indicator | Wickra (per tick)   | talipp (per tick)         |
|-----------|---------------------|---------------------------|
| RSI(14)   | **0.119 µs**        | 1.644 µs (13.8× slower)   |

The streaming gap widens linearly with how much history a batch-only library
has to recompute on every new tick; the table above is the gap at a 5 000-bar
seed followed by 15 000 live updates.

## Practical consequences

- **Mix freely.** A common pattern is "warm up the indicator on historical
  bars in one `batch` call, then drive it tick-by-tick with `update` for
  live data". This is correct because the two paths share state.
- **`is_ready()` is the safe gate.** Don't use a `len(prices) > warmup_period`
  check; trust the indicator's `is_ready()` method, which is `true` exactly
  when at least one `Some` value has been emitted.
- **Multi-output indicators NaN/None together.** Every column of a MACD or
  Bollinger batch transitions from `NaN` to a real value on the same row.
  Use `~np.isnan(out[:, 0])` (Python) or `Number.isFinite(row[0])`
  (Node) as a single mask across all columns.

## See also

- [Quickstart: Python](Quickstart-Python.md) — concrete Python usage of both
  paths.
- [Quickstart: Rust](Quickstart-Rust.md) — the `BatchExt` trait and `?`
  error handling.
- [Warmup Periods](Warmup-Periods.md) — the exact `warmup_period()` for
  every indicator.
- Source: <https://github.com/kingchenc/wickra>
