//! Advance/Decline Ratio — advancing issues divided by declining issues.

use crate::cross_section::CrossSection;
use crate::traits::Indicator;

/// Advance/Decline Ratio (ADR) — the number of advancing symbols divided by the
/// number of declining symbols across a universe.
///
/// On each [`CrossSection`] tick the ratio is `advancers / decliners`: a reading
/// above one means advancing issues outnumber declining ones (broad strength),
/// while a reading below one signals broad weakness. Because it is a ratio rather
/// than a difference, the ADR is comparable across universes of different sizes.
///
/// When a tick has no declining symbols the denominator is floored to one, so the
/// ratio degrades gracefully to the advancer count instead of dividing by zero.
///
/// `Input = CrossSection`, `Output = f64`. The ratio is defined from the first
/// tick, so `warmup_period == 1` and the indicator is ready after one update.
///
/// # Example
///
/// ```
/// use wickra_core::{AdvanceDeclineRatio, CrossSection, Indicator, Member};
///
/// let mut adr = AdvanceDeclineRatio::new();
/// // 3 advancers, 1 decliner -> ratio 3.0.
/// let tick = CrossSection::new(
///     vec![
///         Member::new(1.0, 10.0, false, false),
///         Member::new(0.5, 10.0, false, false),
///         Member::new(2.0, 10.0, false, false),
///         Member::new(-1.0, 10.0, false, false),
///     ],
///     0,
/// )
/// .unwrap();
/// assert_eq!(adr.update(tick), Some(3.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct AdvanceDeclineRatio {
    has_emitted: bool,
}

impl AdvanceDeclineRatio {
    /// Construct a new Advance/Decline Ratio indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for AdvanceDeclineRatio {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        let advancers = section.advancers() as f64;
        let decliners = section.decliners().max(1) as f64;
        self.has_emitted = true;
        Some(advancers / decliners)
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
        "AdvanceDeclineRatio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_section::Member;
    use crate::traits::BatchExt;

    fn section(up: usize, down: usize) -> CrossSection {
        let mut members = Vec::new();
        for _ in 0..up {
            members.push(Member::new(1.0, 10.0, false, false));
        }
        for _ in 0..down {
            members.push(Member::new(-1.0, 10.0, false, false));
        }
        // A non-empty unchanged member guarantees a valid universe when both
        // counts are zero.
        members.push(Member::new(0.0, 10.0, false, false));
        CrossSection::new(members, 0).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let adr = AdvanceDeclineRatio::new();
        assert_eq!(adr.name(), "AdvanceDeclineRatio");
        assert_eq!(adr.warmup_period(), 1);
        assert!(!adr.is_ready());
    }

    #[test]
    fn first_tick_emits_ratio() {
        let mut adr = AdvanceDeclineRatio::new();
        assert_eq!(adr.update(section(3, 1)), Some(3.0));
        assert!(adr.is_ready());
    }

    #[test]
    fn zero_decliners_floors_denominator() {
        let mut adr = AdvanceDeclineRatio::new();
        // 4 advancers, 0 decliners -> 4 / max(0, 1) = 4.0.
        assert_eq!(adr.update(section(4, 0)), Some(4.0));
    }

    #[test]
    fn no_advancers_yields_zero() {
        let mut adr = AdvanceDeclineRatio::new();
        assert_eq!(adr.update(section(0, 5)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut adr = AdvanceDeclineRatio::new();
        adr.update(section(3, 1));
        assert!(adr.is_ready());
        adr.reset();
        assert!(!adr.is_ready());
        assert_eq!(adr.update(section(2, 1)), Some(2.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![section(3, 1), section(4, 0), section(0, 5), section(2, 2)];
        let mut a = AdvanceDeclineRatio::new();
        let mut b = AdvanceDeclineRatio::new();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
