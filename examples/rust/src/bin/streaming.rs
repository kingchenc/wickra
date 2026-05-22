//! Streaming indicators with the Wickra Rust crate.
//!
//! Feeds a synthetic price series through several indicators tick by tick —
//! the same O(1)-per-update model a live trading bot would use — and prints
//! a status line whenever every indicator has warmed up. The Rust
//! counterpart of `examples/python/streaming.py` and
//! `examples/node/streaming.js`.
//!
//! Build with:
//! ```text
//! cargo run --release -p wickra-examples --bin streaming
//! ```

use std::env;

use wickra::{Ema, Indicator, MacdIndicator, Rsi, Sma};

const DEFAULT_TICKS: usize = 120;

/// Deterministic synthetic series matching the Node sibling example's
/// seeded LCG, so a side-by-side run produces visibly comparable streams.
fn make_series(n: usize) -> Vec<f64> {
    let mut seed: u64 = 1_234_567;
    (0..n)
        .map(|t| {
            seed = (seed.wrapping_mul(1_103_515_245).wrapping_add(12_345)) & 0x7FFF_FFFF;
            let rand = seed as f64 / 0x7FFF_FFFF_u64 as f64;
            let tf = t as f64;
            100.0 + tf * 0.05 + (tf * 0.07).sin() * 8.0 + (tf * 0.21).cos() * 3.0 + (rand - 0.5)
        })
        .collect()
}

fn fmt(v: Option<f64>) -> String {
    match v {
        Some(x) if x.is_finite() => format!("{x:7.2}"),
        _ => "  --   ".to_string(),
    }
}

fn parse_ticks() -> Result<usize, Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None => Ok(DEFAULT_TICKS),
        Some("--ticks") => match args.next() {
            Some(n) => n
                .parse::<usize>()
                .map_err(|e| format!("--ticks: {e}").into()),
            None => Err("--ticks requires a value".into()),
        },
        Some(other) => Err(format!("unexpected argument: {other}").into()),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticks = parse_ticks()?;
    if ticks == 0 {
        return Err("--ticks must be positive".into());
    }

    println!("Wickra streaming indicator demo (Rust)\n");

    let mut sma = Sma::new(20)?;
    let mut ema = Ema::new(20)?;
    let mut rsi = Rsi::new(14)?;
    let mut macd = MacdIndicator::new(12, 26, 9)?;

    let prices = make_series(ticks);
    let mut signals = 0usize;
    for (t, &price) in prices.iter().enumerate() {
        let sma_v = sma.update(price);
        let ema_v = ema.update(price);
        let rsi_v = rsi.update(price);
        let macd_v = macd.update(price);

        // Only act once every indicator has produced a value.
        let (Some(sv), Some(ev), Some(rv), Some(m)) = (sma_v, ema_v, rsi_v, macd_v) else {
            continue;
        };

        let overbought = rv > 70.0 && m.histogram < 0.0;
        let oversold = rv < 30.0 && m.histogram > 0.0;
        let tag = if overbought {
            "SELL?"
        } else if oversold {
            "BUY? "
        } else {
            "     "
        };
        if overbought || oversold {
            signals += 1;
        }

        println!(
            "t={t:>3}  price={}  sma={}  ema={}  rsi={}  macd_hist={}  {tag}",
            fmt(Some(price)),
            fmt(Some(sv)),
            fmt(Some(ev)),
            fmt(Some(rv)),
            fmt(Some(m.histogram)),
        );
    }

    println!(
        "\nDone — {signals} candidate signal(s) over {} ticks.",
        prices.len()
    );
    Ok(())
}
