//! Rickshaw Man candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Rickshaw Man — a single-bar indecision signal. A long-legged doji whose tiny
/// body sits near the *middle* of a wide range, the most balanced form of
/// indecision: neither side controlled the close and the midpoint pins it.
///
/// ```text
/// range = high − low
/// doji        = |close − open| <= 0.1 * range
/// long upper  = high − max(open, close) >= 0.3 * range
/// long lower  = min(open, close) − low  >= 0.3 * range
/// centred body = body midpoint within the central 40–60 % of the range
/// ```
///
/// Output is `+1.0` when the rickshaw man prints and `0.0` otherwise. This is a
/// non-directional indecision flag — it never emits `−1.0`. A rickshaw man is a
/// special case of a long-legged doji (the body additionally sits at the centre),
/// so both detectors may flag the same bar. Body and shadow thresholds follow the
/// geometric house style (fixed fractions of the bar range) rather than TA-Lib's
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
/// use wickra_core::{Candle, Indicator, RickshawMan};
///
/// let mut indicator = RickshawMan::new();
/// // Tiny body centred in a wide range, long shadows both sides.
/// let candle = Candle::new(10.0, 12.0, 8.0, 10.0, 1.0, 0).unwrap();
/// assert_eq!(indicator.update(candle), Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct RickshawMan {
    has_emitted: bool,
}

impl RickshawMan {
    /// Construct a new Rickshaw Man detector.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for RickshawMan {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let range = candle.high - candle.low;
        if range <= 0.0 {
            return Some(0.0);
        }
        if (candle.close - candle.open).abs() > 0.1 * range {
            return Some(0.0);
        }
        let upper = candle.high - candle.open.max(candle.close);
        let lower = candle.open.min(candle.close) - candle.low;
        let body_mid = 0.5 * (candle.open + candle.close);
        let pos = (body_mid - candle.low) / range;
        if upper >= 0.3 * range && lower >= 0.3 * range && (0.4..=0.6).contains(&pos) {
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
        "RickshawMan"
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
        let t = RickshawMan::new();
        assert_eq!(t.name(), "RickshawMan");
        assert_eq!(t.warmup_period(), 1);
        assert!(!t.is_ready());
    }

    #[test]
    fn rickshaw_is_plus_one() {
        let mut t = RickshawMan::new();
        assert_eq!(t.update(c(10.0, 12.0, 8.0, 10.0, 0)), Some(1.0));
    }

    #[test]
    fn off_centre_body_yields_zero() {
        let mut t = RickshawMan::new();
        // Long-legged but the body sits near the top, not the middle.
        assert_eq!(t.update(c(11.4, 12.0, 8.0, 11.45, 0)), Some(0.0));
    }

    #[test]
    fn one_sided_shadow_yields_zero() {
        let mut t = RickshawMan::new();
        // Dragonfly shape: no upper shadow -> not a rickshaw man.
        assert_eq!(t.update(c(10.0, 10.05, 6.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn non_doji_yields_zero() {
        let mut t = RickshawMan::new();
        assert_eq!(t.update(c(9.0, 12.0, 8.0, 11.0, 0)), Some(0.0));
    }

    #[test]
    fn zero_range_yields_zero() {
        let mut t = RickshawMan::new();
        assert_eq!(t.update(c(10.0, 10.0, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 2.0, base - 2.0, base + 0.05, i)
            })
            .collect();
        let mut a = RickshawMan::new();
        let mut b = RickshawMan::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = RickshawMan::new();
        t.update(c(10.0, 12.0, 8.0, 10.0, 0));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
    }
}
