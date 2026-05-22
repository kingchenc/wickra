# Cookbook

Practical strategy recipes built on Wickra's streaming indicators. Each
recipe is a small, runnable snippet you can drop into a backtest loop or a
live trading bot. Both paths share the same indicator state, so the same
recipe works in either mode — see [Streaming vs Batch](Streaming-vs-Batch.md).

## 1. RSI mean reversion

Enter when RSI crosses out of an extreme; flatten when it returns to
neutral.

```python
import wickra as ta

rsi = ta.RSI(14)
position = 0  # 0 flat, +1 long, −1 short
for price in price_feed:
    v = rsi.update(price)
    if v is None:
        continue
    if position == 0 and v < 30:
        position = 1
        print(f"BUY at {price:.2f}")
    elif position == 1 and v > 50:
        position = 0
        print(f"EXIT long at {price:.2f}")
    elif position == 0 and v > 70:
        position = -1
        print(f"SHORT at {price:.2f}")
    elif position == -1 and v < 50:
        position = 0
        print(f"COVER short at {price:.2f}")
```

## 2. MACD histogram crossover

Trade in the direction of a MACD-histogram sign change. Zero-crossings of
the histogram (`MACD − signal`) are the canonical trigger and lead the
slower MACD-vs-signal line cross.

```rust
use wickra::{Indicator, MacdIndicator};

let mut macd = MacdIndicator::classic(); // (12, 26, 9)
let mut last_hist: Option<f64> = None;
for &price in &prices {
    if let Some(v) = macd.update(price) {
        if let Some(prev) = last_hist {
            if prev <= 0.0 && v.histogram > 0.0 {
                println!("BUY: MACD histogram turned positive at {price:.2}");
            } else if prev >= 0.0 && v.histogram < 0.0 {
                println!("SELL: MACD histogram turned negative at {price:.2}");
            }
        }
        last_hist = Some(v.histogram);
    }
}
```

## 3. Bollinger band breakout

Trade in the direction of a band-piercing close, taking the bands as a
dynamic support / resistance.

```python
import wickra as ta

bb = ta.BollingerBands(20, 2.0)
for price in price_feed:
    out = bb.update(price)
    if out is None:
        continue
    upper, _middle, lower, _stddev = out
    if price > upper:
        print(f"BREAKOUT (long): {price:.2f} > upper {upper:.2f}")
    elif price < lower:
        print(f"BREAKOUT (short): {price:.2f} < lower {lower:.2f}")
```

## 4. ADX-gated trend filter

Take EMA-crossover signals only when ADX confirms a trend is in place.
This is a textbook way to silence whipsaws in a ranging market.

```python
import wickra as ta

ema_fast = ta.EMA(20)
ema_slow = ta.EMA(50)
adx = ta.ADX(14)

for high, low, close in candle_feed:
    f = ema_fast.update(close)
    s = ema_slow.update(close)
    a = adx.update(high, low, close)   # (plus_di, minus_di, adx) or None
    if f is None or s is None or a is None:
        continue
    _, _, adx_v = a
    if adx_v < 25:
        continue  # ranging market — skip
    if f > s:
        print(f"LONG: EMA20 > EMA50, ADX={adx_v:.1f}")
    elif f < s:
        print(f"SHORT: EMA20 < EMA50, ADX={adx_v:.1f}")
```

## 5. Multi-timeframe confirmation

Only take a 1-minute entry when the 1-hour trend agrees. With Wickra you
keep one streaming indicator per timeframe and feed each only the candles
that belong to it. `wickra-data`'s [`Resampler`](Data-Layer.md) rolls one
candle stream up into a coarser one; the canonical example is
`examples/rust/src/bin/multi_timeframe.rs`.

```rust
use wickra::{Indicator, Rsi};

let mut rsi_1m = Rsi::new(14)?;
let mut rsi_1h = Rsi::new(14)?;

for candle in one_min_candles {
    let fast = rsi_1m.update(candle.close);

    if candle.is_hour_close {
        let slow = rsi_1h.update(candle.close);
        if let (Some(f), Some(s)) = (fast, slow) {
            if f > 70.0 && s > 50.0 {
                println!("strong overbought (1m {f:.1} / 1h {s:.1})");
            } else if f < 30.0 && s < 50.0 {
                println!("strong oversold (1m {f:.1} / 1h {s:.1})");
            }
        }
    }
}
```

## 6. SuperTrend trailing stop

`SuperTrend` is a single-line ATR-banded trailing stop with explicit flip
logic — drop it into a long-only loop to manage exits:

```python
import wickra as ta

st = ta.SuperTrend(10, 3.0)
position = 0  # 0 flat, +1 long
for high, low, close in candle_feed:
    out = st.update(high, low, close)
    if out is None:
        continue
    value, direction = out
    if direction > 0 and position == 0:
        position = 1
        print(f"BUY at {close:.2f}, stop={value:.2f}")
    elif direction < 0 and position == 1:
        position = 0
        print(f"EXIT at {close:.2f} (SuperTrend flipped)")
```

## 7. Chained indicators

When you want an indicator computed *over the output of another*, use the
Rust `Chain` combinator. The chain itself implements `Indicator`, so you
can nest, stack, and feed it into anything that takes an indicator.

```rust
use wickra::{BatchExt, Chain, Ema, Rsi};

// RSI(7) of EMA(14)-smoothed closes.
let mut chain = Chain::new(Ema::new(14)?, Rsi::new(7)?);
let out: Vec<Option<f64>> = chain.batch(&prices);
```

See [Indicator Chaining](Indicator-Chaining.md) for the chained-warmup rule
and three-stage examples.

## See also

- [Indicators Overview](Indicators-Overview.md) — pick the right indicator
  for the question you are asking.
- [Streaming vs Batch](Streaming-vs-Batch.md) — why these recipes work
  bit-identically in both modes.
- [Data Layer](Data-Layer.md) — `Resampler` and the bundled BTCUSDT
  datasets for live multi-timeframe work.
