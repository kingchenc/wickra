//! Volume-Weighted Moving Average.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Volume-Weighted Moving Average over a rolling window of `period` candles.
///
/// Each close is weighted by its own bar volume:
///
/// ```text
/// VWMA_t = Σ(close_i · volume_i) / Σ(volume_i)   over the last `period` bars
/// ```
///
/// High-volume bars pull the average toward their close, so VWMA reacts to
/// price moves that the market actually participated in and largely ignores
/// thin, low-conviction bars.
///
/// If every candle in the window has zero volume the weighted mean is
/// undefined; the indicator then falls back to the **unweighted** mean of the
/// `period` closes, so the output is always finite. The first output lands
/// after exactly `period` candles.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Vwma};
///
/// let mut indicator = Vwma::new(5).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let p = 100.0 + f64::from(i);
///     let candle = Candle::new(p, p + 1.0, p - 1.0, p, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Vwma {
    period: usize,
    /// Rolling window of `(close, volume)` pairs, oldest at the front.
    window: VecDeque<(f64, f64)>,
    sum_pv: f64,
    sum_v: f64,
    sum_close: f64,
    current: Option<f64>,
}

impl Vwma {
    /// Construct a new VWMA with the given period.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
            sum_pv: 0.0,
            sum_v: 0.0,
            sum_close: 0.0,
            current: None,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.current
    }
}

impl Indicator for Vwma {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let close = candle.close;
        let volume = candle.volume;
        if self.window.len() == self.period {
            let (old_close, old_volume) = self.window.pop_front().expect("window is non-empty");
            self.sum_pv -= old_close * old_volume;
            self.sum_v -= old_volume;
            self.sum_close -= old_close;
        }
        self.window.push_back((close, volume));
        self.sum_pv += close * volume;
        self.sum_v += volume;
        self.sum_close += close;
        if self.window.len() < self.period {
            return None;
        }
        let value = if self.sum_v > 0.0 {
            self.sum_pv / self.sum_v
        } else {
            // Degenerate window: every bar had zero volume. Fall back to the
            // plain mean of the closes so the output stays finite.
            self.sum_close / self.period as f64
        };
        self.current = Some(value);
        Some(value)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum_pv = 0.0;
        self.sum_v = 0.0;
        self.sum_close = 0.0;
        self.current = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.current.is_some()
    }

    fn name(&self) -> &'static str {
        "VWMA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    /// Build a flat candle with a given close and volume.
    fn candle(close: f64, volume: f64, ts: i64) -> Candle {
        Candle::new(close, close, close, close, volume, ts).unwrap()
    }

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(Vwma::new(0), Err(Error::PeriodZero)));
    }

    /// Cover the const accessors `period` / `value` (72-79) and the
    /// Indicator-impl `name` body (129-131). Existing tests inspect
    /// VWMA output but never query the metadata.
    #[test]
    fn accessors_and_metadata() {
        let mut v = Vwma::new(5).unwrap();
        assert_eq!(v.period(), 5);
        assert_eq!(v.name(), "VWMA");
        assert_eq!(v.value(), None);
        for i in 1..=5i64 {
            let p = 100.0 + i as f64;
            v.update(Candle::new(p, p, p, p, 1.0, i).unwrap());
        }
        assert!(v.value().is_some());
    }

    #[test]
    fn reference_value() {
        // VWMA(2): (10·1 + 20·3) / (1 + 3) = 70 / 4 = 17.5.
        let mut vwma = Vwma::new(2).unwrap();
        assert_eq!(vwma.update(candle(10.0, 1.0, 0)), None);
        assert_relative_eq!(
            vwma.update(candle(20.0, 3.0, 1)).unwrap(),
            17.5,
            epsilon = 1e-12
        );
        // Window slides: (20·3 + 30·1) / (3 + 1) = 90 / 4 = 22.5.
        assert_relative_eq!(
            vwma.update(candle(30.0, 1.0, 2)).unwrap(),
            22.5,
            epsilon = 1e-12
        );
    }

    #[test]
    fn zero_volume_window_falls_back_to_unweighted_mean() {
        let mut vwma = Vwma::new(2).unwrap();
        assert_eq!(vwma.update(candle(10.0, 0.0, 0)), None);
        // Both bars have zero volume: fall back to mean(10, 20) = 15.
        assert_relative_eq!(
            vwma.update(candle(20.0, 0.0, 1)).unwrap(),
            15.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn constant_series_yields_the_constant() {
        let mut vwma = Vwma::new(5).unwrap();
        let candles: Vec<Candle> = (0..30).map(|i| candle(42.0, 3.0, i)).collect();
        let out = vwma.batch(&candles);
        for x in out.iter().skip(4).flatten() {
            assert_relative_eq!(*x, 42.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn high_volume_bar_pulls_the_average() {
        // A heavy bar at a higher close drags VWMA above the simple mean.
        let mut vwma = Vwma::new(3).unwrap();
        vwma.update(candle(10.0, 1.0, 0));
        vwma.update(candle(10.0, 1.0, 1));
        let v = vwma.update(candle(20.0, 100.0, 2)).unwrap();
        let simple_mean = (10.0 + 10.0 + 20.0) / 3.0;
        assert!(
            v > simple_mean,
            "{v} should exceed simple mean {simple_mean}"
        );
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut vwma = Vwma::new(4).unwrap();
        assert_eq!(vwma.warmup_period(), 4);
        for i in 0..3 {
            assert_eq!(vwma.update(candle(10.0, 1.0, i)), None);
        }
        assert!(vwma.update(candle(10.0, 1.0, 3)).is_some());
    }

    #[test]
    fn reset_clears_state() {
        let mut vwma = Vwma::new(3).unwrap();
        let candles: Vec<Candle> = (0..10).map(|i| candle(10.0 + i as f64, 2.0, i)).collect();
        vwma.batch(&candles);
        assert!(vwma.is_ready());
        vwma.reset();
        assert!(!vwma.is_ready());
        assert_eq!(vwma.update(candle(10.0, 1.0, 0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..50_i64)
            .map(|i| {
                let c = 100.0 + (i as f64 * 0.3).sin() * 8.0;
                candle(c, 1.0 + (i % 7) as f64, i)
            })
            .collect();
        let batch = Vwma::new(8).unwrap().batch(&candles);
        let mut b = Vwma::new(8).unwrap();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }
}
