#![allow(clippy::doc_markdown)]

//! Tristar — a three-doji reversal pattern.
//!
//! A Tristar is three consecutive Doji candles where the middle one gaps away
//! from its neighbours, forming a star. A bearish Tristar (top) has the middle
//! doji sitting above the other two; a bullish Tristar (bottom) has it below.
//!
//! - **Bullish** (`+1.0`): three dojis, the middle doji's body centre below both
//!   neighbours' body centres.
//! - **Bearish** (`-1.0`): three dojis, the middle above both neighbours.
//! - Otherwise the output is `0.0`.
//!
//! A doji is a candle whose body is `<= 0.1 * range`. The three-bar lookback means
//! the first value lands on the third candle.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Body-centre of a candle.
fn body_mid(candle: Candle) -> f64 {
    f64::midpoint(candle.open, candle.close)
}

/// Whether a candle is a doji (body small relative to range).
fn is_doji(candle: Candle) -> bool {
    let body = (candle.close - candle.open).abs();
    let range = candle.high - candle.low;
    range > 0.0 && body <= 0.1 * range
}

/// Tristar — three-doji star reversal detector.
/// # Example
///
/// ```
/// use wickra_core::{Tristar, Candle, Indicator};
///
/// let mut indicator = Tristar::new();
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
pub struct Tristar {
    c1: Option<Candle>,
    c2: Option<Candle>,
    last_value: Option<f64>,
}

impl Tristar {
    /// Construct a new `Tristar`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Latest emitted signal if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for Tristar {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let (Some(first), Some(middle)) = (self.c1, self.c2) else {
            self.c1 = self.c2;
            self.c2 = Some(candle);
            self.last_value = Some(0.0);
            return Some(0.0);
        };
        let v = if is_doji(first) && is_doji(middle) && is_doji(candle) {
            let mid = body_mid(middle);
            let n1 = body_mid(first);
            let n3 = body_mid(candle);
            if mid > n1 && mid > n3 {
                -1.0
            } else if mid < n1 && mid < n3 {
                1.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        self.c1 = self.c2;
        self.c2 = Some(candle);
        self.last_value = Some(v);
        Some(v)
    }

    fn reset(&mut self) {
        self.c1 = None;
        self.c2 = None;
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        3
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "Tristar"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    /// A doji centred at `mid` (tiny body, symmetric shadows).
    fn doji(mid: f64) -> Candle {
        Candle::new_unchecked(mid, mid + 1.0, mid - 1.0, mid + 0.02, 0.0, 0)
    }

    /// A non-doji (big body).
    fn solid(open: f64, close: f64) -> Candle {
        Candle::new_unchecked(
            open,
            open.max(close) + 0.1,
            open.min(close) - 0.1,
            close,
            0.0,
            0,
        )
    }

    #[test]
    fn accessors_and_metadata() {
        let t = Tristar::new();
        assert_eq!(t.warmup_period(), 3);
        assert_eq!(t.name(), "Tristar");
        assert!(!t.is_ready());
        assert_eq!(t.value(), None);
    }

    #[test]
    fn first_two_bars_seed_without_signal() {
        let mut t = Tristar::new();
        assert_eq!(t.update(doji(100.0)), Some(0.0));
        assert_eq!(t.update(doji(100.0)), Some(0.0));
        assert!(t.update(doji(100.0)).is_some());
    }

    #[test]
    fn bearish_tristar_top() {
        // middle doji centred above the two neighbours -> top -> -1.
        let mut t = Tristar::new();
        t.update(doji(100.0));
        t.update(doji(105.0)); // middle, highest
        assert_eq!(t.update(doji(100.0)), Some(-1.0));
    }

    #[test]
    fn bullish_tristar_bottom() {
        let mut t = Tristar::new();
        t.update(doji(100.0));
        t.update(doji(95.0)); // middle, lowest
        assert_eq!(t.update(doji(100.0)), Some(1.0));
    }

    #[test]
    fn non_doji_is_zero() {
        let mut t = Tristar::new();
        t.update(doji(100.0));
        t.update(solid(100.0, 110.0)); // not a doji
        assert_eq!(t.update(doji(100.0)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut t = Tristar::new();
        t.update(doji(100.0));
        t.update(doji(105.0));
        t.update(doji(100.0));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(doji(100.0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| doji(100.0 + (f64::from(i) * 0.4).sin() * 5.0))
            .collect();
        let batch = Tristar::new().batch(&candles);
        let mut b = Tristar::new();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
