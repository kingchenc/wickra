//! Funding Rate — the current perpetual funding rate.

use crate::derivatives::DerivativesTick;
use crate::traits::Indicator;

/// Funding Rate — the funding rate carried by each derivatives tick.
///
/// The funding rate is the periodic payment exchanged between long and short
/// perpetual-swap holders that tethers the perpetual mark to the spot index. A
/// positive rate means longs pay shorts (the perpetual trades at a premium); a
/// negative rate means shorts pay longs (a discount). This indicator simply
/// surfaces the rate from the [`DerivativesTick`] feed so it can be charted,
/// chained or fed to the rolling funding statistics ([`FundingRateMean`],
/// [`FundingRateZScore`]).
///
/// `Input = DerivativesTick`, `Output = f64`. Stateless; ready after the first
/// tick.
///
/// [`FundingRateMean`]: crate::FundingRateMean
/// [`FundingRateZScore`]: crate::FundingRateZScore
///
/// # Example
///
/// ```
/// use wickra_core::{DerivativesTick, FundingRate, Indicator};
///
/// let mut fr = FundingRate::new();
/// let tick = DerivativesTick::new(
///     0.0001, 100.0, 100.0, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0,
/// )
/// .unwrap();
/// assert_eq!(fr.update(tick), Some(0.0001));
/// ```
#[derive(Debug, Clone, Default)]
pub struct FundingRate {
    has_emitted: bool,
}

impl FundingRate {
    /// Construct a new funding-rate indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for FundingRate {
    type Input = DerivativesTick;
    type Output = f64;

    fn update(&mut self, tick: DerivativesTick) -> Option<f64> {
        self.has_emitted = true;
        Some(tick.funding_rate)
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
        "FundingRate"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn tick(funding_rate: f64) -> DerivativesTick {
        DerivativesTick::new_unchecked(
            funding_rate,
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
            0,
        )
    }

    #[test]
    fn accessors_and_metadata() {
        let fr = FundingRate::new();
        assert_eq!(fr.name(), "FundingRate");
        assert_eq!(fr.warmup_period(), 1);
        assert!(!fr.is_ready());
    }

    #[test]
    fn passes_through_funding_rate() {
        let mut fr = FundingRate::new();
        assert_eq!(fr.update(tick(0.0001)), Some(0.0001));
        assert_eq!(fr.update(tick(-0.0003)), Some(-0.0003));
        assert!(fr.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let ticks: Vec<DerivativesTick> =
            (0..20).map(|i| tick(0.0001 * f64::from(i - 10))).collect();
        let mut a = FundingRate::new();
        let mut b = FundingRate::new();
        assert_eq!(
            a.batch(&ticks),
            ticks.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut fr = FundingRate::new();
        fr.update(tick(0.0001));
        assert!(fr.is_ready());
        fr.reset();
        assert!(!fr.is_ready());
    }
}
