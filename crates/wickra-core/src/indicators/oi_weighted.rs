//! Open-Interest-Weighted Price — cumulative mark price weighted by open
//! interest.

use crate::derivatives::DerivativesTick;
use crate::traits::Indicator;

/// Open-Interest-Weighted Price — the running mean mark price, weighting each
/// tick by its open interest.
///
/// ```text
/// oiWeighted = Σ(markPrice · openInterest) / Σ openInterest
/// ```
///
/// Where a plain mean treats every tick equally, the OI-weighted price pulls
/// toward the levels at which the most contracts were actually outstanding — the
/// price the bulk of open positioning sits around, a fair-value anchor for
/// liquidations and mean-reversion. The accumulation runs from construction;
/// call [`reset`] at each session boundary to re-anchor. Until any open interest
/// has accrued the indicator returns the current mark price.
///
/// `Input = DerivativesTick`, `Output = f64`. Ready after the first tick.
///
/// [`reset`]: crate::Indicator::reset
///
/// # Example
///
/// ```
/// use wickra_core::{DerivativesTick, Indicator, OIWeighted};
///
/// fn tick(mark: f64, oi: f64) -> DerivativesTick {
///     DerivativesTick::new(0.0, mark, mark, mark, oi, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0)
///         .unwrap()
/// }
///
/// let mut oiw = OIWeighted::new();
/// assert_eq!(oiw.update(tick(100.0, 10.0)), Some(100.0));
/// // (100·10 + 110·30) / (10 + 30) = 4300 / 40 = 107.5.
/// assert_eq!(oiw.update(tick(110.0, 30.0)), Some(107.5));
/// ```
#[derive(Debug, Clone, Default)]
pub struct OIWeighted {
    sum_weighted: f64,
    sum_oi: f64,
    has_emitted: bool,
}

impl OIWeighted {
    /// Construct a new OI-weighted price indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sum_weighted: 0.0,
            sum_oi: 0.0,
            has_emitted: false,
        }
    }
}

impl Indicator for OIWeighted {
    type Input = DerivativesTick;
    type Output = f64;

    fn update(&mut self, tick: DerivativesTick) -> Option<f64> {
        self.has_emitted = true;
        self.sum_weighted += tick.mark_price * tick.open_interest;
        self.sum_oi += tick.open_interest;
        if self.sum_oi == 0.0 {
            // No open interest has accrued yet: fall back to the mark price.
            return Some(tick.mark_price);
        }
        Some(self.sum_weighted / self.sum_oi)
    }

    fn reset(&mut self) {
        self.sum_weighted = 0.0;
        self.sum_oi = 0.0;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "OIWeighted"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn tick(mark: f64, oi: f64) -> DerivativesTick {
        DerivativesTick::new_unchecked(0.0, mark, mark, mark, oi, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0)
    }

    #[test]
    fn accessors_and_metadata() {
        let oiw = OIWeighted::new();
        assert_eq!(oiw.name(), "OIWeighted");
        assert_eq!(oiw.warmup_period(), 1);
        assert!(!oiw.is_ready());
    }

    #[test]
    fn weights_by_open_interest() {
        let mut oiw = OIWeighted::new();
        assert_eq!(oiw.update(tick(100.0, 10.0)), Some(100.0));
        // (100·10 + 110·30) / 40 = 107.5.
        assert_eq!(oiw.update(tick(110.0, 30.0)), Some(107.5));
        assert!(oiw.is_ready());
    }

    #[test]
    fn zero_open_interest_falls_back_to_mark() {
        let mut oiw = OIWeighted::new();
        assert_eq!(oiw.update(tick(123.0, 0.0)), Some(123.0));
        // Still no OI on the second zero-OI tick.
        assert_eq!(oiw.update(tick(125.0, 0.0)), Some(125.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let ticks: Vec<DerivativesTick> = (0..20)
            .map(|i| tick(100.0 + f64::from(i % 5), 1.0 + f64::from(i % 4)))
            .collect();
        let mut a = OIWeighted::new();
        let mut b = OIWeighted::new();
        assert_eq!(
            a.batch(&ticks),
            ticks.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_re_anchors() {
        let mut oiw = OIWeighted::new();
        oiw.update(tick(100.0, 10.0));
        oiw.update(tick(110.0, 30.0));
        assert!(oiw.is_ready());
        oiw.reset();
        assert!(!oiw.is_ready());
        // After reset the accumulation starts again from the next tick.
        assert_eq!(oiw.update(tick(200.0, 5.0)), Some(200.0));
    }
}
