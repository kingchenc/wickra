//! Advance/Decline Volume Line — cumulative net advancing-minus-declining volume.

use crate::cross_section::CrossSection;
use crate::traits::Indicator;

/// Advance/Decline Volume Line (AD Volume Line) — the running cumulative sum of
/// net advancing volume across a universe.
///
/// On each [`CrossSection`] tick the net is `advancing volume - declining volume`,
/// where advancing volume is the total volume of symbols with a positive change
/// and declining volume the total volume of symbols with a negative change. The
/// line accumulates this net over time, so a rising line means volume is flowing
/// into advancing issues (healthy participation) while a falling line warns that
/// declining issues are carrying the volume — the volume-weighted analogue of the
/// plain Advance/Decline Line.
///
/// `Input = CrossSection`, `Output = f64`, `warmup_period == 1` (defined from the
/// first tick).
///
/// # Example
///
/// ```
/// use wickra_core::{AdVolumeLine, CrossSection, Indicator, Member};
///
/// let mut adv = AdVolumeLine::new();
/// // advancing volume 150, declining volume 50 -> net +100.
/// let tick = CrossSection::new(
///     vec![
///         Member::new(1.0, 150.0, false, false),
///         Member::new(-1.0, 50.0, false, false),
///     ],
///     0,
/// )
/// .unwrap();
/// assert_eq!(adv.update(tick), Some(100.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct AdVolumeLine {
    line: f64,
    has_emitted: bool,
}

impl AdVolumeLine {
    /// Construct a new Advance/Decline Volume Line indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            line: 0.0,
            has_emitted: false,
        }
    }
}

impl Indicator for AdVolumeLine {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        let net = section.advancing_volume() - section.declining_volume();
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
        "AdVolumeLine"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_section::Member;
    use crate::traits::BatchExt;

    fn tick(items: &[(f64, f64)]) -> CrossSection {
        CrossSection::new(
            items
                .iter()
                .map(|&(change, volume)| Member::new(change, volume, false, false))
                .collect(),
            0,
        )
        .unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let adv = AdVolumeLine::new();
        assert_eq!(adv.name(), "AdVolumeLine");
        assert_eq!(adv.warmup_period(), 1);
        assert!(!adv.is_ready());
    }

    #[test]
    fn first_tick_emits_net_volume() {
        let mut adv = AdVolumeLine::new();
        assert_eq!(adv.update(tick(&[(1.0, 150.0), (-1.0, 50.0)])), Some(100.0));
        assert!(adv.is_ready());
    }

    #[test]
    fn line_accumulates_across_ticks() {
        let mut adv = AdVolumeLine::new();
        assert_eq!(adv.update(tick(&[(1.0, 150.0), (-1.0, 50.0)])), Some(100.0));
        assert_eq!(adv.update(tick(&[(1.0, 60.0), (-1.0, 60.0)])), Some(100.0));
        assert_eq!(adv.update(tick(&[(1.0, 30.0)])), Some(130.0));
    }

    #[test]
    fn unchanged_volume_is_ignored() {
        let mut adv = AdVolumeLine::new();
        // Unchanged symbols (zero change) contribute to neither bucket.
        assert_eq!(adv.update(tick(&[(0.0, 1000.0), (1.0, 10.0)])), Some(10.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut adv = AdVolumeLine::new();
        adv.update(tick(&[(1.0, 100.0)]));
        assert!(adv.is_ready());
        adv.reset();
        assert!(!adv.is_ready());
        assert_eq!(adv.update(tick(&[(1.0, 20.0)])), Some(20.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![
            tick(&[(1.0, 150.0), (-1.0, 50.0)]),
            tick(&[(1.0, 60.0), (-1.0, 60.0)]),
            tick(&[(1.0, 30.0)]),
        ];
        let mut a = AdVolumeLine::new();
        let mut b = AdVolumeLine::new();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
