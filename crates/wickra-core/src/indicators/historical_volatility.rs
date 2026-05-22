//! Historical Volatility.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Historical Volatility — the annualised standard deviation of log returns.
///
/// This is the realised (backward-looking) volatility used to price options
/// and size risk:
///
/// ```text
/// r_t = ln(price_t / price_{t−1})
/// HV  = stddev_sample(r over period) · √trading_periods · 100
/// ```
///
/// The log returns over the window are measured with the **sample** standard
/// deviation (divisor `n − 1`, the unbiased estimator), then scaled to an
/// annual figure by `√trading_periods` — `252` for daily bars, `52` for
/// weekly, `12` for monthly — and expressed as a percentage.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, HistoricalVolatility};
///
/// // 20-bar window, 252 trading days per year.
/// let mut indicator = HistoricalVolatility::new(20, 252).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct HistoricalVolatility {
    period: usize,
    trading_periods: usize,
    prev_price: Option<f64>,
    /// Rolling window of the last `period` log returns.
    window: VecDeque<f64>,
    sum: f64,
    sum_sq: f64,
    last: Option<f64>,
}

impl HistoricalVolatility {
    /// Construct a new Historical Volatility indicator.
    ///
    /// `period` is the number of log returns in the rolling window;
    /// `trading_periods` is the annualisation factor (`252` daily, `52`
    /// weekly, `12` monthly).
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period` or `trading_periods` is `0`,
    /// or [`Error::InvalidPeriod`] if `period == 1` (the sample standard
    /// deviation needs at least two returns).
    pub fn new(period: usize, trading_periods: usize) -> Result<Self> {
        if period == 0 || trading_periods == 0 {
            return Err(Error::PeriodZero);
        }
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "historical volatility period must be >= 2",
            });
        }
        Ok(Self {
            period,
            trading_periods,
            prev_price: None,
            window: VecDeque::with_capacity(period),
            sum: 0.0,
            sum_sq: 0.0,
            last: None,
        })
    }

    /// Configured `(period, trading_periods)`.
    pub const fn periods(&self) -> (usize, usize) {
        (self.period, self.trading_periods)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for HistoricalVolatility {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            // Non-finite input is ignored; state is left untouched.
            return self.last;
        }
        let Some(prev) = self.prev_price else {
            self.prev_price = Some(input);
            return None;
        };
        self.prev_price = Some(input);

        let log_return = if prev <= 0.0 || input <= 0.0 {
            // Log return is undefined for non-positive prices.
            0.0
        } else {
            (input / prev).ln()
        };
        if self.window.len() == self.period {
            let old = self.window.pop_front().expect("window is non-empty");
            self.sum -= old;
            self.sum_sq -= old * old;
        }
        self.window.push_back(log_return);
        self.sum += log_return;
        self.sum_sq += log_return * log_return;
        if self.window.len() < self.period {
            return None;
        }
        let n = self.period as f64;
        let mean = self.sum / n;
        // Sample variance (Bessel's correction): Σ(x−mean)² / (n−1).
        let variance = ((self.sum_sq - n * mean * mean) / (n - 1.0)).max(0.0);
        let hv = variance.sqrt() * (self.trading_periods as f64).sqrt() * 100.0;
        self.last = Some(hv);
        Some(hv)
    }

    fn reset(&mut self) {
        self.prev_price = None;
        self.window.clear();
        self.sum = 0.0;
        self.sum_sq = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // The first log return needs a previous price, then the window fills.
        self.period + 1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "HistoricalVolatility"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(
            HistoricalVolatility::new(0, 252),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            HistoricalVolatility::new(20, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn new_rejects_period_one() {
        assert!(matches!(
            HistoricalVolatility::new(1, 252),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut hv = HistoricalVolatility::new(5, 252).unwrap();
        assert_eq!(hv.warmup_period(), 6);
        let out = hv.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        for v in out.iter().take(5) {
            assert!(v.is_none());
        }
        assert!(out[5].is_some());
    }

    #[test]
    fn constant_series_yields_zero() {
        // Flat prices -> all log returns are 0 -> zero volatility.
        let mut hv = HistoricalVolatility::new(10, 252).unwrap();
        let out = hv.batch(&[100.0; 40]);
        for v in out.iter().skip(10).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn geometric_series_yields_zero() {
        // A constant growth factor gives a constant log return -> zero stddev.
        let mut hv = HistoricalVolatility::new(10, 252).unwrap();
        let prices: Vec<f64> = (0..40).map(|i| 100.0 * 1.01_f64.powi(i)).collect();
        let out = hv.batch(&prices);
        for v in out.iter().skip(10).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn output_is_non_negative() {
        let mut hv = HistoricalVolatility::new(20, 252).unwrap();
        let prices: Vec<f64> = (1..=200)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 12.0)
            .collect();
        for v in hv.batch(&prices).into_iter().flatten() {
            assert!(v >= 0.0, "volatility must be non-negative, got {v}");
        }
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut hv = HistoricalVolatility::new(5, 252).unwrap();
        let out = hv.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        let last = *out.last().unwrap();
        assert!(last.is_some());
        assert_eq!(hv.update(f64::NAN), last);
        assert_eq!(hv.update(f64::INFINITY), last);
    }

    #[test]
    fn reset_clears_state() {
        let mut hv = HistoricalVolatility::new(5, 252).unwrap();
        hv.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        assert!(hv.is_ready());
        hv.reset();
        assert!(!hv.is_ready());
        assert_eq!(hv.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 9.0)
            .collect();
        let batch = HistoricalVolatility::new(20, 252).unwrap().batch(&prices);
        let mut b = HistoricalVolatility::new(20, 252).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
