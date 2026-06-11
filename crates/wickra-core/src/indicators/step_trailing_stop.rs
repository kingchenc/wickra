//! Step Trailing Stop.

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Step Trailing Stop — a stop that ratchets in fixed-size discrete steps and
/// flips to the opposite side on a close-through.
///
/// ```text
/// long:   target = close − step_size
///         stop_t = max(stop_{t−1}, floor(target / step_size) · step_size)
///                  while close ≥ stop_{t−1}
/// short:  target = close + step_size
///         stop_t = min(stop_{t−1}, ceil(target / step_size) · step_size)
///                  while close ≤ stop_{t−1}
/// flip-to-long  on close > prev short-stop -> stop = floor((close − step) / step) · step
/// flip-to-short on close < prev long-stop  -> stop = ceil((close + step) / step) · step
/// ```
///
/// Quantising the stop to a multiple of `step_size` keeps the level on a
/// round-number grid, which mirrors how many discretionary traders move stops
/// by hand (in $0.50, $1, or 10-pip increments). The first input seeds a long
/// stop one step below the snapped close.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, StepTrailingStop};
///
/// let mut indicator = StepTrailingStop::new(1.0).unwrap();
/// let mut last = None;
/// for i in 0..20 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct StepTrailingStop {
    step_size: f64,
    prev_stop: Option<f64>,
    long: bool,
}

impl StepTrailingStop {
    /// Construct a Step Trailing Stop with an explicit step size.
    ///
    /// # Errors
    /// Returns [`Error::NonPositiveMultiplier`] if `step_size` is not strictly
    /// positive and finite.
    pub fn new(step_size: f64) -> Result<Self> {
        if !step_size.is_finite() || step_size <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            step_size,
            prev_stop: None,
            long: true,
        })
    }

    /// A common configuration: a `1.0` step size.
    pub fn classic() -> Self {
        Self::new(1.0).expect("classic step is valid")
    }

    /// Configured step size.
    pub const fn step_size(&self) -> f64 {
        self.step_size
    }

    /// Snap `value` down to the nearest `step_size`-grid line below it.
    fn snap_long(&self, close: f64) -> f64 {
        ((close - self.step_size) / self.step_size).floor() * self.step_size
    }

    /// Snap `value` up to the nearest `step_size`-grid line above it.
    fn snap_short(&self, close: f64) -> f64 {
        ((close + self.step_size) / self.step_size).ceil() * self.step_size
    }
}

impl Indicator for StepTrailingStop {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, close: f64) -> Option<f64> {
        if !close.is_finite() {
            return None;
        }
        let stop = match self.prev_stop {
            Some(prev) => {
                if self.long {
                    if close < prev {
                        self.long = false;
                        self.snap_short(close)
                    } else {
                        prev.max(self.snap_long(close))
                    }
                } else if close > prev {
                    self.long = true;
                    self.snap_long(close)
                } else {
                    prev.min(self.snap_short(close))
                }
            }
            None => self.snap_long(close),
        };
        self.prev_stop = Some(stop);
        Some(stop)
    }

    fn reset(&mut self) {
        self.prev_stop = None;
        self.long = true;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.prev_stop.is_some()
    }

    fn name(&self) -> &'static str {
        "StepTrailingStop"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_invalid_step() {
        assert!(StepTrailingStop::new(0.0).is_err());
        assert!(StepTrailingStop::new(-1.0).is_err());
        assert!(StepTrailingStop::new(f64::NAN).is_err());
    }

    #[test]
    fn accessors_and_metadata() {
        let s = StepTrailingStop::classic();
        assert_relative_eq!(s.step_size(), 1.0, epsilon = 1e-12);
        assert_eq!(s.name(), "StepTrailingStop");
        assert_eq!(s.warmup_period(), 1);
    }

    #[test]
    fn first_value_snaps_below_price() {
        let mut s = StepTrailingStop::new(1.0).unwrap();
        // floor((100.4 - 1) / 1) · 1 = 99.
        assert_relative_eq!(s.update(100.4).unwrap(), 99.0, epsilon = 1e-12);
    }

    #[test]
    fn long_stop_ratchets_in_discrete_steps() {
        let mut s = StepTrailingStop::new(1.0).unwrap();
        let out: Vec<f64> = [100.0, 100.5, 101.0, 102.0, 103.5]
            .iter()
            .map(|&p| s.update(p).unwrap())
            .collect();
        // 100 -> 99, 100.5 -> 99 (no advance), 101 -> 100, 102 -> 101, 103.5 -> 102.
        assert_relative_eq!(out[0], 99.0, epsilon = 1e-9);
        assert_relative_eq!(out[1], 99.0, epsilon = 1e-9);
        assert_relative_eq!(out[2], 100.0, epsilon = 1e-9);
        assert_relative_eq!(out[3], 101.0, epsilon = 1e-9);
        assert_relative_eq!(out[4], 102.0, epsilon = 1e-9);
    }

    #[test]
    fn flips_to_short_on_close_through_and_back() {
        let mut s = StepTrailingStop::new(1.0).unwrap();
        s.update(100.0); // 99
        s.update(105.0); // 104
        let flipped = s.update(50.0).unwrap();
        // ceil((50+1)/1)·1 = 51.
        assert_relative_eq!(flipped, 51.0, epsilon = 1e-9);
        // Rally back through 51 -> flip long at 99.
        let back = s.update(100.0).unwrap();
        assert_relative_eq!(back, 99.0, epsilon = 1e-9);
    }

    #[test]
    fn short_stop_ratchets_down() {
        let mut s = StepTrailingStop::new(1.0).unwrap();
        s.update(100.0);
        s.update(50.0); // short at 51
        let v = s.update(40.0).unwrap();
        // ceil((40+1)/1)·1 = 41 -> min(51, 41) = 41.
        assert_relative_eq!(v, 41.0, epsilon = 1e-9);
    }

    #[test]
    fn constant_series_holds_stop() {
        let mut s = StepTrailingStop::new(1.0).unwrap();
        let out = s.batch(&[100.0; 30]);
        for v in out.into_iter().flatten() {
            assert_relative_eq!(v, 99.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut s = StepTrailingStop::new(1.0).unwrap();
        s.update(100.0);
        s.update(50.0);
        assert!(s.is_ready());
        s.reset();
        assert!(!s.is_ready());
        assert_relative_eq!(s.update(200.0).unwrap(), 199.0, epsilon = 1e-12);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..80)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 8.0)
            .collect();
        let mut a = StepTrailingStop::classic();
        let mut b = StepTrailingStop::classic();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
