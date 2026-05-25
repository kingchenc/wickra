//! Market Facilitation Index (Bill Williams).

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Bill Williams' Market Facilitation Index — how much price movement the
/// market produces per unit of volume.
///
/// ```text
/// MFI_BW_t = (high_t − low_t) / volume_t
/// ```
///
/// A rising MFI on rising volume ("green") signals strong participation behind
/// the move; a rising MFI on falling volume ("fake") suggests a low-volume push
/// that may not hold. Williams pairs MFI with a "Squat" or "Fade" classification
/// against the prior bar's MFI/volume — a downstream concern; this struct only
/// emits the per-bar ratio. A bar with zero volume returns `None` (no
/// facilitation can be defined). Output is emitted from the very first bar.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, MarketFacilitationIndex};
///
/// let mut indicator = MarketFacilitationIndex::new();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 50.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct MarketFacilitationIndex {
    has_emitted: bool,
    last_value: f64,
}

impl MarketFacilitationIndex {
    /// Construct a new Market Facilitation Index.
    pub const fn new() -> Self {
        Self {
            has_emitted: false,
            last_value: 0.0,
        }
    }

    /// Most recent value if at least one bar with non-zero volume has been
    /// observed.
    pub const fn value(&self) -> Option<f64> {
        if self.has_emitted {
            Some(self.last_value)
        } else {
            None
        }
    }
}

impl Indicator for MarketFacilitationIndex {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        if candle.volume == 0.0 {
            // No trade activity -> facilitation is undefined.
            return None;
        }
        let v = (candle.high - candle.low) / candle.volume;
        self.last_value = v;
        self.has_emitted = true;
        Some(v)
    }

    fn reset(&mut self) {
        self.has_emitted = false;
        self.last_value = 0.0;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "MarketFacilitationIndex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(open: f64, high: f64, low: f64, close: f64, volume: f64, ts: i64) -> Candle {
        Candle::new(open, high, low, close, volume, ts).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let m = MarketFacilitationIndex::new();
        assert_eq!(m.name(), "MarketFacilitationIndex");
        assert_eq!(m.warmup_period(), 1);
        assert_eq!(m.value(), None);
    }

    #[test]
    fn reference_value() {
        // (12 − 8) / 200 = 0.02.
        let mut m = MarketFacilitationIndex::new();
        let v = m.update(c(10.0, 12.0, 8.0, 11.0, 200.0, 0)).unwrap();
        assert_relative_eq!(v, 0.02, epsilon = 1e-12);
        assert_relative_eq!(m.value().unwrap(), 0.02, epsilon = 1e-12);
    }

    #[test]
    fn constant_series_is_constant() {
        // Same OHLCV every bar -> same ratio every bar.
        let candles: Vec<Candle> = (0..30)
            .map(|i| c(10.0, 11.0, 9.0, 10.0, 100.0, i))
            .collect();
        let mut m = MarketFacilitationIndex::new();
        for v in m.batch(&candles).into_iter().flatten() {
            // 2/100 = 0.02.
            assert_relative_eq!(v, 0.02, epsilon = 1e-12);
        }
    }

    #[test]
    fn zero_volume_returns_none() {
        let mut m = MarketFacilitationIndex::new();
        assert_eq!(m.update(c(10.0, 11.0, 9.0, 10.0, 0.0, 0)), None);
        assert!(!m.is_ready());
        // Subsequent non-zero-volume bar still works.
        let v = m.update(c(10.0, 12.0, 8.0, 10.0, 100.0, 1)).unwrap();
        assert_relative_eq!(v, 0.04, epsilon = 1e-12);
    }

    #[test]
    fn zero_range_bar_yields_zero() {
        // high == low -> ratio = 0.
        let mut m = MarketFacilitationIndex::new();
        let v = m.update(c(10.0, 10.0, 10.0, 10.0, 100.0, 0)).unwrap();
        assert_relative_eq!(v, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..60i64)
            .map(|i| {
                let f = i as f64;
                let mid = 100.0 + (f * 0.3).sin() * 5.0;
                c(
                    mid,
                    mid + 2.0,
                    mid - 2.0,
                    mid + 0.5,
                    50.0 + (i % 5) as f64,
                    i,
                )
            })
            .collect();
        let mut a = MarketFacilitationIndex::new();
        let mut b = MarketFacilitationIndex::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut m = MarketFacilitationIndex::new();
        m.update(c(10.0, 12.0, 8.0, 11.0, 100.0, 0));
        assert!(m.is_ready());
        m.reset();
        assert!(!m.is_ready());
        assert_eq!(m.value(), None);
    }
}
