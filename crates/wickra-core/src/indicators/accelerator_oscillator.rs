//! Accelerator Oscillator (Bill Williams).

use crate::error::Result;
use crate::indicators::awesome_oscillator::AwesomeOscillator;
use crate::indicators::sma::Sma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Accelerator Oscillator — Bill Williams' gauge of *momentum's acceleration*.
///
/// ```text
/// AO = SMA(median, fast) − SMA(median, slow)   (the Awesome Oscillator)
/// AC = AO − SMA(AO, signal)
/// ```
///
/// Where the [`AwesomeOscillator`](crate::AwesomeOscillator) tracks momentum,
/// the Accelerator tracks the *change* in momentum: it is the AO minus a short
/// moving average of itself. Because acceleration leads speed, `AC` tends to
/// turn before the `AO` does. Bill Williams' classic configuration is the
/// `(5, 34)` AO with a `5`-period signal average.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, AcceleratorOscillator};
///
/// let mut indicator = AcceleratorOscillator::classic();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct AcceleratorOscillator {
    ao: AwesomeOscillator,
    signal: Sma,
    ao_fast: usize,
    ao_slow: usize,
    signal_period: usize,
}

impl AcceleratorOscillator {
    /// Construct an Accelerator Oscillator with explicit AO and signal periods.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`](crate::Error::PeriodZero) for a zero
    /// period and [`Error::InvalidPeriod`](crate::Error::InvalidPeriod) if the
    /// AO `fast` period is not strictly below `slow`.
    pub fn new(ao_fast: usize, ao_slow: usize, signal_period: usize) -> Result<Self> {
        Ok(Self {
            ao: AwesomeOscillator::new(ao_fast, ao_slow)?,
            signal: Sma::new(signal_period)?,
            ao_fast,
            ao_slow,
            signal_period,
        })
    }

    /// Bill Williams' classic configuration: `AO(5, 34)` with a `5`-period signal.
    pub fn classic() -> Self {
        Self::new(5, 34, 5).expect("classic Accelerator Oscillator params are valid")
    }

    /// Configured `(ao_fast, ao_slow, signal_period)`.
    pub const fn params(&self) -> (usize, usize, usize) {
        (self.ao_fast, self.ao_slow, self.signal_period)
    }
}

impl Indicator for AcceleratorOscillator {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let ao = self.ao.update(candle)?;
        let signal = self.signal.update(ao)?;
        Some(ao - signal)
    }

    fn reset(&mut self) {
        self.ao.reset();
        self.signal.reset();
    }

    fn warmup_period(&self) -> usize {
        // The AO emits at candle `ao_slow`; the signal SMA then needs
        // `signal_period` AO values.
        self.ao_slow + self.signal_period - 1
    }

    fn is_ready(&self) -> bool {
        self.signal.is_ready()
    }

    fn name(&self) -> &'static str {
        "AcceleratorOscillator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(f64::midpoint(high, low), high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn constant_series_yields_zero() {
        // A flat market gives AO = 0, so its signal average and AC are 0 too.
        let candles: Vec<Candle> = (0..80).map(|i| c(11.0, 9.0, 10.0, i)).collect();
        let mut ac = AcceleratorOscillator::classic();
        for v in ac.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn matches_independent_ao_and_signal() {
        let candles: Vec<Candle> = (0..90)
            .map(|i| {
                let m = 100.0 + (i as f64 * 0.2).sin() * 6.0;
                c(m + 1.5, m - 1.5, m + 0.3, i)
            })
            .collect();
        let mut ac = AcceleratorOscillator::classic();
        let mut ao = AwesomeOscillator::classic();
        let mut signal = Sma::new(5).unwrap();
        for (i, candle) in candles.iter().enumerate() {
            let got = ac.update(*candle);
            match ao.update(*candle) {
                Some(ao_val) => match signal.update(ao_val) {
                    Some(sig) => {
                        assert_relative_eq!(got.unwrap(), ao_val - sig, epsilon = 1e-9);
                    }
                    None => assert!(got.is_none(), "i={i}"),
                },
                None => assert!(got.is_none(), "i={i}"),
            }
        }
    }

    #[test]
    fn first_emission_matches_warmup_period() {
        let candles: Vec<Candle> = (0..60).map(|i| c(11.0, 9.0, 10.0, i)).collect();
        let mut ac = AcceleratorOscillator::classic();
        let out = ac.batch(&candles);
        assert_eq!(ac.warmup_period(), 38);
        for (i, v) in out.iter().enumerate().take(37) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(out[37].is_some(), "first value lands at warmup_period - 1");
    }

    #[test]
    fn rejects_invalid_params() {
        assert!(AcceleratorOscillator::new(0, 34, 5).is_err());
        assert!(AcceleratorOscillator::new(5, 34, 0).is_err());
        assert!(AcceleratorOscillator::new(34, 5, 5).is_err());
    }

    /// Cover the const accessor `params` (69-71) and the Indicator-impl
    /// `name` body (99-101). Existing tests inspect numeric output but
    /// never query the metadata.
    #[test]
    fn accessors_and_metadata() {
        let ac = AcceleratorOscillator::classic();
        assert_eq!(ac.params(), (5, 34, 5));
        assert_eq!(ac.name(), "AcceleratorOscillator");
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..60).map(|i| c(11.0, 9.0, 10.0, i)).collect();
        let mut ac = AcceleratorOscillator::classic();
        ac.batch(&candles);
        assert!(ac.is_ready());
        ac.reset();
        assert!(!ac.is_ready());
        assert_eq!(ac.update(candles[0]), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..90)
            .map(|i| {
                let m = 100.0 + (i as f64 * 0.3).sin() * 8.0;
                c(m + 1.5, m - 1.5, m + 0.5, i)
            })
            .collect();
        let mut a = AcceleratorOscillator::classic();
        let mut b = AcceleratorOscillator::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
