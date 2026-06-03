//! Bullish Percent Index — share of a universe on a point-and-figure buy signal.

use crate::cross_section::CrossSection;
use crate::traits::Indicator;

/// Bullish Percent Index (BPI) — the percentage of symbols in a universe that are
/// currently on a point-and-figure buy signal.
///
/// On each [`CrossSection`] tick the value is `100 * on_buy_signal_count /
/// universe size`, read from the per-symbol `on_buy_signal` flag (the caller
/// evaluates each symbol's point-and-figure chart when it builds the tick). It is
/// a bounded `0..=100` gauge of how many issues are in a confirmed uptrend.
/// Readings above 70 are considered overbought (broad strength, but a crowded
/// market) and below 30 oversold; reversals from those zones are classic BPI
/// buy/sell triggers.
///
/// `Input = CrossSection`, `Output = f64` (a percentage in `0..=100`),
/// `warmup_period == 1`. The universe is non-empty by construction, so the share
/// is always defined.
///
/// # Example
///
/// ```
/// use wickra_core::{BullishPercentIndex, CrossSection, Indicator, Member};
///
/// let mut bpi = BullishPercentIndex::new();
/// // 2 of 4 symbols on a buy signal -> 50%.
/// let tick = CrossSection::new(
///     vec![
///         Member::with_signals(1.0, 10.0, false, false, false, true),
///         Member::with_signals(1.0, 10.0, false, false, false, true),
///         Member::with_signals(-1.0, 10.0, false, false, false, false),
///         Member::with_signals(-1.0, 10.0, false, false, false, false),
///     ],
///     0,
/// )
/// .unwrap();
/// assert_eq!(bpi.update(tick), Some(50.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct BullishPercentIndex {
    has_emitted: bool,
}

impl BullishPercentIndex {
    /// Construct a new Bullish Percent Index indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for BullishPercentIndex {
    type Input = CrossSection;
    type Output = f64;

    fn update(&mut self, section: CrossSection) -> Option<f64> {
        let bullish = section.on_buy_signal_count() as f64;
        let total = section.members.len() as f64;
        self.has_emitted = true;
        Some(100.0 * bullish / total)
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
        "BullishPercentIndex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_section::Member;
    use crate::traits::BatchExt;

    fn tick(bullish: usize, bearish: usize) -> CrossSection {
        let mut members = Vec::new();
        for _ in 0..bullish {
            members.push(Member::with_signals(1.0, 10.0, false, false, false, true));
        }
        for _ in 0..bearish {
            members.push(Member::with_signals(-1.0, 10.0, false, false, false, false));
        }
        CrossSection::new(members, 0).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let bpi = BullishPercentIndex::new();
        assert_eq!(bpi.name(), "BullishPercentIndex");
        assert_eq!(bpi.warmup_period(), 1);
        assert!(!bpi.is_ready());
    }

    #[test]
    fn first_tick_emits_percentage() {
        let mut bpi = BullishPercentIndex::new();
        assert_eq!(bpi.update(tick(2, 2)), Some(50.0));
        assert!(bpi.is_ready());
    }

    #[test]
    fn all_bullish_is_one_hundred() {
        let mut bpi = BullishPercentIndex::new();
        assert_eq!(bpi.update(tick(5, 0)), Some(100.0));
    }

    #[test]
    fn none_bullish_is_zero() {
        let mut bpi = BullishPercentIndex::new();
        assert_eq!(bpi.update(tick(0, 4)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut bpi = BullishPercentIndex::new();
        bpi.update(tick(2, 2));
        assert!(bpi.is_ready());
        bpi.reset();
        assert!(!bpi.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let sections = vec![tick(2, 2), tick(5, 0), tick(0, 4)];
        let mut a = BullishPercentIndex::new();
        let mut b = BullishPercentIndex::new();
        assert_eq!(
            a.batch(&sections),
            sections
                .iter()
                .map(|s| b.update(s.clone()))
                .collect::<Vec<_>>()
        );
    }
}
