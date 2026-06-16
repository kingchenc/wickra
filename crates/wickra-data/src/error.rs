//! Error types specific to the data sources.

use thiserror::Error;

/// Errors produced by the data layer.
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("invalid timeframe: {0}")]
    InvalidTimeframe(String),

    #[error("indicator-core error: {0}")]
    Core(#[from] wickra_core::Error),

    #[error("malformed payload: {0}")]
    Malformed(String),

    /// A live-feed read exceeded its deadline.
    #[error("read timed out")]
    Timeout,

    #[cfg(feature = "live-binance")]
    #[error("websocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[cfg(feature = "live-binance")]
    #[error("JSON decode error: {0}")]
    Json(#[from] serde_json::Error),

    /// Boxed because ureq 2.x's `Error::Status` carries a full `Response`, which
    /// would otherwise make this the dominant enum variant (clippy
    /// `large_enum_variant`).
    #[cfg(feature = "live-binance")]
    #[error("HTTP error: {0}")]
    Http(#[from] Box<ureq::Error>),
}

/// Convenience alias for `Result<T, wickra_data::Error>`.
pub type Result<T> = core::result::Result<T, Error>;
