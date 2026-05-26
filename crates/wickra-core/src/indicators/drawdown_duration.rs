//! Drawdown Duration — bars since the last all-time peak ("time under water").

use crate::traits::Indicator;

/// Cumulative drawdown duration in bars.
///
/// Each `update` receives one equity-curve sample. The indicator tracks the
/// **running all-time peak** seen since construction (or last `reset`) and
/// reports how many bars have elapsed since that peak was set:
///
/// ```text
/// peak_t        = max(input over [0..=t])
/// duration_t    = bars elapsed since peak_t was first set
/// ```
///
/// A new peak resets the duration to `0`. As long as the series stays under
/// water the duration grows linearly with each bar.
///
/// The indicator emits a value on every bar (no warmup beyond the first
/// input) and runs in O(1) per `update`.
///
/// # Example
///
/// ```
/// use wickra_core::{DrawdownDuration, Indicator};
///
/// let mut dd = DrawdownDuration::new();
/// assert_eq!(dd.update(100.0), Some(0));        // first bar -> new peak
/// assert_eq!(dd.update(95.0), Some(1));         // 1 bar under water
/// assert_eq!(dd.update(90.0), Some(2));         // 2 bars under water
/// assert_eq!(dd.update(110.0), Some(0));        // new peak -> reset
/// ```
#[derive(Debug, Clone, Default)]
pub struct DrawdownDuration {
    peak: f64,
    bars_under_water: u32,
    seen: bool,
}

impl DrawdownDuration {
    /// Construct a new Drawdown Duration tracker.
    pub const fn new() -> Self {
        Self {
            peak: f64::NEG_INFINITY,
            bars_under_water: 0,
            seen: false,
        }
    }

    /// Bars elapsed since the running all-time peak was set.
    pub const fn value(&self) -> Option<u32> {
        if self.seen {
            Some(self.bars_under_water)
        } else {
            None
        }
    }
}

impl Indicator for DrawdownDuration {
    type Input = f64;
    type Output = u32;

    fn update(&mut self, input: f64) -> Option<u32> {
        if !input.is_finite() {
            return self.value();
        }
        if !self.seen || input >= self.peak {
            self.peak = input;
            self.bars_under_water = 0;
        } else {
            self.bars_under_water = self.bars_under_water.saturating_add(1);
        }
        self.seen = true;
        Some(self.bars_under_water)
    }

    fn reset(&mut self) {
        self.peak = f64::NEG_INFINITY;
        self.bars_under_water = 0;
        self.seen = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.seen
    }

    fn name(&self) -> &'static str {
        "DrawdownDuration"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    #[test]
    fn accessors_and_metadata() {
        let mut d = DrawdownDuration::new();
        assert_eq!(d.name(), "DrawdownDuration");
        assert_eq!(d.warmup_period(), 1);
        assert_eq!(d.value(), None);
        d.update(100.0);
        assert_eq!(d.value(), Some(0));
    }

    #[test]
    fn first_bar_is_peak() {
        let mut d = DrawdownDuration::new();
        assert_eq!(d.update(100.0), Some(0));
    }

    #[test]
    fn under_water_counter_increments() {
        let mut d = DrawdownDuration::new();
        d.update(100.0);
        assert_eq!(d.update(90.0), Some(1));
        assert_eq!(d.update(80.0), Some(2));
        assert_eq!(d.update(85.0), Some(3));
    }

    #[test]
    fn new_peak_resets_counter() {
        let mut d = DrawdownDuration::new();
        d.update(100.0);
        d.update(90.0);
        d.update(80.0);
        assert_eq!(d.update(105.0), Some(0));
        assert_eq!(d.update(95.0), Some(1));
    }

    #[test]
    fn equal_value_is_treated_as_peak() {
        let mut d = DrawdownDuration::new();
        d.update(100.0);
        assert_eq!(d.update(100.0), Some(0));
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut d = DrawdownDuration::new();
        d.update(100.0);
        d.update(90.0);
        let v = d.value();
        assert_eq!(d.update(f64::NAN), v);
        assert_eq!(d.update(f64::INFINITY), v);
    }

    #[test]
    fn reset_clears_state() {
        let mut d = DrawdownDuration::new();
        d.batch(&[100.0, 90.0, 80.0]);
        assert!(d.is_ready());
        d.reset();
        assert!(!d.is_ready());
        assert_eq!(d.update(100.0), Some(0));
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..30)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 5.0)
            .collect();
        let batch = DrawdownDuration::new().batch(&prices);
        let mut s = DrawdownDuration::new();
        let streamed: Vec<_> = prices.iter().map(|p| s.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
