//! Advance Block candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Advance Block — a 3-bar bearish warning: three green candles still pushing to
/// higher closes, but visibly running out of steam — each real body shrinks while
/// the upper shadows lengthen, hinting the advance is about to stall.
///
/// ```text
/// all three green & higher closes
/// each opens inside the prior body
/// shrinking bodies   (body3 < body2 < body1)
/// upper shadow of bar3 >= upper shadow of bar2 and bar3 has an upper shadow
/// ```
///
/// Output is `−1.0` when the pattern completes and `0.0` otherwise. Advance Block
/// is a single-direction (bearish-only) warning, so it never emits `+1.0`. The
/// first two bars always return `0.0` because the three-bar window is not yet
/// filled. Pattern-shape check only — no trend filter is applied; combine with a
/// trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `−1.0` bearish, `0.0` no pattern — so it drops straight into
/// a machine-learning feature matrix as a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{AdvanceBlock, Candle, Indicator};
///
/// let mut indicator = AdvanceBlock::new();
/// indicator.update(Candle::new(10.0, 13.1, 9.9, 13.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(12.0, 14.3, 11.9, 14.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(13.5, 15.0, 13.4, 14.5, 1.0, 2).unwrap());
/// assert_eq!(out, Some(-1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct AdvanceBlock {
    prev: Option<Candle>,
    prev_prev: Option<Candle>,
    has_emitted: bool,
}

impl AdvanceBlock {
    /// Construct a new Advance Block detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            prev_prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for AdvanceBlock {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let pp = self.prev_prev;
        let p = self.prev;
        self.prev_prev = self.prev;
        self.prev = Some(candle);
        let (Some(bar1), Some(bar2)) = (pp, p) else {
            return Some(0.0);
        };
        let body1 = bar1.close - bar1.open;
        let body2 = bar2.close - bar2.open;
        let body3 = candle.close - candle.open;
        let upper2 = bar2.high - bar2.close;
        let upper3 = candle.high - candle.close;
        if bar1.close > bar1.open
            && bar2.close > bar2.open
            && candle.close > candle.open
            && bar2.close > bar1.close
            && candle.close > bar2.close
            && bar2.open >= bar1.open
            && bar2.open <= bar1.close
            && candle.open >= bar2.open
            && candle.open <= bar2.close
            && body2 < body1
            && body3 < body2
            && upper3 >= upper2
            && upper3 > 0.0
        {
            return Some(-1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.prev = None;
        self.prev_prev = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        3
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "AdvanceBlock"
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
        let t = AdvanceBlock::new();
        assert_eq!(t.name(), "AdvanceBlock");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
    }

    #[test]
    fn advance_block_is_minus_one() {
        let mut t = AdvanceBlock::new();
        assert_eq!(t.update(c(10.0, 13.1, 9.9, 13.0, 0)), Some(0.0));
        assert_eq!(t.update(c(12.0, 14.3, 11.9, 14.0, 1)), Some(0.0));
        assert_eq!(t.update(c(13.5, 15.0, 13.4, 14.5, 2)), Some(-1.0));
    }

    #[test]
    fn strong_advance_yields_zero() {
        let mut t = AdvanceBlock::new();
        // Bodies grow instead of shrinking -> a strong advance, not blocked.
        assert_eq!(t.update(c(10.0, 11.1, 9.9, 11.0, 0)), Some(0.0));
        assert_eq!(t.update(c(10.5, 12.6, 10.4, 12.5, 1)), Some(0.0));
        assert_eq!(t.update(c(11.5, 14.1, 11.4, 14.0, 2)), Some(0.0));
    }

    #[test]
    fn no_upper_shadow_growth_yields_zero() {
        let mut t = AdvanceBlock::new();
        t.update(c(10.0, 13.1, 9.9, 13.0, 0));
        t.update(c(12.0, 14.3, 11.9, 14.0, 1));
        // bar3 shrinking body but no upper shadow -> not blocked.
        assert_eq!(t.update(c(13.5, 14.5, 13.4, 14.5, 2)), Some(0.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = AdvanceBlock::new();
        assert_eq!(t.update(c(10.0, 13.1, 9.9, 13.0, 0)), Some(0.0));
        assert_eq!(t.update(c(12.0, 14.3, 11.9, 14.0, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 2.0, base - 0.2, base + 1.5, i)
            })
            .collect();
        let mut a = AdvanceBlock::new();
        let mut b = AdvanceBlock::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = AdvanceBlock::new();
        t.update(c(10.0, 13.1, 9.9, 13.0, 0));
        t.update(c(12.0, 14.3, 11.9, 14.0, 1));
        t.update(c(13.5, 15.0, 13.4, 14.5, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(10.0, 13.1, 9.9, 13.0, 0)), Some(0.0));
    }
}
