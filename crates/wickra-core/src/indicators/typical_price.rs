//! Typical Price.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Typical Price — the bar's `(high + low + close) / 3`.
///
/// A single representative price per bar that weights the close no more
/// heavily than the two extremes. It is the price series that
/// [`Cci`](crate::Cci) and [`Mfi`](crate::Mfi) are built on, and a common
/// input to feed other indicators in place of the raw close. As a stateless
/// per-bar transform it emits a value from the very first candle.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, TypicalPrice};
///
/// let mut indicator = TypicalPrice::new();
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
pub struct TypicalPrice {
    has_emitted: bool,
}

impl TypicalPrice {
    /// Construct a new Typical Price transform.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for TypicalPrice {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        Some(candle.typical_price())
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
        "TypicalPrice"
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
        // (high + low + close) / 3 = (12 + 6 + 9) / 3 = 9.
        let mut tp = TypicalPrice::new();
        assert_relative_eq!(
            tp.update(candle(9.0, 12.0, 6.0, 9.0, 0)).unwrap(),
            9.0,
            epsilon = 1e-12
        );
    }

    /// Cover the Indicator-impl `name` body (62-64).
    #[test]
    fn name_metadata() {
        let tp = TypicalPrice::new();
        assert_eq!(tp.name(), "TypicalPrice");
    }

    #[test]
    fn emits_from_first_candle() {
        let mut tp = TypicalPrice::new();
        assert_eq!(tp.warmup_period(), 1);
        assert!(!tp.is_ready());
        assert!(tp.update(candle(10.0, 11.0, 9.0, 10.0, 0)).is_some());
        assert!(tp.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut tp = TypicalPrice::new();
        tp.update(candle(10.0, 11.0, 9.0, 10.0, 0));
        assert!(tp.is_ready());
        tp.reset();
        assert!(!tp.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                candle(base, base + 2.0, base - 2.0, base + 1.0, i)
            })
            .collect();
        let mut a = TypicalPrice::new();
        let mut b = TypicalPrice::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
