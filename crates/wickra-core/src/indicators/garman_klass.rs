//! Garman-Klass Volatility (OHLC estimator).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Garman-Klass Volatility — an OHLC realised-volatility estimator.
///
/// Garman & Klass (1980) extended Parkinson's high-low estimator by adding
/// an open-to-close term, removing some of the bias introduced when the
/// closing price drifts within the bar. The per-bar sample is
///
/// ```text
/// s_t = 0.5 · (ln(H_t / L_t))² − (2·ln 2 − 1) · (ln(C_t / O_t))²
/// ```
///
/// and the indicator returns the annualised square root of the rolling
/// mean of `s_t`:
///
/// ```text
/// out = sqrt(max(mean(s_t over `period`), 0)) · sqrt(trading_periods) · 100
/// ```
///
/// Garman & Klass showed the estimator is ~7.4× more statistically efficient
/// than the close-to-close estimator under driftless Geometric Brownian
/// Motion (Parkinson sits at ~5.0×). It is still biased when there is
/// significant overnight drift between bars — use the Yang-Zhang estimator
/// when the dataset has meaningful close-to-open gaps.
///
/// The per-bar sample `s_t` can be slightly negative when the bar's range
/// is small relative to its open-to-close move; this matches the original
/// paper's algebra and is handled by clamping the rolling mean to zero
/// before taking the square root.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, GarmanKlassVolatility, Indicator};
///
/// let mut indicator = GarmanKlassVolatility::new(20, 252).unwrap();
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
pub struct GarmanKlassVolatility {
    period: usize,
    trading_periods: usize,
    window: VecDeque<f64>,
    sum: f64,
    last: Option<f64>,
}

/// `2 · ln 2 − 1` — the Garman-Klass open-to-close weight.
const GK_OC_COEFF: f64 = 0.386_294_361_119_890_6;

impl GarmanKlassVolatility {
    /// Construct a Garman-Klass Volatility estimator.
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

impl Indicator for GarmanKlassVolatility {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        // `Candle::new` enforces finite, positive OHLC with `high >= max(open,
        // low, close)` and `low <= min(open, high, close)`, so every log
        // ratio below is well-defined and `ln(H/L) >= 0`.
        let log_hl = (candle.high / candle.low).ln();
        let log_co = (candle.close / candle.open).ln();
        let sample = 0.5 * log_hl * log_hl - GK_OC_COEFF * log_co * log_co;

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
        // Rolling mean. Garman-Klass samples can be marginally negative on
        // narrow-range bars with large O-to-C moves; the rolling mean is
        // theoretically `>= 0` but a clamp absorbs FP cancellation and the
        // pathological all-negative case.
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
        "GarmanKlassVolatility"
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
            GarmanKlassVolatility::new(0, 252),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            GarmanKlassVolatility::new(20, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let gk = GarmanKlassVolatility::new(20, 252).unwrap();
        assert_eq!(gk.periods(), (20, 252));
        assert_eq!(gk.value(), None);
        assert_eq!(gk.warmup_period(), 20);
        assert_eq!(gk.name(), "GarmanKlassVolatility");
        assert!(!gk.is_ready());
    }

    #[test]
    fn zero_movement_yields_zero() {
        // O == H == L == C -> both log terms are zero -> sigma is zero.
        let candles: Vec<Candle> = (0..30).map(|i| candle(10.0, 10.0, 10.0, 10.0, i)).collect();
        let mut gk = GarmanKlassVolatility::new(14, 1).unwrap();
        for v in gk.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn constant_bar_shape_yields_constant_sigma() {
        // Every bar has identical O/H/L/C ratios -> per-bar sample is a
        // constant `k`, so the rolling mean is `k` and the output is
        // `sqrt(k) * 100` (trading_periods = 1).
        let candles: Vec<Candle> = (0..30).map(|i| candle(10.0, 11.0, 9.0, 10.2, i)).collect();
        let log_hl = (11.0_f64 / 9.0_f64).ln();
        let log_co = (10.2_f64 / 10.0_f64).ln();
        let k = 0.5 * log_hl * log_hl - GK_OC_COEFF * log_co * log_co;
        let expected = k.max(0.0).sqrt() * 100.0;

        let mut gk = GarmanKlassVolatility::new(10, 1).unwrap();
        let out = gk.batch(&candles);
        for v in out.iter().skip(9).flatten() {
            assert_relative_eq!(*v, expected, epsilon = 1e-9);
        }
    }

    #[test]
    fn output_is_non_negative() {
        let mut gk = GarmanKlassVolatility::new(14, 252).unwrap();
        let candles: Vec<Candle> = (0..200)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.3).sin() * 12.0;
                let half = 0.5 + (f64::from(i) * 0.13).cos().abs() * 1.5;
                let open = base - 0.1;
                let close = base + 0.2;
                candle(open, base + half, base - half, close, i64::from(i))
            })
            .collect();
        for v in gk.batch(&candles).into_iter().flatten() {
            assert!(v >= 0.0, "Garman-Klass must be non-negative: {v}");
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
        let raw = GarmanKlassVolatility::new(10, 1).unwrap().batch(&candles);
        let annual = GarmanKlassVolatility::new(10, 252).unwrap().batch(&candles);
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
        let candles: Vec<Candle> = (0..20).map(|i| candle(10.0, 11.0, 9.0, 10.2, i)).collect();
        let mut gk = GarmanKlassVolatility::new(5, 1).unwrap();
        let out = gk.batch(&candles);
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
        let batch = GarmanKlassVolatility::new(14, 252).unwrap().batch(&candles);
        let mut streamer = GarmanKlassVolatility::new(14, 252).unwrap();
        let streamed: Vec<_> = candles.iter().map(|c| streamer.update(*c)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..30).map(|i| candle(10.0, 11.0, 9.0, 10.2, i)).collect();
        let mut gk = GarmanKlassVolatility::new(14, 252).unwrap();
        gk.batch(&candles);
        assert!(gk.is_ready());
        gk.reset();
        assert!(!gk.is_ready());
        assert_eq!(gk.value(), None);
        assert_eq!(gk.update(candles[0]), None);
    }
}
