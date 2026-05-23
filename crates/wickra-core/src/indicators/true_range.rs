//! True Range.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// True Range — the single-bar building block of every ATR-based indicator.
///
/// ```text
/// TR = max( high − low, |high − close_prev|, |low − close_prev| )
/// ```
///
/// True Range is the greatest of the bar's own range and the two gaps to the
/// previous close, so it captures volatility that opens *between* bars rather
/// than only within them. The first bar has no previous close and falls back
/// to `high − low`. Where [`Atr`](crate::Atr) smooths this series, `TrueRange`
/// exposes it raw, one value per bar.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, TrueRange};
///
/// let mut indicator = TrueRange::new();
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
pub struct TrueRange {
    prev_close: Option<f64>,
    has_emitted: bool,
}

impl TrueRange {
    /// Construct a new True Range indicator.
    pub const fn new() -> Self {
        Self {
            prev_close: None,
            has_emitted: false,
        }
    }
}

impl Indicator for TrueRange {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let tr = candle.true_range(self.prev_close);
        self.prev_close = Some(candle.close);
        self.has_emitted = true;
        Some(tr)
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "TrueRange"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(f64::midpoint(high, low), high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn reference_values() {
        // Bar 1 has no previous close -> TR = high - low = 12 - 8 = 4.
        // Bar 2: prev close 11, TR = max(10-9, |10-11|, |9-11|) = max(1, 1, 2) = 2.
        let mut tr = TrueRange::new();
        let out = tr.batch(&[c(12.0, 8.0, 11.0, 0), c(10.0, 9.0, 9.5, 1)]);
        assert_relative_eq!(out[0].unwrap(), 4.0, epsilon = 1e-12);
        assert_relative_eq!(out[1].unwrap(), 2.0, epsilon = 1e-12);
    }

    /// Cover the Indicator-impl `name` body (73-75).
    #[test]
    fn name_metadata() {
        let tr = TrueRange::new();
        assert_eq!(tr.name(), "TrueRange");
    }

    #[test]
    fn emits_from_first_candle() {
        let mut tr = TrueRange::new();
        assert_eq!(tr.warmup_period(), 1);
        assert!(!tr.is_ready());
        assert!(tr.update(c(11.0, 9.0, 10.0, 0)).is_some());
        assert!(tr.is_ready());
    }

    #[test]
    fn never_negative() {
        let candles: Vec<Candle> = (0..120)
            .map(|i| {
                let base = 100.0 + (i as f64 * 0.3).sin() * 5.0;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut tr = TrueRange::new();
        for v in tr.batch(&candles).into_iter().flatten() {
            assert!(v >= 0.0, "true range must be non-negative, got {v}");
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut tr = TrueRange::new();
        tr.batch(&[c(12.0, 8.0, 10.0, 0), c(13.0, 9.0, 11.0, 1)]);
        assert!(tr.is_ready());
        tr.reset();
        assert!(!tr.is_ready());
        // After reset the next bar again has no previous close.
        assert_relative_eq!(
            tr.update(c(12.0, 8.0, 10.0, 0)).unwrap(),
            4.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.3).sin() * 8.0;
                c(mid + 1.5, mid - 1.5, mid + 0.5, i)
            })
            .collect();
        let mut a = TrueRange::new();
        let mut b = TrueRange::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
