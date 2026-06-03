//! Minus Directional Indicator (-DI), Wilder-smoothed.

use crate::error::{Error, Result};
use crate::indicators::adx::directional_movement;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Wilder's Minus Directional Indicator (`MINUS_DI`).
///
/// `-DI = 100 · smoothed(-DM) / smoothed(TR)`, where both the minus directional
/// movement and the true range are Wilder-smoothed over `period` bars. It is the
/// bearish half of the directional system that drives [`Adx`](crate::Adx);
/// readings above [`PlusDi`](crate::PlusDi) mark a down-trending regime.
///
/// The first `period` raw values seed the two running sums; from then on each
/// applies the Wilder recursion `smoothed − smoothed / period + raw`. Because a
/// bar's directional movement and true range both need the previous bar, the
/// first value is emitted after `period + 1` candles. When the smoothed true
/// range is zero (a perfectly flat market) the indicator returns `0`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, MinusDi};
///
/// let mut indicator = MinusDi::new(5).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let base = 100.0 - f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base - 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct MinusDi {
    period: usize,
    prev: Option<Candle>,
    dm_seed: f64,
    tr_seed: f64,
    seed_count: usize,
    dm_smooth: Option<f64>,
    tr_smooth: Option<f64>,
}

impl MinusDi {
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            prev: None,
            dm_seed: 0.0,
            tr_seed: 0.0,
            seed_count: 0,
            dm_smooth: None,
            tr_smooth: None,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for MinusDi {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev) = self.prev else {
            self.prev = Some(candle);
            return None;
        };
        self.prev = Some(candle);

        let (_, minus_dm) = directional_movement(&prev, &candle);
        let tr = candle.true_range(Some(prev.close));
        let n = self.period as f64;

        let (dm_v, tr_v) = if let (Some(d), Some(t)) = (self.dm_smooth, self.tr_smooth) {
            let d_new = d - d / n + minus_dm;
            let t_new = t - t / n + tr;
            self.dm_smooth = Some(d_new);
            self.tr_smooth = Some(t_new);
            (d_new, t_new)
        } else {
            self.dm_seed += minus_dm;
            self.tr_seed += tr;
            self.seed_count += 1;
            if self.seed_count < self.period {
                return None;
            }
            self.dm_smooth = Some(self.dm_seed);
            self.tr_smooth = Some(self.tr_seed);
            (self.dm_seed, self.tr_seed)
        };

        let di = if tr_v == 0.0 {
            0.0
        } else {
            100.0 * dm_v / tr_v
        };
        Some(di)
    }

    fn reset(&mut self) {
        self.prev = None;
        self.dm_seed = 0.0;
        self.tr_seed = 0.0;
        self.seed_count = 0;
        self.dm_smooth = None;
        self.tr_smooth = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.dm_smooth.is_some()
    }

    fn name(&self) -> &'static str {
        "MINUS_DI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(h: f64, l: f64, cl: f64) -> Candle {
        Candle::new(cl, h, l, cl, 1.0, 0).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(MinusDi::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_report_config() {
        let di = MinusDi::new(7).unwrap();
        assert_eq!(di.period(), 7);
        assert_eq!(di.name(), "MINUS_DI");
        assert_eq!(di.warmup_period(), 7);
        assert!(!di.is_ready());
    }

    #[test]
    fn downtrend_drives_minus_di_high() {
        // Strict downtrend: -DM dominates, so -DI is large and bounded by 100.
        let candles: Vec<Candle> = (0..12)
            .map(|i| {
                let base = 140.0 - f64::from(i) * 2.0;
                c(base + 0.5, base - 1.0, base - 0.5)
            })
            .collect();
        let mut di = MinusDi::new(3).unwrap();
        let out: Vec<Option<f64>> = di.batch(&candles);
        assert_eq!(out[0], None);
        assert!(out[3].is_some());
        let last = out.into_iter().flatten().last().unwrap();
        assert!(last > 0.0 && last <= 100.0);
        assert!(di.is_ready());
    }

    #[test]
    fn flat_market_returns_zero() {
        let candles: Vec<Candle> = (0..6).map(|_| c(50.0, 50.0, 50.0)).collect();
        let mut di = MinusDi::new(3).unwrap();
        let last = di.batch(&candles).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn reset_restores_initial_state() {
        let candles: Vec<Candle> = (0..6)
            .map(|i| {
                let base = 140.0 - f64::from(i) * 2.0;
                c(base + 0.5, base - 1.0, base - 0.5)
            })
            .collect();
        let mut di = MinusDi::new(3).unwrap();
        let _ = di.batch(&candles);
        assert!(di.is_ready());
        di.reset();
        assert!(!di.is_ready());
        assert_eq!(di.update(candles[0]), None);
    }
}
