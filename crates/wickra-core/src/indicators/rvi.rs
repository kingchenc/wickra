//! Relative Vigor Index (RVI).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Relative Vigor Index — Donald Dorsey's ratio of intra-bar drive (close − open)
/// to intra-bar range (high − low), averaged over a `period`-bar window.
///
/// The reading is `SMA(close − open, period) / SMA(high − low, period)`. A
/// positive value means the average bar in the window closed above where it
/// opened (bullish "vigor"); a negative value means the average closed below.
/// The denominator's rolling-window SMA can fall to zero on a perfectly flat
/// stretch, in which case the recurrence is undefined and the indicator holds
/// its previous value.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Rvi};
///
/// let mut rvi = Rvi::new(10).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let o = 100.0 + f64::from(i);
///     let c = o + 0.5;
///     let candle = Candle::new(o, c + 0.2, o - 0.2, c, 1.0, i64::from(i)).unwrap();
///     last = rvi.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Rvi {
    period: usize,
    window: VecDeque<(f64, f64)>,
    sum_num: f64,
    sum_den: f64,
    current: Option<f64>,
}

impl Rvi {
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
            sum_num: 0.0,
            sum_den: 0.0,
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

impl Indicator for Rvi {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let num = candle.close - candle.open;
        let den = candle.high - candle.low;
        if self.window.len() == self.period {
            let (old_n, old_d) = self.window.pop_front().expect("window is non-empty");
            self.sum_num -= old_n;
            self.sum_den -= old_d;
        }
        self.window.push_back((num, den));
        self.sum_num += num;
        self.sum_den += den;
        if self.window.len() < self.period {
            return None;
        }
        if self.sum_den <= 0.0 {
            // Window of perfectly flat (zero-range) bars: ratio undefined.
            // Hold the previous value rather than emitting NaN / inf.
            return self.current;
        }
        let value = self.sum_num / self.sum_den;
        self.current = Some(value);
        Some(value)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum_num = 0.0;
        self.sum_den = 0.0;
        self.current = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.current.is_some()
    }

    fn name(&self) -> &'static str {
        "RVI"
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
    fn rejects_zero_period() {
        assert!(matches!(Rvi::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let mut r = Rvi::new(10).unwrap();
        assert_eq!(r.period(), 10);
        assert_eq!(r.warmup_period(), 10);
        assert_eq!(r.name(), "RVI");
        assert_eq!(r.value(), None);
        for i in 0..10 {
            r.update(candle(10.0, 11.0, 9.0, 10.5, i));
        }
        assert!(r.value().is_some());
    }

    #[test]
    fn reference_value_period_2() {
        // Two bars with (open, high, low, close) = (10, 11, 9, 10.5) and
        // (10.5, 11.5, 10, 11). Per bar:
        //   num1 = 0.5, num2 = 0.5; sum = 1.0
        //   den1 = 2.0, den2 = 1.5; sum = 3.5
        //   RVI = 1.0 / 3.5 ≈ 0.2857142857
        let mut r = Rvi::new(2).unwrap();
        assert_eq!(r.update(candle(10.0, 11.0, 9.0, 10.5, 0)), None);
        let v = r.update(candle(10.5, 11.5, 10.0, 11.0, 1)).unwrap();
        assert_relative_eq!(v, 1.0 / 3.5, epsilon = 1e-12);
    }

    #[test]
    fn warmup_emits_first_value_at_period() {
        let mut r = Rvi::new(3).unwrap();
        for i in 0..2 {
            assert_eq!(r.update(candle(10.0, 11.0, 9.0, 10.5, i)), None);
        }
        assert!(r.update(candle(10.5, 11.5, 10.0, 11.0, 2)).is_some());
    }

    #[test]
    fn pure_uptrend_is_positive() {
        // Every bar closes above its open and has a non-zero range: RVI > 0.
        let mut r = Rvi::new(5).unwrap();
        for i in 0..10 {
            let o = 10.0 + f64::from(i);
            let c = o + 0.5;
            r.update(candle(o, c + 0.2, o - 0.2, c, i64::from(i)));
        }
        let v = r.value().unwrap();
        assert!(v > 0.0, "uptrend RVI should be positive: {v}");
    }

    #[test]
    fn zero_range_window_holds_value() {
        // Window of perfectly flat bars (high == low): ratio undefined,
        // indicator holds.
        let mut r = Rvi::new(3).unwrap();
        r.update(candle(10.0, 10.0, 10.0, 10.0, 0));
        r.update(candle(10.0, 10.0, 10.0, 10.0, 1));
        assert_eq!(r.update(candle(10.0, 10.0, 10.0, 10.0, 2)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40_i64)
            .map(|i| {
                let o = 100.0 + (i as f64 * 0.3).sin() * 5.0;
                let c = o + (i as f64 * 0.1).cos();
                candle(o, o.max(c) + 0.5, o.min(c) - 0.5, c, i)
            })
            .collect();
        let batch = Rvi::new(10).unwrap().batch(&candles);
        let mut b = Rvi::new(10).unwrap();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn reset_clears_state() {
        let mut r = Rvi::new(5).unwrap();
        for i in 0..10 {
            r.update(candle(10.0, 11.0, 9.0, 10.5, i));
        }
        assert!(r.is_ready());
        r.reset();
        assert!(!r.is_ready());
        assert_eq!(r.update(candle(10.0, 11.0, 9.0, 10.5, 0)), None);
    }
}
