//! Percent Above Moving Average — share of a universe trading above its MA.

use crate::cross_section::CrossSection;
use crate::traits::Indicator;

/// Percent Above Moving Average — the percentage of symbols in a universe that
/// are trading above their reference moving average.
///
/// On each [`CrossSection`] tick the value is `100 * above_ma_count / universe
/// size`, read from the per-symbol `above_ma` flag (the caller decides which MA —
/// 50-day, 200-day — when it builds the tick). It is a bounded `0..=100` breadth
/// gauge: readings near 100 mean almost the whole universe is in an uptrend
/// (broad participation, but also a potential overbought extreme), readings near
/// zero mark washouts. Crosses of the 50 line are read as bull/bear regime flips.
///
/// `Input = CrossSection`, `Output = f64` (a percentage in `0..=100`),
/// `warmup_period == 1`. The universe is non-empty by construction, so the share
/// is always defined.
///
/// # Example
///
/// ```
/// use wickra_core::{CrossSection, Indicator, Member, PercentAboveMa};
///
/// let mut pct = PercentAboveMa::new();
/// // 3 of 4 symbols above their MA -> 75%.
/// let tick = CrossSection::new(
///     vec![
///         Member::with_signals(1.0, 10.0, false, false, true, false),
///         Member::with_signals(1.0, 10.0, false, false, true, false),
///         Member::with_signals(-1.0, 10.0, false, false, true, false),
///         Member::with_signals(-1.0, 10.0, false, false, false, false),
///     ],
///     0,
/// )
/// .unwrap();
/// assert_eq!(pct.update(tick), Some(75.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct PercentAboveMa {
    has_emitted: bool,
}

impl PercentAboveMa {
    /// Construct a new Percent Above Moving Average indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for PercentAboveMa {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        let above = section.above_ma_count() as f64;
        let total = section.members.len() as f64;
        self.has_emitted = true;
        Some(100.0 * above / total)
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
        "PercentAboveMa"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_section::Member;
    use crate::traits::BatchExt;

    fn tick(above: usize, below: usize) -> CrossSection {
        let mut members = Vec::new();
        for _ in 0..above {
            members.push(Member::with_signals(1.0, 10.0, false, false, true, false));
        }
        for _ in 0..below {
            members.push(Member::with_signals(-1.0, 10.0, false, false, false, false));
        }
        CrossSection::new(members, 0).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let pct = PercentAboveMa::new();
        assert_eq!(pct.name(), "PercentAboveMa");
        assert_eq!(pct.warmup_period(), 1);
        assert!(!pct.is_ready());
    }

    #[test]
    fn first_tick_emits_percentage() {
        let mut pct = PercentAboveMa::new();
        assert_eq!(pct.update(tick(3, 1)), Some(75.0));
        assert!(pct.is_ready());
    }

    #[test]
    fn all_above_is_one_hundred() {
        let mut pct = PercentAboveMa::new();
        assert_eq!(pct.update(tick(4, 0)), Some(100.0));
    }

    #[test]
    fn none_above_is_zero() {
        let mut pct = PercentAboveMa::new();
        assert_eq!(pct.update(tick(0, 5)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut pct = PercentAboveMa::new();
        pct.update(tick(3, 1));
        assert!(pct.is_ready());
        pct.reset();
        assert!(!pct.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![tick(3, 1), tick(4, 0), tick(0, 5)];
        let mut a = PercentAboveMa::new();
        let mut b = PercentAboveMa::new();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
