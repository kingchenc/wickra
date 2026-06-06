//! Volatility of Volatility — the dispersion of a rolling volatility series.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Sample standard deviation from a running `(sum, sum_of_squares, count)`.
///
/// Uses Bessel's correction (divisor `n − 1`) and clamps a tiny negative
/// floating-point residual to zero before the square root.
fn sample_stddev(sum: f64, sum_sq: f64, count: usize) -> f64 {
    let n = count as f64;
    let mean = sum / n;
    let variance = ((sum_sq - n * mean * mean) / (n - 1.0)).max(0.0);
    variance.sqrt()
}

/// Volatility of Volatility — the standard deviation of a rolling realized-
/// volatility series ("vol-of-vol").
///
/// ```text
/// r_t   = ln(price_t / price_{t−1})
/// vol_t = stddev_sample(r over vol_window)          (rolling realized volatility)
/// VoV   = stddev_sample(vol over vov_window)         (dispersion of that series)
/// ```
///
/// This is a two-stage estimator: the first stage measures the rolling sample
/// volatility of log returns (the same quantity
/// [`HistoricalVolatility`](crate::HistoricalVolatility) annualises), and the
/// second stage measures how much *that* volatility itself moves. A high
/// vol-of-vol means the volatility regime is unstable — turbulent periods
/// alternate with calm ones — which is exactly the convexity that long-gamma and
/// volatility-trading strategies care about. Both stages use the unbiased
/// `n − 1` sample standard deviation. Each `update` is O(1).
///
/// Non-finite and non-positive prices are ignored (the log return would be
/// undefined): the tick is dropped, state is left untouched, and the last value
/// is returned.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, VolatilityOfVolatility};
///
/// let mut indicator = VolatilityOfVolatility::new(20, 20).unwrap();
/// let mut last = None;
/// for i in 0..120 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct VolatilityOfVolatility {
    vol_window: usize,
    vov_window: usize,
    prev_price: Option<f64>,
    /// Rolling window of log returns (stage one).
    returns: VecDeque<f64>,
    ret_sum: f64,
    ret_sum_sq: f64,
    /// Rolling window of realized-volatility readings (stage two).
    vols: VecDeque<f64>,
    vol_sum: f64,
    vol_sum_sq: f64,
    last: Option<f64>,
}

impl VolatilityOfVolatility {
    /// Construct a new vol-of-vol indicator.
    ///
    /// `vol_window` is the window for the inner realized-volatility series;
    /// `vov_window` is the window over which its dispersion is measured.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if either window is `0`, or
    /// [`Error::InvalidPeriod`] if either is `1` (a sample standard deviation
    /// needs at least two observations).
    pub fn new(vol_window: usize, vov_window: usize) -> Result<Self> {
        if vol_window == 0 || vov_window == 0 {
            return Err(Error::PeriodZero);
        }
        if vol_window < 2 || vov_window < 2 {
            return Err(Error::InvalidPeriod {
                message: "vol-of-vol windows must both be >= 2",
            });
        }
        Ok(Self {
            vol_window,
            vov_window,
            prev_price: None,
            returns: VecDeque::with_capacity(vol_window),
            ret_sum: 0.0,
            ret_sum_sq: 0.0,
            vols: VecDeque::with_capacity(vov_window),
            vol_sum: 0.0,
            vol_sum_sq: 0.0,
            last: None,
        })
    }

    /// Configured `(vol_window, vov_window)`.
    pub const fn windows(&self) -> (usize, usize) {
        (self.vol_window, self.vov_window)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for VolatilityOfVolatility {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        // Non-finite / non-positive prices are skipped: `ln(input / prev)` is
        // undefined, so the tick must not enter the return window.
        if !input.is_finite() || input <= 0.0 {
            return self.last;
        }
        let Some(prev) = self.prev_price else {
            self.prev_price = Some(input);
            return None;
        };
        self.prev_price = Some(input);
        // `prev` came from `self.prev_price`, gated by the guard above, so it is
        // finite and positive — the log return is always well-defined.
        let r = (input / prev).ln();

        // Stage one: rolling sample volatility of log returns.
        if self.returns.len() == self.vol_window {
            let old = self.returns.pop_front().expect("returns window non-empty");
            self.ret_sum -= old;
            self.ret_sum_sq -= old * old;
        }
        self.returns.push_back(r);
        self.ret_sum += r;
        self.ret_sum_sq += r * r;
        if self.returns.len() < self.vol_window {
            return None;
        }
        let vol = sample_stddev(self.ret_sum, self.ret_sum_sq, self.vol_window);

        // Stage two: rolling sample dispersion of the volatility series.
        if self.vols.len() == self.vov_window {
            let old = self.vols.pop_front().expect("vols window non-empty");
            self.vol_sum -= old;
            self.vol_sum_sq -= old * old;
        }
        self.vols.push_back(vol);
        self.vol_sum += vol;
        self.vol_sum_sq += vol * vol;
        if self.vols.len() < self.vov_window {
            return None;
        }
        let vov = sample_stddev(self.vol_sum, self.vol_sum_sq, self.vov_window);
        self.last = Some(vov);
        Some(vov)
    }

    fn reset(&mut self) {
        self.prev_price = None;
        self.returns.clear();
        self.ret_sum = 0.0;
        self.ret_sum_sq = 0.0;
        self.vols.clear();
        self.vol_sum = 0.0;
        self.vol_sum_sq = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // One previous price for the first return, `vol_window` returns for the
        // first volatility, then `vov_window` volatilities for the dispersion.
        // The two windows overlap on the bar axis, so this is the sum.
        self.vol_window + self.vov_window
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "VolatilityOfVolatility"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use crate::HistoricalVolatility;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_window() {
        assert!(matches!(
            VolatilityOfVolatility::new(0, 10),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            VolatilityOfVolatility::new(10, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn rejects_window_one() {
        assert!(matches!(
            VolatilityOfVolatility::new(1, 10),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            VolatilityOfVolatility::new(10, 1),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let vov = VolatilityOfVolatility::new(20, 10).unwrap();
        assert_eq!(vov.windows(), (20, 10));
        assert_eq!(vov.warmup_period(), 30);
        assert_eq!(vov.name(), "VolatilityOfVolatility");
        assert!(!vov.is_ready());
        assert_eq!(vov.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut vov = VolatilityOfVolatility::new(3, 3).unwrap();
        let prices: Vec<f64> = (1..=20)
            .map(|i| 100.0 + (f64::from(i) * 0.7).sin() * 4.0)
            .collect();
        let out = vov.batch(&prices);
        let warmup = vov.warmup_period(); // 6
        for v in out.iter().take(warmup - 1) {
            assert!(v.is_none());
        }
        assert!(out[warmup - 1].is_some());
    }

    #[test]
    fn matches_two_stage_reference() {
        // Stage one equals HistoricalVolatility(vol_window, 1) / 100 (sample
        // stddev of log returns); stage two is the sample stddev of that series.
        let (vol_window, vov_window) = (3, 3);
        let prices: Vec<f64> = [100.0, 102.0, 101.0, 104.0, 103.5, 106.0, 105.0, 108.0].to_vec();

        let mut hv = HistoricalVolatility::new(vol_window, 1).unwrap();
        let vol_series: Vec<f64> = hv
            .batch(&prices)
            .into_iter()
            .flatten()
            .map(|v| v / 100.0)
            .collect();
        // Sample stddev of the last `vov_window` volatilities.
        let tail = &vol_series[vol_series.len() - vov_window..];
        let sum: f64 = tail.iter().sum();
        let sum_sq: f64 = tail.iter().map(|v| v * v).sum();
        let expected = sample_stddev(sum, sum_sq, vov_window);

        let mut vov = VolatilityOfVolatility::new(vol_window, vov_window).unwrap();
        let out = vov.batch(&prices);
        assert_relative_eq!(out.last().unwrap().unwrap(), expected, epsilon = 1e-9);
    }

    #[test]
    fn constant_series_yields_zero() {
        let mut vov = VolatilityOfVolatility::new(5, 5).unwrap();
        for v in vov.batch(&[100.0; 60]).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn output_is_non_negative() {
        let mut vov = VolatilityOfVolatility::new(10, 10).unwrap();
        let prices: Vec<f64> = (1..=300)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 12.0)
            .collect();
        for v in vov.batch(&prices).into_iter().flatten() {
            assert!(v >= 0.0, "vol-of-vol must be non-negative, got {v}");
        }
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut vov = VolatilityOfVolatility::new(3, 3).unwrap();
        let out = vov.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        let last = *out.last().unwrap();
        assert!(last.is_some());
        assert_eq!(vov.update(f64::NAN), last);
        assert_eq!(vov.update(f64::INFINITY), last);
    }

    #[test]
    fn skips_non_positive_prices() {
        let mut vov = VolatilityOfVolatility::new(3, 3).unwrap();
        let warmup = vov.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        let baseline = warmup.last().copied().flatten().expect("warmed up");
        assert_eq!(vov.update(-5.0), Some(baseline));
        assert_eq!(vov.update(0.0), Some(baseline));
        // State untouched: a clone advanced by the same real tick agrees.
        let mut control = vov.clone();
        let after = vov.update(41.0).expect("ready");
        assert_eq!(control.update(41.0).expect("ready"), after);
    }

    #[test]
    fn reset_clears_state() {
        let mut vov = VolatilityOfVolatility::new(3, 3).unwrap();
        vov.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        assert!(vov.is_ready());
        vov.reset();
        assert!(!vov.is_ready());
        assert_eq!(vov.value(), None);
        assert_eq!(vov.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=200)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 9.0)
            .collect();
        let batch = VolatilityOfVolatility::new(10, 10).unwrap().batch(&prices);
        let mut b = VolatilityOfVolatility::new(10, 10).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
