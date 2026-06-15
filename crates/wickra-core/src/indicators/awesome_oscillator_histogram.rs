//! Awesome Oscillator Histogram.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::indicators::awesome_oscillator::AwesomeOscillator;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Awesome Oscillator Histogram — the bar-to-bar **momentum** of the Awesome
/// Oscillator over a `lookback` window. This is the value behind the coloured
/// histogram bars in Bill Williams' charts: each bar shows how much `AO` has
/// changed, so positive values mean `AO` is rising (the histogram "greens up")
/// and negative values mean it is falling.
///
/// ```text
/// AO     = SMA(median, fast) − SMA(median, slow)
/// AOHist = AO_t − AO_{t−lookback}
/// ```
///
/// This is distinct from the two related indicators Wickra ships: the raw
/// [`AwesomeOscillator`](crate::AwesomeOscillator) is `AO` itself, and the
/// [`AcceleratorOscillator`](crate::AcceleratorOscillator) is `AO − SMA(AO, n)`.
/// The histogram instead reports `AO`'s rate of change. The default `lookback`
/// is `1` (the classic one-bar histogram delta).
///
/// # Example
///
/// ```
/// use wickra_core::{AwesomeOscillatorHistogram, Candle, Indicator};
///
/// let mut hist = AwesomeOscillatorHistogram::classic();
/// let mut last = None;
/// for i in 0..80 {
///     let p = 100.0 + f64::from(i);
///     let candle = Candle::new(p, p + 0.5, p - 0.5, p, 1.0, i64::from(i)).unwrap();
///     last = hist.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct AwesomeOscillatorHistogram {
    fast_period: usize,
    slow_period: usize,
    lookback: usize,
    ao: AwesomeOscillator,
    history: VecDeque<f64>,
    emitted: bool,
}

impl AwesomeOscillatorHistogram {
    /// # Errors
    /// - [`Error::PeriodZero`] if any period is zero.
    /// - [`Error::InvalidPeriod`] if `fast >= slow`.
    pub fn new(fast: usize, slow: usize, lookback: usize) -> Result<Self> {
        if fast == 0 || slow == 0 || lookback == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "AwesomeOscillatorHistogram fast must be strictly less than slow",
            });
        }
        Ok(Self {
            fast_period: fast,
            slow_period: slow,
            lookback,
            ao: AwesomeOscillator::new(fast, slow)?,
            history: VecDeque::with_capacity(lookback + 1),
            emitted: false,
        })
    }

    /// Bill Williams' defaults with a one-bar histogram delta `(5, 34, 1)`.
    pub fn classic() -> Self {
        Self::new(5, 34, 1).expect("classic Awesome Oscillator Histogram parameters are valid")
    }

    /// Configured `(fast_period, slow_period, lookback)`.
    pub const fn periods(&self) -> (usize, usize, usize) {
        (self.fast_period, self.slow_period, self.lookback)
    }
}

impl Indicator for AwesomeOscillatorHistogram {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let ao = self.ao.update(candle)?;
        self.history.push_back(ao);
        if self.history.len() <= self.lookback {
            return None;
        }
        let prev = self.history.pop_front().expect("history is non-empty");
        self.emitted = true;
        Some(ao - prev)
    }

    fn reset(&mut self) {
        self.ao.reset();
        self.history.clear();
        self.emitted = false;
    }

    fn warmup_period(&self) -> usize {
        // AO first emits at `slow` candles; `lookback` more AO values are then
        // needed before `AO_t − AO_{t−lookback}` can be formed.
        self.slow_period + self.lookback
    }

    fn is_ready(&self) -> bool {
        self.emitted
    }

    fn name(&self) -> &'static str {
        "AwesomeOscillatorHistogram"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(price: f64, ts: i64) -> Candle {
        Candle::new(price, price + 0.5, price - 0.5, price, 1.0, ts).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(
            AwesomeOscillatorHistogram::new(0, 34, 1),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            AwesomeOscillatorHistogram::new(5, 0, 1),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            AwesomeOscillatorHistogram::new(5, 34, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn rejects_fast_geq_slow() {
        assert!(matches!(
            AwesomeOscillatorHistogram::new(34, 5, 1),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let hist = AwesomeOscillatorHistogram::classic();
        assert_eq!(hist.periods(), (5, 34, 1));
        assert_eq!(hist.warmup_period(), 35);
        assert_eq!(hist.name(), "AwesomeOscillatorHistogram");
    }

    #[test]
    fn constant_series_yields_zero() {
        // AO of a flat series is 0, so its momentum is 0.
        let mut hist = AwesomeOscillatorHistogram::new(3, 5, 1).unwrap();
        let candles: Vec<Candle> = (0..30).map(|i| candle(42.0, i)).collect();
        let out = hist.batch(&candles);
        for v in out.iter().skip(hist.warmup_period() - 1).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn warmup_emits_first_value_at_warmup_period() {
        let mut hist = AwesomeOscillatorHistogram::new(2, 4, 2).unwrap();
        assert_eq!(hist.warmup_period(), 6);
        let candles: Vec<Candle> = (0..8)
            .map(|i| candle(10.0 + f64::from(i), i64::from(i)))
            .collect();
        let out = hist.batch(&candles);
        for v in out.iter().take(5) {
            assert!(v.is_none());
        }
        assert!(out[5].is_some());
    }

    #[test]
    fn equals_ao_difference() {
        // The histogram must equal AO_t − AO_{t−lookback} bar for bar.
        let candles: Vec<Candle> = (0..60_i64)
            .map(|i| candle(100.0 + (i as f64 * 0.3).sin() * 5.0, i))
            .collect();
        let lookback = 1;
        let ao_series = AwesomeOscillator::new(5, 34).unwrap().batch(&candles);
        let hist = AwesomeOscillatorHistogram::new(5, 34, lookback)
            .unwrap()
            .batch(&candles);
        for i in 0..candles.len() {
            if let Some(h) = hist[i] {
                let ao_now = ao_series[i].expect("AO present once histogram emits");
                let ao_prev =
                    ao_series[i - lookback].expect("prior AO present once histogram emits");
                assert_relative_eq!(h, ao_now - ao_prev, epsilon = 1e-9);
            }
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..100_i64)
            .map(|i| candle(100.0 + (i as f64 * 0.3).sin() * 5.0, i))
            .collect();
        let batch = AwesomeOscillatorHistogram::classic().batch(&candles);
        let mut b = AwesomeOscillatorHistogram::classic();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn reset_clears_state() {
        let mut hist = AwesomeOscillatorHistogram::classic();
        let candles: Vec<Candle> = (0..80)
            .map(|i| candle(10.0 + f64::from(i), i64::from(i)))
            .collect();
        hist.batch(&candles);
        assert!(hist.is_ready());
        hist.reset();
        assert!(!hist.is_ready());
    }
}
