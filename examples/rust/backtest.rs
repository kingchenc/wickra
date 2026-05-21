//! Rust example: backtest a basket of indicators against a CSV file.
//!
//! Build with:
//! ```text
//! cargo run --release --example backtest -- path/to/ohlcv.csv
//! ```

use std::env;

use wickra::{Adx, Atr, BatchExt, BollingerBands, Ema, Indicator, MacdIndicator, Obv, Rsi};
use wickra_data::csv::CandleReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).ok_or("usage: backtest <ohlcv.csv>")?;

    let mut reader = CandleReader::open(path)?;
    let candles = reader.read_all()?;
    if candles.is_empty() {
        return Err("CSV is empty".into());
    }
    let n = candles.len();
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    let rsi = Rsi::new(14)?.batch(&closes);
    let ema = Ema::new(20)?.batch(&closes);
    let bb = BollingerBands::classic().batch(&closes);
    let macd = MacdIndicator::classic().batch(&closes);

    let mut atr = Atr::new(14)?;
    let atr_series: Vec<_> = candles.iter().map(|c| atr.update(*c)).collect();

    let mut adx = Adx::new(14)?;
    let adx_series: Vec<_> = candles.iter().map(|c| adx.update(*c)).collect();

    let mut obv = Obv::new();
    let obv_series: Vec<_> = candles.iter().map(|c| obv.update(*c)).collect();

    let last_rsi = rsi
        .iter()
        .rev()
        .flatten()
        .next()
        .copied()
        .unwrap_or(f64::NAN);
    let last_ema = ema
        .iter()
        .rev()
        .flatten()
        .next()
        .copied()
        .unwrap_or(f64::NAN);
    let last_bb = bb.iter().rev().flatten().next().copied();
    let last_macd = macd.iter().rev().flatten().next().copied();
    let last_atr = atr_series
        .iter()
        .rev()
        .flatten()
        .next()
        .copied()
        .unwrap_or(f64::NAN);
    let last_adx = adx_series.iter().rev().flatten().next().copied();
    let last_obv = obv_series
        .iter()
        .rev()
        .flatten()
        .next()
        .copied()
        .unwrap_or(f64::NAN);

    println!("backtest summary for {path} ({n} bars)");
    println!("  RSI(14)  = {last_rsi:>9.4}");
    println!("  EMA(20)  = {last_ema:>9.4}");
    if let Some(bb) = last_bb {
        println!(
            "  BB(20,2) upper={:>9.4}  middle={:>9.4}  lower={:>9.4}  sd={:>8.4}",
            bb.upper, bb.middle, bb.lower, bb.stddev
        );
    }
    if let Some(m) = last_macd {
        println!(
            "  MACD     macd={:>9.4}  signal={:>9.4}  hist={:>9.4}",
            m.macd, m.signal, m.histogram
        );
    }
    println!("  ATR(14)  = {last_atr:>9.4}");
    if let Some(a) = last_adx {
        println!(
            "  ADX(14)  +DI={:>6.2}  -DI={:>6.2}  ADX={:>6.2}",
            a.plus_di, a.minus_di, a.adx
        );
    }
    println!("  OBV      = {last_obv:>14.2}");
    Ok(())
}
