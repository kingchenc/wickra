//! Relative Volatility Index (Donald Dorsey).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Relative Volatility Index — Donald Dorsey's RSI-shaped volatility gauge.
///
/// Where RSI partitions price changes into gains and losses and Wilder-smooths
/// each side, RVI partitions the rolling standard deviation of price into "up
/// volatility" (when price rose since the previous bar) and "down volatility"
/// (when price fell), then applies the same Wilder smoothing and ratio:
///
/// ```text
/// sd_t        = stddev_pop(close over `period`)            // single scalar each bar
/// up_t        = sd_t if close_t > close_{t-1}, else 0
/// down_t      = sd_t if close_t < close_{t-1}, else 0
/// AvgUp_t     = Wilder(up,   period)
/// AvgDown_t   = Wilder(down, period)
/// RVI_t       = 100 · AvgUp_t / (AvgUp_t + AvgDown_t)
/// ```
///
/// The output is bounded on `[0, 100]`. A series with no down-bars saturates
/// at `100`; a series with no up-bars saturates at `0`. A completely flat
/// series (no movement, both averages zero) returns `50` by the same
/// undefined-RS convention as `RSI` (`crates/wickra-core/src/indicators/rsi.rs`).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, RviVolatility};
///
/// let mut indicator = RviVolatility::new(10).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct RviVolatility {
    period: usize,
    // Rolling-stddev state.
    window: VecDeque<f64>,
    sum: f64,
    sum_sq: f64,
    // Direction tracking.
    prev_close: Option<f64>,
    // Wilder-smoothed up/down volatility.
    seed_up: Vec<f64>,
    seed_down: Vec<f64>,
    avg_up: Option<f64>,
    avg_down: Option<f64>,
    last_value: Option<f64>,
}

impl RviVolatility {
    /// Construct an RVI with the given period.
    ///
    /// `period` is used both as the standard-deviation window length and as
    /// the Wilder smoothing constant for the up/down averages.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`, or
    /// [`Error::InvalidPeriod`] if `period == 1` (a 1-bar rolling standard
    /// deviation is always zero and the indicator would never produce a
    /// meaningful reading).
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "RVI period must be >= 2",
            });
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
            sum: 0.0,
            sum_sq: 0.0,
            prev_close: None,
            seed_up: Vec::with_capacity(period),
            seed_down: Vec::with_capacity(period),
            avg_up: None,
            avg_down: None,
            last_value: None,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }

    fn ratio(avg_up: f64, avg_down: f64) -> f64 {
        let denom = avg_up + avg_down;
        if denom == 0.0 {
            // No volatility on either side. Match RSI's undefined-RS convention.
            50.0
        } else {
            100.0 * avg_up / denom
        }
    }
}

impl Indicator for RviVolatility {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            // Non-finite input leaves state untouched, mirrors `StdDev` / `Rsi`.
            return self.last_value;
        }

        // 1. Roll the standard-deviation window.
        if self.window.len() == self.period {
            let old = self.window.pop_front().expect("window is non-empty");
            self.sum -= old;
            self.sum_sq -= old * old;
        }
        self.window.push_back(input);
        self.sum += input;
        self.sum_sq += input * input;

        if self.window.len() < self.period {
            // Track previous close from the very first input so that the first
            // ready stddev sample is paired with a valid direction.
            self.prev_close = Some(input);
            return None;
        }

        let n = self.period as f64;
        let mean = self.sum / n;
        // Population variance with a non-negativity clamp for FP cancellation.
        let variance = (self.sum_sq / n - mean * mean).max(0.0);
        let sd = variance.sqrt();

        // 2. Classify the stddev sample as up- or down-volatility.
        let prev = self
            .prev_close
            .expect("prev_close is set on every input before this point");
        let (up, down) = if input > prev {
            (sd, 0.0)
        } else if input < prev {
            (0.0, sd)
        } else {
            (0.0, 0.0)
        };
        self.prev_close = Some(input);

        // 3. Wilder-smooth the up/down series.
        if let (Some(au), Some(ad)) = (self.avg_up, self.avg_down) {
            let new_au = au.mul_add(n - 1.0, up) / n;
            let new_ad = ad.mul_add(n - 1.0, down) / n;
            self.avg_up = Some(new_au);
            self.avg_down = Some(new_ad);
            let v = Self::ratio(new_au, new_ad);
            self.last_value = Some(v);
            return Some(v);
        }

        self.seed_up.push(up);
        self.seed_down.push(down);
        if self.seed_up.len() == self.period {
            let au = self.seed_up.iter().sum::<f64>() / n;
            let ad = self.seed_down.iter().sum::<f64>() / n;
            self.avg_up = Some(au);
            self.avg_down = Some(ad);
            let v = Self::ratio(au, ad);
            self.last_value = Some(v);
            return Some(v);
        }
        None
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum = 0.0;
        self.sum_sq = 0.0;
        self.prev_close = None;
        self.seed_up.clear();
        self.seed_down.clear();
        self.avg_up = None;
        self.avg_down = None;
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        // `period` bars to fill the stddev window plus another `period − 1`
        // bars to seed the Wilder averages with up/down samples. The two
        // phases overlap by one bar (the `period`-th input produces both the
        // first stddev sample and the first up/down classification), so the
        // first ready RVI lands at index `2 · period − 2`, i.e. the
        // `(2·period − 1)`-th input.
        2 * self.period - 1
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "RVIVolatility"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(RviVolatility::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn rejects_period_one() {
        assert!(matches!(
            RviVolatility::new(1),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let rvi = RviVolatility::new(14).unwrap();
        assert_eq!(rvi.period(), 14);
        assert_eq!(rvi.name(), "RVIVolatility");
        assert_eq!(rvi.value(), None);
        assert_eq!(rvi.warmup_period(), 27);
        assert!(!rvi.is_ready());
    }

    #[test]
    fn constant_series_yields_fifty() {
        // Flat input -> stddev is zero every bar and direction is "unchanged",
        // so both avg_up and avg_down stay at zero -> the undefined-RS
        // convention returns 50.
        let mut rvi = RviVolatility::new(5).unwrap();
        let out = rvi.batch(&[42.0; 40]);
        for v in out.iter().skip(9).flatten() {
            assert_relative_eq!(*v, 50.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn pure_uptrend_saturates_to_one_hundred() {
        // Every bar's close is above the previous -> every stddev sample is
        // classified as up, every down sample is zero -> RVI = 100.
        let mut rvi = RviVolatility::new(5).unwrap();
        let prices: Vec<f64> = (1..=40).map(f64::from).collect();
        let out = rvi.batch(&prices);
        for v in out.iter().skip(9).flatten() {
            assert_relative_eq!(*v, 100.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn pure_downtrend_saturates_to_zero() {
        let mut rvi = RviVolatility::new(5).unwrap();
        let prices: Vec<f64> = (1..=40).rev().map(f64::from).collect();
        let out = rvi.batch(&prices);
        for v in out.iter().skip(9).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn output_is_bounded() {
        let mut rvi = RviVolatility::new(10).unwrap();
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 12.0)
            .collect();
        for v in rvi.batch(&prices).into_iter().flatten() {
            assert!((0.0..=100.0).contains(&v), "RVI out of range: {v}");
        }
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut rvi = RviVolatility::new(5).unwrap();
        assert_eq!(rvi.warmup_period(), 9);
        let prices: Vec<f64> = (0..30)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 3.0)
            .collect();
        let out = rvi.batch(&prices);
        for v in out.iter().take(8) {
            assert!(v.is_none(), "indicator must still be warming up");
        }
        assert!(
            out[8].is_some(),
            "first value lands at warmup_period - 1 = 8"
        );
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 9.0)
            .collect();
        let batch = RviVolatility::new(10).unwrap().batch(&prices);
        let mut streamer = RviVolatility::new(10).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| streamer.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn reset_clears_state() {
        let mut rvi = RviVolatility::new(5).unwrap();
        rvi.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        assert!(rvi.is_ready());
        rvi.reset();
        assert!(!rvi.is_ready());
        assert_eq!(rvi.value(), None);
        assert_eq!(rvi.update(1.0), None);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut rvi = RviVolatility::new(5).unwrap();
        let out = rvi.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        let last = *out.last().unwrap();
        assert!(last.is_some());
        assert_eq!(rvi.update(f64::NAN), last);
        assert_eq!(rvi.update(f64::INFINITY), last);
    }
}
