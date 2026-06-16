//! Live exchange feeds. Each adapter is feature-gated; the Binance adapter
//! lives behind the `live-binance` feature.

pub mod binance;
pub mod binance_rest;
