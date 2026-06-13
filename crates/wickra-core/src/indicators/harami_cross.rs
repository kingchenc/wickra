#![allow(clippy::doc_markdown)]

//! Harami Cross — a Harami whose second candle is a Doji.
//!
//! A Harami Cross is a stronger Harami: a large real body followed by a Doji whose
//! body sits *within* the prior body. The Doji's total indecision after a strong
//! move makes the reversal signal more potent than a plain Harami.
//!
//! - **Bullish** (`+1.0`): the prior candle is a large **bearish** body
//!   (`close < open`) and the current candle is a Doji whose open and close lie
//!   within the prior body.
//! - **Bearish** (`-1.0`): the prior candle is a large **bullish** body and the
//!   current is a contained Doji.
//! - Otherwise the output is `0.0`.
//!
//! A doji is a candle whose body is `<= 0.1 * range`. The two-bar lookback means
//! the first value lands on the second candle.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

fn is_doji(candle: Candle) -> bool {
    let body = (candle.close - candle.open).abs();
    let range = candle.high - candle.low;
    range > 0.0 && body <= 0.1 * range
}

/// Harami Cross — large-body-then-contained-doji reversal detector.
/// # Example
///
/// ```
/// use wickra_core::{HaramiCross, Candle, Indicator};
///
/// let mut indicator = HaramiCross::new();
/// // `None` during warmup, then `Some(_)` once enough bars are seen.
/// let mut out = None;
/// for i in 0..40i64 {
///     let p = 100.0 + (i as f64 * 0.4).sin() * 5.0;
///     let candle = Candle::new(p, p + 1.5, p - 1.5, p + 0.3, 1_000.0, i).unwrap();
///     out = indicator.update(candle);
/// }
/// let _ = out;
/// ```
#[derive(Debug, Clone, Default)]
pub struct HaramiCross {
    prev: Option<Candle>,
    last_value: Option<f64>,
}

impl HaramiCross {
    /// Construct a new `HaramiCross`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Latest emitted signal if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for HaramiCross {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev) = self.prev else {
            self.prev = Some(candle);
            self.last_value = Some(0.0);
            return Some(0.0);
        };
        let prev_body_low = prev.open.min(prev.close);
        let prev_body_high = prev.open.max(prev.close);
        let prev_is_solid = !is_doji(prev);
        let curr_is_doji = is_doji(candle);
        let contained = candle.open >= prev_body_low
            && candle.open <= prev_body_high
            && candle.close >= prev_body_low
            && candle.close <= prev_body_high;

        let v = if prev_is_solid && curr_is_doji && contained {
            if prev.close < prev.open {
                1.0
            } else {
                -1.0
            }
        } else {
            0.0
        };
        self.prev = Some(candle);
        self.last_value = Some(v);
        Some(v)
    }

    fn reset(&mut self) {
        self.prev = None;
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "HaramiCross"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn solid(open: f64, close: f64) -> Candle {
        Candle::new_unchecked(
            open,
            open.max(close) + 0.2,
            open.min(close) - 0.2,
            close,
            0.0,
            0,
        )
    }

    fn doji(mid: f64) -> Candle {
        Candle::new_unchecked(mid, mid + 1.0, mid - 1.0, mid + 0.02, 0.0, 0)
    }

    #[test]
    fn accessors_and_metadata() {
        let h = HaramiCross::new();
        assert_eq!(h.warmup_period(), 2);
        assert_eq!(h.name(), "HaramiCross");
        assert!(!h.is_ready());
        assert_eq!(h.value(), None);
    }

    #[test]
    fn first_bar_seeds_without_signal() {
        let mut h = HaramiCross::new();
        assert_eq!(h.update(solid(110.0, 100.0)), Some(0.0));
        assert!(h.update(doji(105.0)).is_some());
    }

    #[test]
    fn bullish_harami_cross() {
        // prior big bearish body [100, 110]; doji centred at 105 inside it -> +1.
        let mut h = HaramiCross::new();
        h.update(solid(110.0, 100.0));
        assert_eq!(h.update(doji(105.0)), Some(1.0));
    }

    #[test]
    fn bearish_harami_cross() {
        // prior big bullish body [100, 110]; doji inside -> -1.
        let mut h = HaramiCross::new();
        h.update(solid(100.0, 110.0));
        assert_eq!(h.update(doji(105.0)), Some(-1.0));
    }

    #[test]
    fn doji_outside_body_is_zero() {
        let mut h = HaramiCross::new();
        h.update(solid(110.0, 100.0));
        // doji centred at 120, outside the prior body -> 0.
        assert_eq!(h.update(doji(120.0)), Some(0.0));
    }

    #[test]
    fn non_doji_second_is_zero() {
        let mut h = HaramiCross::new();
        h.update(solid(110.0, 100.0));
        assert_eq!(h.update(solid(104.0, 106.0)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut h = HaramiCross::new();
        h.update(solid(110.0, 100.0));
        h.update(doji(105.0));
        assert!(h.is_ready());
        h.reset();
        assert!(!h.is_ready());
        assert_eq!(h.update(solid(110.0, 100.0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                if i % 2 == 0 {
                    solid(110.0, 100.0)
                } else {
                    doji(105.0)
                }
            })
            .collect();
        let batch = HaramiCross::new().batch(&candles);
        let mut b = HaramiCross::new();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
