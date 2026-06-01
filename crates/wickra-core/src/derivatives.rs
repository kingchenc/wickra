//! Derivatives value type: the perpetual / futures tick.
//!
//! [`DerivativesTick`] is the non-OHLCV input consumed by the derivatives /
//! perpetual-futures indicator family. A single tick bundles the funding,
//! price, open-interest, positioning, taker-flow and liquidation fields a
//! perp/futures venue publishes per update; each indicator reads only the
//! subset it needs (the same one-rich-type-per-family pattern as [`Trade`] /
//! [`OrderBook`] in [`crate::microstructure`]).
//!
//! [`Trade`]: crate::microstructure::Trade
//! [`OrderBook`]: crate::microstructure::OrderBook

use crate::error::{Error, Result};

/// A single derivatives / perpetual-futures market tick.
///
/// Field invariants enforced by [`new`](DerivativesTick::new):
///
/// - `funding_rate` is finite and **may be negative** (a negative funding rate
///   means shorts pay longs).
/// - `mark_price`, `index_price` and `futures_price` are finite and strictly
///   positive.
/// - `open_interest`, `long_size`, `short_size`, `taker_buy_volume`,
///   `taker_sell_volume`, `long_liquidation` and `short_liquidation` are finite
///   and non-negative.
///
/// `timestamp` is a caller-defined epoch / resolution and is not validated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DerivativesTick {
    /// Current funding rate for the interval (finite; may be negative).
    pub funding_rate: f64,
    /// Perpetual mark price (finite, strictly positive).
    pub mark_price: f64,
    /// Spot / index price the perpetual tracks (finite, strictly positive).
    pub index_price: f64,
    /// Dated (e.g. quarterly) futures mark price (finite, strictly positive).
    pub futures_price: f64,
    /// Open interest — outstanding contracts / notional (finite, non-negative).
    pub open_interest: f64,
    /// Aggregate long size / long account count (finite, non-negative).
    pub long_size: f64,
    /// Aggregate short size / short account count (finite, non-negative).
    pub short_size: f64,
    /// Taker buy (ask-lifting) volume (finite, non-negative).
    pub taker_buy_volume: f64,
    /// Taker sell (bid-hitting) volume (finite, non-negative).
    pub taker_sell_volume: f64,
    /// Long-side liquidation notional (finite, non-negative).
    pub long_liquidation: f64,
    /// Short-side liquidation notional (finite, non-negative).
    pub short_liquidation: f64,
    /// Tick timestamp (caller-defined epoch / resolution).
    pub timestamp: i64,
}

impl DerivativesTick {
    /// Construct a derivatives tick, validating every field invariant.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidDerivatives`] if `funding_rate` is not finite;
    /// any of `mark_price`, `index_price`, `futures_price` is not a finite
    /// positive number; or any of the six size / volume / liquidation fields is
    /// not a finite non-negative number.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        funding_rate: f64,
        mark_price: f64,
        index_price: f64,
        futures_price: f64,
        open_interest: f64,
        long_size: f64,
        short_size: f64,
        taker_buy_volume: f64,
        taker_sell_volume: f64,
        long_liquidation: f64,
        short_liquidation: f64,
        timestamp: i64,
    ) -> Result<Self> {
        if !funding_rate.is_finite() {
            return Err(Error::InvalidDerivatives {
                message: "funding_rate must be finite",
            });
        }
        for price in [mark_price, index_price, futures_price] {
            if !price.is_finite() || price <= 0.0 {
                return Err(Error::InvalidDerivatives {
                    message:
                        "mark_price, index_price and futures_price must be finite and positive",
                });
            }
        }
        for amount in [
            open_interest,
            long_size,
            short_size,
            taker_buy_volume,
            taker_sell_volume,
            long_liquidation,
            short_liquidation,
        ] {
            if !amount.is_finite() || amount < 0.0 {
                return Err(Error::InvalidDerivatives {
                    message: "open interest, sizes, volumes and liquidations must be finite and non-negative",
                });
            }
        }
        Ok(Self {
            funding_rate,
            mark_price,
            index_price,
            futures_price,
            open_interest,
            long_size,
            short_size,
            taker_buy_volume,
            taker_sell_volume,
            long_liquidation,
            short_liquidation,
            timestamp,
        })
    }

    /// Construct a derivatives tick without validation. The caller asserts that
    /// every field invariant documented on [`DerivativesTick`] holds.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new_unchecked(
        funding_rate: f64,
        mark_price: f64,
        index_price: f64,
        futures_price: f64,
        open_interest: f64,
        long_size: f64,
        short_size: f64,
        taker_buy_volume: f64,
        taker_sell_volume: f64,
        long_liquidation: f64,
        short_liquidation: f64,
        timestamp: i64,
    ) -> Self {
        Self {
            funding_rate,
            mark_price,
            index_price,
            futures_price,
            open_interest,
            long_size,
            short_size,
            taker_buy_volume,
            taker_sell_volume,
            long_liquidation,
            short_liquidation,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fully valid tick used as a baseline; individual tests override one
    /// field to exercise a single reject branch.
    fn valid() -> DerivativesTick {
        DerivativesTick::new(
            0.0001, 100.0, 99.5, 100.5, 1_000.0, 600.0, 400.0, 50.0, 40.0, 5.0, 3.0, 42,
        )
        .unwrap()
    }

    #[test]
    fn new_accepts_valid() {
        let tick = valid();
        assert_eq!(tick.funding_rate, 0.0001);
        assert_eq!(tick.mark_price, 100.0);
        assert_eq!(tick.index_price, 99.5);
        assert_eq!(tick.futures_price, 100.5);
        assert_eq!(tick.open_interest, 1_000.0);
        assert_eq!(tick.long_size, 600.0);
        assert_eq!(tick.short_size, 400.0);
        assert_eq!(tick.taker_buy_volume, 50.0);
        assert_eq!(tick.taker_sell_volume, 40.0);
        assert_eq!(tick.long_liquidation, 5.0);
        assert_eq!(tick.short_liquidation, 3.0);
        assert_eq!(tick.timestamp, 42);
    }

    #[test]
    fn new_accepts_negative_funding_and_zero_amounts() {
        let tick = DerivativesTick::new(
            -0.0005, 100.0, 100.0, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0,
        )
        .unwrap();
        assert_eq!(tick.funding_rate, -0.0005);
        assert_eq!(tick.open_interest, 0.0);
    }

    #[test]
    fn new_rejects_non_finite_funding() {
        assert!(matches!(
            DerivativesTick::new(
                f64::NAN,
                100.0,
                100.0,
                100.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0
            ),
            Err(Error::InvalidDerivatives { .. })
        ));
        assert!(matches!(
            DerivativesTick::new(
                f64::INFINITY,
                100.0,
                100.0,
                100.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0
            ),
            Err(Error::InvalidDerivatives { .. })
        ));
    }

    #[test]
    fn new_rejects_non_positive_mark() {
        assert!(matches!(
            DerivativesTick::new(0.0, 0.0, 100.0, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0),
            Err(Error::InvalidDerivatives { .. })
        ));
    }

    #[test]
    fn new_rejects_non_positive_index() {
        assert!(matches!(
            DerivativesTick::new(0.0, 100.0, -1.0, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0),
            Err(Error::InvalidDerivatives { .. })
        ));
    }

    #[test]
    fn new_rejects_non_finite_futures() {
        assert!(matches!(
            DerivativesTick::new(
                0.0,
                100.0,
                100.0,
                f64::NAN,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0
            ),
            Err(Error::InvalidDerivatives { .. })
        ));
    }

    #[test]
    fn new_rejects_negative_open_interest() {
        assert!(matches!(
            DerivativesTick::new(0.0, 100.0, 100.0, 100.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0),
            Err(Error::InvalidDerivatives { .. })
        ));
    }

    #[test]
    fn new_rejects_non_finite_size() {
        assert!(matches!(
            DerivativesTick::new(
                0.0,
                100.0,
                100.0,
                100.0,
                0.0,
                f64::INFINITY,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0
            ),
            Err(Error::InvalidDerivatives { .. })
        ));
    }

    #[test]
    fn new_rejects_negative_liquidation() {
        assert!(matches!(
            DerivativesTick::new(0.0, 100.0, 100.0, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -2.0, 0),
            Err(Error::InvalidDerivatives { .. })
        ));
    }

    #[test]
    fn new_unchecked_preserves_fields() {
        let tick = DerivativesTick::new_unchecked(
            -1.0, -2.0, -3.0, -4.0, -5.0, -6.0, -7.0, -8.0, -9.0, -10.0, -11.0, 7,
        );
        assert_eq!(tick.funding_rate, -1.0);
        assert_eq!(tick.mark_price, -2.0);
        assert_eq!(tick.short_liquidation, -11.0);
        assert_eq!(tick.timestamp, 7);
    }
}
