//! Fibonacci Confluence — the strongest retracement cluster across recent legs.

use crate::indicators::pattern_swing::{
    approx_equal, SwingTracker, LEVEL_TOLERANCE, SWING_THRESHOLD,
};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// How many recent pivots to consider; six pivots yield up to five legs.
const PIVOT_HISTORY: usize = 6;

/// The retracement ratios contributed by each leg to the confluence search.
const RATIOS: [f64; 3] = [0.382, 0.5, 0.618];

/// The strongest Fibonacci confluence zone found across recent swing legs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FibConfluenceOutput {
    /// Mean price of the densest cluster of retracement levels.
    pub price: f64,
    /// Number of retracement levels that fall inside the cluster (its strength).
    pub strength: f64,
}

/// Fibonacci Confluence (`FibConfluence`).
///
/// Computes the 38.2% / 50% / 61.8% retracement prices of every leg among the
/// last six confirmed pivots, then reports the densest price cluster — where
/// levels from different legs stack up, the zone the market is most likely to
/// react to. `price` is the cluster mean; `strength` is how many levels it
/// gathers.
///
/// Parameter-free; construction is infallible. Returns `None` until at least two
/// legs (three pivots) exist.
///
/// See `crates/wickra-core/src/indicators/fib_confluence.rs`.
/// # Example
///
/// ```
/// use wickra_core::{FibConfluence, Candle, Indicator};
///
/// let mut indicator = FibConfluence::new();
/// // `None` during warmup, then `Some(_)` once enough bars are seen.
/// let mut out = None;
/// for i in 0..40i64 {
///     let p = 100.0 + (i as f64 * 0.4).sin() * 5.0;
///     let candle = Candle::new(p, p + 1.5, p - 1.5, p + 0.3, 1_000.0, i).unwrap();
///     out = indicator.update(candle);
/// }
/// let _ = out;
/// ```
#[derive(Debug, Clone)]
pub struct FibConfluence {
    swing: SwingTracker,
}

impl FibConfluence {
    /// Construct a new Fibonacci Confluence tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, PIVOT_HISTORY),
        }
    }

    fn confluence(&self) -> Option<FibConfluenceOutput> {
        let pivots = self.swing.pivots();
        if pivots.len() < 3 {
            return None;
        }
        let levels: Vec<f64> = pivots
            .windows(2)
            .flat_map(|leg| {
                let (start, end) = (leg[0].price, leg[1].price);
                RATIOS.map(|r| end + r * (start - end))
            })
            .collect();
        // The `len < 3` guard guarantees at least two legs, hence a non-empty
        // level set, so `max_by` always yields a cluster.
        let (count, total) = levels
            .iter()
            .map(|&center| {
                let members: Vec<f64> = levels
                    .iter()
                    .copied()
                    .filter(|&x| approx_equal(x, center, LEVEL_TOLERANCE))
                    .collect();
                (members.len(), members.iter().sum::<f64>())
            })
            .max_by(|a, b| a.0.cmp(&b.0))
            .expect("at least two legs guarantee a non-empty level set");
        Some(FibConfluenceOutput {
            price: total / count as f64,
            strength: count as f64,
        })
    }
}

impl Default for FibConfluence {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for FibConfluence {
    type Input = Candle;
    type Output = FibConfluenceOutput;

    fn update(&mut self, candle: Candle) -> Option<FibConfluenceOutput> {
        self.swing.update(candle);
        self.confluence()
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
        "FibConfluence"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::pattern_swing::candles_for_pivots;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn accessors_and_metadata() {
        let indicator = FibConfluence::new();
        assert_eq!(indicator.name(), "FibConfluence");
        assert_eq!(indicator.warmup_period(), 3);
        assert!(!indicator.is_ready());
        assert!(!FibConfluence::default().is_ready());
    }

    #[test]
    fn no_output_before_two_legs() {
        let mut indicator = FibConfluence::new();
        let outputs: Vec<_> = candles_for_pivots(&[200.0, 100.0])
            .into_iter()
            .map(|c| indicator.update(c))
            .collect();
        assert!(outputs.iter().all(Option::is_none));
        assert!(!indicator.is_ready());
    }

    #[test]
    fn picks_the_densest_cluster() {
        // Legs 200->100 and 100->160. The 38.2% of each (138.2 and ~137.08)
        // sit within 3% of each other and form the densest cluster (strength 2).
        let mut indicator = FibConfluence::new();
        let mut last = None;
        for candle in candles_for_pivots(&[200.0, 100.0, 160.0]) {
            last = indicator.update(candle);
        }
        let v = last.unwrap();
        assert!(indicator.is_ready());
        assert_relative_eq!(v.strength, 2.0);
        let want = (138.2 + (160.0 + 0.382 * (100.0 - 160.0))) / 2.0;
        assert_relative_eq!(v.price, want, epsilon = 1e-9);
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = FibConfluence::new();
        for candle in candles_for_pivots(&[200.0, 100.0, 160.0]) {
            let _ = indicator.update(candle);
        }
        assert!(indicator.is_ready());
        indicator.reset();
        assert!(!indicator.is_ready());
        let c = Candle::new(99.5, 100.0, 99.5, 99.5, 1.0, 0).unwrap();
        assert!(indicator.update(c).is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = candles_for_pivots(&[200.0, 100.0, 160.0, 120.0]);
        let mut a = FibConfluence::new();
        let mut b = FibConfluence::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
