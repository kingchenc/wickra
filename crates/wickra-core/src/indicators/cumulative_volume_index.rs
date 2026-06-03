//! Cumulative Volume Index — running total of volume-normalised net advancing volume.

use crate::cross_section::CrossSection;
use crate::traits::Indicator;

/// Cumulative Volume Index (CVI) — the running total of *volume-normalised* net
/// advancing volume across a universe.
///
/// On each [`CrossSection`] tick the increment is `(advancing volume - declining
/// volume) / total volume`: the share of the tick's total volume that flowed,
/// net, into advancing issues. The index accumulates this share over time. Where
/// the raw [`AdVolumeLine`](crate::AdVolumeLine) sums *absolute* net volume — and
/// so drifts with secular growth in trading activity — the CVI normalises each
/// tick by its own total volume, so a one-share-net day in a thin market counts
/// the same as in a heavy one. This keeps the index comparable across regimes of
/// very different volume.
///
/// When a tick has zero total volume the net is necessarily zero too, so the
/// increment is zero and the index is unchanged (the divisor is floored to the
/// smallest positive `f64` purely to keep the division defined).
///
/// `Input = CrossSection`, `Output = f64`, `warmup_period == 1`.
///
/// # Example
///
/// ```
/// use wickra_core::{CrossSection, CumulativeVolumeIndex, Indicator, Member};
///
/// let mut cvi = CumulativeVolumeIndex::new();
/// // adv vol 150, dec vol 50, total 200 -> (150 - 50) / 200 = 0.5.
/// let tick = CrossSection::new(
///     vec![
///         Member::new(1.0, 150.0, false, false),
///         Member::new(-1.0, 50.0, false, false),
///     ],
///     0,
/// )
/// .unwrap();
/// assert_eq!(cvi.update(tick), Some(0.5));
/// ```
#[derive(Debug, Clone, Default)]
pub struct CumulativeVolumeIndex {
    index: f64,
    has_emitted: bool,
}

impl CumulativeVolumeIndex {
    /// Construct a new Cumulative Volume Index indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            index: 0.0,
            has_emitted: false,
        }
    }
}

impl Indicator for CumulativeVolumeIndex {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        let net = section.advancing_volume() - section.declining_volume();
        let total = section.total_volume().max(f64::MIN_POSITIVE);
        self.index += net / total;
        self.has_emitted = true;
        Some(self.index)
    }

    fn reset(&mut self) {
        self.index = 0.0;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "CumulativeVolumeIndex"
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
        let cvi = CumulativeVolumeIndex::new();
        assert_eq!(cvi.name(), "CumulativeVolumeIndex");
        assert_eq!(cvi.warmup_period(), 1);
        assert!(!cvi.is_ready());
    }

    #[test]
    fn first_tick_emits_normalised_net() {
        let mut cvi = CumulativeVolumeIndex::new();
        assert_eq!(cvi.update(tick(&[(1.0, 150.0), (-1.0, 50.0)])), Some(0.5));
        assert!(cvi.is_ready());
    }

    #[test]
    fn index_accumulates_normalised_shares() {
        let mut cvi = CumulativeVolumeIndex::new();
        assert_eq!(cvi.update(tick(&[(1.0, 150.0), (-1.0, 50.0)])), Some(0.5));
        // adv 60, dec 60, total 120 -> net 0 -> index unchanged.
        assert_eq!(cvi.update(tick(&[(1.0, 60.0), (-1.0, 60.0)])), Some(0.5));
    }

    #[test]
    fn zero_total_volume_leaves_index_unchanged() {
        let mut cvi = CumulativeVolumeIndex::new();
        cvi.update(tick(&[(1.0, 150.0), (-1.0, 50.0)]));
        // A tick with no volume at all: net 0 / floored divisor -> 0 increment.
        assert_eq!(cvi.update(tick(&[(0.0, 0.0)])), Some(0.5));
    }

    #[test]
    fn reset_clears_state() {
        let mut cvi = CumulativeVolumeIndex::new();
        cvi.update(tick(&[(1.0, 150.0), (-1.0, 50.0)]));
        assert!(cvi.is_ready());
        cvi.reset();
        assert!(!cvi.is_ready());
        assert_eq!(cvi.update(tick(&[(1.0, 100.0)])), Some(1.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![
            tick(&[(1.0, 150.0), (-1.0, 50.0)]),
            tick(&[(1.0, 60.0), (-1.0, 60.0)]),
            tick(&[(0.0, 0.0)]),
        ];
        let mut a = CumulativeVolumeIndex::new();
        let mut b = CumulativeVolumeIndex::new();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
