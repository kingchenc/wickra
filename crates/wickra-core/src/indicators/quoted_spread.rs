//! Quoted Spread — top-of-book spread in basis points.

use crate::microstructure::OrderBook;
use crate::traits::Indicator;

/// Quoted Spread — the top-of-book bid-ask spread expressed in basis points of
/// the mid price.
///
/// ```text
/// mid           = (bidPrice₁ + askPrice₁) / 2
/// quotedSpread  = (askPrice₁ − bidPrice₁) / mid · 10_000   (bps)
/// ```
///
/// This is the round-trip cost of crossing the spread at the touch, normalised
/// by price so it is comparable across instruments. For a valid (uncrossed)
/// book the result is non-negative. An empty book yields `0`.
///
/// `Input = OrderBook`, `Output = f64`. Stateless; ready after the first
/// snapshot.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Level, OrderBook, QuotedSpread};
///
/// let book = OrderBook::new(
///     vec![Level::new(100.0, 1.0).unwrap()],
///     vec![Level::new(100.5, 1.0).unwrap()],
/// )
/// .unwrap();
/// let mut qs = QuotedSpread::new();
/// // spread 0.5, mid 100.25 -> 0.5 / 100.25 * 10_000 ≈ 49.875 bps.
/// let bps = qs.update(book).unwrap();
/// assert!((bps - 49.875_311_72).abs() < 1e-6);
/// ```
#[derive(Debug, Clone, Default)]
pub struct QuotedSpread {
    has_emitted: bool,
}

impl QuotedSpread {
    /// Construct a new quoted-spread indicator.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for QuotedSpread {
    type Input = OrderBook;
    type Output = f64;

    fn update(&mut self, book: OrderBook) -> Option<f64> {
        self.has_emitted = true;
        let (Some(bid), Some(ask)) = (book.best_bid(), book.best_ask()) else {
            return Some(0.0);
        };
        let mid = f64::midpoint(bid.price, ask.price);
        Some((ask.price - bid.price) / mid * 10_000.0)
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
        "QuotedSpread"
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
        let qs = QuotedSpread::new();
        assert_eq!(qs.name(), "QuotedSpread");
        assert_eq!(qs.warmup_period(), 1);
        assert!(!qs.is_ready());
    }

    #[test]
    fn known_value_in_bps() {
        let mut qs = QuotedSpread::new();
        // spread 1.0, mid 100.5 -> 1 / 100.5 * 10_000 ≈ 99.5025 bps.
        let bps = qs.update(book(&[(100.0, 1.0)], &[(101.0, 1.0)])).unwrap();
        assert!((bps - 99.502_487_56).abs() < 1e-6);
        assert!(qs.is_ready());
    }

    #[test]
    fn tight_book_is_small() {
        let mut qs = QuotedSpread::new();
        let bps = qs.update(book(&[(100.0, 1.0)], &[(100.01, 1.0)])).unwrap();
        assert!(bps > 0.0 && bps < 2.0);
    }

    #[test]
    fn empty_book_is_zero() {
        let mut qs = QuotedSpread::new();
        assert_eq!(
            qs.update(OrderBook::new_unchecked(vec![], vec![])),
            Some(0.0)
        );
    }

    #[test]
    fn batch_equals_streaming() {
        let books: Vec<OrderBook> = (0..20)
            .map(|i| {
                let ask = 100.5 + f64::from(i % 4) * 0.1;
                book(&[(100.0, 1.0)], &[(ask, 1.0)])
            })
            .collect();
        let mut a = QuotedSpread::new();
        let mut b = QuotedSpread::new();
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
        let mut qs = QuotedSpread::new();
        qs.update(book(&[(100.0, 1.0)], &[(101.0, 1.0)]));
        assert!(qs.is_ready());
        qs.reset();
        assert!(!qs.is_ready());
    }
}
