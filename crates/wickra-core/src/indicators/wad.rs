//! Williams Accumulation/Distribution (WAD) — Larry Williams' cumulative line.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Williams Accumulation/Distribution — a cumulative price-only line that adds
/// the day's accumulation on up-closes and subtracts the day's distribution on
/// down-closes.
///
/// ```text
/// if close > prev_close:  AD =  close − min(low,  prev_close)   (true low)
/// if close < prev_close:  AD =  close − max(high, prev_close)   (true high)
/// if close = prev_close:  AD =  0
/// WAD_t = WAD_{t−1} + AD
/// ```
///
/// Larry Williams' A/D line (distinct from Chaikin's volume-based
/// [`Adl`](crate::Adl)) uses **no volume at all** — it measures accumulation as
/// how far price closed above the *true low* on up-days and distribution as how
/// far it closed below the *true high* on down-days, then accumulates the result.
/// A rising WAD that diverges from a flat or falling price is the classic
/// accumulation signal; a falling WAD under a rising price warns of distribution.
///
/// The line is unbounded and its absolute level is meaningless — only its slope
/// and divergences against price matter. The first candle has no previous close,
/// so it seeds the reference and emits nothing; thereafter every bar emits the
/// running total. Each `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Wad};
///
/// let mut indicator = Wad::new();
/// let mut last = None;
/// for i in 0..20 {
///     let base = 100.0 + f64::from(i);
///     let c = Candle::new(base, base + 1.0, base - 1.0, base + 0.5, 1_000.0, 0).unwrap();
///     last = indicator.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct Wad {
    prev_close: Option<f64>,
    line: f64,
    last: Option<f64>,
}

impl Wad {
    /// Construct a new Williams A/D line. The line is parameter-free.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for Wad {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev_close) = self.prev_close else {
            self.prev_close = Some(candle.close);
            return None;
        };
        let ad = if candle.close > prev_close {
            candle.close - candle.low.min(prev_close)
        } else if candle.close < prev_close {
            candle.close - candle.high.max(prev_close)
        } else {
            0.0
        };
        self.line += ad;
        self.prev_close = Some(candle.close);
        self.last = Some(self.line);
        Some(self.line)
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.line = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // The first bar only seeds the reference close; the first value lands on
        // the second bar.
        2
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "Wad"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(high: f64, low: f64, close: f64) -> Candle {
        Candle::new_unchecked(low, high, low, close, 1_000.0, 0)
    }

    #[test]
    fn accessors_and_metadata() {
        let wad = Wad::new();
        assert_eq!(wad.warmup_period(), 2);
        assert_eq!(wad.name(), "Wad");
        assert!(!wad.is_ready());
        assert_eq!(wad.value(), None);
    }

    #[test]
    fn first_bar_seeds_without_output() {
        let mut wad = Wad::new();
        assert_eq!(wad.update(candle(101.0, 99.0, 100.0)), None);
        assert!(wad.update(candle(102.0, 100.0, 101.0)).is_some());
    }

    #[test]
    fn up_close_accumulates() {
        // close rises from 100 -> 101; true low = min(low, prev_close) = min(100,100)=100;
        // AD = 101 - 100 = 1.
        let mut wad = Wad::new();
        wad.update(candle(101.0, 99.0, 100.0));
        let v = wad.update(candle(102.0, 100.0, 101.0)).unwrap();
        assert_relative_eq!(v, 1.0, epsilon = 1e-9);
    }

    #[test]
    fn down_close_distributes() {
        // close falls 100 -> 99; true high = max(high, prev_close) = max(101,100)=101;
        // AD = 99 - 101 = -2.
        let mut wad = Wad::new();
        wad.update(candle(102.0, 100.0, 100.0));
        let v = wad.update(candle(101.0, 98.0, 99.0)).unwrap();
        assert_relative_eq!(v, -2.0, epsilon = 1e-9);
    }

    #[test]
    fn unchanged_close_adds_nothing() {
        let mut wad = Wad::new();
        wad.update(candle(101.0, 99.0, 100.0));
        let v = wad.update(candle(105.0, 95.0, 100.0)).unwrap();
        assert_relative_eq!(v, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn pure_uptrend_is_monotone() {
        let mut wad = Wad::new();
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                let base = 100.0 + f64::from(i);
                candle(base + 1.0, base - 1.0, base)
            })
            .collect();
        let mut prev = f64::NEG_INFINITY;
        for v in wad.batch(&candles).into_iter().flatten() {
            assert!(v >= prev, "WAD must rise in an uptrend");
            prev = v;
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut wad = Wad::new();
        let candles: Vec<Candle> = (0..10)
            .map(|i| {
                let base = 100.0 + f64::from(i);
                candle(base + 1.0, base - 1.0, base)
            })
            .collect();
        wad.batch(&candles);
        assert!(wad.is_ready());
        wad.reset();
        assert!(!wad.is_ready());
        assert_eq!(wad.value(), None);
        assert_eq!(wad.update(candle(101.0, 99.0, 100.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.3).sin() * 8.0;
                candle(base + 2.0, base - 2.0, base + 0.5)
            })
            .collect();
        let batch = Wad::new().batch(&candles);
        let mut b = Wad::new();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }
}
