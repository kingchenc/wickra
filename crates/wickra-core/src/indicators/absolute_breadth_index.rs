//! Absolute Breadth Index — the magnitude of net advancing-minus-declining issues.

use crate::cross_section::CrossSection;
use crate::traits::Indicator;

/// Absolute Breadth Index (ABI) — the absolute value of net advancing issues,
/// `|advancers - decliners|`.
///
/// The ABI ignores the *direction* of breadth and measures only its *magnitude*:
/// a high reading means the universe moved decisively one way or the other (high
/// internal activity / volatility), while a low reading means advances and
/// declines were nearly balanced (a quiet, directionless market). It is sometimes
/// called a "market thermometer" because elevated readings often cluster around
/// turning points.
///
/// `Input = CrossSection`, `Output = f64`, `warmup_period == 1`.
///
/// # Example
///
/// ```
/// use wickra_core::{AbsoluteBreadthIndex, CrossSection, Indicator, Member};
///
/// let mut abi = AbsoluteBreadthIndex::new();
/// // 2 advancers, 5 decliners -> |2 - 5| = 3.
/// let tick = CrossSection::new(
///     vec![
///         Member::new(1.0, 10.0, false, false),
///         Member::new(1.0, 10.0, false, false),
///         Member::new(-1.0, 10.0, false, false),
///         Member::new(-1.0, 10.0, false, false),
///         Member::new(-1.0, 10.0, false, false),
///         Member::new(-1.0, 10.0, false, false),
///         Member::new(-1.0, 10.0, false, false),
///     ],
///     0,
/// )
/// .unwrap();
/// assert_eq!(abi.update(tick), Some(3.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct AbsoluteBreadthIndex {
    has_emitted: bool,
}

impl AbsoluteBreadthIndex {
    /// Construct a new Absolute Breadth Index indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for AbsoluteBreadthIndex {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        let net = section.advancers() as f64 - section.decliners() as f64;
        self.has_emitted = true;
        Some(net.abs())
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
        "AbsoluteBreadthIndex"
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
        members.push(Member::new(0.0, 10.0, false, false));
        CrossSection::new(members, 0).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let abi = AbsoluteBreadthIndex::new();
        assert_eq!(abi.name(), "AbsoluteBreadthIndex");
        assert_eq!(abi.warmup_period(), 1);
        assert!(!abi.is_ready());
    }

    #[test]
    fn magnitude_ignores_direction() {
        let mut abi = AbsoluteBreadthIndex::new();
        assert_eq!(abi.update(section(2, 5)), Some(3.0));
        // Same magnitude with the direction reversed.
        let mut abi2 = AbsoluteBreadthIndex::new();
        assert_eq!(abi2.update(section(5, 2)), Some(3.0));
    }

    #[test]
    fn balanced_universe_yields_zero() {
        let mut abi = AbsoluteBreadthIndex::new();
        assert_eq!(abi.update(section(3, 3)), Some(0.0));
        assert!(abi.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut abi = AbsoluteBreadthIndex::new();
        abi.update(section(2, 5));
        assert!(abi.is_ready());
        abi.reset();
        assert!(!abi.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![section(2, 5), section(5, 2), section(3, 3)];
        let mut a = AbsoluteBreadthIndex::new();
        let mut b = AbsoluteBreadthIndex::new();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
