//! Fibonacci Arcs — semicircular retracement levels centred on the swing end,
//! decaying back toward it as time elapses.

use crate::indicators::pattern_swing::{SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// The three arc ratios drawn (38.2% / 50% / 61.8%).
const RATIOS: [f64; 3] = [0.382, 0.5, 0.618];

/// Fibonacci Arc prices evaluated at the current bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FibArcsOutput {
    /// Price of the 38.2% arc at the current bar.
    pub arc_382: f64,
    /// Price of the 50% arc at the current bar.
    pub arc_500: f64,
    /// Price of the 61.8% arc at the current bar.
    pub arc_618: f64,
}

/// Fibonacci Arcs (`FibArcs`).
///
/// Three arcs centred on the end of the most recent confirmed swing leg. Time is
/// normalised by the leg's bar-width so the construction is chart-scale-free: at
/// the leg's end bar each arc sits exactly on its retracement level, and as time
/// elapses the arc curves back toward the swing-end price, reaching it one leg
/// width later.
///
/// ```text
/// u       = (cur - end_bar) / (end_bar - start_bar)
/// arc(r)  = end + (start - end) * r * sqrt(max(0, 1 - u^2))
/// ```
///
/// Parameter-free; construction is infallible. Returns `None` until the first
/// leg is complete.
///
/// See `crates/wickra-core/src/indicators/fib_arcs.rs`.
#[derive(Debug, Clone)]
pub struct FibArcs {
    swing: SwingTracker,
}

impl FibArcs {
    /// Construct a new Fibonacci Arcs tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 2),
        }
    }

    fn arcs(&self) -> Option<FibArcsOutput> {
        let pivots = self.swing.pivots();
        let start = pivots.first()?;
        let end = pivots.get(1)?;
        // Consecutive pivots occur at strictly increasing bars → span >= 1 bar.
        let span_bars = (end.bar - start.bar) as f64;
        let u = (self.swing.current_bar() - end.bar) as f64 / span_bars;
        let curve = (1.0 - u * u).max(0.0).sqrt();
        let arc = |r: f64| end.price + (start.price - end.price) * r * curve;
        Some(FibArcsOutput {
            arc_382: arc(RATIOS[0]),
            arc_500: arc(RATIOS[1]),
            arc_618: arc(RATIOS[2]),
        })
    }
}

impl Default for FibArcs {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for FibArcs {
    type Input = Candle;
    type Output = FibArcsOutput;

    fn update(&mut self, candle: Candle) -> Option<FibArcsOutput> {
        self.swing.update(candle);
        self.arcs()
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
        "FibArcs"
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

    /// Leg start=200 (bar 0) -> end=100 (bar 2), confirmed at bar 3 so the arc is
    /// first reported with `u = (3 - 2) / (2 - 0) = 0.5`.
    fn down_leg() -> Vec<Candle> {
        vec![
            c(200.0, 199.0, 0),
            c(190.0, 160.0, 1), // confirm high @200
            c(150.0, 100.0, 2), // extend low to 100 (bar 2)
            c(110.0, 105.0, 3), // confirm low @100 -> two pivots
        ]
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = FibArcs::new();
        assert_eq!(indicator.name(), "FibArcs");
        assert_eq!(indicator.warmup_period(), 2);
        assert!(!indicator.is_ready());
        assert!(!FibArcs::default().is_ready());
    }

    #[test]
    fn no_output_before_two_pivots() {
        let mut indicator = FibArcs::new();
        let outputs: Vec<_> = [c(200.0, 199.0, 0), c(190.0, 150.0, 1)]
            .into_iter()
            .map(|x| indicator.update(x))
            .collect();
        assert!(outputs.iter().all(Option::is_none));
        assert!(!indicator.is_ready());
    }

    #[test]
    fn arcs_curve_back_toward_the_swing_end() {
        let mut indicator = FibArcs::new();
        let mut last = None;
        for candle in down_leg() {
            last = indicator.update(candle);
        }
        let v = last.unwrap();
        assert!(indicator.is_ready());
        // u = 0.5 → curve = sqrt(0.75); arc(r) = 100 + 100 * r * curve.
        let curve = 0.75_f64.sqrt();
        assert_relative_eq!(v.arc_382, 100.0 + 100.0 * 0.382 * curve);
        assert_relative_eq!(v.arc_500, 100.0 + 100.0 * 0.5 * curve);
        assert_relative_eq!(v.arc_618, 100.0 + 100.0 * 0.618 * curve);
    }

    #[test]
    fn arc_clamps_to_zero_beyond_one_leg_width() {
        // Extend far past the end pivot so u > 1; the curve clamps to 0 and the
        // arcs collapse onto the swing-end price.
        let mut indicator = FibArcs::new();
        for candle in down_leg() {
            let _ = indicator.update(candle);
        }
        // Feed flat bars that neither extend nor confirm a new pivot.
        let mut last = None;
        for ts in 4..12 {
            last = indicator.update(c(108.0, 106.0, ts));
        }
        let v = last.unwrap();
        assert_relative_eq!(v.arc_382, 100.0);
        assert_relative_eq!(v.arc_618, 100.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = FibArcs::new();
        for candle in down_leg() {
            let _ = indicator.update(candle);
        }
        indicator.reset();
        assert!(!indicator.is_ready());
        assert!(indicator.update(c(100.0, 99.5, 0)).is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = down_leg();
        let mut a = FibArcs::new();
        let mut b = FibArcs::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
