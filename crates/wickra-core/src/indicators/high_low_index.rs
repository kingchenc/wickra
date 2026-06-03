//! High-Low Index — a moving average of the record-high percentage.

use crate::cross_section::CrossSection;
use crate::error::Result;
use crate::traits::Indicator;
use crate::Sma;

/// High-Low Index — a simple moving average of the *record high percent*,
/// `100 * new_highs / (new_highs + new_lows)`.
///
/// The record high percent is the share of new-extreme issues that are new
/// *highs* rather than new *lows*; smoothing it over a window (the classic period
/// is 10) gives the High-Low Index. Readings above 50 mean new highs dominate
/// (a healthy, broadening trend), readings below 50 mean new lows dominate. The
/// 30 and 70 lines are watched as oversold / overbought breadth thresholds.
///
/// Each tick floors the new-extreme count to one, so a tick with no new highs or
/// lows contributes a defined `0.0` instead of dividing by zero. The reading is
/// `None` until `period` ticks have been seen.
///
/// `Input = CrossSection`, `Output = f64` (a percentage in `0..=100`),
/// `warmup_period == period`.
///
/// # Example
///
/// ```
/// use wickra_core::{CrossSection, HighLowIndex, Indicator, Member};
///
/// let mut hli = HighLowIndex::new(2).unwrap();
/// let highs = CrossSection::new(vec![Member::new(1.0, 1.0, true, false)], 0).unwrap();
/// assert_eq!(hli.update(highs.clone()), None); // warming up
/// assert_eq!(hli.update(highs), Some(100.0)); // all new highs
/// ```
#[derive(Debug, Clone)]
pub struct HighLowIndex {
    sma: Sma,
}

impl HighLowIndex {
    /// Construct a new High-Low Index over the given window length.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`](crate::Error::PeriodZero) if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            sma: Sma::new(period)?,
        })
    }

    /// Configured window length.
    #[must_use]
    pub const fn period(&self) -> usize {
        self.sma.period()
    }
}

impl Indicator for HighLowIndex {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        let new_highs = section.new_highs();
        let new_lows = section.new_lows();
        let extremes = (new_highs + new_lows).max(1) as f64;
        let record_high_percent = 100.0 * new_highs as f64 / extremes;
        self.sma.update(record_high_percent)
    }

    fn reset(&mut self) {
        self.sma.reset();
    }

    fn warmup_period(&self) -> usize {
        self.sma.period()
    }

    fn is_ready(&self) -> bool {
        self.sma.value().is_some()
    }

    fn name(&self) -> &'static str {
        "HighLowIndex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_section::Member;
    use crate::error::Error;
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
        let hli = HighLowIndex::new(10).unwrap();
        assert_eq!(hli.name(), "HighLowIndex");
        assert_eq!(hli.warmup_period(), 10);
        assert_eq!(hli.period(), 10);
        assert!(!hli.is_ready());
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(HighLowIndex::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn averages_the_record_high_percent() {
        let mut hli = HighLowIndex::new(2).unwrap();
        // 8 highs / 10 extremes -> 80% ; window not full.
        assert_eq!(hli.update(flags(8, 2)), None);
        // 6 highs / 10 extremes -> 60% ; SMA(2) = (80 + 60) / 2 = 70.
        let value = hli.update(flags(6, 4)).unwrap();
        assert!((value - 70.0).abs() < 1e-9);
        assert!(hli.is_ready());
    }

    #[test]
    fn no_extremes_floors_to_zero_percent() {
        let mut hli = HighLowIndex::new(1).unwrap();
        // No new highs or lows -> 0 / max(0, 1) -> 0%.
        assert_eq!(hli.update(flags(0, 0)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut hli = HighLowIndex::new(2).unwrap();
        hli.update(flags(8, 2));
        hli.update(flags(6, 4));
        assert!(hli.is_ready());
        hli.reset();
        assert!(!hli.is_ready());
        assert_eq!(hli.update(flags(8, 2)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![flags(8, 2), flags(6, 4), flags(3, 7), flags(0, 0)];
        let mut a = HighLowIndex::new(2).unwrap();
        let mut b = HighLowIndex::new(2).unwrap();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
