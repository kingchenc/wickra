//! Calendar Spread — the dated future's relative premium to the perpetual.

use crate::derivatives::DerivativesTick;
use crate::traits::Indicator;

/// Calendar Spread — the relative spread between a dated (e.g. quarterly)
/// futures price and the perpetual mark price.
///
/// ```text
/// spread = (futuresPrice − markPrice) / markPrice
/// ```
///
/// A calendar (or inter-delivery) spread trades the *near* leg against the
/// *far* leg — here the perpetual against a dated future. The relative spread is
/// the roll yield available between the two contracts: positive when the future
/// trades over the perpetual (contango roll), negative when under
/// (backwardation). Where [`TermStructureBasis`] measures the future against
/// spot, this measures it against the perpetual — the leg a perp-vs-future
/// basis trade actually holds. The output is a fraction; multiply by `10_000`
/// for basis points.
///
/// `Input = DerivativesTick`, `Output = f64`. Stateless; ready after the first
/// tick.
///
/// [`TermStructureBasis`]: crate::TermStructureBasis
///
/// # Example
///
/// ```
/// use wickra_core::{CalendarSpread, DerivativesTick, Indicator};
///
/// fn tick(futures: f64, mark: f64) -> DerivativesTick {
///     DerivativesTick::new(0.0, mark, mark, futures, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0)
///         .unwrap()
/// }
///
/// let mut cs = CalendarSpread::new();
/// // futures 101 vs perpetual mark 100 -> 0.01.
/// assert!((cs.update(tick(101.0, 100.0)).unwrap() - 0.01).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Default)]
pub struct CalendarSpread {
    has_emitted: bool,
}

impl CalendarSpread {
    /// Construct a new calendar-spread indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for CalendarSpread {
    type Input = DerivativesTick;
    type Output = f64;

    fn update(&mut self, tick: DerivativesTick) -> Option<f64> {
        self.has_emitted = true;
        Some((tick.futures_price - tick.mark_price) / tick.mark_price)
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
        "CalendarSpread"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn tick(futures: f64, mark: f64) -> DerivativesTick {
        DerivativesTick::new_unchecked(
            0.0, mark, mark, futures, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0,
        )
    }

    #[test]
    fn accessors_and_metadata() {
        let cs = CalendarSpread::new();
        assert_eq!(cs.name(), "CalendarSpread");
        assert_eq!(cs.warmup_period(), 1);
        assert!(!cs.is_ready());
    }

    #[test]
    fn future_over_perp_is_positive() {
        let mut cs = CalendarSpread::new();
        let out = cs.update(tick(101.0, 100.0)).unwrap();
        assert!((out - 0.01).abs() < 1e-12);
        assert!(cs.is_ready());
    }

    #[test]
    fn future_under_perp_is_negative() {
        let mut cs = CalendarSpread::new();
        let out = cs.update(tick(99.0, 100.0)).unwrap();
        assert!((out + 0.01).abs() < 1e-12);
    }

    #[test]
    fn flat_is_zero() {
        let mut cs = CalendarSpread::new();
        assert_eq!(cs.update(tick(100.0, 100.0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let ticks: Vec<DerivativesTick> = (0..20)
            .map(|i| tick(100.0 + f64::from(i % 5), 100.0))
            .collect();
        let mut a = CalendarSpread::new();
        let mut b = CalendarSpread::new();
        assert_eq!(
            a.batch(&ticks),
            ticks.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut cs = CalendarSpread::new();
        cs.update(tick(101.0, 100.0));
        assert!(cs.is_ready());
        cs.reset();
        assert!(!cs.is_ready());
    }
}
