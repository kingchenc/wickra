//! `wickra-data`: offline and online data sources for the Wickra indicator engine.
//!
//! - [`csv`]: stream OHLCV bars out of CSV files without buffering the whole
//!   history in memory.
//! - [`aggregator`]: roll trade ticks up into candles of arbitrary timeframes.
//! - [`resample`]: convert a stream of candles from one timeframe to a coarser one.
//! - [`live`] (feature `live-binance`): connect to exchange websockets for live
//!   klines, or pull historical klines over REST — both yield typed candles
//!   compatible with the rest of the crate.

#![cfg_attr(docsrs, feature(doc_cfg))]
// `tokio_tungstenite::Error` is large by itself (~200 B). Boxing every Err
// variant per clippy::result_large_err just shifts allocation pressure into
// the hot path. We accept the size because errors are rare in this crate.
#![allow(clippy::result_large_err)]

pub mod aggregator;
pub mod csv;
pub mod error;
pub mod resample;

#[cfg(feature = "live-binance")]
pub mod live;

pub use error::{Error, Result};
