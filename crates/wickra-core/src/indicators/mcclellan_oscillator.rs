//! McClellan Oscillator — the spread between a fast and slow EMA of breadth.

use crate::cross_section::CrossSection;
use crate::traits::Indicator;

/// Fast EMA smoothing constant — the classic McClellan 19-period weight
/// `2 / (19 + 1)`.
const ALPHA_FAST: f64 = 0.1;
/// Slow EMA smoothing constant — the classic McClellan 39-period weight
/// `2 / (39 + 1)`.
const ALPHA_SLOW: f64 = 0.05;
/// Scale applied to the ratio-adjusted net advances so readings land on the
/// familiar McClellan amplitude.
const RANA_SCALE: f64 = 1000.0;

/// McClellan Oscillator — the difference between a 19-period and a 39-period
/// exponential moving average of *ratio-adjusted net advances*.
///
/// Each tick's breadth is reduced to ratio-adjusted net advances (RANA),
/// `(advancers - decliners) / (advancers + decliners) * 1000`. Dividing by the
/// number of participating issues makes the reading independent of universe size,
/// so the oscillator stays comparable as the universe grows or shrinks. The
/// oscillator is then the fast EMA minus the slow EMA of that series, using the
/// classic McClellan smoothing constants `0.10` (19-period) and `0.05`
/// (39-period). Both EMAs are seeded from the first tick's RANA, so the
/// oscillator is defined from the first update (`warmup_period == 1`); it starts
/// at `0.0` and crosses zero as breadth momentum shifts.
///
/// A tick with no advancing or declining issues yields a RANA of `0.0` (the
/// participating count is floored to one).
///
/// `Input = CrossSection`, `Output = f64`.
///
/// # Example
///
/// ```
/// use wickra_core::{CrossSection, Indicator, McClellanOscillator, Member};
///
/// let mut osc = McClellanOscillator::new();
/// let tick = CrossSection::new(
///     vec![
///         Member::new(1.0, 10.0, false, false),
///         Member::new(1.0, 10.0, false, false),
///         Member::new(1.0, 10.0, false, false),
///         Member::new(-1.0, 10.0, false, false),
///     ],
///     0,
/// )
/// .unwrap();
/// // First tick seeds both EMAs to the same value -> oscillator 0.
/// assert_eq!(osc.update(tick), Some(0.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct McClellanOscillator {
    ema_fast: f64,
    ema_slow: f64,
    seeded: bool,
    has_emitted: bool,
}

impl McClellanOscillator {
    /// Construct a new McClellan Oscillator with the classic 19/39 smoothing.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ema_fast: 0.0,
            ema_slow: 0.0,
            seeded: false,
            has_emitted: false,
        }
    }

    /// Feed a cross-section tick and return the oscillator value, which is defined
    /// on every tick. Shared with [`McClellanSummationIndex`] so the summation
    /// index can accumulate the oscillator without an `Option` round-trip.
    ///
    /// [`McClellanSummationIndex`]: crate::McClellanSummationIndex
    pub(crate) fn step(&mut self, section: &CrossSection) -> f64 {
        let advancers = section.advancers();
        let decliners = section.decliners();
        let net = advancers as f64 - decliners as f64;
        let participating = (advancers + decliners).max(1) as f64;
        let rana = net / participating * RANA_SCALE;
        if self.seeded {
            self.ema_fast += ALPHA_FAST * (rana - self.ema_fast);
            self.ema_slow += ALPHA_SLOW * (rana - self.ema_slow);
        } else {
            self.ema_fast = rana;
            self.ema_slow = rana;
            self.seeded = true;
        }
        self.has_emitted = true;
        self.ema_fast - self.ema_slow
    }
}

impl Indicator for McClellanOscillator {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        Some(self.step(&section))
    }

    fn reset(&mut self) {
        self.ema_fast = 0.0;
        self.ema_slow = 0.0;
        self.seeded = false;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "McClellanOscillator"
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
        let osc = McClellanOscillator::new();
        assert_eq!(osc.name(), "McClellanOscillator");
        assert_eq!(osc.warmup_period(), 1);
        assert!(!osc.is_ready());
    }

    #[test]
    fn seeds_to_zero_on_first_tick() {
        let mut osc = McClellanOscillator::new();
        // RANA = (3 - 1) / 4 * 1000 = 500 ; both EMAs seed to 500 -> spread 0.
        assert_eq!(osc.update(section(3, 1)), Some(0.0));
        assert!(osc.is_ready());
    }

    #[test]
    fn tracks_breadth_momentum_after_seeding() {
        let mut osc = McClellanOscillator::new();
        osc.update(section(3, 1)); // seed at RANA 500
                                   // RANA = (1 - 3) / 4 * 1000 = -500.
                                   // fast = 500 + 0.1 * (-1000) = 400 ; slow = 500 + 0.05 * (-1000) = 450.
        let value = osc.update(section(1, 3)).unwrap();
        assert!((value - (-50.0)).abs() < 1e-9);
        // RANA = 0. fast = 400 + 0.1 * (-400) = 360 ; slow = 450 + 0.05 * (-450) = 427.5.
        let value = osc.update(section(2, 2)).unwrap();
        assert!((value - (-67.5)).abs() < 1e-9);
    }

    #[test]
    fn empty_participation_yields_zero_rana() {
        let mut osc = McClellanOscillator::new();
        // No advancers or decliners -> RANA 0 ; seeds both EMAs to 0 -> spread 0.
        assert_eq!(osc.update(section(0, 0)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut osc = McClellanOscillator::new();
        osc.update(section(3, 1));
        osc.update(section(1, 3));
        assert!(osc.is_ready());
        osc.reset();
        assert!(!osc.is_ready());
        // After reset the next tick re-seeds to spread 0.
        assert_eq!(osc.update(section(1, 3)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![section(3, 1), section(1, 3), section(2, 2), section(0, 0)];
        let mut a = McClellanOscillator::new();
        let mut b = McClellanOscillator::new();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
