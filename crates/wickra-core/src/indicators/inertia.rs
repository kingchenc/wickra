//! Inertia (Donald Dorsey).

use crate::error::{Error, Result};
use crate::indicators::linreg::LinearRegression;
use crate::indicators::rvi::Rvi;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Donald Dorsey's Inertia — a Linear-Regression-smoothed `RVI` (Relative Vigor
/// Index). The endpoint of an `n`-bar least-squares fit of the `RVI` series is
/// taken as the indicator's reading, smoothing the underlying ratio while
/// preserving its trend direction.
///
/// ```text
/// Inertia_t = LinearRegression(RVI(close - open, high - low; rvi_period), linreg_period)_t
/// ```
///
/// Dorsey's recommended defaults are `(rvi_period = 14, linreg_period = 20)`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Inertia};
///
/// let mut inertia = Inertia::new(14, 20).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     let o = 100.0 + f64::from(i);
///     let c = o + 0.5;
///     let candle = Candle::new(o, c + 0.2, o - 0.2, c, 1.0, i64::from(i)).unwrap();
///     last = inertia.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Inertia {
    rvi_period: usize,
    linreg_period: usize,
    rvi: Rvi,
    linreg: LinearRegression,
}

impl Inertia {
    /// # Errors
    /// Returns [`Error::PeriodZero`] if either period is zero.
    pub fn new(rvi_period: usize, linreg_period: usize) -> Result<Self> {
        if rvi_period == 0 || linreg_period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            rvi_period,
            linreg_period,
            rvi: Rvi::new(rvi_period)?,
            linreg: LinearRegression::new(linreg_period)?,
        })
    }

    /// Dorsey's recommended defaults `(rvi_period = 14, linreg_period = 20)`.
    pub fn classic() -> Self {
        Self::new(14, 20).expect("classic Inertia parameters are valid")
    }

    /// Configured `(rvi_period, linreg_period)`.
    pub const fn periods(&self) -> (usize, usize) {
        (self.rvi_period, self.linreg_period)
    }
}

impl Indicator for Inertia {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let rvi = self.rvi.update(candle)?;
        self.linreg.update(rvi)
    }

    fn reset(&mut self) {
        self.rvi.reset();
        self.linreg.reset();
    }

    fn warmup_period(&self) -> usize {
        // RVI emits at `rvi_period` candles; the LinearRegression then needs
        // `linreg_period − 1` more RVI values to fill its window.
        self.rvi_period + self.linreg_period - 1
    }

    fn is_ready(&self) -> bool {
        self.linreg.is_ready()
    }

    fn name(&self) -> &'static str {
        "Inertia"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(open: f64, high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(open, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(Inertia::new(0, 20), Err(Error::PeriodZero)));
        assert!(matches!(Inertia::new(14, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let inertia = Inertia::classic();
        assert_eq!(inertia.periods(), (14, 20));
        assert_eq!(inertia.warmup_period(), 33);
        assert_eq!(inertia.name(), "Inertia");
    }

    #[test]
    fn classic_factory() {
        assert_eq!(Inertia::classic().periods(), (14, 20));
    }

    #[test]
    fn warmup_emits_first_value_at_warmup_period() {
        // Smaller periods for a fast test: RVI(3) emits at 3 candles, then
        // LinReg(4) needs 4 RVI values -> total 3 + 4 - 1 = 6.
        let mut inertia = Inertia::new(3, 4).unwrap();
        assert_eq!(inertia.warmup_period(), 6);
        for i in 0..5 {
            assert_eq!(inertia.update(candle(10.0, 11.0, 9.0, 10.5, i)), None);
        }
        assert!(inertia.update(candle(10.0, 11.0, 9.0, 10.5, 5)).is_some());
    }

    #[test]
    fn constant_rvi_yields_constant_inertia() {
        // Every bar identical -> RVI is constant -> LinReg of a constant
        // series equals that constant after warmup.
        let mut inertia = Inertia::new(3, 4).unwrap();
        let mut last = None;
        for i in 0..40 {
            last = inertia.update(candle(10.0, 11.0, 9.0, 10.5, i));
        }
        // RVI = SMA(c-o, 3) / SMA(h-l, 3) = 0.5 / 2.0 = 0.25 on every bar.
        let v = last.unwrap();
        assert_relative_eq!(v, 0.25, epsilon = 1e-12);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80_i64)
            .map(|i| {
                let o = 100.0 + (i as f64 * 0.3).sin() * 5.0;
                let c = o + (i as f64 * 0.1).cos();
                candle(o, o.max(c) + 0.5, o.min(c) - 0.5, c, i)
            })
            .collect();
        let batch = Inertia::classic().batch(&candles);
        let mut b = Inertia::classic();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn reset_clears_state() {
        let mut inertia = Inertia::classic();
        for i in 0..50 {
            inertia.update(candle(10.0, 11.0, 9.0, 10.5, i));
        }
        assert!(inertia.is_ready());
        inertia.reset();
        assert!(!inertia.is_ready());
        assert_eq!(inertia.update(candle(10.0, 11.0, 9.0, 10.5, 0)), None);
    }
}
