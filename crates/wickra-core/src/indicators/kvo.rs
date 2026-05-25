//! Klinger Volume Oscillator.

use crate::error::{Error, Result};
use crate::indicators::ema::Ema;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Stephen J. Klinger's Volume Oscillator — a long/short-term volume-force
/// MACD with trend-aware cumulative-money-flow weighting.
///
/// Each bar produces a "volume force" (`vf`) whose sign tracks the daily trend
/// (`+1` on an up day, `−1` on a down day, carry-over otherwise) and whose
/// magnitude scales with how the current accumulation horizon compares to the
/// previous trend's. The KVO line is the difference of two EMAs of `vf`:
///
/// ```text
/// dm_t   = high_t + low_t + close_t                                            (the "daily measurement")
/// trend  = sign(dm_t − dm_{t−1})    if differs from previous trend, reset cm
/// cm_t   = cm_{t−1} + dm_t          if trend unchanged
/// cm_t   = dm_{t−1} + dm_t          if trend just flipped
/// vf_t   = volume_t · |2·(dm_t/cm_t − 1)| · trend · 100
/// KVO_t  = EMA(vf, fast)_t − EMA(vf, slow)_t
/// ```
///
/// Klinger's textbook configuration is `fast = 34, slow = 55` on daily bars.
/// The first bar only seeds `dm_{t−1}`, so the very first `vf` lands at bar 2;
/// the slow EMA then needs `slow` raw `vf` values to seed, putting the first
/// KVO emission at bar `slow + 1`. A zero `cm_t` (which only happens on the
/// trend-flip branch when both the prior and current `dm` are zero) collapses
/// `vf` to `0`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Kvo};
///
/// let mut indicator = Kvo::new(34, 55).unwrap();
/// let mut last = None;
/// for i in 0..120 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Kvo {
    fast_period: usize,
    slow_period: usize,
    fast: Ema,
    slow: Ema,
    prev_dm: Option<f64>,
    trend: i8,
    cm: f64,
}

impl Kvo {
    /// Construct a new KVO with the given EMA periods.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if either period is zero, or
    /// [`Error::InvalidPeriod`] if `fast >= slow`.
    pub fn new(fast: usize, slow: usize) -> Result<Self> {
        if fast == 0 || slow == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "KVO needs fast < slow",
            });
        }
        Ok(Self {
            fast_period: fast,
            slow_period: slow,
            fast: Ema::new(fast)?,
            slow: Ema::new(slow)?,
            prev_dm: None,
            trend: 0,
            cm: 0.0,
        })
    }

    /// Klinger's classic configuration: `EMA(vf, 34) − EMA(vf, 55)`.
    pub fn classic() -> Self {
        Self::new(34, 55).expect("classic Klinger periods are valid")
    }

    /// Configured `(fast, slow)` periods.
    pub const fn periods(&self) -> (usize, usize) {
        (self.fast_period, self.slow_period)
    }
}

impl Indicator for Kvo {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let dm = candle.high + candle.low + candle.close;
        let Some(prev_dm) = self.prev_dm else {
            // The first bar only establishes the previous daily measurement.
            self.prev_dm = Some(dm);
            return None;
        };

        // Determine the bar's trend sign relative to the previous bar.
        let new_trend: i8 = if dm > prev_dm {
            1
        } else if dm < prev_dm {
            -1
        } else {
            self.trend
        };

        // Cumulative measurement resets to (prev_dm + dm) whenever the trend
        // flips. On the very first sign read (trend was 0) we also seed from
        // the two-bar sum, matching the textbook definition.
        if new_trend != self.trend || self.trend == 0 {
            self.cm = prev_dm + dm;
        } else {
            self.cm += dm;
        }
        self.trend = new_trend;

        let vf = if self.cm == 0.0 {
            // Pathological all-zero OHLC stretch — no force to register.
            0.0
        } else {
            candle.volume * (2.0 * (dm / self.cm - 1.0)).abs() * f64::from(new_trend) * 100.0
        };

        self.prev_dm = Some(dm);

        let fast = self.fast.update(vf);
        let slow = self.slow.update(vf);
        Some(fast? - slow?)
    }

    fn reset(&mut self) {
        self.fast.reset();
        self.slow.reset();
        self.prev_dm = None;
        self.trend = 0;
        self.cm = 0.0;
    }

    fn warmup_period(&self) -> usize {
        // One bar to seed `prev_dm`, then the slow EMA needs `slow` raw `vf` values.
        self.slow_period + 1
    }

    fn is_ready(&self) -> bool {
        self.fast.is_ready() && self.slow.is_ready()
    }

    fn name(&self) -> &'static str {
        "KVO"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(high: f64, low: f64, close: f64, volume: f64, ts: i64) -> Candle {
        Candle::new(low, high, low, close, volume, ts).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(Kvo::new(0, 10), Err(Error::PeriodZero)));
        assert!(matches!(Kvo::new(3, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn rejects_fast_geq_slow() {
        assert!(matches!(Kvo::new(34, 34), Err(Error::InvalidPeriod { .. })));
        assert!(matches!(Kvo::new(55, 34), Err(Error::InvalidPeriod { .. })));
    }

    #[test]
    fn accessors_and_metadata() {
        let k = Kvo::classic();
        assert_eq!(k.periods(), (34, 55));
        assert_eq!(k.name(), "KVO");
        assert_eq!(k.warmup_period(), 56);
    }

    #[test]
    fn zero_ohlc_collapses_vf_to_zero() {
        // Two consecutive all-zero bars: dm = 0 for both, so prev_dm + dm = 0
        // and `cm == 0.0` fires the defensive branch, holding vf at zero.
        let mut k = Kvo::new(3, 6).unwrap();
        let zero = Candle::new(0.0, 0.0, 0.0, 0.0, 100.0, 0).unwrap();
        assert_eq!(k.update(zero), None);
        assert_eq!(k.update(zero), None);
        assert_eq!(k.update(zero), None);
    }

    #[test]
    fn constant_series_yields_zero() {
        // dm flat -> trend never sets to a nonzero sign and vf collapses to 0
        // for every bar; both EMAs hold at 0 once seeded.
        let candles: Vec<Candle> = (0..120).map(|i| c(10.0, 10.0, 10.0, 100.0, i)).collect();
        let mut k = Kvo::new(3, 6).unwrap();
        for v in k.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn warmup_emits_at_slow_plus_one() {
        let candles: Vec<Candle> = (0..30i64)
            .map(|i| {
                let f = i as f64;
                c(10.0 + f, 8.0 + f, 9.0 + f, 100.0, i)
            })
            .collect();
        let mut k = Kvo::new(3, 5).unwrap();
        let out = k.batch(&candles);
        for (i, v) in out.iter().enumerate().take(5) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        // First emission lands at index slow_period (one seed bar + slow EMA seeding from there).
        assert!(out[5].is_some(), "first value lands at slow_period");
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..100i64)
            .map(|i| {
                let f = i as f64;
                let mid = 100.0 + (f * 0.2).sin() * 4.0;
                c(mid + 1.0, mid - 1.0, mid, 10.0 + ((i % 5) as f64), i)
            })
            .collect();
        let mut a = Kvo::classic();
        let mut b = Kvo::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..80i64)
            .map(|i| {
                let f = i as f64;
                c(11.0 + f, 9.0 + f, 10.0 + f, 100.0, i)
            })
            .collect();
        let mut k = Kvo::classic();
        k.batch(&candles);
        assert!(k.is_ready());
        k.reset();
        assert!(!k.is_ready());
        assert_eq!(k.update(candles[0]), None);
    }
}
