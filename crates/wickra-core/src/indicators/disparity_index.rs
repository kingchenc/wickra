//! Disparity Index.

use crate::error::Result;
use crate::indicators::sma::Sma;
use crate::traits::Indicator;

/// Disparity Index — the percentage gap between price and its moving average.
///
/// ```text
/// Disparity = 100 * (price - SMA(price, period)) / SMA(price, period)
/// ```
///
/// Originating in Japanese technical analysis (*kairi*), the disparity index
/// expresses how far price has stretched from its `period`-bar simple moving
/// average, as a percentage of that average. Positive readings mean price is
/// above the mean (potentially overbought / strong), negative readings mean it
/// is below (potentially oversold / weak); the magnitude measures how
/// over-extended the move is.
///
/// The first output lands once the inner SMA is ready (input `period`). If the
/// moving average is exactly zero the gap percentage is undefined and the index
/// returns `0.0`.
///
/// # Example
///
/// ```
/// use wickra_core::{DisparityIndex, Indicator};
///
/// let mut indicator = DisparityIndex::new(14).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct DisparityIndex {
    period: usize,
    sma: Sma,
}

impl DisparityIndex {
    /// Construct a disparity index over `period` inputs.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            period,
            sma: Sma::new(period)?,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for DisparityIndex {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        let mean = self.sma.update(input)?;
        if mean == 0.0 {
            return Some(0.0);
        }
        Some(100.0 * (input - mean) / mean)
    }

    fn reset(&mut self) {
        self.sma.reset();
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.sma.is_ready()
    }

    fn name(&self) -> &'static str {
        "DisparityIndex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(DisparityIndex::new(0).is_err());
    }

    /// Cover the const accessor `period` and the Indicator-impl `warmup_period`
    /// + `name`.
    #[test]
    fn accessors_and_metadata() {
        let di = DisparityIndex::new(14).unwrap();
        assert_eq!(di.period(), 14);
        assert_eq!(di.warmup_period(), 14);
        assert_eq!(di.name(), "DisparityIndex");
    }

    #[test]
    fn warmup_then_known_value() {
        // SMA(3) of [2, 4, 6] = 4; price 6 -> 100 * (6 - 4) / 4 = 50.
        let mut di = DisparityIndex::new(3).unwrap();
        assert_eq!(di.update(2.0), None);
        assert_eq!(di.update(4.0), None);
        assert_relative_eq!(di.update(6.0).unwrap(), 50.0, epsilon = 1e-12);
    }

    #[test]
    fn constant_series_is_zero() {
        // Price equals its own mean -> zero disparity.
        let mut di = DisparityIndex::new(5).unwrap();
        for v in di.batch(&[42.0; 20]).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn negative_when_below_mean() {
        // SMA(3) of [10, 8, 6] = 8; price 6 -> 100 * (6 - 8) / 8 = -25.
        let mut di = DisparityIndex::new(3).unwrap();
        let v = di.batch(&[10.0, 8.0, 6.0]);
        assert_relative_eq!(v[2].unwrap(), -25.0, epsilon = 1e-12);
    }

    #[test]
    fn zero_mean_returns_zero() {
        // A window summing to zero (mean 0) makes the percentage undefined; the
        // index returns 0.0 rather than a non-finite value.
        let mut di = DisparityIndex::new(2).unwrap();
        assert_eq!(di.update(-3.0), None);
        // SMA(2) of [-3, 3] = 0 -> guarded to 0.0.
        assert_relative_eq!(di.update(3.0).unwrap(), 0.0, epsilon = 1e-12);
    }

    #[test]
    fn reset_clears_state() {
        let mut di = DisparityIndex::new(5).unwrap();
        di.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        assert!(di.is_ready());
        di.reset();
        assert!(!di.is_ready());
        assert_eq!(di.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=30)
            .map(|i| 50.0 + (f64::from(i) * 0.3).sin() * 10.0)
            .collect();
        let mut a = DisparityIndex::new(7).unwrap();
        let mut b = DisparityIndex::new(7).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }
}
