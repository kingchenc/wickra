//! TRIN / Arms Index — the advance-decline ratio over the up-down volume ratio.

use crate::cross_section::CrossSection;
use crate::traits::Indicator;

/// TRIN (Arms Index) — `(advancers / decliners) / (advancing volume / declining
/// volume)`.
///
/// The TRIN compares the breadth of a move in *issues* to the breadth of the move
/// in *volume*. A value near `1.0` means advancing issues and advancing volume are
/// in balance; a value below `1.0` is bullish (volume is concentrated in advancing
/// issues relative to their count); a value above `1.0` is bearish (declining
/// issues are absorbing disproportionate volume).
///
/// To stay finite on degenerate ticks the decliner count is floored to one and
/// both volume sums are floored to `1.0`, so a tick with no declining issues or no
/// volume on one side still yields a defined reading instead of a division by
/// zero.
///
/// `Input = CrossSection`, `Output = f64`, `warmup_period == 1`.
///
/// # Example
///
/// ```
/// use wickra_core::{CrossSection, Indicator, Member, Trin};
///
/// let mut trin = Trin::new();
/// // 3 advancers / 1 decliner = 3; adv vol 150 / dec vol 50 = 3; TRIN = 1.0.
/// let tick = CrossSection::new(
///     vec![
///         Member::new(1.0, 50.0, false, false),
///         Member::new(1.0, 50.0, false, false),
///         Member::new(1.0, 50.0, false, false),
///         Member::new(-1.0, 50.0, false, false),
///     ],
///     0,
/// )
/// .unwrap();
/// assert_eq!(trin.update(tick), Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Trin {
    has_emitted: bool,
}

impl Trin {
    /// Construct a new TRIN / Arms Index indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for Trin {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        let advancers = section.advancers() as f64;
        let decliners = section.decliners().max(1) as f64;
        let advancing_volume = section.advancing_volume().max(1.0);
        let declining_volume = section.declining_volume().max(1.0);
        let ad_ratio = advancers / decliners;
        let volume_ratio = advancing_volume / declining_volume;
        self.has_emitted = true;
        Some(ad_ratio / volume_ratio)
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
        "Trin"
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
        let trin = Trin::new();
        assert_eq!(trin.name(), "Trin");
        assert_eq!(trin.warmup_period(), 1);
        assert!(!trin.is_ready());
    }

    #[test]
    fn balanced_breadth_yields_one() {
        let mut trin = Trin::new();
        let value = trin
            .update(tick(&[(1.0, 50.0), (1.0, 50.0), (1.0, 50.0), (-1.0, 50.0)]))
            .unwrap();
        assert!((value - 1.0).abs() < 1e-9);
        assert!(trin.is_ready());
    }

    #[test]
    fn zero_decliners_and_volume_are_floored() {
        let mut trin = Trin::new();
        // 2 advancers, 0 decliners, adv vol 100, dec vol 0.
        // ad_ratio = 2 / max(0,1) = 2; volume_ratio = 100 / max(0,1) = 100; TRIN = 0.02.
        let value = trin.update(tick(&[(1.0, 50.0), (1.0, 50.0)])).unwrap();
        assert!((value - 0.02).abs() < 1e-9);
    }

    #[test]
    fn heavy_declining_volume_pushes_above_one() {
        let mut trin = Trin::new();
        // 2 adv / 2 dec = 1; adv vol 20 / dec vol 80 = 0.25; TRIN = 4.0.
        let value = trin
            .update(tick(&[
                (1.0, 10.0),
                (1.0, 10.0),
                (-1.0, 40.0),
                (-1.0, 40.0),
            ]))
            .unwrap();
        assert!((value - 4.0).abs() < 1e-9);
    }

    #[test]
    fn reset_clears_state() {
        let mut trin = Trin::new();
        trin.update(tick(&[(1.0, 10.0), (-1.0, 10.0)]));
        assert!(trin.is_ready());
        trin.reset();
        assert!(!trin.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![
            tick(&[(1.0, 50.0), (1.0, 50.0), (1.0, 50.0), (-1.0, 50.0)]),
            tick(&[(1.0, 50.0), (1.0, 50.0)]),
            tick(&[(1.0, 10.0), (1.0, 10.0), (-1.0, 40.0), (-1.0, 40.0)]),
        ];
        let mut a = Trin::new();
        let mut b = Trin::new();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
