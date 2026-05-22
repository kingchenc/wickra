//! Multi-timeframe indicators with the Wickra Rust stack.
//!
//! Reads the bundled 1-minute BTCUSDT CSV (or a path passed on the command
//! line), resamples it via [`wickra_data::resample::resample_all`] to 5m /
//! 15m / 1h / 4h / 1d, and prints the last RSI(14), MACD(12,26,9) histogram
//! and ADX(14) at each timeframe. The Rust counterpart of
//! `examples/python/multi_timeframe.py`.
//!
//! Run with:
//! ```text
//! cargo run --release -p wickra-examples --bin multi_timeframe
//! ```

use std::env;
use std::path::PathBuf;

use wickra::{Adx, Candle, Indicator, MacdIndicator, Rsi};
use wickra_data::aggregator::Timeframe;
use wickra_data::csv::CandleReader;
use wickra_data::resample::resample_all;

const ONE_MINUTE_MS: i64 = 60_000;

fn default_csv() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("data")
        .join("btcusdt-1m.csv")
}

fn summarize(label: &str, candles: &[Candle]) {
    if candles.is_empty() {
        println!("  {label:<5} (empty)");
        return;
    }
    let mut rsi = Rsi::new(14).expect("RSI(14) constructor cannot fail");
    let mut macd = MacdIndicator::new(12, 26, 9).expect("MACD(12,26,9) constructor cannot fail");
    let mut adx = Adx::new(14).expect("ADX(14) constructor cannot fail");

    let mut last_rsi: Option<f64> = None;
    let mut last_hist: Option<f64> = None;
    let mut last_adx: Option<f64> = None;
    for c in candles {
        if let Some(v) = rsi.update(c.close) {
            last_rsi = Some(v);
        }
        if let Some(m) = macd.update(c.close) {
            last_hist = Some(m.histogram);
        }
        if let Some(a) = adx.update(*c) {
            last_adx = Some(a.adx);
        }
    }
    let last_close = candles.last().expect("non-empty").close;
    println!(
        "  {label:<5} bars={:>5}  last_close={last_close:>10.2}  rsi={}  macd_hist={}  adx={}",
        candles.len(),
        last_rsi.map_or_else(|| "    --".to_string(), |v| format!("{v:>6.2}")),
        last_hist.map_or_else(|| " --   ".to_string(), |v| format!("{v:+6.2}")),
        last_adx.map_or_else(|| "    --".to_string(), |v| format!("{v:>6.2}")),
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).map_or_else(default_csv, PathBuf::from);

    println!("Multi-timeframe view of {}", path.display());

    let mut reader = CandleReader::open(&path)?;
    let ones: Vec<Candle> = reader.read_all()?;
    summarize("1m", &ones);

    for &(label, minutes) in &[
        ("5m", 5_i64),
        ("15m", 15),
        ("1h", 60),
        ("4h", 240),
        ("1d", 1440),
    ] {
        let tf = Timeframe::millis(minutes * ONE_MINUTE_MS)?;
        let resampled = resample_all(tf, ones.iter().copied().map(Ok))?;
        summarize(label, &resampled);
    }
    Ok(())
}
