//! Error types used across `wickra-core`.

use thiserror::Error;

/// Errors that can occur when constructing or operating on an indicator.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    /// A period (window length) must be at least one.
    #[error("period must be greater than zero")]
    PeriodZero,

    /// A specific minimum period requirement was not met (e.g. MACD needs slow > fast).
    #[error("invalid period: {message}")]
    InvalidPeriod { message: &'static str },

    /// A non-finite value (NaN or infinity) was passed where a finite price was expected.
    #[error("input value must be finite (got NaN or infinity)")]
    NonFiniteInput,

    /// A candle whose components do not form a valid bar (e.g. high < low) was provided.
    #[error("invalid candle: {message}")]
    InvalidCandle { message: &'static str },

    /// A tick whose components do not satisfy the tick invariants (e.g. negative
    /// volume) was provided. Ticks are a different concept from candles and
    /// surface as their own variant so consumers of a tick-stream pipeline
    /// can match on a semantically-correct error instead of `InvalidCandle`.
    #[error("invalid tick: {message}")]
    InvalidTick { message: &'static str },

    /// A multiplier or factor must be strictly positive.
    #[error("multiplier must be greater than zero")]
    NonPositiveMultiplier,

    /// An order-book snapshot whose levels do not satisfy the book invariants
    /// (e.g. a crossed book, non-finite price, negative size, or mis-sorted
    /// levels) was provided. Order books are a microstructure input distinct
    /// from candles and ticks, so they surface as their own variant.
    #[error("invalid order book: {message}")]
    InvalidOrderBook { message: &'static str },

    /// A trade whose components do not satisfy the trade invariants (e.g.
    /// non-finite price or negative size) was provided.
    #[error("invalid trade: {message}")]
    InvalidTrade { message: &'static str },

    /// A derivatives tick whose components do not satisfy the tick invariants
    /// (e.g. a non-positive price, a non-finite funding rate, or a negative
    /// size/volume/liquidation) was provided. Derivatives ticks (funding /
    /// open-interest / liquidation feeds) are a perpetual-futures input
    /// distinct from candles, order books and trades, so they surface as their
    /// own variant.
    #[error("invalid derivatives tick: {message}")]
    InvalidDerivatives { message: &'static str },

    /// A market-breadth cross-section whose members do not satisfy the
    /// cross-section invariants (an empty universe, a non-finite change, or a
    /// negative / non-finite volume) was provided. A cross-section is a
    /// breadth input distinct from candles, ticks, order books and trades, so
    /// it surfaces as its own variant.
    #[error("invalid cross-section: {message}")]
    InvalidCrossSection { message: &'static str },

    /// A real-valued configuration parameter was outside its admissible range
    /// (e.g. a non-positive standard-deviation multiplier, or a Kalman filter
    /// covariance that is not strictly positive). This is the floating-point
    /// analogue of [`Error::InvalidPeriod`], which only covers integer windows.
    #[error("invalid parameter: {message}")]
    InvalidParameter { message: &'static str },
}

/// Convenience alias for `Result<T, wickra_core::Error>`.
pub type Result<T> = core::result::Result<T, Error>;
