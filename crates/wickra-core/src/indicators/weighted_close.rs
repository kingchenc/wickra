//! Weighted Close.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Weighted Close — the bar's `(high + low + 2·close) / 4`.
///
/// A representative per-bar price that, unlike the [`TypicalPrice`](crate::TypicalPrice),
/// gives the close double weight — useful when the closing print matters more
/// than the extremes for your strategy. As a stateless per-bar transform it
/// emits a value from the very first candle.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, WeightedClose};
///
/// let mut indicator = WeightedClose::new();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct WeightedClose {
    has_emitted: bool,
}

impl WeightedClose {
    /// Construct a new Weighted Close transform.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for WeightedClose {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        Some(candle.weighted_close())
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
        "WeightedClose"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(open: f64, high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(open, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn reference_value() {
        // (high + low + 2·close) / 4 = (12 + 8 + 2·11) / 4 = 42 / 4 = 10.5.
        let mut wc = WeightedClose::new();
        assert_relative_eq!(
            wc.update(candle(10.0, 12.0, 8.0, 11.0, 0)).unwrap(),
            10.5,
            epsilon = 1e-12
        );
    }

    /// Cover the Indicator-impl `name` body (61-63).
    #[test]
    fn name_metadata() {
        let wc = WeightedClose::new();
        assert_eq!(wc.name(), "WeightedClose");
    }

    #[test]
    fn emits_from_first_candle() {
        let mut wc = WeightedClose::new();
        assert_eq!(wc.warmup_period(), 1);
        assert!(!wc.is_ready());
        assert!(wc.update(candle(10.0, 11.0, 9.0, 10.0, 0)).is_some());
        assert!(wc.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut wc = WeightedClose::new();
        wc.update(candle(10.0, 11.0, 9.0, 10.0, 0));
        assert!(wc.is_ready());
        wc.reset();
        assert!(!wc.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                candle(base, base + 2.0, base - 2.0, base + 1.0, i)
            })
            .collect();
        let mut a = WeightedClose::new();
        let mut b = WeightedClose::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
