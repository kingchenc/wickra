//! New Highs − New Lows — net count of fresh period extremes across a universe.

use crate::cross_section::CrossSection;
use crate::traits::Indicator;

/// New Highs − New Lows — the number of symbols printing a new period high minus
/// the number printing a new period low across a universe.
///
/// On each [`CrossSection`] tick the value is `new_highs - new_lows`, read from the
/// per-symbol `new_high` / `new_low` flags. A persistently positive reading means
/// fresh leadership is broad (many names making new highs); a negative reading
/// during an index advance is a classic breadth divergence warning that the rally
/// is narrowing.
///
/// `Input = CrossSection`, `Output = f64`, `warmup_period == 1`.
///
/// # Example
///
/// ```
/// use wickra_core::{CrossSection, Indicator, Member, NewHighsNewLows};
///
/// let mut nhnl = NewHighsNewLows::new();
/// // 2 new highs, 1 new low -> net +1.
/// let tick = CrossSection::new(
///     vec![
///         Member::new(1.0, 10.0, true, false),
///         Member::new(1.0, 10.0, true, false),
///         Member::new(-1.0, 10.0, false, true),
///     ],
///     0,
/// )
/// .unwrap();
/// assert_eq!(nhnl.update(tick), Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct NewHighsNewLows {
    has_emitted: bool,
}

impl NewHighsNewLows {
    /// Construct a new New Highs − New Lows indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for NewHighsNewLows {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        let net = section.new_highs() as f64 - section.new_lows() as f64;
        self.has_emitted = true;
        Some(net)
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
        "NewHighsNewLows"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_section::Member;
    use crate::traits::BatchExt;

    fn flags(highs: usize, lows: usize) -> CrossSection {
        let mut members = Vec::new();
        for _ in 0..highs {
            members.push(Member::new(1.0, 10.0, true, false));
        }
        for _ in 0..lows {
            members.push(Member::new(-1.0, 10.0, false, true));
        }
        members.push(Member::new(0.0, 10.0, false, false));
        CrossSection::new(members, 0).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let nhnl = NewHighsNewLows::new();
        assert_eq!(nhnl.name(), "NewHighsNewLows");
        assert_eq!(nhnl.warmup_period(), 1);
        assert!(!nhnl.is_ready());
    }

    #[test]
    fn first_tick_emits_net_extremes() {
        let mut nhnl = NewHighsNewLows::new();
        assert_eq!(nhnl.update(flags(5, 2)), Some(3.0));
        assert!(nhnl.is_ready());
    }

    #[test]
    fn more_lows_than_highs_is_negative() {
        let mut nhnl = NewHighsNewLows::new();
        assert_eq!(nhnl.update(flags(1, 4)), Some(-3.0));
    }

    #[test]
    fn no_extremes_yields_zero() {
        let mut nhnl = NewHighsNewLows::new();
        assert_eq!(nhnl.update(flags(0, 0)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut nhnl = NewHighsNewLows::new();
        nhnl.update(flags(3, 1));
        assert!(nhnl.is_ready());
        nhnl.reset();
        assert!(!nhnl.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![flags(5, 2), flags(1, 4), flags(0, 0)];
        let mut a = NewHighsNewLows::new();
        let mut b = NewHighsNewLows::new();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
