//! Breadth Thrust (Zweig) — a moving average of the advancing-issues share.

use crate::cross_section::CrossSection;
use crate::error::Result;
use crate::traits::Indicator;
use crate::Sma;

/// Breadth Thrust (Zweig) — a simple moving average of the advancing-issues
/// share, `advancers / (advancers + decliners)`.
///
/// Martin Zweig's breadth thrust smooths the fraction of participating issues
/// that are advancing over a short window (the classic period is 10). A "thrust"
/// fires when this average climbs from below ~0.40 (oversold, washed-out breadth)
/// to above ~0.615 within about ten sessions — historically a rare, reliable
/// signal that a powerful new advance has begun with broad participation.
///
/// Each tick's share floors the participating count to one, so a tick with no
/// advancing or declining issues contributes a defined `0.0` instead of dividing
/// by zero. The reading is `None` until `period` ticks have been seen.
///
/// `Input = CrossSection`, `Output = f64` (a share in `0..=1`),
/// `warmup_period == period`.
///
/// # Example
///
/// ```
/// use wickra_core::{BreadthThrust, CrossSection, Indicator, Member};
///
/// let mut bt = BreadthThrust::new(2).unwrap();
/// let up = CrossSection::new(vec![Member::new(1.0, 1.0, false, false)], 0).unwrap();
/// assert_eq!(bt.update(up.clone()), None); // warming up
/// assert_eq!(bt.update(up), Some(1.0)); // both ticks 100% advancing
/// ```
#[derive(Debug, Clone)]
pub struct BreadthThrust {
    sma: Sma,
}

impl BreadthThrust {
    /// Construct a new Breadth Thrust over the given window length.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`](crate::Error::PeriodZero) if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            sma: Sma::new(period)?,
        })
    }

    /// Configured window length.
    #[must_use]
    pub const fn period(&self) -> usize {
        self.sma.period()
    }
}

impl Indicator for BreadthThrust {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        let advancers = section.advancers();
        let decliners = section.decliners();
        let participating = (advancers + decliners).max(1) as f64;
        let share = advancers as f64 / participating;
        self.sma.update(share)
    }

    fn reset(&mut self) {
        self.sma.reset();
    }

    fn warmup_period(&self) -> usize {
        self.sma.period()
    }

    fn is_ready(&self) -> bool {
        self.sma.value().is_some()
    }

    fn name(&self) -> &'static str {
        "BreadthThrust"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_section::Member;
    use crate::error::Error;
    use crate::traits::BatchExt;

    fn section(up: usize, down: usize) -> CrossSection {
        let mut members = Vec::new();
        for _ in 0..up {
            members.push(Member::new(1.0, 10.0, false, false));
        }
        for _ in 0..down {
            members.push(Member::new(-1.0, 10.0, false, false));
        }
        members.push(Member::new(0.0, 10.0, false, false));
        CrossSection::new(members, 0).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let bt = BreadthThrust::new(10).unwrap();
        assert_eq!(bt.name(), "BreadthThrust");
        assert_eq!(bt.warmup_period(), 10);
        assert_eq!(bt.period(), 10);
        assert!(!bt.is_ready());
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(BreadthThrust::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn averages_the_advancing_share() {
        let mut bt = BreadthThrust::new(2).unwrap();
        // share = 8 / 10 = 0.8 ; window not full yet.
        assert_eq!(bt.update(section(8, 2)), None);
        // share = 6 / 10 = 0.6 ; SMA(2) = (0.8 + 0.6) / 2 = 0.7.
        let value = bt.update(section(6, 4)).unwrap();
        assert!((value - 0.7).abs() < 1e-9);
        assert!(bt.is_ready());
        // share = 5 / 10 = 0.5 ; SMA(2) = (0.6 + 0.5) / 2 = 0.55.
        let value = bt.update(section(5, 5)).unwrap();
        assert!((value - 0.55).abs() < 1e-9);
    }

    #[test]
    fn empty_participation_floors_to_zero_share() {
        let mut bt = BreadthThrust::new(1).unwrap();
        // No advancers or decliners -> 0 / max(0, 1) = 0.0.
        assert_eq!(bt.update(section(0, 0)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut bt = BreadthThrust::new(2).unwrap();
        bt.update(section(8, 2));
        bt.update(section(6, 4));
        assert!(bt.is_ready());
        bt.reset();
        assert!(!bt.is_ready());
        assert_eq!(bt.update(section(8, 2)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![section(8, 2), section(6, 4), section(5, 5), section(0, 0)];
        let mut a = BreadthThrust::new(2).unwrap();
        let mut b = BreadthThrust::new(2).unwrap();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
