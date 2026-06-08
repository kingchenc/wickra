//! Perpetual Premium Index — the perp mark price relative to spot.

use crate::derivatives::DerivativesTick;
use crate::traits::Indicator;

/// Perpetual Premium Index — the perpetual's mark price relative to the spot index
/// it tracks, as a fraction.
///
/// ```text
/// premium = (mark_price − index_price) / index_price
/// ```
///
/// A perpetual swap is pegged to spot by the funding mechanism, but it can still
/// trade at a premium (above spot) or discount (below). A positive premium signals
/// net long demand willing to pay up to hold the perp — bullish positioning, and
/// the proximate driver of positive funding; a negative premium signals the
/// reverse. Sustained extremes flag crowded positioning ripe for a funding-driven
/// mean reversion.
///
/// The output is centred on zero and dimensionless (a fraction; multiply by `100`
/// for percent). `index_price` is validated strictly positive on the tick, so the
/// division is always defined. It is stateless — each tick yields one value (no
/// warmup). Each `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{DerivativesTick, Indicator, PerpetualPremiumIndex};
///
/// let mut indicator = PerpetualPremiumIndex::new();
/// // Mark 101 vs index 100 -> +1% premium.
/// let tick = DerivativesTick::new(0.0, 101.0, 100.0, 101.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0).unwrap();
/// let premium = indicator.update(tick).unwrap();
/// assert!((premium - 0.01).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Default)]
pub struct PerpetualPremiumIndex {
    ready: bool,
}

impl PerpetualPremiumIndex {
    /// Construct a new Perpetual Premium Index. The indicator is parameter-free.
    #[must_use]
    pub const fn new() -> Self {
        Self { ready: false }
    }
}

impl Indicator for PerpetualPremiumIndex {
    type Input = DerivativesTick;
    type Output = f64;

    fn update(&mut self, tick: DerivativesTick) -> Option<f64> {
        let premium = (tick.mark_price - tick.index_price) / tick.index_price;
        self.ready = true;
        Some(premium)
    }

    fn reset(&mut self) {
        self.ready = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn name(&self) -> &'static str {
        "PerpetualPremiumIndex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn tick(mark: f64, index: f64) -> DerivativesTick {
        DerivativesTick::new_unchecked(0.0, mark, index, mark, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0)
    }

    #[test]
    fn accessors_and_metadata() {
        let p = PerpetualPremiumIndex::new();
        assert_eq!(p.warmup_period(), 1);
        assert_eq!(p.name(), "PerpetualPremiumIndex");
        assert!(!p.is_ready());
    }

    #[test]
    fn premium_reference_value() {
        let mut p = PerpetualPremiumIndex::new();
        assert_relative_eq!(p.update(tick(101.0, 100.0)).unwrap(), 0.01, epsilon = 1e-12);
    }

    #[test]
    fn discount_is_negative() {
        let mut p = PerpetualPremiumIndex::new();
        assert!(p.update(tick(99.0, 100.0)).unwrap() < 0.0);
    }

    #[test]
    fn at_par_is_zero() {
        let mut p = PerpetualPremiumIndex::new();
        assert_relative_eq!(p.update(tick(100.0, 100.0)).unwrap(), 0.0, epsilon = 1e-12);
    }

    #[test]
    fn ready_after_first_update() {
        let mut p = PerpetualPremiumIndex::new();
        assert!(!p.is_ready());
        p.update(tick(100.0, 100.0));
        assert!(p.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut p = PerpetualPremiumIndex::new();
        p.update(tick(101.0, 100.0));
        assert!(p.is_ready());
        p.reset();
        assert!(!p.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let ticks: Vec<DerivativesTick> = (0..40)
            .map(|i| tick(100.0 + (f64::from(i) * 0.3).sin(), 100.0))
            .collect();
        let batch = PerpetualPremiumIndex::new().batch(&ticks);
        let mut b = PerpetualPremiumIndex::new();
        let streamed: Vec<_> = ticks.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
