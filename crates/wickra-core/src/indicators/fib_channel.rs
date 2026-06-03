//! Fibonacci Channel — a sloped base trendline plus parallel lines offset by
//! Fibonacci multiples of the channel width.

use crate::indicators::pattern_swing::{SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// The parallel-line ratios above the base (61.8% / 100% / 161.8% of the width).
const RATIOS: [f64; 3] = [0.618, 1.0, 1.618];

/// Fibonacci Channel line prices evaluated at the current bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FibChannelOutput {
    /// The base trendline price at the current bar.
    pub base: f64,
    /// Base + 61.8% of the channel width.
    pub level_618: f64,
    /// Base + 100% of the width — the opposite channel boundary.
    pub level_1000: f64,
    /// Base + 161.8% of the width.
    pub level_1618: f64,
}

/// Fibonacci Channel (`FibChannel`).
///
/// From the last three confirmed pivots, the two same-direction outer pivots
/// define a sloped base trendline and the opposite middle pivot sets the channel
/// width (its signed distance from the base line). Parallel lines are then offset
/// by Fibonacci multiples of that width and reported at the current bar.
///
/// ```text
/// slope     = (p2 - p0) / (bar2 - bar0)
/// base(bar) = p0 + slope * (bar - bar0)
/// width     = p1 - base(bar1)
/// level(r)  = base(cur) + r * width
/// ```
///
/// Parameter-free; construction is infallible. Returns `None` until three pivots
/// have confirmed.
///
/// See `crates/wickra-core/src/indicators/fib_channel.rs`.
#[derive(Debug, Clone)]
pub struct FibChannel {
    swing: SwingTracker,
}

impl FibChannel {
    /// Construct a new Fibonacci Channel tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 3),
        }
    }

    fn channel(&self) -> Option<FibChannelOutput> {
        let pivots = self.swing.pivots();
        let p0 = pivots.first()?;
        let p1 = pivots.get(1)?;
        let p2 = pivots.get(2)?;
        // p0 and p2 are the same-direction outer pivots; their bars differ
        // strictly, so the slope denominator is non-zero.
        let slope = (p2.price - p0.price) / (p2.bar - p0.bar) as f64;
        let base_at = |bar: usize| p0.price + slope * (bar - p0.bar) as f64;
        let width = p1.price - base_at(p1.bar);
        let base = base_at(self.swing.current_bar());
        Some(FibChannelOutput {
            base,
            level_618: base + RATIOS[0] * width,
            level_1000: base + RATIOS[1] * width,
            level_1618: base + RATIOS[2] * width,
        })
    }
}

impl Default for FibChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for FibChannel {
    type Input = Candle;
    type Output = FibChannelOutput;

    fn update(&mut self, candle: Candle) -> Option<FibChannelOutput> {
        self.swing.update(candle);
        self.channel()
    }

    fn reset(&mut self) {
        self.swing.reset();
    }

    fn warmup_period(&self) -> usize {
        3
    }

    fn is_ready(&self) -> bool {
        self.swing.pivots().len() >= 3
    }

    fn name(&self) -> &'static str {
        "FibChannel"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(high: f64, low: f64, ts: i64) -> Candle {
        Candle::new(low, high, low, low, 1.0, ts).unwrap()
    }

    /// Pivots: high 200 (bar 0), low 100 (bar 1), high 220 (bar 3); confirmed at
    /// bar 4 so the channel is first reported at current bar 4.
    fn three_pivots() -> Vec<Candle> {
        vec![
            c(200.0, 199.0, 0),
            c(190.0, 100.0, 1), // confirm high @200, low candidate @100
            c(110.0, 108.0, 2), // confirm low @100, high candidate @110
            c(220.0, 210.0, 3), // extend high to 220 (bar 3)
            c(200.0, 150.0, 4), // confirm high @220 -> three pivots
        ]
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = FibChannel::new();
        assert_eq!(indicator.name(), "FibChannel");
        assert_eq!(indicator.warmup_period(), 3);
        assert!(!indicator.is_ready());
        assert!(!FibChannel::default().is_ready());
    }

    #[test]
    fn no_output_before_three_pivots() {
        let mut indicator = FibChannel::new();
        let outputs: Vec<_> = [c(200.0, 199.0, 0), c(190.0, 100.0, 1), c(110.0, 108.0, 2)]
            .into_iter()
            .map(|x| indicator.update(x))
            .collect();
        // Only two pivots confirm within these three bars.
        assert!(outputs.iter().all(Option::is_none));
        assert!(!indicator.is_ready());
    }

    #[test]
    fn channel_levels_from_three_pivots() {
        let mut indicator = FibChannel::new();
        let mut last = None;
        for candle in three_pivots() {
            last = indicator.update(candle);
        }
        let v = last.unwrap();
        assert!(indicator.is_ready());
        // Base through highs (0,200) and (3,220); width from low (1,100); cur = 4.
        let slope = (220.0 - 200.0) / 3.0;
        let base_cur = 200.0 + slope * 4.0;
        let width = 100.0 - (200.0 + slope * 1.0);
        assert_relative_eq!(v.base, base_cur);
        assert_relative_eq!(v.level_1000, base_cur + width);
        assert_relative_eq!(v.level_618, base_cur + 0.618 * width);
        assert_relative_eq!(v.level_1618, base_cur + 1.618 * width);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = FibChannel::new();
        for candle in three_pivots() {
            let _ = indicator.update(candle);
        }
        assert!(indicator.is_ready());
        indicator.reset();
        assert!(!indicator.is_ready());
        assert!(indicator.update(c(100.0, 99.5, 0)).is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = three_pivots();
        let mut a = FibChannel::new();
        let mut b = FibChannel::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
