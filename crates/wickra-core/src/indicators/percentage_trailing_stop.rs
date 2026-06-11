//! Percentage Trailing Stop.

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Percentage Trailing Stop — a fixed-percentage stop that ratchets with the
/// trend and flips to the opposite side on a close-through.
///
/// ```text
/// step = price · percent / 100
///
/// long:   stop_t = max(stop_{t−1}, close − step)   while close ≥ stop_{t−1}
/// short:  stop_t = min(stop_{t−1}, close + step)   while close ≤ stop_{t−1}
/// flip-to-long  on a close > prev short-stop -> stop = close − step
/// flip-to-short on a close < prev long-stop  -> stop = close + step
/// ```
///
/// The first input seeds a long stop `percent` below price. Unlike the ATR
/// trailing stop the band is purely proportional to the latest price, so it
/// scales naturally across instruments without re-tuning. A common
/// configuration is `5.0` (a 5% trail).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, PercentageTrailingStop};
///
/// let mut indicator = PercentageTrailingStop::new(5.0).unwrap();
/// let mut last = None;
/// for i in 0..20 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct PercentageTrailingStop {
    percent: f64,
    prev_stop: Option<f64>,
    /// `true` while a long trail is active; flipped on a close-through.
    long: bool,
}

impl PercentageTrailingStop {
    /// Construct a Percentage Trailing Stop with an explicit trail size
    /// (e.g. `5.0` for a 5% trail).
    ///
    /// # Errors
    /// Returns [`Error::NonPositiveMultiplier`] if `percent` is not strictly
    /// positive and finite.
    pub fn new(percent: f64) -> Result<Self> {
        if !percent.is_finite() || percent <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            percent,
            prev_stop: None,
            long: true,
        })
    }

    /// A common configuration: a `5.0%` trail.
    pub fn classic() -> Self {
        Self::new(5.0).expect("classic percent is valid")
    }

    /// Configured trail percent.
    pub const fn percent(&self) -> f64 {
        self.percent
    }
}

impl Indicator for PercentageTrailingStop {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, close: f64) -> Option<f64> {
        if !close.is_finite() {
            return None;
        }
        let step = close.abs() * self.percent / 100.0;
        let stop = match self.prev_stop {
            Some(prev) => {
                if self.long {
                    if close < prev {
                        // Close-through long stop — flip to short.
                        self.long = false;
                        close + step
                    } else {
                        prev.max(close - step)
                    }
                } else if close > prev {
                    // Close-through short stop — flip to long.
                    self.long = true;
                    close - step
                } else {
                    prev.min(close + step)
                }
            }
            None => close - step,
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
        "PercentageTrailingStop"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_invalid_percent() {
        assert!(PercentageTrailingStop::new(0.0).is_err());
        assert!(PercentageTrailingStop::new(-1.0).is_err());
        assert!(PercentageTrailingStop::new(f64::NAN).is_err());
        assert!(PercentageTrailingStop::new(f64::INFINITY).is_err());
    }

    #[test]
    fn accessors_and_metadata() {
        let s = PercentageTrailingStop::classic();
        assert_relative_eq!(s.percent(), 5.0, epsilon = 1e-12);
        assert_eq!(s.name(), "PercentageTrailingStop");
        assert_eq!(s.warmup_period(), 1);
    }

    #[test]
    fn first_value_is_step_below_price() {
        let mut s = PercentageTrailingStop::new(10.0).unwrap();
        // 100 · 0.10 = 10, so seed stop = 90.
        assert_relative_eq!(s.update(100.0).unwrap(), 90.0, epsilon = 1e-12);
        assert!(s.is_ready());
    }

    #[test]
    fn long_stop_ratchets_up_with_price() {
        let mut s = PercentageTrailingStop::new(10.0).unwrap();
        // Rising series: each new high pulls the stop up.
        let prices = [100.0, 110.0, 120.0, 130.0];
        let out: Vec<f64> = prices.iter().map(|&p| s.update(p).unwrap()).collect();
        assert_relative_eq!(out[0], 90.0, epsilon = 1e-9);
        assert_relative_eq!(out[1], 99.0, epsilon = 1e-9);
        assert_relative_eq!(out[2], 108.0, epsilon = 1e-9);
        assert_relative_eq!(out[3], 117.0, epsilon = 1e-9);
    }

    #[test]
    fn flips_to_short_on_close_through() {
        let mut s = PercentageTrailingStop::new(10.0).unwrap();
        // Seed long at 100 -> stop 90.
        s.update(100.0);
        // Climb to 130 -> stop ratchets to 117.
        s.update(130.0);
        // Crash to 50 -> closes through 117 -> flip to short stop at 55.
        let flipped = s.update(50.0).unwrap();
        assert_relative_eq!(flipped, 55.0, epsilon = 1e-9);
    }

    #[test]
    fn short_stop_ratchets_down_and_flips_back() {
        let mut s = PercentageTrailingStop::new(10.0).unwrap();
        s.update(100.0); // long stop 90
        s.update(50.0); // flip short, stop 55
        let v = s.update(40.0).unwrap();
        // Short side ratchets down: min(55, 44) = 44.
        assert_relative_eq!(v, 44.0, epsilon = 1e-9);
        // Rally back through 44 -> flip long, stop = 100 · 0.9 = 90.
        let v = s.update(100.0).unwrap();
        assert_relative_eq!(v, 90.0, epsilon = 1e-9);
    }

    #[test]
    fn constant_series_holds_stop() {
        let mut s = PercentageTrailingStop::new(5.0).unwrap();
        let out = s.batch(&[100.0; 30]);
        for v in out.into_iter().flatten() {
            assert_relative_eq!(v, 95.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut s = PercentageTrailingStop::new(5.0).unwrap();
        s.update(100.0);
        s.update(50.0); // flip short
        assert!(s.is_ready());
        s.reset();
        assert!(!s.is_ready());
        // After reset the next update seeds a fresh long stop.
        let v = s.update(200.0).unwrap();
        assert_relative_eq!(v, 190.0, epsilon = 1e-12);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..80)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 8.0)
            .collect();
        let mut a = PercentageTrailingStop::classic();
        let mut b = PercentageTrailingStop::classic();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
