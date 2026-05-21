//! Connect to Binance, run incremental RSI on the closed kline events.
//!
//! Build with:
//! ```text
//! cargo run --release --example live_binance --features wickra-data/live-binance -- BTCUSDT
//! ```
//!
//! The example prints a line per bar close. Hit Ctrl+C to stop.

use std::env;

use wickra::{Indicator, Rsi};
use wickra_data::live::binance::{BinanceKlineStream, Interval};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let symbol = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "BTCUSDT".to_string());

    let mut stream =
        BinanceKlineStream::connect(std::slice::from_ref(&symbol), Interval::OneMinute).await?;
    let mut rsi = Rsi::new(14)?;
    println!("Listening for {symbol} 1m closes…");

    while let Some(event) = stream.next_event().await? {
        if !event.is_closed {
            continue;
        }
        let close = event.candle.close;
        let v = rsi.update(close);
        if let Some(value) = v {
            println!("{}  close={:.4}  rsi={:.2}", event.symbol, close, value);
        } else {
            println!("{}  close={:.4}  rsi=…warmup", event.symbol, close);
        }
    }
    Ok(())
}
