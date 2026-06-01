//! Microprice — size-weighted fair value of the top of book.

use crate::microstructure::OrderBook;
use crate::traits::Indicator;

/// Microprice — the size-weighted mid of the top of book.
///
/// The microprice tilts the mid toward the side that is *more likely to be
/// hit*: it weights each touch price by the size resting on the **opposite**
/// side, so a heavy ask (sell pressure) pulls the fair value down toward the
/// bid, and vice versa:
///
/// ```text
/// microprice = (bidPrice₁·askSize₁ + askPrice₁·bidSize₁) / (bidSize₁ + askSize₁)
/// ```
///
/// When both top sizes are zero the weighting is undefined and the plain mid
/// `(bidPrice₁ + askPrice₁) / 2` is returned. An empty book yields `0`.
///
/// `Input = OrderBook`, `Output = f64`. Stateless; ready after the first
/// snapshot.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Level, Microprice, OrderBook};
///
/// let book = OrderBook::new(
///     vec![Level::new(100.0, 1.0).unwrap()],
///     vec![Level::new(101.0, 3.0).unwrap()],
/// )
/// .unwrap();
/// let mut mp = Microprice::new();
/// // (100·3 + 101·1) / (1 + 3) = 401 / 4 = 100.25 — pulled toward the bid.
/// assert_eq!(mp.update(book), Some(100.25));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Microprice {
    has_emitted: bool,
}

impl Microprice {
    /// Construct a new microprice indicator.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for Microprice {
    type Input = OrderBook;
    type Output = f64;

    fn update(&mut self, book: OrderBook) -> Option<f64> {
        self.has_emitted = true;
        let (Some(bid), Some(ask)) = (book.best_bid(), book.best_ask()) else {
            return Some(0.0);
        };
        let total = bid.size + ask.size;
        if total <= 0.0 {
            return Some(f64::midpoint(bid.price, ask.price));
        }
        Some((bid.price * ask.size + ask.price * bid.size) / total)
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
        "Microprice"
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
        let mp = Microprice::new();
        assert_eq!(mp.name(), "Microprice");
        assert_eq!(mp.warmup_period(), 1);
        assert!(!mp.is_ready());
    }

    #[test]
    fn weights_toward_thin_side() {
        let mut mp = Microprice::new();
        // Heavy ask -> microprice pulled toward bid.
        assert_eq!(
            mp.update(book(&[(100.0, 1.0)], &[(101.0, 3.0)])),
            Some(100.25)
        );
        assert!(mp.is_ready());
    }

    #[test]
    fn balanced_top_equals_mid() {
        let mut mp = Microprice::new();
        assert_eq!(
            mp.update(book(&[(100.0, 2.0)], &[(101.0, 2.0)])),
            Some(100.5)
        );
    }

    #[test]
    fn zero_size_falls_back_to_mid() {
        let mut mp = Microprice::new();
        assert_eq!(
            mp.update(book(&[(100.0, 0.0)], &[(102.0, 0.0)])),
            Some(101.0)
        );
    }

    #[test]
    fn empty_book_is_zero() {
        let mut mp = Microprice::new();
        assert_eq!(
            mp.update(OrderBook::new_unchecked(vec![], vec![])),
            Some(0.0)
        );
    }

    #[test]
    fn batch_equals_streaming() {
        let books: Vec<OrderBook> = (0..20)
            .map(|i| {
                let ask = 1.0 + f64::from(i % 4);
                book(&[(100.0, 2.0)], &[(101.0, ask)])
            })
            .collect();
        let mut a = Microprice::new();
        let mut b = Microprice::new();
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
        let mut mp = Microprice::new();
        mp.update(book(&[(100.0, 1.0)], &[(101.0, 1.0)]));
        assert!(mp.is_ready());
        mp.reset();
        assert!(!mp.is_ready());
    }
}
