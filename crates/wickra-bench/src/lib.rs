//! Internal cross-library benchmark harness for Wickra.
//!
//! This crate is `publish = false`. It exists only to host the Criterion
//! benchmark in `benches/cross_lib.rs`, which compares Wickra against the
//! Rust technical-analysis crates `kand`, `ta` (ta-rs) and `yata` on an
//! identical candle series. It deliberately carries no library code.
