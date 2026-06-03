//! Advance/Decline Line — cumulative net advancing-minus-declining issues.

use crate::cross_section::CrossSection;
use crate::traits::Indicator;

/// Advance/Decline Line (A/D Line) — the running cumulative sum of net advancing
/// issues across a universe.
///
/// On each [`CrossSection`] tick the net breadth is `advancers - decliners`:
/// the number of symbols with a positive price change minus the number with a
/// negative change (unchanged symbols are ignored). The line accumulates this
/// net value over time, so a rising line means advancers have persistently
/// outnumbered decliners — broad participation — while a falling line warns that
/// a rally is being carried by fewer and fewer names (a breadth divergence when
/// the index itself is still rising).
///
/// `Input = CrossSection`, `Output = f64`. The line is defined from the very
/// first tick, so `warmup_period == 1` and the indicator is ready after one
/// update.
///
/// # Example
///
/// ```
/// use wickra_core::{AdvanceDecline, CrossSection, Indicator, Member};
///
/// let mut ad = AdvanceDecline::new();
/// // 3 advancers, 1 decliner -> net +2.
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
/// assert_eq!(ad.update(tick), Some(2.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct AdvanceDecline {
    line: f64,
    has_emitted: bool,
}

impl AdvanceDecline {
    /// Construct a new Advance/Decline Line indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            line: 0.0,
            has_emitted: false,
        }
    }
}

impl Indicator for AdvanceDecline {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        let net = section.advancers() as f64 - section.decliners() as f64;
        self.line += net;
        self.has_emitted = true;
        Some(self.line)
    }

    fn reset(&mut self) {
        self.line = 0.0;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "AdvanceDecline"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_section::Member;
    use crate::traits::BatchExt;

    /// Build a cross-section with `up` advancers, `down` decliners and `flat`
    /// unchanged symbols.
    fn section(up: usize, down: usize, flat: usize) -> CrossSection {
        let mut members = Vec::new();
        for _ in 0..up {
            members.push(Member::new(1.0, 10.0, false, false));
        }
        for _ in 0..down {
            members.push(Member::new(-1.0, 10.0, false, false));
        }
        for _ in 0..flat {
            members.push(Member::new(0.0, 10.0, false, false));
        }
        CrossSection::new(members, 0).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let ad = AdvanceDecline::new();
        assert_eq!(ad.name(), "AdvanceDecline");
        assert_eq!(ad.warmup_period(), 1);
        assert!(!ad.is_ready());
    }

    #[test]
    fn first_tick_emits_net_breadth() {
        let mut ad = AdvanceDecline::new();
        assert_eq!(ad.update(section(3, 1, 0)), Some(2.0));
        assert!(ad.is_ready());
    }

    #[test]
    fn line_accumulates_across_ticks() {
        let mut ad = AdvanceDecline::new();
        assert_eq!(ad.update(section(3, 1, 0)), Some(2.0)); // +2 -> 2
        assert_eq!(ad.update(section(1, 4, 0)), Some(-1.0)); // -3 -> -1
        assert_eq!(ad.update(section(2, 0, 0)), Some(1.0)); // +2 -> 1
    }

    #[test]
    fn unchanged_symbols_are_ignored() {
        let mut ad = AdvanceDecline::new();
        // 2 up, 2 down, 5 unchanged -> net 0, line stays flat.
        assert_eq!(ad.update(section(2, 2, 5)), Some(0.0));
        assert_eq!(ad.update(section(2, 2, 5)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut ad = AdvanceDecline::new();
        ad.update(section(5, 0, 0));
        assert!(ad.is_ready());
        ad.reset();
        assert!(!ad.is_ready());
        // Line restarts from zero, not from the pre-reset value.
        assert_eq!(ad.update(section(1, 0, 0)), Some(1.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![
            section(3, 1, 2),
            section(1, 4, 0),
            section(2, 2, 1),
            section(5, 0, 3),
        ];
        let mut a = AdvanceDecline::new();
        let mut b = AdvanceDecline::new();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
