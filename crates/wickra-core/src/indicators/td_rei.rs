#![allow(clippy::doc_markdown)]

//! Tom DeMark Range Expansion Index (TD REI).
//!
//! The TD REI is a `period`-bar bounded oscillator in `[-100, 100]` that
//! detects exhaustion via comparisons of the current bar's range to the bars
//! two and five-or-six bars earlier. The canonical TD REI uses a `period` of
//! 5.
//!
//! Per bar `i` (requires history through `i - 7`):
//!
//! ```text
//! cond1 = (high[i] >= low[i-5])  OR (high[i] >= low[i-6])
//! cond2 = (low[i]  <= high[i-5]) OR (low[i]  <= high[i-6])
//!
//! if cond1 AND cond2:
//!     numerator   = (high[i] - high[i-2]) + (low[i] - low[i-2])
//! else:
//!     numerator   = 0
//!
//! denominator = |high[i] - high[i-2]| + |low[i] - low[i-2]|
//!
//! REI(i) = 100 * sum(numerator, period) / sum(denominator, period)
//! ```
//!
//! When the windowed denominator is zero the indicator falls back to `0` (the
//! neutral midpoint). Readings above `+60` are typically considered
//! overbought; below `-60` oversold.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// TD Range Expansion Index oscillator.
#[derive(Debug, Clone)]
pub struct TdRei {
    period: usize,
    // Need at least the last 7 candles for the lookback comparisons; we keep a
    // rolling window long enough for the rule plus enough numerator/
    // denominator history.
    candles: VecDeque<Candle>,
    numerators: VecDeque<f64>,
    denominators: VecDeque<f64>,
    last_value: Option<f64>,
}

/// Minimum history required to evaluate the TD REI per-bar rule. The
/// numerator and denominator both reference `bar[i-2]` and the long
/// conditional references `bar[i-5]` and `bar[i-6]`, so we need the candle
/// six bars before the current one to be available.
const LOOKBACK: usize = 7;

impl TdRei {
    /// Construct a TD REI with the given averaging window. The classic
    /// DeMark configuration is `period = 5`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            candles: VecDeque::with_capacity(LOOKBACK),
            numerators: VecDeque::with_capacity(period),
            denominators: VecDeque::with_capacity(period),
            last_value: None,
        })
    }

    /// DeMark's classic configuration: `period = 5`.
    pub fn classic() -> Self {
        Self::new(5).expect("classic TD REI parameters are valid")
    }

    /// Configured window.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Latest emitted value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for TdRei {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        // Maintain a rolling window of the last `LOOKBACK` candles (front =
        // 6 bars ago when full).
        if self.candles.len() == LOOKBACK {
            self.candles.pop_front();
        }
        if self.candles.len() < LOOKBACK - 1 {
            // Need 6 previous candles before we can evaluate the rule on the
            // current one.
            self.candles.push_back(candle);
            return None;
        }
        // candles currently holds the 6 most recent bars (in order); the new
        // candle is the 7th. After the rule fires we push it onto the back.
        // Indexing convention: index 0 is the oldest in the window (i.e. 6
        // bars ago); index 5 is the bar just before the current one.
        // For the rule we need:
        //   bar[i-2] -> candles[len-2]  (here len == 6)
        //   bar[i-5] -> candles[1]
        //   bar[i-6] -> candles[0]
        let prev2 = self.candles[self.candles.len() - 2];
        let prev5 = self.candles[1];
        let prev6 = self.candles[0];

        let cond1 = candle.high >= prev5.low || candle.high >= prev6.low;
        let cond2 = candle.low <= prev5.high || candle.low <= prev6.high;

        let raw_num = (candle.high - prev2.high) + (candle.low - prev2.low);
        let denominator = (candle.high - prev2.high).abs() + (candle.low - prev2.low).abs();
        let numerator = if cond1 && cond2 { raw_num } else { 0.0 };

        if self.numerators.len() == self.period {
            self.numerators.pop_front();
            self.denominators.pop_front();
        }
        self.numerators.push_back(numerator);
        self.denominators.push_back(denominator);
        self.candles.push_back(candle);

        if self.numerators.len() < self.period {
            return None;
        }
        let sum_num: f64 = self.numerators.iter().sum();
        let sum_den: f64 = self.denominators.iter().sum();
        let v = if sum_den == 0.0 {
            0.0
        } else {
            100.0 * sum_num / sum_den
        };
        self.last_value = Some(v);
        Some(v)
    }

    fn reset(&mut self) {
        self.candles.clear();
        self.numerators.clear();
        self.denominators.clear();
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        // 6 bars to fill the lookback plus `period` updates to fill the
        // numerator / denominator buffers.
        (LOOKBACK - 1) + self.period
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "TDREI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new_unchecked(close, high, low, close, 0.0, ts)
    }

    #[test]
    fn flat_market_yields_neutral_zero() {
        // All highs and lows equal -> denominator is identically zero, so the
        // indicator emits its neutral fallback of 0.
        let candles: Vec<Candle> = (0..40).map(|i| c(11.0, 9.0, 10.0, i)).collect();
        let mut rei = TdRei::classic();
        let out = rei.batch(&candles);
        for v in out.iter().skip(rei.warmup_period()).copied().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn pure_uptrend_pegs_indicator_at_100() {
        // Every bar makes strictly higher highs and lows. Both range-overlap
        // conditions hold (current high > all previous lows; current low > all
        // previous highs is false, but we need current low <= some prev
        // high). For a slow steady uptrend cond2 still holds because
        // current low < prev5/prev6 highs as long as the slope is moderate.
        // With slope 1 and spread 2 (low to high), cond2 fails after ~3 bars.
        // Use a smaller slope so cond2 holds throughout.
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let m = 100.0 + f64::from(i) * 0.1;
                c(m + 1.0, m - 1.0, m, i64::from(i))
            })
            .collect();
        let mut rei = TdRei::classic();
        let last = rei.batch(&candles).into_iter().flatten().last().unwrap();
        // Every numerator is positive (price moving up) and equals the
        // denominator in magnitude (no sign flips), so REI saturates at 100.
        assert_relative_eq!(last, 100.0, epsilon = 1e-9);
    }

    #[test]
    fn pure_downtrend_pegs_indicator_at_minus_100() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let m = 100.0 - f64::from(i) * 0.1;
                c(m + 1.0, m - 1.0, m, i64::from(i))
            })
            .collect();
        let mut rei = TdRei::classic();
        let last = rei.batch(&candles).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, -100.0, epsilon = 1e-9);
    }

    #[test]
    fn stays_in_minus_100_to_100() {
        let candles: Vec<Candle> = (0..200)
            .map(|i| {
                let m = 50.0 + (f64::from(i) * 0.2).sin() * 5.0;
                c(m + 1.0, m - 1.0, m, i64::from(i))
            })
            .collect();
        let mut rei = TdRei::classic();
        for v in rei.batch(&candles).into_iter().flatten() {
            assert!((-100.0..=100.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.3).sin() * 5.0;
                c(m + 1.0, m - 1.0, m, i64::from(i))
            })
            .collect();
        let mut a = TdRei::classic();
        let mut b = TdRei::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(TdRei::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let m = 100.0 + f64::from(i) * 0.1;
                c(m + 1.0, m - 1.0, m, i64::from(i))
            })
            .collect();
        let mut rei = TdRei::classic();
        rei.batch(&candles);
        assert!(rei.is_ready());
        rei.reset();
        assert!(!rei.is_ready());
        assert_eq!(rei.update(candles[0]), None);
        assert_eq!(rei.value(), None);
    }

    #[test]
    fn accessors_and_metadata() {
        let rei = TdRei::classic();
        assert_eq!(rei.period(), 5);
        assert_eq!(rei.warmup_period(), 6 + 5);
        assert_eq!(rei.name(), "TDREI");
    }
}
