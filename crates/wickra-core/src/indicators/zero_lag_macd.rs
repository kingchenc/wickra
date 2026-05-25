//! Zero-Lag MACD — MACD computed on `ZLEMA` instead of `EMA`.

use crate::error::{Error, Result};
use crate::indicators::zlema::Zlema;
use crate::traits::Indicator;

/// Multi-output for Zero-Lag MACD: the MACD line, its signal line, and the
/// histogram (line − signal).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZeroLagMacdOutput {
    /// Fast `ZLEMA` minus slow `ZLEMA`.
    pub macd: f64,
    /// `ZLEMA(macd, signal_period)`.
    pub signal: f64,
    /// `macd − signal`.
    pub histogram: f64,
}

/// Zero-Lag MACD — the standard `MACD` topology with `ZLEMA` substituted for
/// `EMA` everywhere. `ZLEMA`'s de-lagged construction makes the MACD line
/// react faster to trend changes at the cost of slightly noisier readings.
///
/// ```text
/// macd_t      = ZLEMA(close, fast)_t − ZLEMA(close, slow)_t
/// signal_t    = ZLEMA(macd, signal_period)_t
/// histogram_t = macd_t − signal_t
/// ```
///
/// Default parameters mirror MACD: `(fast = 12, slow = 26, signal = 9)`.
/// `fast` must be strictly less than `slow`.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, ZeroLagMacd};
///
/// let mut zmacd = ZeroLagMacd::classic();
/// let mut last = None;
/// for i in 0..120 {
///     last = zmacd.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct ZeroLagMacd {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    fast: Zlema,
    slow: Zlema,
    signal: Zlema,
}

impl ZeroLagMacd {
    /// # Errors
    /// - [`Error::PeriodZero`] if any period is zero.
    /// - [`Error::InvalidPeriod`] if `fast >= slow`.
    pub fn new(fast: usize, slow: usize, signal: usize) -> Result<Self> {
        if fast == 0 || slow == 0 || signal == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "ZeroLagMACD fast period must be strictly less than slow",
            });
        }
        Ok(Self {
            fast_period: fast,
            slow_period: slow,
            signal_period: signal,
            fast: Zlema::new(fast)?,
            slow: Zlema::new(slow)?,
            signal: Zlema::new(signal)?,
        })
    }

    /// MACD-style defaults: `(fast = 12, slow = 26, signal = 9)`.
    pub fn classic() -> Self {
        Self::new(12, 26, 9).expect("classic Zero-Lag MACD parameters are valid")
    }

    /// Configured `(fast, slow, signal)`.
    pub const fn periods(&self) -> (usize, usize, usize) {
        (self.fast_period, self.slow_period, self.signal_period)
    }
}

impl Indicator for ZeroLagMacd {
    type Input = f64;
    type Output = ZeroLagMacdOutput;

    fn update(&mut self, input: f64) -> Option<ZeroLagMacdOutput> {
        // Feed both inner ZLEMAs on every input so the slow one warms in
        // parallel with the fast one.
        let f = self.fast.update(input);
        let s = self.slow.update(input);
        let (f, s) = (f?, s?);
        let macd = f - s;
        let signal = self.signal.update(macd)?;
        Some(ZeroLagMacdOutput {
            macd,
            signal,
            histogram: macd - signal,
        })
    }

    fn reset(&mut self) {
        self.fast.reset();
        self.slow.reset();
        self.signal.reset();
    }

    fn warmup_period(&self) -> usize {
        // ZLEMA(period) warmup is `(period − 1) / 2 + period` = `lag + period`.
        // Both fast and slow run in parallel; the slow one dominates. The
        // signal ZLEMA then needs its own `lag + period` MACD values on top.
        let zlema_warmup = |period: usize| ((period - 1) / 2).saturating_add(period);
        zlema_warmup(self.slow_period) + zlema_warmup(self.signal_period) - 1
    }

    fn is_ready(&self) -> bool {
        self.signal.is_ready()
    }

    fn name(&self) -> &'static str {
        "ZeroLagMACD"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(ZeroLagMacd::new(0, 26, 9), Err(Error::PeriodZero)));
        assert!(matches!(ZeroLagMacd::new(12, 0, 9), Err(Error::PeriodZero)));
        assert!(matches!(
            ZeroLagMacd::new(12, 26, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn rejects_fast_geq_slow() {
        assert!(matches!(
            ZeroLagMacd::new(26, 12, 9),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let z = ZeroLagMacd::classic();
        assert_eq!(z.periods(), (12, 26, 9));
        assert_eq!(z.name(), "ZeroLagMACD");
    }

    #[test]
    fn classic_factory() {
        assert_eq!(ZeroLagMacd::classic().periods(), (12, 26, 9));
    }

    #[test]
    fn constant_series_converges_to_zero() {
        // Each ZLEMA reproduces a constant, so macd, signal and histogram
        // are all 0 after the slowest branch warms.
        let mut z = ZeroLagMacd::new(3, 5, 3).unwrap();
        let out = z.batch(&[42.0_f64; 60]);
        for v in out.iter().rev().take(5).flatten() {
            assert_relative_eq!(v.macd, 0.0, epsilon = 1e-12);
            assert_relative_eq!(v.signal, 0.0, epsilon = 1e-12);
            assert_relative_eq!(v.histogram, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn histogram_is_macd_minus_signal() {
        let mut z = ZeroLagMacd::classic();
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 5.0)
            .collect();
        for v in z.batch(&prices).iter().flatten() {
            assert_relative_eq!(v.histogram, v.macd - v.signal, epsilon = 1e-12);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 5.0)
            .collect();
        let mut a = ZeroLagMacd::classic();
        let mut b = ZeroLagMacd::classic();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut z = ZeroLagMacd::classic();
        z.batch(&(1..=120).map(f64::from).collect::<Vec<_>>());
        assert!(z.is_ready());
        z.reset();
        assert!(!z.is_ready());
    }

    #[test]
    fn warmup_period_matches_zlema_chain() {
        // warmup = zlema_warmup(slow) + zlema_warmup(signal) - 1
        // zlema_warmup(p) = (p - 1) / 2 + p
        // (12, 26, 9): zlema_warmup(26) = 12 + 26 = 38;
        //              zlema_warmup(9)  = 4 + 9 = 13.
        //              warmup = 38 + 13 - 1 = 50.
        let z = ZeroLagMacd::new(12, 26, 9).unwrap();
        assert_eq!(z.warmup_period(), 50);
        // (3, 5, 3): zlema_warmup(5) = 2 + 5 = 7; zlema_warmup(3) = 1 + 3 = 4.
        //            warmup = 7 + 4 - 1 = 10.
        let z = ZeroLagMacd::new(3, 5, 3).unwrap();
        assert_eq!(z.warmup_period(), 10);
    }
}
