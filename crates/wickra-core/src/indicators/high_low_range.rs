//! High-Low Range — the bar range as a fraction of close.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// High-Low Range — the bar's high-low range expressed as a fraction of its
/// close price.
///
/// ```text
/// HighLowRange = (high − low) / close
/// ```
///
/// A scale-free, single-bar volatility proxy: the absolute range `high − low`
/// grows with the nominal price level, so dividing by the close makes a `2$`
/// range on a `100$` instrument (`0.02`) directly comparable to a `200$` range
/// on a `10000$` one (`0.02`). It is the per-bar cousin of average-true-range
/// style measures without the smoothing — useful as an instant intrabar
/// volatility read or a normaliser for other features. The output is `≥ 0`
/// for positive prices. A zero close carries no scale and yields `0`.
///
/// This is a stateless per-bar transform: every candle produces one value.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, HighLowRange};
///
/// let mut indicator = HighLowRange::new();
/// // range 104 - 98 = 6, close 100 -> 0.06.
/// let c = Candle::new(99.0, 104.0, 98.0, 100.0, 10.0, 0).unwrap();
/// assert!((indicator.update(c).unwrap() - 0.06).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Default)]
pub struct HighLowRange {
    has_emitted: bool,
}

impl HighLowRange {
    /// Construct a new High-Low Range transform.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for HighLowRange {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let out = if candle.close == 0.0 {
            // A zero close carries no scale to normalise the range against.
            0.0
        } else {
            (candle.high - candle.low) / candle.close
        };
        Some(out)
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
        "HighLowRange"
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
        // (104 - 98) / 100 = 0.06.
        let mut hlr = HighLowRange::new();
        assert_relative_eq!(
            hlr.update(candle(99.0, 104.0, 98.0, 100.0, 0)).unwrap(),
            0.06,
            epsilon = 1e-12
        );
    }

    #[test]
    fn zero_range_bar_yields_zero() {
        // high == low -> range 0 -> 0 regardless of close.
        let mut hlr = HighLowRange::new();
        assert_relative_eq!(
            hlr.update(candle(10.0, 10.0, 10.0, 10.0, 0)).unwrap(),
            0.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn zero_close_yields_zero() {
        // Candle permits a zero close (only finiteness + OHLC ordering checked):
        // open 0, high 1, low 0, close 0 satisfies high >= all, low <= all.
        let mut hlr = HighLowRange::new();
        assert_relative_eq!(
            hlr.update(candle(0.0, 1.0, 0.0, 0.0, 0)).unwrap(),
            0.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn output_is_non_negative() {
        let candles: Vec<Candle> = (0..100)
            .map(|i| {
                let mid = 100.0 + (f64::from(i) * 0.2).sin() * 8.0;
                candle(mid, mid + 3.0, mid - 3.0, mid, i64::from(i))
            })
            .collect();
        let mut hlr = HighLowRange::new();
        for v in hlr.batch(&candles).into_iter().flatten() {
            assert!(v >= 0.0, "HighLowRange {v} must be non-negative");
        }
    }

    #[test]
    fn name_metadata() {
        let hlr = HighLowRange::new();
        assert_eq!(hlr.name(), "HighLowRange");
    }

    #[test]
    fn emits_from_first_candle() {
        let mut hlr = HighLowRange::new();
        assert_eq!(hlr.warmup_period(), 1);
        assert!(!hlr.is_ready());
        assert!(hlr.update(candle(10.0, 11.0, 9.0, 10.0, 0)).is_some());
        assert!(hlr.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut hlr = HighLowRange::new();
        hlr.update(candle(10.0, 11.0, 9.0, 10.0, 0));
        assert!(hlr.is_ready());
        hlr.reset();
        assert!(!hlr.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + f64::from(i);
                candle(base, base + 2.0, base - 2.0, base + 1.0, i64::from(i))
            })
            .collect();
        let mut a = HighLowRange::new();
        let mut b = HighLowRange::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
