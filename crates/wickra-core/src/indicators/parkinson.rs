//! Parkinson Volatility (high-low estimator).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Parkinson Volatility — a high-low realised-volatility estimator.
///
/// Michael Parkinson (1980) noted that the extreme range of a bar carries
/// more variance information than the closing price alone: a wide bar that
/// closes near its open is far more "volatile" than a narrow bar that
/// happens to close at the same level. The estimator is
///
/// ```text
/// sigma² = (1 / (4n · ln 2)) · Σ_{i=1..n} (ln(H_i / L_i))²
/// sigma  = √sigma²
/// out    = sigma · √trading_periods · 100
/// ```
///
/// The output is annualised to a percent in the same style as
/// [`HistoricalVolatility`](crate::HistoricalVolatility) — `trading_periods`
/// of `252` for daily bars, `52` for weekly, `12` for monthly. Pass
/// `trading_periods = 1` for the raw per-bar `sigma · 100` figure.
///
/// Under a driftless Geometric-Brownian-Motion assumption, Parkinson's
/// estimator has roughly `1/5` the variance of the close-to-close
/// estimator — i.e. five close-to-close samples give the same statistical
/// efficiency as one Parkinson sample.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, ParkinsonVolatility};
///
/// let mut indicator = ParkinsonVolatility::new(20, 252).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let base = 100.0 + f64::from(i);
///     let candle = Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 1.0, i64::from(i))
///         .unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct ParkinsonVolatility {
    period: usize,
    trading_periods: usize,
    window: VecDeque<f64>,
    sum_sq: f64,
    last: Option<f64>,
}

/// `1 / (4 · ln 2)` — the Parkinson normalisation constant, evaluated once at
/// `const` to keep the per-update path branch-free.
const PARKINSON_FACTOR: f64 = 0.360_673_760_222_241_2;

impl ParkinsonVolatility {
    /// Construct a Parkinson Volatility estimator.
    ///
    /// `period` is the rolling window of bars; `trading_periods` is the
    /// annualisation factor (`252` daily, `52` weekly, `12` monthly, or
    /// `1` for raw per-bar volatility).
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if either parameter is `0`.
    pub fn new(period: usize, trading_periods: usize) -> Result<Self> {
        if period == 0 || trading_periods == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            trading_periods,
            window: VecDeque::with_capacity(period),
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

impl Indicator for ParkinsonVolatility {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        // `Candle::new` already guarantees finite, positive `high` and `low`
        // with `high >= low`, so the log ratio is always well-defined and
        // non-negative.
        let log_hl = (candle.high / candle.low).ln();
        let sample = log_hl * log_hl;

        if self.window.len() == self.period {
            let old = self.window.pop_front().expect("window is non-empty");
            self.sum_sq -= old;
        }
        self.window.push_back(sample);
        self.sum_sq += sample;

        if self.window.len() < self.period {
            return None;
        }

        let n = self.period as f64;
        let variance = (PARKINSON_FACTOR * self.sum_sq / n).max(0.0);
        let sigma = variance.sqrt();
        let out = sigma * (self.trading_periods as f64).sqrt() * 100.0;
        self.last = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum_sq = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "ParkinsonVolatility"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(h: f64, l: f64, c: f64, ts: i64) -> Candle {
        Candle::new(f64::midpoint(h, l), h, l, c, 1.0, ts).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(
            ParkinsonVolatility::new(0, 252),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            ParkinsonVolatility::new(20, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let pv = ParkinsonVolatility::new(20, 252).unwrap();
        assert_eq!(pv.periods(), (20, 252));
        assert_eq!(pv.value(), None);
        assert_eq!(pv.warmup_period(), 20);
        assert_eq!(pv.name(), "ParkinsonVolatility");
        assert!(!pv.is_ready());
    }

    #[test]
    fn zero_range_yields_zero() {
        // H == L every bar -> ln(H/L) = 0 -> sigma = 0.
        let candles: Vec<Candle> = (0..30).map(|i| candle(10.0, 10.0, 10.0, i)).collect();
        let mut pv = ParkinsonVolatility::new(14, 1).unwrap();
        for v in pv.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn constant_range_yields_constant_sigma() {
        // Every bar has the same H/L ratio -> every (ln H/L)² is the same
        // constant -> the rolling sum is `n * k` and the variance simplifies
        // to `factor * k`. The output is `sqrt(factor * k) * 100` (with
        // trading_periods = 1).
        let candles: Vec<Candle> = (0..30).map(|i| candle(11.0, 9.0, 10.0, i)).collect();
        let mut pv = ParkinsonVolatility::new(10, 1).unwrap();
        let out = pv.batch(&candles);

        let k = (11.0_f64 / 9.0_f64).ln().powi(2);
        let expected = (PARKINSON_FACTOR * k).sqrt() * 100.0;
        for v in out.iter().skip(9).flatten() {
            assert_relative_eq!(*v, expected, epsilon = 1e-9);
        }
    }

    #[test]
    fn output_is_non_negative() {
        let mut pv = ParkinsonVolatility::new(14, 252).unwrap();
        let candles: Vec<Candle> = (0..200)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.3).sin() * 12.0;
                let half = 0.5 + (f64::from(i) * 0.13).cos().abs() * 1.5;
                candle(base + half, base - half, base, i64::from(i))
            })
            .collect();
        for v in pv.batch(&candles).into_iter().flatten() {
            assert!(v >= 0.0, "Parkinson volatility must be non-negative: {v}");
        }
    }

    #[test]
    fn annualisation_scales_by_sqrt_trading_periods() {
        // Same candles run through (period, 1) and (period, 252) -> the
        // 252-version is `sqrt(252)` times the raw version, bar-for-bar.
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.3).sin() * 5.0;
                let half = 1.0 + (f64::from(i) * 0.2).cos().abs();
                candle(base + half, base - half, base, i64::from(i))
            })
            .collect();
        let raw = ParkinsonVolatility::new(10, 1).unwrap().batch(&candles);
        let annual = ParkinsonVolatility::new(10, 252).unwrap().batch(&candles);
        let scale = (252.0_f64).sqrt();
        for (r, a) in raw.iter().zip(annual.iter()) {
            assert_eq!(r.is_some(), a.is_some(), "warmup mismatch");
            if let (Some(r), Some(a)) = (r, a) {
                assert_relative_eq!(*a, r * scale, epsilon = 1e-9);
            }
        }
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let candles: Vec<Candle> = (0..20).map(|i| candle(11.0, 9.0, 10.0, i)).collect();
        let mut pv = ParkinsonVolatility::new(5, 1).unwrap();
        let out = pv.batch(&candles);
        for v in out.iter().take(4) {
            assert!(v.is_none());
        }
        assert!(out[4].is_some());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.25).sin() * 6.0;
                let half = 1.0 + (f64::from(i) * 0.15).cos().abs();
                candle(base + half, base - half, base, i64::from(i))
            })
            .collect();
        let batch = ParkinsonVolatility::new(14, 252).unwrap().batch(&candles);
        let mut streamer = ParkinsonVolatility::new(14, 252).unwrap();
        let streamed: Vec<_> = candles.iter().map(|c| streamer.update(*c)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..30).map(|i| candle(11.0, 9.0, 10.0, i)).collect();
        let mut pv = ParkinsonVolatility::new(14, 252).unwrap();
        pv.batch(&candles);
        assert!(pv.is_ready());
        pv.reset();
        assert!(!pv.is_ready());
        assert_eq!(pv.value(), None);
        assert_eq!(pv.update(candles[0]), None);
    }
}
