//! Depth Slope — how fast resting liquidity accumulates away from the mid.

use crate::microstructure::{Level, OrderBook};
use crate::traits::Indicator;

/// Ordinary-least-squares slope of cumulative resting size against distance
/// from the mid, over the levels of one book side.
///
/// `signed_distance` is `+1.0` for the ask side (price above the mid) and
/// `−1.0` for the bid side (price below the mid), so the regressor `x` —
/// distance from the mid — is non-negative on both sides. The response `y` is
/// the cumulative size walking outward from the touch. Returns `0.0` for a
/// degenerate fit where every level sits at the same distance (zero variance in
/// `x`).
fn cumulative_slope(levels: &[Level], mid: f64, signed_distance: f64) -> f64 {
    let count = levels.len() as f64;
    let mut cumulative = 0.0;
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    let mut sum_xx = 0.0;
    for level in levels {
        let x = signed_distance * (level.price - mid);
        cumulative += level.size;
        sum_x += x;
        sum_y += cumulative;
        sum_xy += x * cumulative;
        sum_xx += x * x;
    }
    let denom = count * sum_xx - sum_x * sum_x;
    if denom == 0.0 {
        return 0.0;
    }
    (count * sum_xy - sum_x * sum_y) / denom
}

/// Depth Slope — the average rate at which cumulative resting size grows with
/// distance from the mid, across the bid and ask sides of the book.
///
/// For each side the indicator runs an ordinary-least-squares regression of
/// cumulative size (walking outward from the touch) on the level's distance
/// from the mid, then reports the mean of the two slopes:
///
/// ```text
/// slope_side = OLS slope of (|priceᵢ − mid|, Σ_{j≤i} sizeⱼ)
/// depthSlope = (slope_bid + slope_ask) / 2
/// ```
///
/// Because the response is *cumulative* size it never decreases with distance,
/// so the slope is non-negative: it is a magnitude, not a direction. A large
/// slope means cumulative liquidity builds quickly away from the touch — a deep
/// book that absorbs large orders with little walking; a small slope is a thin,
/// shallow book. A book whose size is concentrated at the touch and thins out
/// behind it (a fragile book) reads a *smaller* slope than one of equal total
/// depth that thickens with distance.
///
/// A side with fewer than two levels carries no slope, so the indicator returns
/// `0.0` whenever either side has fewer than two levels (including an empty
/// book).
///
/// `Input = OrderBook`, `Output = f64`. Stateless; ready after the first
/// snapshot.
///
/// # Example
///
/// ```
/// use wickra_core::{DepthSlope, Indicator, Level, OrderBook};
///
/// // Both sides thicken linearly away from the mid (sizes 1, 2, 3 …).
/// let book = OrderBook::new(
///     vec![Level::new(99.0, 1.0).unwrap(), Level::new(98.0, 2.0).unwrap()],
///     vec![Level::new(101.0, 1.0).unwrap(), Level::new(102.0, 2.0).unwrap()],
/// )
/// .unwrap();
/// let mut ds = DepthSlope::new();
/// assert!(ds.update(book).unwrap() > 0.0);
/// ```
#[derive(Debug, Clone, Default)]
pub struct DepthSlope {
    has_emitted: bool,
}

impl DepthSlope {
    /// Construct a new depth-slope indicator.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for DepthSlope {
    type Input = OrderBook;
    type Output = f64;

    fn update(&mut self, book: OrderBook) -> Option<f64> {
        self.has_emitted = true;
        let Some(mid) = book.mid() else {
            return Some(0.0);
        };
        if book.bids.len() < 2 || book.asks.len() < 2 {
            return Some(0.0);
        }
        let bid_slope = cumulative_slope(&book.bids, mid, -1.0);
        let ask_slope = cumulative_slope(&book.asks, mid, 1.0);
        Some(f64::midpoint(bid_slope, ask_slope))
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
        "DepthSlope"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let ds = DepthSlope::new();
        assert_eq!(ds.name(), "DepthSlope");
        assert_eq!(ds.warmup_period(), 1);
        assert!(!ds.is_ready());
    }

    #[test]
    fn thickening_book_has_positive_slope() {
        let mut ds = DepthSlope::new();
        let out = ds
            .update(book(
                &[(99.0, 1.0), (98.0, 2.0), (97.0, 3.0)],
                &[(101.0, 1.0), (102.0, 2.0), (103.0, 3.0)],
            ))
            .unwrap();
        assert!(out > 0.0);
        assert!(ds.is_ready());
    }

    #[test]
    fn front_loaded_book_has_smaller_slope_than_back_loaded() {
        // Same total depth (6 per side), but one book thickens away from the
        // touch and the other thins. Cumulative slope is non-negative for both;
        // the back-loaded book accumulates faster, so its slope is larger.
        let mut back = DepthSlope::new();
        let back_slope = back
            .update(book(
                &[(99.0, 1.0), (98.0, 2.0), (97.0, 3.0)],
                &[(101.0, 1.0), (102.0, 2.0), (103.0, 3.0)],
            ))
            .unwrap();
        let mut front = DepthSlope::new();
        let front_slope = front
            .update(book(
                &[(99.0, 3.0), (98.0, 2.0), (97.0, 1.0)],
                &[(101.0, 3.0), (102.0, 2.0), (103.0, 1.0)],
            ))
            .unwrap();
        assert!(front_slope >= 0.0);
        assert!(back_slope > front_slope);
    }

    #[test]
    fn known_slope_value() {
        // Symmetric book, each side: distances 1, 2; cumulative sizes 1, 3.
        // OLS slope of (1->1, 2->3) = 2. Mean of two equal sides = 2.
        let mut ds = DepthSlope::new();
        let out = ds
            .update(book(
                &[(99.0, 1.0), (98.0, 2.0)],
                &[(101.0, 1.0), (102.0, 2.0)],
            ))
            .unwrap();
        assert!((out - 2.0).abs() < 1e-9);
    }

    #[test]
    fn single_level_side_is_zero() {
        let mut ds = DepthSlope::new();
        // Bid side has only one level -> no slope -> 0.
        assert_eq!(
            ds.update(book(&[(100.0, 1.0)], &[(101.0, 1.0), (102.0, 1.0)])),
            Some(0.0)
        );
    }

    #[test]
    fn empty_book_is_zero() {
        let mut ds = DepthSlope::new();
        assert_eq!(
            ds.update(OrderBook::new_unchecked(vec![], vec![])),
            Some(0.0)
        );
    }

    #[test]
    fn degenerate_distance_slope_is_zero() {
        // Two levels at the same distance from mid carry zero x-variance.
        let levels = [
            Level::new_unchecked(100.0, 1.0),
            Level::new_unchecked(100.0, 2.0),
        ];
        assert_eq!(cumulative_slope(&levels, 100.0, 1.0), 0.0);
    }

    #[test]
    fn batch_equals_streaming() {
        let books: Vec<OrderBook> = (0..20)
            .map(|i| {
                let extra = f64::from(i % 4);
                book(
                    &[(99.0, 1.0 + extra), (98.0, 2.0)],
                    &[(101.0, 1.0), (102.0, 2.0 + extra)],
                )
            })
            .collect();
        let mut a = DepthSlope::new();
        let mut b = DepthSlope::new();
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
        let mut ds = DepthSlope::new();
        ds.update(book(
            &[(99.0, 1.0), (98.0, 2.0)],
            &[(101.0, 1.0), (102.0, 2.0)],
        ));
        assert!(ds.is_ready());
        ds.reset();
        assert!(!ds.is_ready());
    }
}
