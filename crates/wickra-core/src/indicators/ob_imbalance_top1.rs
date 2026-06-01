//! Order-Book Imbalance at the top of book.

use crate::microstructure::OrderBook;
use crate::traits::Indicator;

/// Order-Book Imbalance (top-of-book).
///
/// Measures the pressure between the best bid and best ask by comparing their
/// resting sizes:
///
/// ```text
/// imbalance = (bidSize₁ − askSize₁) / (bidSize₁ + askSize₁)
/// ```
///
/// The output lies in `[−1, +1]`: `+1` means all size sits on the bid (buy
/// pressure), `−1` means all size sits on the ask (sell pressure), `0` means a
/// balanced top of book. A book with zero size on both top levels yields `0`.
///
/// `Input = OrderBook`, `Output = f64`. The indicator is stateless and ready
/// after the first snapshot.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Level, OrderBook, OrderBookImbalanceTop1};
///
/// let book = OrderBook::new(
///     vec![Level::new(100.0, 3.0).unwrap()],
///     vec![Level::new(101.0, 1.0).unwrap()],
/// )
/// .unwrap();
/// let mut obi = OrderBookImbalanceTop1::new();
/// assert_eq!(obi.update(book), Some(0.5)); // (3 − 1) / (3 + 1)
/// ```
#[derive(Debug, Clone, Default)]
pub struct OrderBookImbalanceTop1 {
    has_emitted: bool,
}

impl OrderBookImbalanceTop1 {
    /// Construct a new top-of-book imbalance indicator.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for OrderBookImbalanceTop1 {
    type Input = OrderBook;
    type Output = f64;

    fn update(&mut self, book: OrderBook) -> Option<f64> {
        self.has_emitted = true;
        let (Some(bid), Some(ask)) = (book.best_bid(), book.best_ask()) else {
            return Some(0.0);
        };
        let total = bid.size + ask.size;
        if total <= 0.0 {
            return Some(0.0);
        }
        Some((bid.size - ask.size) / total)
    }

    fn reset(&mut self) {
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "OrderBookImbalanceTop1"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::microstructure::Level;
    use crate::traits::BatchExt;

    fn book(bids: &[(f64, f64)], asks: &[(f64, f64)]) -> OrderBook {
        let to_levels = |xs: &[(f64, f64)]| {
            xs.iter()
                .map(|&(p, s)| Level::new(p, s).unwrap())
                .collect::<Vec<_>>()
        };
        OrderBook::new(to_levels(bids), to_levels(asks)).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let obi = OrderBookImbalanceTop1::new();
        assert_eq!(obi.name(), "OrderBookImbalanceTop1");
        assert_eq!(obi.warmup_period(), 1);
        assert!(!obi.is_ready());
    }

    #[test]
    fn balanced_top_is_zero() {
        let mut obi = OrderBookImbalanceTop1::new();
        assert_eq!(
            obi.update(book(&[(100.0, 2.0)], &[(101.0, 2.0)])),
            Some(0.0)
        );
        assert!(obi.is_ready());
    }

    #[test]
    fn bid_heavy_is_positive() {
        let mut obi = OrderBookImbalanceTop1::new();
        assert_eq!(
            obi.update(book(&[(100.0, 3.0)], &[(101.0, 1.0)])),
            Some(0.5)
        );
    }

    #[test]
    fn ask_heavy_is_negative() {
        let mut obi = OrderBookImbalanceTop1::new();
        assert_eq!(
            obi.update(book(&[(100.0, 1.0)], &[(101.0, 3.0)])),
            Some(-0.5)
        );
    }

    #[test]
    fn zero_size_top_is_zero() {
        let mut obi = OrderBookImbalanceTop1::new();
        assert_eq!(
            obi.update(book(&[(100.0, 0.0)], &[(101.0, 0.0)])),
            Some(0.0)
        );
    }

    #[test]
    fn empty_book_is_zero() {
        let mut obi = OrderBookImbalanceTop1::new();
        assert_eq!(
            obi.update(OrderBook::new_unchecked(vec![], vec![])),
            Some(0.0)
        );
    }

    #[test]
    fn batch_equals_streaming() {
        let books: Vec<OrderBook> = (0..20)
            .map(|i| {
                let bid = 1.0 + f64::from(i % 5);
                book(&[(100.0, bid)], &[(101.0, 2.0)])
            })
            .collect();
        let mut a = OrderBookImbalanceTop1::new();
        let mut b = OrderBookImbalanceTop1::new();
        assert_eq!(
            a.batch(&books),
            books
                .iter()
                .map(|x| b.update(x.clone()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut obi = OrderBookImbalanceTop1::new();
        obi.update(book(&[(100.0, 1.0)], &[(101.0, 1.0)]));
        assert!(obi.is_ready());
        obi.reset();
        assert!(!obi.is_ready());
    }
}
