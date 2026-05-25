//! Positive Volume Index.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Default starting value; matches Norman Fosback's textbook convention.
const STARTING_INDEX: f64 = 1000.0;

/// Positive Volume Index (Paul Dysart, popularised by Norman Fosback).
///
/// The PVI only updates when **volume expands** — Fosback's interpretation is
/// that the crowd ("uninformed money") trades on volume spikes, so the PVI
/// tracks the crowd-driven leg of price action. When today's volume is at or
/// below yesterday's, the PVI is left unchanged.
///
/// ```text
/// PVI_t = PVI_{t−1} · (1 + (close_t − close_{t−1}) / close_{t−1})   if volume_t > volume_{t−1}
/// PVI_t = PVI_{t−1}                                                  otherwise
/// ```
///
/// The first bar establishes the baseline at `1000.0`. A bar whose previous
/// close is zero contributes no return.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Pvi};
///
/// let mut indicator = Pvi::new();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Pvi {
    prev_close: Option<f64>,
    prev_volume: Option<f64>,
    index: f64,
    has_emitted: bool,
}

impl Pvi {
    /// Construct a new PVI starting at `1000.0`.
    pub const fn new() -> Self {
        Self {
            prev_close: None,
            prev_volume: None,
            index: STARTING_INDEX,
            has_emitted: false,
        }
    }

    /// Construct a new PVI with a custom starting baseline.
    pub const fn with_baseline(baseline: f64) -> Self {
        Self {
            prev_close: None,
            prev_volume: None,
            index: baseline,
            has_emitted: false,
        }
    }

    /// Current cumulative value if at least one candle has been ingested.
    pub const fn value(&self) -> Option<f64> {
        if self.has_emitted {
            Some(self.index)
        } else {
            None
        }
    }
}

impl Default for Pvi {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for Pvi {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        if let (Some(pc), Some(pv)) = (self.prev_close, self.prev_volume) {
            if candle.volume > pv && pc != 0.0 {
                let ret = (candle.close - pc) / pc;
                self.index += self.index * ret;
            }
        }
        self.prev_close = Some(candle.close);
        self.prev_volume = Some(candle.volume);
        self.has_emitted = true;
        Some(self.index)
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.prev_volume = None;
        self.index = STARTING_INDEX;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "PVI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(close: f64, volume: f64, ts: i64) -> Candle {
        Candle::new(close, close, close, close, volume, ts).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let mut p = Pvi::new();
        assert_eq!(p.warmup_period(), 1);
        assert_eq!(p.name(), "PVI");
        assert_eq!(p.value(), None);
        p.update(c(10.0, 100.0, 0));
        assert_eq!(p.value(), Some(1000.0));
    }

    #[test]
    fn default_matches_new() {
        let a = Pvi::default();
        let b = Pvi::new();
        assert_eq!(a.warmup_period(), b.warmup_period());
        assert_eq!(a.value(), b.value());
        assert_eq!(a.is_ready(), b.is_ready());
    }

    #[test]
    fn first_bar_seeds_baseline() {
        let mut p = Pvi::new();
        assert_relative_eq!(
            p.update(c(10.0, 100.0, 0)).unwrap(),
            1000.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn volume_rise_applies_percent_change() {
        // 1000 * (1 + (11 - 10)/10) = 1100.
        let mut p = Pvi::new();
        p.update(c(10.0, 100.0, 0));
        let v = p.update(c(11.0, 200.0, 1)).unwrap();
        assert_relative_eq!(v, 1100.0, epsilon = 1e-12);
    }

    #[test]
    fn volume_fall_leaves_index_unchanged() {
        let mut p = Pvi::new();
        p.update(c(10.0, 200.0, 0));
        let v = p.update(c(11.0, 100.0, 1)).unwrap();
        assert_relative_eq!(v, 1000.0, epsilon = 1e-12);
    }

    #[test]
    fn equal_volume_leaves_index_unchanged() {
        let mut p = Pvi::new();
        p.update(c(10.0, 100.0, 0));
        let v = p.update(c(11.0, 100.0, 1)).unwrap();
        assert_relative_eq!(v, 1000.0, epsilon = 1e-12);
    }

    #[test]
    fn zero_previous_close_contributes_no_return() {
        let mut p = Pvi::new();
        p.update(c(0.0, 100.0, 0));
        let v = p.update(c(5.0, 200.0, 1)).unwrap();
        assert_relative_eq!(v, 1000.0, epsilon = 1e-12);
    }

    #[test]
    fn custom_baseline() {
        let mut p = Pvi::with_baseline(100.0);
        assert_relative_eq!(p.update(c(10.0, 100.0, 0)).unwrap(), 100.0, epsilon = 1e-12);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80i64)
            .map(|i| {
                let f = i as f64;
                c(
                    100.0 + (f * 0.3).sin() * 5.0,
                    50.0 + ((i % 7) as f64) * 10.0,
                    i,
                )
            })
            .collect();
        let mut a = Pvi::new();
        let mut b = Pvi::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut p = Pvi::new();
        p.batch(&[c(10.0, 100.0, 0), c(11.0, 200.0, 1)]);
        assert!(p.is_ready());
        p.reset();
        assert!(!p.is_ready());
        assert_eq!(p.value(), None);
    }
}
