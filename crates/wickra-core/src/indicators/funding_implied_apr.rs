//! Funding-Implied APR — the per-interval funding rate annualised.

use crate::derivatives::DerivativesTick;
use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Funding-Implied APR — the perpetual's per-interval funding rate scaled to an
/// annualised rate.
///
/// ```text
/// APR = funding_rate · intervals_per_year
/// ```
///
/// Funding is paid in small per-interval amounts (commonly every 8 hours, i.e.
/// `1095` intervals per year). Annualising it converts the headline funding number
/// into the carry cost (or yield) of holding the position for a year, which is far
/// easier to reason about and to compare against spot lending rates, basis trades,
/// and other yields. A large positive APR means longs pay a steep carry to shorts
/// (and vice versa) — the economic incentive behind cash-and-carry and
/// funding-arbitrage strategies.
///
/// The output is a fraction (multiply by `100` for percent) and may be negative.
/// It is stateless — each tick yields one value (no warmup). Each `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{DerivativesTick, Indicator, FundingImpliedApr};
///
/// // 0.01% per 8h funding -> 0.0001 * 1095 ≈ 10.95% APR.
/// let mut indicator = FundingImpliedApr::new(1095.0).unwrap();
/// let tick = DerivativesTick::new(0.0001, 100.0, 100.0, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0).unwrap();
/// let apr = indicator.update(tick).unwrap();
/// assert!((apr - 0.1095).abs() < 1e-9);
/// ```
#[derive(Debug, Clone)]
pub struct FundingImpliedApr {
    intervals_per_year: f64,
    ready: bool,
}

impl FundingImpliedApr {
    /// Construct a Funding-Implied APR with the number of funding intervals per
    /// year (e.g. `1095` for 8-hour funding, `365` for daily).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if `intervals_per_year` is not finite
    /// and positive.
    pub fn new(intervals_per_year: f64) -> Result<Self> {
        if !intervals_per_year.is_finite() || intervals_per_year <= 0.0 {
            return Err(Error::InvalidParameter {
                message: "intervals_per_year must be finite and positive",
            });
        }
        Ok(Self {
            intervals_per_year,
            ready: false,
        })
    }

    /// Configured intervals per year.
    pub const fn intervals_per_year(&self) -> f64 {
        self.intervals_per_year
    }
}

impl Indicator for FundingImpliedApr {
    type Input = DerivativesTick;
    type Output = f64;

    fn update(&mut self, tick: DerivativesTick) -> Option<f64> {
        self.ready = true;
        Some(tick.funding_rate * self.intervals_per_year)
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
        "FundingImpliedApr"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn tick(funding: f64) -> DerivativesTick {
        DerivativesTick::new_unchecked(
            funding, 100.0, 100.0, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0,
        )
    }

    #[test]
    fn rejects_invalid_intervals() {
        assert!(matches!(
            FundingImpliedApr::new(0.0),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            FundingImpliedApr::new(-1.0),
            Err(Error::InvalidParameter { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let f = FundingImpliedApr::new(1095.0).unwrap();
        assert_relative_eq!(f.intervals_per_year(), 1095.0, epsilon = 1e-12);
        assert_eq!(f.warmup_period(), 1);
        assert_eq!(f.name(), "FundingImpliedApr");
        assert!(!f.is_ready());
    }

    #[test]
    fn apr_reference_value() {
        let mut f = FundingImpliedApr::new(1095.0).unwrap();
        assert_relative_eq!(f.update(tick(0.0001)).unwrap(), 0.1095, epsilon = 1e-9);
    }

    #[test]
    fn negative_funding_is_negative_apr() {
        let mut f = FundingImpliedApr::new(1095.0).unwrap();
        assert!(f.update(tick(-0.0001)).unwrap() < 0.0);
    }

    #[test]
    fn zero_funding_is_zero() {
        let mut f = FundingImpliedApr::new(365.0).unwrap();
        assert_relative_eq!(f.update(tick(0.0)).unwrap(), 0.0, epsilon = 1e-12);
    }

    #[test]
    fn reset_clears_state() {
        let mut f = FundingImpliedApr::new(1095.0).unwrap();
        f.update(tick(0.0001));
        assert!(f.is_ready());
        f.reset();
        assert!(!f.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let ticks: Vec<DerivativesTick> = (0..40)
            .map(|i| tick(0.0001 * (f64::from(i) * 0.3).sin()))
            .collect();
        let batch = FundingImpliedApr::new(1095.0).unwrap().batch(&ticks);
        let mut b = FundingImpliedApr::new(1095.0).unwrap();
        let streamed: Vec<_> = ticks.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
