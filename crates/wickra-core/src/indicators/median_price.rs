//! Median Price.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Median Price — the bar's `(high + low) / 2`.
///
/// The midpoint of the bar's range, ignoring where it opened or closed. It is
/// the price series Bill Williams' [`AwesomeOscillator`](crate::AwesomeOscillator)
/// is built on, and a smoother stand-in for the close when feeding other
/// indicators. As a stateless per-bar transform it emits a value from the
/// very first candle.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, MedianPrice};
///
/// let mut indicator = MedianPrice::new();
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
pub struct MedianPrice {
    has_emitted: bool,
}

impl MedianPrice {
    /// Construct a new Median Price transform.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for MedianPrice {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        Some(candle.median_price())
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
        "MedianPrice"
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
        // (high + low) / 2 = (12 + 8) / 2 = 10.
        let mut mp = MedianPrice::new();
        assert_relative_eq!(
            mp.update(candle(10.0, 12.0, 8.0, 11.0, 0)).unwrap(),
            10.0,
            epsilon = 1e-12
        );
    }

    /// Cover the Indicator-impl `name` body (62-64).
    #[test]
    fn name_metadata() {
        let mp = MedianPrice::new();
        assert_eq!(mp.name(), "MedianPrice");
    }

    #[test]
    fn emits_from_first_candle() {
        let mut mp = MedianPrice::new();
        assert_eq!(mp.warmup_period(), 1);
        assert!(!mp.is_ready());
        assert!(mp.update(candle(10.0, 11.0, 9.0, 10.0, 0)).is_some());
        assert!(mp.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut mp = MedianPrice::new();
        mp.update(candle(10.0, 11.0, 9.0, 10.0, 0));
        assert!(mp.is_ready());
        mp.reset();
        assert!(!mp.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                candle(base, base + 2.0, base - 2.0, base + 1.0, i)
            })
            .collect();
        let mut a = MedianPrice::new();
        let mut b = MedianPrice::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
