//! Rogers-Satchell Volatility (drift-free OHLC estimator).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Rogers-Satchell Volatility — a drift-free OHLC realised-volatility
/// estimator.
///
/// Rogers, Satchell & Yoon (1994) extended the Garman-Klass framework to
/// handle non-zero drift between bars without introducing the bias the
/// Garman-Klass estimator picks up in trending markets. The per-bar sample
/// is
///
/// ```text
/// s_t = ln(H_t / C_t) · ln(H_t / O_t) + ln(L_t / C_t) · ln(L_t / O_t)
/// ```
///
/// and the indicator returns the annualised square root of the rolling
/// mean of `s_t`:
///
/// ```text
/// out = sqrt(max(mean(s_t over `period`), 0)) · sqrt(trading_periods) · 100
/// ```
///
/// The estimator is exact under a Brownian Motion with arbitrary drift —
/// the drift component cancels out algebraically. Each per-bar sample is
/// also guaranteed non-negative (both products contribute non-negative
/// terms by construction: `H >= O,C` and `L <= O,C`), so the rolling mean
/// cannot drift below zero except through FP cancellation.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, RogersSatchellVolatility};
///
/// let mut indicator = RogersSatchellVolatility::new(20, 252).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let base = 100.0 + f64::from(i);
///     let candle = Candle::new(base, base + 2.0, base - 2.0, base + 0.5, 1.0, i64::from(i))
///         .unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct RogersSatchellVolatility {
    period: usize,
    trading_periods: usize,
    window: VecDeque<f64>,
    sum: f64,
    last: Option<f64>,
}

impl RogersSatchellVolatility {
    /// Construct a Rogers-Satchell Volatility estimator.
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
            sum: 0.0,
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

impl Indicator for RogersSatchellVolatility {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        // `Candle::new` guarantees finite, positive OHLC with `high >=
        // max(open, low, close)` and `low <= min(open, high, close)`. The
        // factors below thus have predictable signs:
        //   ln(H/C) >= 0,  ln(H/O) >= 0,  ln(L/C) <= 0,  ln(L/O) <= 0
        // so both products are non-negative and the per-bar sample is
        // guaranteed `>= 0` by construction.
        let log_hc = (candle.high / candle.close).ln();
        let log_ho = (candle.high / candle.open).ln();
        let log_lc = (candle.low / candle.close).ln();
        let log_lo = (candle.low / candle.open).ln();
        let sample = log_hc.mul_add(log_ho, log_lc * log_lo);

        if self.window.len() == self.period {
            let old = self.window.pop_front().expect("window is non-empty");
            self.sum -= old;
        }
        self.window.push_back(sample);
        self.sum += sample;

        if self.window.len() < self.period {
            return None;
        }

        let n = self.period as f64;
        // The clamp absorbs FP cancellation; the mathematical value is
        // already `>= 0` by the sign argument above.
        let variance = (self.sum / n).max(0.0);
        let sigma = variance.sqrt();
        let out = sigma * (self.trading_periods as f64).sqrt() * 100.0;
        self.last = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "RogersSatchellVolatility"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(o: f64, h: f64, l: f64, c: f64, ts: i64) -> Candle {
        Candle::new(o, h, l, c, 1.0, ts).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(
            RogersSatchellVolatility::new(0, 252),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            RogersSatchellVolatility::new(20, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let rs = RogersSatchellVolatility::new(20, 252).unwrap();
        assert_eq!(rs.periods(), (20, 252));
        assert_eq!(rs.value(), None);
        assert_eq!(rs.warmup_period(), 20);
        assert_eq!(rs.name(), "RogersSatchellVolatility");
        assert!(!rs.is_ready());
    }

    #[test]
    fn zero_movement_yields_zero() {
        let candles: Vec<Candle> = (0..30).map(|i| candle(10.0, 10.0, 10.0, 10.0, i)).collect();
        let mut rs = RogersSatchellVolatility::new(14, 1).unwrap();
        for v in rs.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn constant_bar_shape_yields_constant_sigma() {
        // Each bar has identical OHLC -> per-bar sample is a constant `k`.
        let candles: Vec<Candle> = (0..30).map(|i| candle(10.0, 11.0, 9.0, 10.5, i)).collect();
        let log_hc = (11.0_f64 / 10.5_f64).ln();
        let log_ho = (11.0_f64 / 10.0_f64).ln();
        let log_lc = (9.0_f64 / 10.5_f64).ln();
        let log_lo = (9.0_f64 / 10.0_f64).ln();
        let k = log_hc * log_ho + log_lc * log_lo;
        let expected = k.max(0.0).sqrt() * 100.0;

        let mut rs = RogersSatchellVolatility::new(10, 1).unwrap();
        let out = rs.batch(&candles);
        for v in out.iter().skip(9).flatten() {
            assert_relative_eq!(*v, expected, epsilon = 1e-9);
        }
    }

    #[test]
    fn output_is_non_negative() {
        let mut rs = RogersSatchellVolatility::new(14, 252).unwrap();
        let candles: Vec<Candle> = (0..200)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.3).sin() * 12.0;
                let half = 0.5 + (f64::from(i) * 0.13).cos().abs() * 1.5;
                let open = base - 0.1;
                let close = base + 0.2;
                candle(open, base + half, base - half, close, i64::from(i))
            })
            .collect();
        for v in rs.batch(&candles).into_iter().flatten() {
            assert!(v >= 0.0, "Rogers-Satchell must be non-negative: {v}");
        }
    }

    #[test]
    fn annualisation_scales_by_sqrt_trading_periods() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.3).sin() * 5.0;
                let half = 1.0 + (f64::from(i) * 0.2).cos().abs();
                candle(base, base + half, base - half, base + 0.3, i64::from(i))
            })
            .collect();
        let raw = RogersSatchellVolatility::new(10, 1)
            .unwrap()
            .batch(&candles);
        let annual = RogersSatchellVolatility::new(10, 252)
            .unwrap()
            .batch(&candles);
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
        let candles: Vec<Candle> = (0..20).map(|i| candle(10.0, 11.0, 9.0, 10.5, i)).collect();
        let mut rs = RogersSatchellVolatility::new(5, 1).unwrap();
        let out = rs.batch(&candles);
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
                candle(base, base + half, base - half, base + 0.5, i64::from(i))
            })
            .collect();
        let batch = RogersSatchellVolatility::new(14, 252)
            .unwrap()
            .batch(&candles);
        let mut streamer = RogersSatchellVolatility::new(14, 252).unwrap();
        let streamed: Vec<_> = candles.iter().map(|c| streamer.update(*c)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..30).map(|i| candle(10.0, 11.0, 9.0, 10.5, i)).collect();
        let mut rs = RogersSatchellVolatility::new(14, 252).unwrap();
        rs.batch(&candles);
        assert!(rs.is_ready());
        rs.reset();
        assert!(!rs.is_ready());
        assert_eq!(rs.value(), None);
        assert_eq!(rs.update(candles[0]), None);
    }
}
