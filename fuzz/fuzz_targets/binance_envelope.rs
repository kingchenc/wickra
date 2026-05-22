#![no_main]
//! Fuzz the Binance combined-stream envelope deserializer.
//!
//! `RawWsEnvelope` is what a kline frame is decoded into; feeding it
//! arbitrary strings exercises the serde path that runs on every WebSocket
//! frame. It must reject malformed input with an `Err`, never panic.

use libfuzzer_sys::fuzz_target;
use wickra_data::live::binance::RawWsEnvelope;

fuzz_target!(|data: &str| {
    let _ = serde_json::from_str::<RawWsEnvelope>(data);
});
