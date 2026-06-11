//! Linear Regression Angle.

use crate::error::Result;
use crate::indicators::linreg_slope::LinRegSlope;
use crate::traits::Indicator;

/// Linear Regression Angle — the slope of the rolling least-squares fit,
/// expressed as an angle in degrees.
///
/// ```text
/// LinRegAngle = atan(LinRegSlope) · 180 / π
/// ```
///
/// It carries exactly the same information as [`LinRegSlope`](crate::LinRegSlope)
/// — positive while price trends up, negative while it trends down — but maps
/// the unbounded slope through `atan` onto `(−90°, +90°)`. That bounded,
/// price-unit-free scale makes "how steep is the trend" comparable at a glance
/// and across instruments. This is TA-Lib's `LINEARREG_ANGLE`.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, LinRegAngle};
///
/// let mut indicator = LinRegAngle::new(14).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct LinRegAngle {
    slope: LinRegSlope,
}

impl LinRegAngle {
    /// Construct a new rolling linear-regression angle over `period` inputs.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`](crate::Error::InvalidPeriod) if
    /// `period < 2` — a regression line is undefined for fewer than two points.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            slope: LinRegSlope::new(period)?,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.slope.period()
    }
}

impl Indicator for LinRegAngle {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        self.slope.update(value).map(|s| s.atan().to_degrees())
    }

    fn reset(&mut self) {
        self.slope.reset();
    }

    fn warmup_period(&self) -> usize {
        self.slope.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.slope.is_ready()
    }

    fn name(&self) -> &'static str {
        "LinRegAngle"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn unit_slope_is_forty_five_degrees() {
        // A series rising by exactly 1 per step has slope 1, and atan(1) = 45°.
        let mut angle = LinRegAngle::new(5).unwrap();
        let out = angle.batch(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        for (i, v) in out.iter().enumerate().take(4) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert_relative_eq!(out[4].unwrap(), 45.0, epsilon = 1e-9);
        assert_relative_eq!(out[5].unwrap(), 45.0, epsilon = 1e-9);
    }

    #[test]
    fn reference_value_steep_slope() {
        // period 3 over [1, 2, 9]: slope 4, angle = atan(4) in degrees.
        let mut angle = LinRegAngle::new(3).unwrap();
        let out = angle.batch(&[1.0, 2.0, 9.0]);
        assert_relative_eq!(out[2].unwrap(), 4.0_f64.atan().to_degrees(), epsilon = 1e-9);
    }

    #[test]
    fn constant_series_has_zero_angle() {
        let mut angle = LinRegAngle::new(8).unwrap();
        for v in angle.batch(&[42.0; 20]).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn falling_series_has_negative_angle() {
        let prices: Vec<f64> = (0..30).map(|i| 100.0 - f64::from(i)).collect();
        let mut angle = LinRegAngle::new(10).unwrap();
        for v in angle.batch(&prices).into_iter().flatten() {
            assert!(v < 0.0, "a falling series must have a negative angle");
        }
    }

    #[test]
    fn stays_within_ninety_degrees() {
        let prices: Vec<f64> = (0..60)
            .map(|i| 50.0 + (f64::from(i) * 0.3).sin() * 1000.0)
            .collect();
        let mut angle = LinRegAngle::new(14).unwrap();
        for v in angle.batch(&prices).into_iter().flatten() {
            assert!(v > -90.0 && v < 90.0, "angle {v} outside (-90, 90)");
        }
    }

    #[test]
    fn rejects_period_below_two() {
        assert!(LinRegAngle::new(0).is_err());
        assert!(LinRegAngle::new(1).is_err());
        assert!(LinRegAngle::new(2).is_ok());
    }

    /// Cover the const accessor `period` (50-52) and the Indicator-impl
    /// `warmup_period` (67-69) + `name` (75-77). Existing tests inspect
    /// angle output but never query the metadata.
    #[test]
    fn accessors_and_metadata() {
        let a = LinRegAngle::new(14).unwrap();
        assert_eq!(a.period(), 14);
        assert_eq!(a.warmup_period(), 14);
        assert_eq!(a.name(), "LinRegAngle");
    }

    #[test]
    fn reset_clears_state() {
        let mut angle = LinRegAngle::new(5).unwrap();
        angle.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(angle.is_ready());
        angle.reset();
        assert!(!angle.is_ready());
        assert_eq!(angle.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..60)
            .map(|i| 50.0 + (f64::from(i) * 0.3).sin() * 10.0)
            .collect();
        let mut a = LinRegAngle::new(14).unwrap();
        let mut b = LinRegAngle::new(14).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
