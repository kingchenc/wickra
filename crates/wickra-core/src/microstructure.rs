//! Microstructure value types: order-book snapshots and trades.
//!
//! These are the non-OHLCV inputs consumed by the order-book / trade-flow
//! indicator family. An [`OrderBook`] is a depth snapshot (sorted bid and ask
//! levels); a [`Trade`] is a single executed trade with an aggressor [`Side`];
//! a [`TradeQuote`] pairs a trade with the mid-price prevailing at execution,
//! the input for spread- and price-impact measures.

use crate::error::{Error, Result};

/// A single order-book price level: a resting quantity at a price.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Level {
    /// Price of the level (strictly positive).
    pub price: f64,
    /// Resting size / quantity at this price (non-negative).
    pub size: f64,
}

impl Level {
    /// Construct a level, validating that `price` is finite and strictly
    /// positive and `size` is finite and non-negative.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidOrderBook`] if the price is not a finite
    /// positive number, or the size is not a finite non-negative number.
    pub fn new(price: f64, size: f64) -> Result<Self> {
        if !price.is_finite() || price <= 0.0 {
            return Err(Error::InvalidOrderBook {
                message: "level price must be finite and positive",
            });
        }
        if !size.is_finite() || size < 0.0 {
            return Err(Error::InvalidOrderBook {
                message: "level size must be finite and non-negative",
            });
        }
        Ok(Self { price, size })
    }

    /// Construct a level without validation. The caller asserts that `price`
    /// is finite and positive and `size` is finite and non-negative.
    pub const fn new_unchecked(price: f64, size: f64) -> Self {
        Self { price, size }
    }
}

/// An order-book depth snapshot.
///
/// Bids are stored best-first (strictly descending price); asks are stored
/// best-first (strictly ascending price). A valid book is non-empty on both
/// sides and uncrossed (`best_bid < best_ask`).
#[derive(Debug, Clone, PartialEq)]
pub struct OrderBook {
    /// Bid levels, best (highest price) first.
    pub bids: Vec<Level>,
    /// Ask levels, best (lowest price) first.
    pub asks: Vec<Level>,
}

impl OrderBook {
    /// Construct an order book, validating the level and ordering invariants.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidOrderBook`] if either side is empty, any level
    /// has a non-finite/non-positive price or non-finite/negative size, the
    /// bids are not strictly descending in price, the asks are not strictly
    /// ascending in price, or the book is crossed/locked (`best_bid >=
    /// best_ask`).
    pub fn new(bids: Vec<Level>, asks: Vec<Level>) -> Result<Self> {
        if bids.is_empty() || asks.is_empty() {
            return Err(Error::InvalidOrderBook {
                message: "order book must have at least one bid and one ask",
            });
        }
        for level in bids.iter().chain(asks.iter()) {
            if !level.price.is_finite() || level.price <= 0.0 {
                return Err(Error::InvalidOrderBook {
                    message: "level price must be finite and positive",
                });
            }
            if !level.size.is_finite() || level.size < 0.0 {
                return Err(Error::InvalidOrderBook {
                    message: "level size must be finite and non-negative",
                });
            }
        }
        for pair in bids.windows(2) {
            if pair[0].price <= pair[1].price {
                return Err(Error::InvalidOrderBook {
                    message: "bids must be strictly descending in price",
                });
            }
        }
        for pair in asks.windows(2) {
            if pair[0].price >= pair[1].price {
                return Err(Error::InvalidOrderBook {
                    message: "asks must be strictly ascending in price",
                });
            }
        }
        if bids[0].price >= asks[0].price {
            return Err(Error::InvalidOrderBook {
                message: "order book must be uncrossed (best_bid < best_ask)",
            });
        }
        Ok(Self { bids, asks })
    }

    /// Construct an order book without validation. The caller asserts that all
    /// level and ordering invariants hold.
    pub const fn new_unchecked(bids: Vec<Level>, asks: Vec<Level>) -> Self {
        Self { bids, asks }
    }

    /// The best (highest-price) bid level, or `None` if the bid side is empty.
    pub fn best_bid(&self) -> Option<Level> {
        self.bids.first().copied()
    }

    /// The best (lowest-price) ask level, or `None` if the ask side is empty.
    pub fn best_ask(&self) -> Option<Level> {
        self.asks.first().copied()
    }

    /// The mid price `(best_bid + best_ask) / 2`, or `None` if either side is
    /// empty.
    pub fn mid(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(f64::midpoint(bid.price, ask.price)),
            _ => None,
        }
    }
}

/// The aggressor side of a trade: the side that crossed the spread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// A buyer-initiated (aggressive buy) trade.
    Buy,
    /// A seller-initiated (aggressive sell) trade.
    Sell,
}

impl Side {
    /// The signed multiplier for this side: `+1.0` for a buy, `−1.0` for a
    /// sell.
    pub const fn sign(self) -> f64 {
        match self {
            Side::Buy => 1.0,
            Side::Sell => -1.0,
        }
    }
}

/// A single executed trade with an aggressor side.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trade {
    /// Execution price (strictly positive).
    pub price: f64,
    /// Executed size / quantity (non-negative).
    pub size: f64,
    /// Aggressor side.
    pub side: Side,
    /// Trade timestamp (caller-defined epoch / resolution).
    pub timestamp: i64,
}

impl Trade {
    /// Construct a trade, validating that `price` is finite and strictly
    /// positive and `size` is finite and non-negative.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTrade`] if the price is not a finite positive
    /// number, or the size is not a finite non-negative number.
    pub fn new(price: f64, size: f64, side: Side, timestamp: i64) -> Result<Self> {
        if !price.is_finite() || price <= 0.0 {
            return Err(Error::InvalidTrade {
                message: "trade price must be finite and positive",
            });
        }
        if !size.is_finite() || size < 0.0 {
            return Err(Error::InvalidTrade {
                message: "trade size must be finite and non-negative",
            });
        }
        Ok(Self {
            price,
            size,
            side,
            timestamp,
        })
    }

    /// Construct a trade without validation. The caller asserts that `price`
    /// is finite and positive and `size` is finite and non-negative.
    pub const fn new_unchecked(price: f64, size: f64, side: Side, timestamp: i64) -> Self {
        Self {
            price,
            size,
            side,
            timestamp,
        }
    }
}

/// A trade paired with the mid-price prevailing at execution.
///
/// This is the input for spread- and price-impact measures (effective spread,
/// realized spread, Kyle's lambda), which relate an executed trade to the
/// quote it traded against.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TradeQuote {
    /// The executed trade.
    pub trade: Trade,
    /// The mid-price prevailing at execution (strictly positive).
    pub mid: f64,
}

impl TradeQuote {
    /// Construct a trade-quote, validating that `mid` is finite and strictly
    /// positive. The `trade` is assumed already valid.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTrade`] if `mid` is not a finite positive
    /// number.
    pub fn new(trade: Trade, mid: f64) -> Result<Self> {
        if !mid.is_finite() || mid <= 0.0 {
            return Err(Error::InvalidTrade {
                message: "trade-quote mid must be finite and positive",
            });
        }
        Ok(Self { trade, mid })
    }

    /// Construct a trade-quote without validation. The caller asserts that
    /// `mid` is finite and positive.
    pub const fn new_unchecked(trade: Trade, mid: f64) -> Self {
        Self { trade, mid }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_new_accepts_valid() {
        let level = Level::new(100.5, 2.0).unwrap();
        assert_eq!(level.price, 100.5);
        assert_eq!(level.size, 2.0);
    }

    #[test]
    fn level_new_accepts_zero_size() {
        assert!(Level::new(100.0, 0.0).is_ok());
    }

    #[test]
    fn level_new_rejects_non_finite_price() {
        assert!(matches!(
            Level::new(f64::NAN, 1.0),
            Err(Error::InvalidOrderBook { .. })
        ));
        assert!(matches!(
            Level::new(f64::INFINITY, 1.0),
            Err(Error::InvalidOrderBook { .. })
        ));
    }

    #[test]
    fn level_new_rejects_non_positive_price() {
        assert!(matches!(
            Level::new(0.0, 1.0),
            Err(Error::InvalidOrderBook { .. })
        ));
        assert!(matches!(
            Level::new(-1.0, 1.0),
            Err(Error::InvalidOrderBook { .. })
        ));
    }

    #[test]
    fn level_new_rejects_bad_size() {
        assert!(matches!(
            Level::new(100.0, -1.0),
            Err(Error::InvalidOrderBook { .. })
        ));
        assert!(matches!(
            Level::new(100.0, f64::NAN),
            Err(Error::InvalidOrderBook { .. })
        ));
    }

    #[test]
    fn level_new_unchecked_preserves_fields() {
        let level = Level::new_unchecked(-5.0, -2.0);
        assert_eq!(level.price, -5.0);
        assert_eq!(level.size, -2.0);
    }

    fn lvl(price: f64, size: f64) -> Level {
        Level::new(price, size).unwrap()
    }

    #[test]
    fn order_book_new_accepts_valid() {
        let book = OrderBook::new(
            vec![lvl(100.0, 2.0), lvl(99.0, 3.0)],
            vec![lvl(101.0, 1.0), lvl(102.0, 4.0)],
        )
        .unwrap();
        assert_eq!(book.best_bid(), Some(lvl(100.0, 2.0)));
        assert_eq!(book.best_ask(), Some(lvl(101.0, 1.0)));
        assert_eq!(book.mid(), Some(100.5));
    }

    #[test]
    fn order_book_new_rejects_empty_side() {
        assert!(matches!(
            OrderBook::new(vec![], vec![lvl(101.0, 1.0)]),
            Err(Error::InvalidOrderBook { .. })
        ));
        assert!(matches!(
            OrderBook::new(vec![lvl(100.0, 1.0)], vec![]),
            Err(Error::InvalidOrderBook { .. })
        ));
    }

    #[test]
    fn order_book_new_rejects_bad_level() {
        assert!(matches!(
            OrderBook::new(
                vec![Level::new_unchecked(100.0, -1.0)],
                vec![lvl(101.0, 1.0)]
            ),
            Err(Error::InvalidOrderBook { .. })
        ));
        assert!(matches!(
            OrderBook::new(
                vec![lvl(100.0, 1.0)],
                vec![Level::new_unchecked(f64::NAN, 1.0)]
            ),
            Err(Error::InvalidOrderBook { .. })
        ));
    }

    #[test]
    fn order_book_new_rejects_misordered_bids() {
        assert!(matches!(
            OrderBook::new(vec![lvl(99.0, 1.0), lvl(100.0, 1.0)], vec![lvl(101.0, 1.0)]),
            Err(Error::InvalidOrderBook { .. })
        ));
    }

    #[test]
    fn order_book_new_rejects_misordered_asks() {
        assert!(matches!(
            OrderBook::new(
                vec![lvl(100.0, 1.0)],
                vec![lvl(102.0, 1.0), lvl(101.0, 1.0)]
            ),
            Err(Error::InvalidOrderBook { .. })
        ));
    }

    #[test]
    fn order_book_new_rejects_crossed() {
        assert!(matches!(
            OrderBook::new(vec![lvl(101.0, 1.0)], vec![lvl(101.0, 1.0)]),
            Err(Error::InvalidOrderBook { .. })
        ));
        assert!(matches!(
            OrderBook::new(vec![lvl(102.0, 1.0)], vec![lvl(101.0, 1.0)]),
            Err(Error::InvalidOrderBook { .. })
        ));
    }

    #[test]
    fn order_book_new_unchecked_allows_empty() {
        let book = OrderBook::new_unchecked(vec![], vec![]);
        assert_eq!(book.best_bid(), None);
        assert_eq!(book.best_ask(), None);
        assert_eq!(book.mid(), None);
    }

    #[test]
    fn side_sign() {
        assert_eq!(Side::Buy.sign(), 1.0);
        assert_eq!(Side::Sell.sign(), -1.0);
    }

    #[test]
    fn trade_new_accepts_valid() {
        let trade = Trade::new(100.0, 1.5, Side::Buy, 42).unwrap();
        assert_eq!(trade.price, 100.0);
        assert_eq!(trade.size, 1.5);
        assert_eq!(trade.side, Side::Buy);
        assert_eq!(trade.timestamp, 42);
    }

    #[test]
    fn trade_new_rejects_bad_price() {
        assert!(matches!(
            Trade::new(0.0, 1.0, Side::Buy, 0),
            Err(Error::InvalidTrade { .. })
        ));
        assert!(matches!(
            Trade::new(f64::NAN, 1.0, Side::Sell, 0),
            Err(Error::InvalidTrade { .. })
        ));
    }

    #[test]
    fn trade_new_rejects_bad_size() {
        assert!(matches!(
            Trade::new(100.0, -1.0, Side::Buy, 0),
            Err(Error::InvalidTrade { .. })
        ));
        assert!(matches!(
            Trade::new(100.0, f64::INFINITY, Side::Buy, 0),
            Err(Error::InvalidTrade { .. })
        ));
    }

    #[test]
    fn trade_new_unchecked_preserves_fields() {
        let trade = Trade::new_unchecked(-1.0, -2.0, Side::Sell, 7);
        assert_eq!(trade.price, -1.0);
        assert_eq!(trade.size, -2.0);
        assert_eq!(trade.side, Side::Sell);
        assert_eq!(trade.timestamp, 7);
    }

    #[test]
    fn trade_quote_new_accepts_valid() {
        let trade = Trade::new(100.0, 1.0, Side::Buy, 0).unwrap();
        let tq = TradeQuote::new(trade, 99.5).unwrap();
        assert_eq!(tq.trade, trade);
        assert_eq!(tq.mid, 99.5);
    }

    #[test]
    fn trade_quote_new_rejects_bad_mid() {
        let trade = Trade::new(100.0, 1.0, Side::Buy, 0).unwrap();
        assert!(matches!(
            TradeQuote::new(trade, 0.0),
            Err(Error::InvalidTrade { .. })
        ));
        assert!(matches!(
            TradeQuote::new(trade, f64::NAN),
            Err(Error::InvalidTrade { .. })
        ));
    }

    #[test]
    fn trade_quote_new_unchecked_preserves_fields() {
        let trade = Trade::new_unchecked(100.0, 1.0, Side::Buy, 0);
        let tq = TradeQuote::new_unchecked(trade, -1.0);
        assert_eq!(tq.mid, -1.0);
        assert_eq!(tq.trade, trade);
    }
}
