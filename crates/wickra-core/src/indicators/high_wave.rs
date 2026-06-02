//! High-Wave candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// High-Wave — a single-bar extreme-indecision signal. A small body with very
/// long shadows on *both* sides: price swung far up and far down yet finished
/// near the open, a sign that trend conviction has evaporated.
///
/// ```text
/// range = high − low
/// long upper = high − max(open, close) >= 0.4 * range
/// long lower = min(open, close) − low  >= 0.4 * range
/// ```
///
/// The two long-shadow conditions force the body below `0.2 * range`, so no
/// separate body test is needed. Output is `+1.0` when the high-wave prints and
/// `0.0` otherwise — a non-directional indecision flag, it never emits `−1.0`.
/// Shadow thresholds follow the geometric house style rather than TA-Lib's
/// rolling averages. Pattern-shape check only — no trend filter is applied;
/// combine with a trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` detected, `0.0` no pattern — so it drops straight into
/// a machine-learning feature matrix as a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, HighWave, Indicator};
///
/// let mut indicator = HighWave::new();
/// // Small body, long shadows both sides.
/// let candle = Candle::new(10.0, 12.0, 8.0, 10.3, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct HighWave {
    has_emitted: bool,
}

impl HighWave {
    /// Construct a new High-Wave detector.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for HighWave {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let range = candle.high - candle.low;
        if range <= 0.0 {
            return Some(0.0);
        }
        let upper = candle.high - candle.open.max(candle.close);
        let lower = candle.open.min(candle.close) - candle.low;
        if upper >= 0.4 * range && lower >= 0.4 * range {
            return Some(1.0);
        }
        Some(0.0)
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
        "HighWave"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(open: f64, high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(open, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let t = HighWave::new();
        assert_eq!(t.name(), "HighWave");
        assert_eq!(t.warmup_period(), 1);
        assert!(!t.is_ready());
    }

    #[test]
    fn high_wave_is_plus_one() {
        let mut t = HighWave::new();
        assert_eq!(t.update(c(10.0, 12.0, 8.0, 10.3, 0)), Some(1.0));
    }

    #[test]
    fn short_upper_shadow_yields_zero() {
        let mut t = HighWave::new();
        // Long lower shadow but short upper -> not a high-wave.
        assert_eq!(t.update(c(11.5, 12.0, 8.0, 11.7, 0)), Some(0.0));
    }

    #[test]
    fn short_lower_shadow_yields_zero() {
        let mut t = HighWave::new();
        // Long upper shadow but short lower -> not a high-wave.
        assert_eq!(t.update(c(8.3, 12.0, 8.0, 8.5, 0)), Some(0.0));
    }

    #[test]
    fn big_body_yields_zero() {
        let mut t = HighWave::new();
        // A large body cannot leave both shadows long.
        assert_eq!(t.update(c(8.5, 12.0, 8.0, 11.5, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = HighWave::new();
        assert_eq!(t.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 3.0, base - 3.0, base + 0.2, i)
            })
            .collect();
        let mut a = HighWave::new();
        let mut b = HighWave::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = HighWave::new();
        t.update(c(10.0, 12.0, 8.0, 10.3, 0));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
    }
}
