//! Derivative Oscillator (Constance Brown).

use crate::error::{Error, Result};
use crate::indicators::ema::Ema;
use crate::indicators::rsi::Rsi;
use crate::indicators::sma::Sma;
use crate::traits::Indicator;

/// Derivative Oscillator — Constance Brown's double-smoothed RSI histogram.
///
/// The RSI is smoothed twice with EMAs, then a simple moving average of that
/// double-smoothed line is subtracted as a signal, leaving a zero-centered
/// histogram:
///
/// ```text
/// rsi   = RSI(price, rsi_period)
/// s1    = EMA(rsi, smooth1)
/// s2    = EMA(s1,  smooth2)          // double-smoothed RSI
/// signal = SMA(s2, signal_period)
/// DerivativeOscillator = s2 - signal
/// ```
///
/// The double EMA smoothing strips the RSI's high-frequency noise, and
/// subtracting the SMA signal removes the residual level, so the result
/// oscillates around zero: positive (and rising) bars mark accelerating bullish
/// momentum, negative bars bearish. Brown's defaults are `rsi_period = 14`,
/// `smooth1 = 5`, `smooth2 = 3`, `signal_period = 9`.
///
/// The first value lands after `rsi_period + smooth1 + smooth2 + signal_period − 2`
/// inputs, the point at which the whole RSI → EMA → EMA → SMA chain is seeded.
///
/// # Example
///
/// ```
/// use wickra_core::{DerivativeOscillator, Indicator};
///
/// let mut indicator = DerivativeOscillator::new(14, 5, 3, 9).unwrap();
/// let mut last = None;
/// for i in 0..120 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.2).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct DerivativeOscillator {
    rsi: Rsi,
    ema1: Ema,
    ema2: Ema,
    signal: Sma,
    warmup: usize,
}

impl DerivativeOscillator {
    /// Construct a Derivative Oscillator with the RSI, two EMA smoothing, and
    /// SMA signal periods.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if any period is `0`.
    pub fn new(
        rsi_period: usize,
        smooth1: usize,
        smooth2: usize,
        signal_period: usize,
    ) -> Result<Self> {
        if rsi_period == 0 || smooth1 == 0 || smooth2 == 0 || signal_period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            rsi: Rsi::new(rsi_period)?,
            ema1: Ema::new(smooth1)?,
            ema2: Ema::new(smooth2)?,
            signal: Sma::new(signal_period)?,
            // RSI seeds at rsi_period + 1, then each stage adds (len - 1).
            warmup: rsi_period + smooth1 + smooth2 + signal_period - 2,
        })
    }

    /// Total warmup length (also returned by `warmup_period`).
    pub const fn warmup(&self) -> usize {
        self.warmup
    }
}

impl Indicator for DerivativeOscillator {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        let rsi = self.rsi.update(input)?;
        let s1 = self.ema1.update(rsi)?;
        let s2 = self.ema2.update(s1)?;
        let signal = self.signal.update(s2)?;
        Some(s2 - signal)
    }

    fn reset(&mut self) {
        self.rsi.reset();
        self.ema1.reset();
        self.ema2.reset();
        self.signal.reset();
    }

    fn warmup_period(&self) -> usize {
        self.warmup
    }

    fn is_ready(&self) -> bool {
        self.signal.is_ready()
    }

    fn name(&self) -> &'static str {
        "DerivativeOscillator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_periods() {
        assert!(matches!(
            DerivativeOscillator::new(0, 5, 3, 9),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            DerivativeOscillator::new(14, 0, 3, 9),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            DerivativeOscillator::new(14, 5, 0, 9),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            DerivativeOscillator::new(14, 5, 3, 0),
            Err(Error::PeriodZero)
        ));
    }

    /// Cover the const accessor `warmup` and the Indicator-impl `warmup_period`
    /// + `name`.
    #[test]
    fn accessors_and_metadata() {
        let d = DerivativeOscillator::new(14, 5, 3, 9).unwrap();
        // 14 + 5 + 3 + 9 - 2 = 29.
        assert_eq!(d.warmup(), 29);
        assert_eq!(d.warmup_period(), 29);
        assert_eq!(d.name(), "DerivativeOscillator");
    }

    #[test]
    fn first_emission_matches_warmup_period() {
        let prices: Vec<f64> = (0..60)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 6.0)
            .collect();
        let mut d = DerivativeOscillator::new(14, 5, 3, 9).unwrap();
        let out = d.batch(&prices);
        let warmup = d.warmup_period();
        for (i, v) in out.iter().enumerate().take(warmup - 1) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(
            out[warmup - 1].is_some(),
            "first value must land at warmup_period - 1"
        );
    }

    #[test]
    fn matches_manual_chain() {
        // Equals RSI -> EMA -> EMA, minus the SMA signal of that line.
        let prices: Vec<f64> = (0..80)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 8.0)
            .collect();
        let mut d = DerivativeOscillator::new(14, 5, 3, 9).unwrap();
        let mut rsi = Rsi::new(14).unwrap();
        let mut e1 = Ema::new(5).unwrap();
        let mut e2 = Ema::new(3).unwrap();
        let mut sig = Sma::new(9).unwrap();
        for (i, &p) in prices.iter().enumerate() {
            let got = d.update(p);
            let want = rsi
                .update(p)
                .and_then(|r| e1.update(r))
                .and_then(|x| e2.update(x))
                .and_then(|s2| sig.update(s2).map(|s| s2 - s));
            assert_eq!(got.is_some(), want.is_some(), "readiness mismatch at {i}");
            if let (Some(a), Some(b)) = (got, want) {
                assert_relative_eq!(a, b, epsilon = 1e-9);
            }
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut d = DerivativeOscillator::new(14, 5, 3, 9).unwrap();
        d.batch(&(0..60).map(|i| 100.0 + f64::from(i)).collect::<Vec<_>>());
        assert!(d.is_ready());
        d.reset();
        assert!(!d.is_ready());
        assert_eq!(d.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..80)
            .map(|i| 50.0 + (f64::from(i) * 0.5).sin() * 10.0)
            .collect();
        let mut a = DerivativeOscillator::new(14, 5, 3, 9).unwrap();
        let mut b = DerivativeOscillator::new(14, 5, 3, 9).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }
}
