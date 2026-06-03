//! Fibonacci Fan — trendlines fanning from a swing start through the
//! retracement levels at the swing end, extended to the current bar.

use crate::indicators::pattern_swing::{SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// The three fan ratios drawn (38.2% / 50% / 61.8%).
const RATIOS: [f64; 3] = [0.382, 0.5, 0.618];

/// Fibonacci Fan line prices evaluated at the current bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FibFanOutput {
    /// Price of the 38.2% fan line at the current bar.
    pub fan_382: f64,
    /// Price of the 50% fan line at the current bar.
    pub fan_500: f64,
    /// Price of the 61.8% fan line at the current bar.
    pub fan_618: f64,
}

/// Fibonacci Fan (`FibFan`).
///
/// Anchored at the start of the most recent confirmed swing leg, three lines fan
/// out through the 38.2% / 50% / 61.8% retracement levels located at the leg's
/// end bar, then extend to the current bar. Each line's price is reported as the
/// fan opens with elapsed time.
///
/// ```text
/// line(r) = start + r * (end - start) * (cur - start_bar) / (end_bar - start_bar)
/// ```
///
/// Parameter-free; construction is infallible. Returns `None` until the first
/// leg is complete.
///
/// See `crates/wickra-core/src/indicators/fib_fan.rs`.
#[derive(Debug, Clone)]
pub struct FibFan {
    swing: SwingTracker,
}

impl FibFan {
    /// Construct a new Fibonacci Fan tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 2),
        }
    }

    fn fan(&self) -> Option<FibFanOutput> {
        let pivots = self.swing.pivots();
        let start = pivots.first()?;
        let end = pivots.get(1)?;
        // Consecutive pivots occur at strictly increasing bars, so the span is
        // always at least one bar — no division by zero.
        let span_bars = (end.bar - start.bar) as f64;
        let elapsed = (self.swing.current_bar() - start.bar) as f64;
        let progress = elapsed / span_bars;
        let line = |r: f64| start.price + r * (end.price - start.price) * progress;
        Some(FibFanOutput {
            fan_382: line(RATIOS[0]),
            fan_500: line(RATIOS[1]),
            fan_618: line(RATIOS[2]),
        })
    }
}

impl Default for FibFan {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for FibFan {
    type Input = Candle;
    type Output = FibFanOutput;

    fn update(&mut self, candle: Candle) -> Option<FibFanOutput> {
        self.swing.update(candle);
        self.fan()
    }

    fn reset(&mut self) {
        self.swing.reset();
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.swing.pivots().len() >= 2
    }

    fn name(&self) -> &'static str {
        "FibFan"
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

    /// Drive a leg start=200 (bar 0) -> end=100 (bar 2), confirmed at bar 3, so
    /// the fan is first reported at bar 3 with `progress = 3 / 2 = 1.5`.
    fn down_leg() -> Vec<Candle> {
        vec![
            c(200.0, 199.0, 0), // bootstrap high @200 (bar 0)
            c(190.0, 160.0, 1), // confirm high @200, low candidate @160
            c(150.0, 100.0, 2), // extend low to 100 (bar 2)
            c(110.0, 105.0, 3), // confirm low @100 -> two pivots
        ]
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = FibFan::new();
        assert_eq!(indicator.name(), "FibFan");
        assert_eq!(indicator.warmup_period(), 2);
        assert!(!indicator.is_ready());
        assert!(!FibFan::default().is_ready());
    }

    #[test]
    fn no_output_before_two_pivots() {
        let mut indicator = FibFan::new();
        // Only the high confirms here; no end pivot yet.
        let outputs: Vec<_> = [c(200.0, 199.0, 0), c(190.0, 150.0, 1)]
            .into_iter()
            .map(|x| indicator.update(x))
            .collect();
        assert!(outputs.iter().all(Option::is_none));
        assert!(!indicator.is_ready());
    }

    #[test]
    fn fan_lines_open_with_elapsed_time() {
        let mut indicator = FibFan::new();
        let mut last = None;
        for candle in down_leg() {
            last = indicator.update(candle);
        }
        let v = last.unwrap();
        assert!(indicator.is_ready());
        // progress = (3 - 0) / (2 - 0) = 1.5; line(r) = 200 + r*(-100)*1.5.
        assert_relative_eq!(v.fan_382, 200.0 - 0.382 * 150.0);
        assert_relative_eq!(v.fan_500, 125.0);
        assert_relative_eq!(v.fan_618, 200.0 - 0.618 * 150.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = FibFan::new();
        for candle in down_leg() {
            let _ = indicator.update(candle);
        }
        assert!(indicator.is_ready());
        indicator.reset();
        assert!(!indicator.is_ready());
        assert!(indicator.update(c(100.0, 99.5, 0)).is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = down_leg();
        let mut a = FibFan::new();
        let mut b = FibFan::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
