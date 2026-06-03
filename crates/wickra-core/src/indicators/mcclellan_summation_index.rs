//! McClellan Summation Index — the running total of the McClellan Oscillator.

use crate::cross_section::CrossSection;
use crate::indicators::mcclellan_oscillator::McClellanOscillator;
use crate::traits::Indicator;

/// McClellan Summation Index — the running cumulative sum of the
/// [`McClellanOscillator`].
///
/// Where the oscillator measures the *momentum* of breadth, the summation index
/// integrates it into a longer-term breadth trend: it rises while the oscillator
/// is positive and falls while it is negative, so it behaves like a slow,
/// smoothed advance/decline line. Sustained readings far above or below zero mark
/// strong bull or bear breadth regimes, and crosses of the zero line are read as
/// major trend changes.
///
/// The index embeds a [`McClellanOscillator`] and adds its value on every tick.
/// Because the oscillator seeds to `0.0` on the first tick, the summation index
/// also starts at `0.0` and is defined from the first update
/// (`warmup_period == 1`).
///
/// `Input = CrossSection`, `Output = f64`.
///
/// # Example
///
/// ```
/// use wickra_core::{CrossSection, Indicator, McClellanSummationIndex, Member};
///
/// let mut msi = McClellanSummationIndex::new();
/// let tick = CrossSection::new(
///     vec![
///         Member::new(1.0, 10.0, false, false),
///         Member::new(-1.0, 10.0, false, false),
///     ],
///     0,
/// )
/// .unwrap();
/// // First tick: oscillator seeds to 0, so the summation index is 0.
/// assert_eq!(msi.update(tick), Some(0.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct McClellanSummationIndex {
    oscillator: McClellanOscillator,
    sum: f64,
    has_emitted: bool,
}

impl McClellanSummationIndex {
    /// Construct a new McClellan Summation Index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            oscillator: McClellanOscillator::new(),
            sum: 0.0,
            has_emitted: false,
        }
    }
}

impl Indicator for McClellanSummationIndex {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        let oscillator = self.oscillator.step(&section);
        self.sum += oscillator;
        self.has_emitted = true;
        Some(self.sum)
    }

    fn reset(&mut self) {
        self.oscillator.reset();
        self.sum = 0.0;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "McClellanSummationIndex"
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
        let msi = McClellanSummationIndex::new();
        assert_eq!(msi.name(), "McClellanSummationIndex");
        assert_eq!(msi.warmup_period(), 1);
        assert!(!msi.is_ready());
    }

    #[test]
    fn first_tick_starts_at_zero() {
        let mut msi = McClellanSummationIndex::new();
        assert_eq!(msi.update(section(3, 1)), Some(0.0));
        assert!(msi.is_ready());
    }

    #[test]
    fn accumulates_the_oscillator() {
        let mut msi = McClellanSummationIndex::new();
        assert_eq!(msi.update(section(3, 1)), Some(0.0)); // osc 0 -> sum 0
                                                          // osc -50 -> sum -50.
        let value = msi.update(section(1, 3)).unwrap();
        assert!((value - (-50.0)).abs() < 1e-9);
        // osc -67.5 -> sum -117.5.
        let value = msi.update(section(2, 2)).unwrap();
        assert!((value - (-117.5)).abs() < 1e-9);
    }

    #[test]
    fn reset_clears_state() {
        let mut msi = McClellanSummationIndex::new();
        msi.update(section(3, 1));
        msi.update(section(1, 3));
        assert!(msi.is_ready());
        msi.reset();
        assert!(!msi.is_ready());
        // Oscillator re-seeds, so the summation index restarts at 0.
        assert_eq!(msi.update(section(1, 3)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![section(3, 1), section(1, 3), section(2, 2)];
        let mut a = McClellanSummationIndex::new();
        let mut b = McClellanSummationIndex::new();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
