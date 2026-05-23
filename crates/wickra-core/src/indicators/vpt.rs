//! Volume-Price Trend.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Volume-Price Trend — a cumulative volume line weighted by percentage price
/// change.
///
/// VPT is a close relative of [`Obv`](crate::Obv), but instead of adding the
/// full bar volume on every up-close it adds volume scaled by the *size* of
/// the move:
///
/// ```text
/// VPT_t = VPT_{t−1} + volume_t · (close_t − close_{t−1}) / close_{t−1}
/// ```
///
/// A big move on heavy volume moves the line far; a small move on the same
/// volume barely nudges it. The running total is unbounded — its slope and its
/// divergence from price are what carry the signal. The first bar establishes
/// the baseline at `0`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, VolumePriceTrend};
///
/// let mut indicator = VolumePriceTrend::new();
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
pub struct VolumePriceTrend {
    prev_close: Option<f64>,
    total: f64,
    has_emitted: bool,
}

impl VolumePriceTrend {
    /// Construct a new Volume-Price Trend starting at zero.
    pub const fn new() -> Self {
        Self {
            prev_close: None,
            total: 0.0,
            has_emitted: false,
        }
    }

    /// Current cumulative value if at least one candle has been ingested.
    pub const fn value(&self) -> Option<f64> {
        if self.has_emitted {
            Some(self.total)
        } else {
            None
        }
    }
}

impl Indicator for VolumePriceTrend {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let Some(prev) = self.prev_close else {
            // The first candle establishes the baseline at 0.
            self.prev_close = Some(candle.close);
            return Some(self.total);
        };
        let roc = if prev == 0.0 {
            // Undefined ratio against a zero previous close.
            0.0
        } else {
            (candle.close - prev) / prev
        };
        self.total += candle.volume * roc;
        self.prev_close = Some(candle.close);
        Some(self.total)
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.total = 0.0;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "VPT"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(close: f64, volume: f64, ts: i64) -> Candle {
        Candle::new(close, close, close, close, volume, ts).unwrap()
    }

    #[test]
    fn reference_values() {
        // closes [10, 11, 9], volumes [100, 200, 300]:
        //   bar 1: baseline 0.
        //   bar 2: VPT += 200 · (11-10)/10 = 20  -> 20.
        //   bar 3: VPT += 300 · (9-11)/11 = -600/11 -> 20 - 600/11.
        let mut vpt = VolumePriceTrend::new();
        let out = vpt.batch(&[
            candle(10.0, 100.0, 0),
            candle(11.0, 200.0, 1),
            candle(9.0, 300.0, 2),
        ]);
        assert_relative_eq!(out[0].unwrap(), 0.0, epsilon = 1e-12);
        assert_relative_eq!(out[1].unwrap(), 20.0, epsilon = 1e-12);
        assert_relative_eq!(out[2].unwrap(), 20.0 - 600.0 / 11.0, epsilon = 1e-12);
    }

    /// Cover the `value()` Some branch (line 57) and the Indicator-impl
    /// `name` body (100-102). `reset_clears_state` hits only the None
    /// branch; the name was never queried.
    #[test]
    fn accessors_and_metadata() {
        let mut vpt = VolumePriceTrend::new();
        assert_eq!(vpt.name(), "VPT");
        assert_eq!(vpt.value(), None);
        vpt.update(candle(100.0, 50.0, 0));
        assert_eq!(vpt.value(), Some(0.0));
    }

    /// Cover the `prev == 0.0` defensive branch (line 77) — the previous
    /// close is exactly 0, making the percentage ROC undefined. The
    /// indicator must contribute 0 to the running total rather than NaN.
    #[test]
    fn zero_previous_close_contributes_zero() {
        let mut vpt = VolumePriceTrend::new();
        vpt.update(candle(0.0, 100.0, 0)); // baseline; prev_close = 0
        let v = vpt.update(candle(50.0, 200.0, 1)).expect("emits");
        // ROC fallback is 0, so total stays at 0.
        assert_eq!(v, 0.0);
    }

    #[test]
    fn emits_from_first_candle_at_zero() {
        let mut vpt = VolumePriceTrend::new();
        assert_eq!(vpt.warmup_period(), 1);
        assert_eq!(vpt.update(candle(100.0, 50.0, 0)), Some(0.0));
    }

    #[test]
    fn constant_close_keeps_line_flat() {
        // No price change -> no contribution regardless of volume.
        let mut vpt = VolumePriceTrend::new();
        let candles: Vec<Candle> = (0..20).map(|i| candle(100.0, 500.0, i)).collect();
        for v in vpt.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut vpt = VolumePriceTrend::new();
        vpt.batch(&[
            candle(10.0, 100.0, 0),
            candle(11.0, 100.0, 1),
            candle(12.0, 100.0, 2),
        ]);
        assert!(vpt.is_ready());
        vpt.reset();
        assert!(!vpt.is_ready());
        assert_eq!(vpt.value(), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                candle(
                    100.0 + (i as f64 * 0.3).sin() * 8.0,
                    10.0 + (i % 5) as f64,
                    i,
                )
            })
            .collect();
        let batch = VolumePriceTrend::new().batch(&candles);
        let mut b = VolumePriceTrend::new();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }
}
