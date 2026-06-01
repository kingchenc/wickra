//! Funding Basis — the perpetual mark's relative premium to the spot index.

use crate::derivatives::DerivativesTick;
use crate::traits::Indicator;

/// Funding Basis — the relative basis between the perpetual mark price and the
/// spot index it tracks.
///
/// ```text
/// basis = (markPrice − indexPrice) / indexPrice
/// ```
///
/// The basis is the spread that the funding mechanism continuously pulls toward
/// zero: a positive basis (perpetual above spot) goes hand in hand with positive
/// funding (longs pay), a negative basis with negative funding. Reading the
/// instantaneous basis alongside the [funding rate] separates a genuine premium
/// from a stale-funding artefact and sizes the carry available to a cash-and-carry
/// or basis-arbitrage trade. The output is a fraction (e.g. `0.001` = 10 bps);
/// multiply by `10_000` for basis points.
///
/// `Input = DerivativesTick`, `Output = f64`. Stateless; ready after the first
/// tick.
///
/// [funding rate]: crate::FundingRate
///
/// # Example
///
/// ```
/// use wickra_core::{DerivativesTick, FundingBasis, Indicator};
///
/// let mut fb = FundingBasis::new();
/// // mark 100.5 vs index 100.0 -> (100.5 - 100.0) / 100.0 = 0.005.
/// let tick = DerivativesTick::new(
///     0.0, 100.5, 100.0, 100.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0,
/// )
/// .unwrap();
/// assert!((fb.update(tick).unwrap() - 0.005).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Default)]
pub struct FundingBasis {
    has_emitted: bool,
}

impl FundingBasis {
    /// Construct a new funding-basis indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for FundingBasis {
    type Input = DerivativesTick;
    type Output = f64;

    fn update(&mut self, tick: DerivativesTick) -> Option<f64> {
        self.has_emitted = true;
        Some((tick.mark_price - tick.index_price) / tick.index_price)
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
        "FundingBasis"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn tick(mark: f64, index: f64) -> DerivativesTick {
        DerivativesTick::new_unchecked(0.0, mark, index, mark, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0)
    }

    #[test]
    fn accessors_and_metadata() {
        let fb = FundingBasis::new();
        assert_eq!(fb.name(), "FundingBasis");
        assert_eq!(fb.warmup_period(), 1);
        assert!(!fb.is_ready());
    }

    #[test]
    fn premium_is_positive() {
        let mut fb = FundingBasis::new();
        let out = fb.update(tick(100.5, 100.0)).unwrap();
        assert!((out - 0.005).abs() < 1e-12);
        assert!(fb.is_ready());
    }

    #[test]
    fn discount_is_negative() {
        let mut fb = FundingBasis::new();
        let out = fb.update(tick(99.5, 100.0)).unwrap();
        assert!((out + 0.005).abs() < 1e-12);
    }

    #[test]
    fn at_par_is_zero() {
        let mut fb = FundingBasis::new();
        assert_eq!(fb.update(tick(100.0, 100.0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let ticks: Vec<DerivativesTick> = (0..20)
            .map(|i| tick(100.0 + f64::from(i % 5) * 0.1, 100.0))
            .collect();
        let mut a = FundingBasis::new();
        let mut b = FundingBasis::new();
        assert_eq!(
            a.batch(&ticks),
            ticks.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut fb = FundingBasis::new();
        fb.update(tick(100.5, 100.0));
        assert!(fb.is_ready());
        fb.reset();
        assert!(!fb.is_ready());
    }
}
