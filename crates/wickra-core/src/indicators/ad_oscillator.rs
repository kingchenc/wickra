//! Williams Accumulation/Distribution.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Larry Williams' Accumulation/Distribution — a cumulative volume-less price
/// flow that classifies each bar as accumulation or distribution based on its
/// close relative to the previous close, then sums the directional component.
///
/// Williams' definition (1972) uses a *true* high/low that includes the prior
/// close as an anchor — the same idea that motivates true range:
///
/// ```text
/// TR_h_t = max(close_{t−1}, high_t)
/// TR_l_t = min(close_{t−1}, low_t)
/// AD_t   = AD_{t−1} + (close_t − TR_l_t)   if close_t > close_{t−1}   (accumulation)
/// AD_t   = AD_{t−1} + (close_t − TR_h_t)   if close_t < close_{t−1}   (distribution)
/// AD_t   = AD_{t−1}                        if close_t == close_{t−1}  (no change)
/// ```
///
/// Unlike Chaikin's Accumulation/Distribution Line, the Williams A/D ignores
/// volume entirely — Williams argued that the relative position of the close
/// already encodes the day's "true" buying or selling pressure. The series is
/// unbounded and used primarily for divergence analysis. The first candle only
/// seeds the previous close; the first emission lands at bar 2.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, AdOscillator};
///
/// let mut indicator = AdOscillator::new();
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
pub struct AdOscillator {
    prev_close: Option<f64>,
    total: f64,
    has_emitted: bool,
}

impl AdOscillator {
    /// Construct a new Williams A/D starting at zero.
    pub const fn new() -> Self {
        Self {
            prev_close: None,
            total: 0.0,
            has_emitted: false,
        }
    }

    /// Current cumulative value if at least one emission has happened.
    pub const fn value(&self) -> Option<f64> {
        if self.has_emitted {
            Some(self.total)
        } else {
            None
        }
    }
}

impl Indicator for AdOscillator {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev) = self.prev_close else {
            // The first bar only establishes the previous close anchor.
            self.prev_close = Some(candle.close);
            return None;
        };
        let delta = if candle.close > prev {
            // Accumulation: distance from the true low.
            let tr_l = prev.min(candle.low);
            candle.close - tr_l
        } else if candle.close < prev {
            // Distribution: distance from the true high (negative).
            let tr_h = prev.max(candle.high);
            candle.close - tr_h
        } else {
            // Unchanged close contributes nothing.
            0.0
        };
        self.total += delta;
        self.prev_close = Some(candle.close);
        self.has_emitted = true;
        Some(self.total)
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.total = 0.0;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        // One seed bar; the second bar is the first emission.
        2
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "WilliamsAD"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(open: f64, high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(open, high, low, close, 100.0, ts).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let ad = AdOscillator::new();
        assert_eq!(ad.name(), "WilliamsAD");
        assert_eq!(ad.warmup_period(), 2);
        assert_eq!(ad.value(), None);
    }

    #[test]
    fn first_bar_only_seeds() {
        let mut ad = AdOscillator::new();
        assert_eq!(ad.update(c(10.0, 11.0, 9.0, 10.0, 0)), None);
        assert!(!ad.is_ready());
    }

    #[test]
    fn accumulation_adds_distance_from_true_low() {
        // prev close = 10, today low = 8, today close = 12 (up day).
        //   TR_l = min(10, 8) = 8, delta = 12 - 8 = 4. AD = 0 + 4 = 4.
        let mut ad = AdOscillator::new();
        ad.update(c(10.0, 11.0, 9.0, 10.0, 0));
        let v = ad.update(c(11.0, 13.0, 8.0, 12.0, 1)).unwrap();
        assert_relative_eq!(v, 4.0, epsilon = 1e-12);
    }

    #[test]
    fn distribution_adds_distance_from_true_high() {
        // prev close = 10, today high = 11, today close = 7 (down day).
        //   TR_h = max(10, 11) = 11, delta = 7 - 11 = -4. AD = -4.
        let mut ad = AdOscillator::new();
        ad.update(c(10.0, 11.0, 9.0, 10.0, 0));
        let v = ad.update(c(10.0, 11.0, 7.0, 7.0, 1)).unwrap();
        assert_relative_eq!(v, -4.0, epsilon = 1e-12);
    }

    #[test]
    fn unchanged_close_keeps_total() {
        // close equals prev close -> no contribution.
        let mut ad = AdOscillator::new();
        ad.update(c(10.0, 11.0, 9.0, 10.0, 0));
        let v = ad.update(c(10.0, 12.0, 8.0, 10.0, 1)).unwrap();
        assert_relative_eq!(v, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn constant_series_yields_zero() {
        // Every close equals the previous -> AD stays at zero forever.
        let candles: Vec<Candle> = (0..40).map(|i| c(10.0, 11.0, 9.0, 10.0, i)).collect();
        let mut ad = AdOscillator::new();
        for v in ad.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80i64)
            .map(|i| {
                let f = i as f64;
                let mid = 100.0 + (f * 0.3).sin() * 5.0;
                c(mid, mid + 2.0, mid - 2.0, mid + 0.5, i)
            })
            .collect();
        let mut a = AdOscillator::new();
        let mut b = AdOscillator::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut ad = AdOscillator::new();
        ad.batch(&[
            c(10.0, 11.0, 9.0, 10.0, 0),
            c(10.0, 12.0, 9.0, 11.0, 1),
            c(11.0, 13.0, 10.0, 12.0, 2),
        ]);
        assert!(ad.is_ready());
        ad.reset();
        assert!(!ad.is_ready());
        assert_eq!(ad.value(), None);
        assert_eq!(ad.update(c(10.0, 11.0, 9.0, 10.0, 3)), None);
    }
}
