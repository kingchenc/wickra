//! Trend Strength Index — the signed coefficient of determination of a linear
//! regression of price against time.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Trend Strength Index: fits an ordinary-least-squares line to the last
/// `period` prices against their bar index and reports the coefficient of
/// determination `r^2`, signed by the slope of the fit.
///
/// ```text
/// regress y = close on x = 0..period-1
/// r^2  = (n·Σxy − Σx·Σy)^2 / [ (n·Σx² − (Σx)²)(n·Σy² − (Σy)²) ]
/// TSI  = sign(slope) · r^2          (slope sign = sign of n·Σxy − Σx·Σy)
/// ```
///
/// `r^2` in `[0, 1]` measures how well a straight line explains the price over
/// the window — how *trendy* the segment is, regardless of direction. Carrying
/// the slope sign turns it into a directional reading in `[-1, 1]`: values near
/// `+1` are a strong, clean uptrend; near `-1` a strong downtrend; near `0` a
/// flat or noisy market with no linear structure. A window of constant prices
/// (zero variance in `y`) has no defined trend and returns `0`.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, TrendStrengthIndex};
///
/// let mut indicator = TrendStrengthIndex::new(20).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// // A clean ramp is a perfect uptrend -> r^2 = 1.
/// assert!((last.unwrap() - 1.0).abs() < 1e-9);
/// ```
#[derive(Debug, Clone)]
pub struct TrendStrengthIndex {
    period: usize,
    buf: VecDeque<f64>,
}

impl TrendStrengthIndex {
    /// Construct a Trend Strength Index over the given window.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`, or [`Error::InvalidPeriod`]
    /// if `period == 1` (a regression needs at least two points).
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        if period == 1 {
            return Err(Error::InvalidPeriod {
                message: "period must be >= 2 for a regression",
            });
        }
        Ok(Self {
            period,
            buf: VecDeque::with_capacity(period),
        })
    }

    /// Configured window length.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for TrendStrengthIndex {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, price: f64) -> Option<f64> {
        if !price.is_finite() {
            return None;
        }
        self.buf.push_back(price);
        if self.buf.len() > self.period {
            self.buf.pop_front();
        }
        if self.buf.len() < self.period {
            return None;
        }

        let count = self.period as f64;
        let mut sum_x = 0.0;
        let mut sum_xx = 0.0;
        let mut sum_y = 0.0;
        let mut sum_yy = 0.0;
        let mut sum_xy = 0.0;
        for (idx, &price) in self.buf.iter().enumerate() {
            let x = idx as f64;
            sum_x += x;
            sum_xx += x * x;
            sum_y += price;
            sum_yy += price * price;
            sum_xy += x * price;
        }

        let cov = count.mul_add(sum_xy, -(sum_x * sum_y));
        let var_x = count.mul_add(sum_xx, -(sum_x * sum_x));
        let var_y = count.mul_add(sum_yy, -(sum_y * sum_y));
        if var_y <= 0.0 {
            return Some(0.0);
        }
        let r2 = (cov * cov) / (var_x * var_y);
        Some(if cov >= 0.0 { r2 } else { -r2 })
    }

    fn reset(&mut self) {
        self.buf.clear();
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.buf.len() >= self.period
    }

    fn name(&self) -> &'static str {
        "TrendStrengthIndex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_invalid_period() {
        assert!(matches!(TrendStrengthIndex::new(0), Err(Error::PeriodZero)));
        assert!(matches!(
            TrendStrengthIndex::new(1),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let tsi = TrendStrengthIndex::new(20).unwrap();
        assert_eq!(tsi.period(), 20);
        assert_eq!(tsi.warmup_period(), 20);
        assert_eq!(tsi.name(), "TrendStrengthIndex");
        assert!(!tsi.is_ready());
    }

    #[test]
    fn warmup_emits_at_period() {
        let mut tsi = TrendStrengthIndex::new(4).unwrap();
        let inputs: Vec<f64> = (0..6).map(f64::from).collect();
        let out = tsi.batch(&inputs);
        assert!(out[2].is_none());
        assert!(out[3].is_some());
    }

    #[test]
    fn perfect_uptrend_is_plus_one() {
        let mut tsi = TrendStrengthIndex::new(10).unwrap();
        let inputs: Vec<f64> = (0..10).map(f64::from).collect();
        let last = tsi.batch(&inputs).last().unwrap().unwrap();
        assert_relative_eq!(last, 1.0, epsilon = 1e-9);
    }

    #[test]
    fn perfect_downtrend_is_minus_one() {
        let mut tsi = TrendStrengthIndex::new(10).unwrap();
        let inputs: Vec<f64> = (0..10).map(|i| 100.0 - f64::from(i)).collect();
        let last = tsi.batch(&inputs).last().unwrap().unwrap();
        assert_relative_eq!(last, -1.0, epsilon = 1e-9);
    }

    #[test]
    fn flat_market_returns_zero() {
        let mut tsi = TrendStrengthIndex::new(8).unwrap();
        let inputs = [42.0; 12];
        let last = tsi.batch(&inputs).last().unwrap().unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn noisy_trend_is_between() {
        // An upward drift with noise: positive but not a perfect fit.
        let mut tsi = TrendStrengthIndex::new(12).unwrap();
        let inputs: Vec<f64> = (0..12)
            .map(|i| f64::from(i) + if i % 2 == 0 { 0.0 } else { 3.0 })
            .collect();
        let last = tsi.batch(&inputs).last().unwrap().unwrap();
        assert!(last > 0.0 && last < 1.0, "tsi {last} should be in (0, 1)");
    }

    #[test]
    fn reset_clears_state() {
        let mut tsi = TrendStrengthIndex::new(10).unwrap();
        let inputs: Vec<f64> = (0..10).map(f64::from).collect();
        tsi.batch(&inputs);
        assert!(tsi.is_ready());
        tsi.reset();
        assert!(!tsi.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let inputs: Vec<f64> = (0..80)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 5.0)
            .collect();
        let mut a = TrendStrengthIndex::new(15).unwrap();
        let mut b = TrendStrengthIndex::new(15).unwrap();
        assert_eq!(
            a.batch(&inputs),
            inputs.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
