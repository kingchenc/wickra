//! Term-Structure Basis — the dated future's relative premium to spot.

use crate::derivatives::DerivativesTick;
use crate::traits::Indicator;

/// Term-Structure Basis — the relative basis between a dated (e.g. quarterly)
/// futures price and the spot index.
///
/// ```text
/// basis = (futuresPrice − indexPrice) / indexPrice
/// ```
///
/// Where [`FundingBasis`] measures the *perpetual*'s premium to spot, this
/// measures a *dated future*'s — the term-structure carry that a calendar or
/// cash-and-carry trade harvests as the contract converges to spot at expiry. A
/// positive basis is contango (futures above spot), a negative one backwardation.
/// The output is a fraction (e.g. `0.02` = 2%); multiply by `10_000` for basis
/// points.
///
/// `Input = DerivativesTick`, `Output = f64`. Stateless; ready after the first
/// tick.
///
/// [`FundingBasis`]: crate::FundingBasis
///
/// # Example
///
/// ```
/// use wickra_core::{DerivativesTick, Indicator, TermStructureBasis};
///
/// fn tick(futures: f64, index: f64) -> DerivativesTick {
///     DerivativesTick::new(0.0, index, index, futures, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0)
///         .unwrap()
/// }
///
/// let mut ts = TermStructureBasis::new();
/// // futures 102 vs index 100 -> 0.02 (2% contango).
/// assert!((ts.update(tick(102.0, 100.0)).unwrap() - 0.02).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Default)]
pub struct TermStructureBasis {
    has_emitted: bool,
}

impl TermStructureBasis {
    /// Construct a new term-structure basis indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for TermStructureBasis {
    type Input = DerivativesTick;
    type Output = f64;

    fn update(&mut self, tick: DerivativesTick) -> Option<f64> {
        self.has_emitted = true;
        Some((tick.futures_price - tick.index_price) / tick.index_price)
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
        "TermStructureBasis"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn tick(futures: f64, index: f64) -> DerivativesTick {
        DerivativesTick::new_unchecked(
            0.0, index, index, futures, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0,
        )
    }

    #[test]
    fn accessors_and_metadata() {
        let ts = TermStructureBasis::new();
        assert_eq!(ts.name(), "TermStructureBasis");
        assert_eq!(ts.warmup_period(), 1);
        assert!(!ts.is_ready());
    }

    #[test]
    fn contango_is_positive() {
        let mut ts = TermStructureBasis::new();
        let out = ts.update(tick(102.0, 100.0)).unwrap();
        assert!((out - 0.02).abs() < 1e-12);
        assert!(ts.is_ready());
    }

    #[test]
    fn backwardation_is_negative() {
        let mut ts = TermStructureBasis::new();
        let out = ts.update(tick(98.0, 100.0)).unwrap();
        assert!((out + 0.02).abs() < 1e-12);
    }

    #[test]
    fn at_par_is_zero() {
        let mut ts = TermStructureBasis::new();
        assert_eq!(ts.update(tick(100.0, 100.0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let ticks: Vec<DerivativesTick> = (0..20)
            .map(|i| tick(100.0 + f64::from(i % 5), 100.0))
            .collect();
        let mut a = TermStructureBasis::new();
        let mut b = TermStructureBasis::new();
        assert_eq!(
            a.batch(&ticks),
            ticks.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut ts = TermStructureBasis::new();
        ts.update(tick(102.0, 100.0));
        assert!(ts.is_ready());
        ts.reset();
        assert!(!ts.is_ready());
    }
}
