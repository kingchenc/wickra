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

    /// A multiplier or factor must be strictly positive.
    #[error("multiplier must be greater than zero")]
    NonPositiveMultiplier,
}

/// Convenience alias for `Result<T, wickra_core::Error>`.
pub type Result<T> = core::result::Result<T, Error>;
