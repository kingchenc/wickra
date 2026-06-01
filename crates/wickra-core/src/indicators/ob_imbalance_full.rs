//! Order-Book Imbalance over the full visible depth.

use crate::microstructure::OrderBook;
use crate::traits::Indicator;

/// Order-Book Imbalance aggregated over the full visible depth of each side.
///
/// Sums the resting size of every bid level and every ask level in the
/// snapshot and compares them:
///
/// ```text
/// bidDepth  = Σ size of all bids
/// askDepth  = Σ size of all asks
/// imbalance = (bidDepth − askDepth) / (bidDepth + askDepth)
/// ```
///
/// The output lies in `[−1, +1]`. A book with zero total size yields `0`. Use
/// [`crate::OrderBookImbalanceTopN`] to bound the depth to the most relevant
/// near-touch levels instead of the full visible book.
///
/// `Input = OrderBook`, `Output = f64`. Stateless; ready after the first
/// snapshot.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Level, OrderBook, OrderBookImbalanceFull};
///
/// let book = OrderBook::new(
///     vec![Level::new(100.0, 2.0).unwrap(), Level::new(99.0, 1.0).unwrap()],
///     vec![Level::new(101.0, 0.5).unwrap(), Level::new(102.0, 0.5).unwrap()],
/// )
/// .unwrap();
/// let mut obi = OrderBookImbalanceFull::new();
/// assert_eq!(obi.update(book), Some(0.5)); // (3 − 1) / (3 + 1)
/// ```
#[derive(Debug, Clone, Default)]
pub struct OrderBookImbalanceFull {
    has_emitted: bool,
}

impl OrderBookImbalanceFull {
    /// Construct a new full-depth imbalance indicator.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for OrderBookImbalanceFull {
    type Input = OrderBook;
    type Output = f64;

    fn update(&mut self, book: OrderBook) -> Option<f64> {
        self.has_emitted = true;
        let bid_depth: f64 = book.bids.iter().map(|l| l.size).sum();
        let ask_depth: f64 = book.asks.iter().map(|l| l.size).sum();
        let total = bid_depth + ask_depth;
        if total <= 0.0 {
            return Some(0.0);
        }
        Some((bid_depth - ask_depth) / total)
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
        "OrderBookImbalanceFull"
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
        let obi = OrderBookImbalanceFull::new();
        assert_eq!(obi.name(), "OrderBookImbalanceFull");
        assert_eq!(obi.warmup_period(), 1);
        assert!(!obi.is_ready());
    }

    #[test]
    fn sums_full_depth() {
        let mut obi = OrderBookImbalanceFull::new();
        let b = book(&[(100.0, 2.0), (99.0, 2.0)], &[(101.0, 1.0), (102.0, 1.0)]);
        // bidDepth 4, askDepth 2 -> (4 - 2) / 6 = 1/3.
        assert_eq!(obi.update(b), Some(1.0 / 3.0));
        assert!(obi.is_ready());
    }

    #[test]
    fn ask_heavy_full_depth_is_negative() {
        let mut obi = OrderBookImbalanceFull::new();
        let b = book(&[(100.0, 1.0)], &[(101.0, 2.0), (102.0, 1.0)]);
        // (1 - 3) / 4 = -0.5.
        assert_eq!(obi.update(b), Some(-0.5));
    }

    #[test]
    fn zero_size_is_zero() {
        let mut obi = OrderBookImbalanceFull::new();
        assert_eq!(
            obi.update(book(&[(100.0, 0.0)], &[(101.0, 0.0)])),
            Some(0.0)
        );
    }

    #[test]
    fn batch_equals_streaming() {
        let books: Vec<OrderBook> = (0..20)
            .map(|i| {
                let bid = 1.0 + f64::from(i % 3);
                book(&[(100.0, bid), (99.0, 1.0)], &[(101.0, 2.0), (102.0, 1.0)])
            })
            .collect();
        let mut a = OrderBookImbalanceFull::new();
        let mut b = OrderBookImbalanceFull::new();
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
        let mut obi = OrderBookImbalanceFull::new();
        obi.update(book(&[(100.0, 1.0)], &[(101.0, 1.0)]));
        assert!(obi.is_ready());
        obi.reset();
        assert!(!obi.is_ready());
    }
}
