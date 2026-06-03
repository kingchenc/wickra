//! Up/Down Volume Ratio — advancing volume divided by declining volume.

use crate::cross_section::CrossSection;
use crate::traits::Indicator;

/// Up/Down Volume Ratio — total advancing volume divided by total declining
/// volume across a universe.
///
/// On each [`CrossSection`] tick the ratio is `advancing volume / declining
/// volume`. A reading above one means more volume is trading in advancing issues
/// than declining ones (accumulation); a reading below one means distribution.
/// Sustained extremes are used to flag breadth thrusts and washout bottoms.
///
/// When a tick has no declining volume the denominator is floored to `1.0`, so the
/// ratio stays finite (it degrades to the advancing-volume total) instead of
/// dividing by zero.
///
/// `Input = CrossSection`, `Output = f64`, `warmup_period == 1`.
///
/// # Example
///
/// ```
/// use wickra_core::{CrossSection, Indicator, Member, UpDownVolumeRatio};
///
/// let mut udv = UpDownVolumeRatio::new();
/// // advancing volume 150, declining volume 50 -> ratio 3.0.
/// let tick = CrossSection::new(
///     vec![
///         Member::new(1.0, 150.0, false, false),
///         Member::new(-1.0, 50.0, false, false),
///     ],
///     0,
/// )
/// .unwrap();
/// assert_eq!(udv.update(tick), Some(3.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct UpDownVolumeRatio {
    has_emitted: bool,
}

impl UpDownVolumeRatio {
    /// Construct a new Up/Down Volume Ratio indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for UpDownVolumeRatio {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        let advancing_volume = section.advancing_volume();
        let declining_volume = section.declining_volume().max(1.0);
        self.has_emitted = true;
        Some(advancing_volume / declining_volume)
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
        "UpDownVolumeRatio"
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
        let udv = UpDownVolumeRatio::new();
        assert_eq!(udv.name(), "UpDownVolumeRatio");
        assert_eq!(udv.warmup_period(), 1);
        assert!(!udv.is_ready());
    }

    #[test]
    fn first_tick_emits_ratio() {
        let mut udv = UpDownVolumeRatio::new();
        assert_eq!(udv.update(tick(&[(1.0, 150.0), (-1.0, 50.0)])), Some(3.0));
        assert!(udv.is_ready());
    }

    #[test]
    fn zero_declining_volume_floors_denominator() {
        let mut udv = UpDownVolumeRatio::new();
        // advancing volume 100, declining volume 0 -> 100 / max(0, 1) = 100.0.
        assert_eq!(udv.update(tick(&[(1.0, 100.0)])), Some(100.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut udv = UpDownVolumeRatio::new();
        udv.update(tick(&[(1.0, 10.0), (-1.0, 10.0)]));
        assert!(udv.is_ready());
        udv.reset();
        assert!(!udv.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![
            tick(&[(1.0, 150.0), (-1.0, 50.0)]),
            tick(&[(1.0, 100.0)]),
            tick(&[(1.0, 20.0), (-1.0, 80.0)]),
        ];
        let mut a = UpDownVolumeRatio::new();
        let mut b = UpDownVolumeRatio::new();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
