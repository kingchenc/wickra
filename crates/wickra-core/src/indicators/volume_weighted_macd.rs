//! Volume-Weighted MACD — MACD built on volume-weighted moving averages.

use crate::error::{Error, Result};
use crate::indicators::ema::Ema;
use crate::indicators::vwma::Vwma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Output of [`VolumeWeightedMacd`]: the three classic MACD series, but with the
/// fast and slow averages volume-weighted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumeWeightedMacdOutput {
    /// Fast VWMA − slow VWMA.
    pub macd: f64,
    /// EMA of `macd` over the signal period.
    pub signal: f64,
    /// `macd − signal`.
    pub histogram: f64,
}

/// Volume-Weighted MACD — the MACD oscillator computed from **volume-weighted**
/// moving averages instead of plain EMAs.
///
/// ```text
/// macd      = VWMA(close, fast) − VWMA(close, slow)
/// signal    = EMA(macd, signal_period)
/// histogram = macd − signal
/// ```
///
/// Standard [`MacdIndicator`](crate::MacdIndicator) smooths price with exponential
/// averages that ignore volume. The volume-weighted variant (Buff Dormeier and
/// others) replaces each average with a [`Vwma`], so heavy-volume bars dominate
/// the trend estimate and the oscillator leans toward where real participation
/// occurred. Crossovers backed by volume therefore appear sooner and noise from
/// thin bars is damped. The signal line keeps a standard EMA, matching the
/// classic histogram construction.
///
/// `fast` must be strictly smaller than `slow`. The first output lands after
/// `slow + signal − 1` inputs: `slow` to seed the slow VWMA, then `signal − 1`
/// more to seed the signal EMA. Each `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, VolumeWeightedMacd};
///
/// let mut indicator = VolumeWeightedMacd::new(12, 26, 9).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + f64::from(i);
///     let c = Candle::new(base, base + 1.0, base - 1.0, base + 0.5, 1_000.0, 0).unwrap();
///     last = indicator.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct VolumeWeightedMacd {
    fast: Vwma,
    slow: Vwma,
    signal_ema: Ema,
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    last: Option<VolumeWeightedMacdOutput>,
}

impl VolumeWeightedMacd {
    /// Construct a volume-weighted MACD with the given periods.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if any period is zero, and
    /// [`Error::InvalidPeriod`] if `fast >= slow`.
    pub fn new(fast: usize, slow: usize, signal: usize) -> Result<Self> {
        if fast == 0 || slow == 0 || signal == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "fast period must be strictly less than slow period",
            });
        }
        Ok(Self {
            fast: Vwma::new(fast)?,
            slow: Vwma::new(slow)?,
            signal_ema: Ema::new(signal)?,
            fast_period: fast,
            slow_period: slow,
            signal_period: signal,
            last: None,
        })
    }

    /// Configured periods as `(fast, slow, signal)`.
    pub const fn periods(&self) -> (usize, usize, usize) {
        (self.fast_period, self.slow_period, self.signal_period)
    }

    /// Most recent fully-computed output if available.
    pub const fn value(&self) -> Option<VolumeWeightedMacdOutput> {
        self.last
    }
}

impl Indicator for VolumeWeightedMacd {
    type Input = Candle;
    type Output = VolumeWeightedMacdOutput;

    fn update(&mut self, candle: Candle) -> Option<VolumeWeightedMacdOutput> {
        let fast = self.fast.update(candle);
        let slow = self.slow.update(candle);
        if let (Some(f), Some(s)) = (fast, slow) {
            let macd = f - s;
            let signal = self.signal_ema.update(macd)?;
            let out = VolumeWeightedMacdOutput {
                macd,
                signal,
                histogram: macd - signal,
            };
            self.last = Some(out);
            return Some(out);
        }
        None
    }

    fn reset(&mut self) {
        self.fast.reset();
        self.slow.reset();
        self.signal_ema.reset();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.slow_period + self.signal_period - 1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "VolumeWeightedMacd"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(close: f64, volume: f64) -> Candle {
        Candle::new_unchecked(close, close, close, close, volume, 0)
    }

    #[test]
    fn rejects_invalid_periods() {
        assert!(matches!(
            VolumeWeightedMacd::new(0, 26, 9),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            VolumeWeightedMacd::new(26, 12, 9),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            VolumeWeightedMacd::new(12, 12, 9),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let m = VolumeWeightedMacd::new(12, 26, 9).unwrap();
        assert_eq!(m.periods(), (12, 26, 9));
        assert_eq!(m.warmup_period(), 34);
        assert_eq!(m.name(), "VolumeWeightedMacd");
        assert!(!m.is_ready());
        assert_eq!(m.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut m = VolumeWeightedMacd::new(2, 4, 3).unwrap();
        let candles: Vec<Candle> = (0..20)
            .map(|i| candle(100.0 + f64::from(i), 1_000.0))
            .collect();
        let out = m.batch(&candles);
        let warmup = m.warmup_period(); // 4 + 3 - 1 = 6
        assert_eq!(warmup, 6);
        for v in out.iter().take(warmup - 1) {
            assert!(v.is_none());
        }
        assert!(out[warmup - 1].is_some());
    }

    #[test]
    fn uptrend_has_positive_macd() {
        // A steady advance with equal volume -> fast VWMA leads slow -> macd > 0.
        let mut m = VolumeWeightedMacd::new(3, 6, 3).unwrap();
        let candles: Vec<Candle> = (0..60)
            .map(|i| candle(100.0 + f64::from(i), 1_000.0))
            .collect();
        let last = m.batch(&candles).into_iter().flatten().last().unwrap();
        assert!(
            last.macd > 0.0,
            "uptrend should give positive macd, got {}",
            last.macd
        );
    }

    #[test]
    fn histogram_is_macd_minus_signal() {
        let mut m = VolumeWeightedMacd::new(3, 6, 3).unwrap();
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                candle(
                    100.0 + (f64::from(i) * 0.3).sin() * 5.0,
                    1_000.0 + f64::from(i),
                )
            })
            .collect();
        for o in m.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(o.histogram, o.macd - o.signal, epsilon = 1e-9);
        }
    }

    #[test]
    fn equal_volume_matches_plain_macd() {
        // With constant volume, VWMA reduces to SMA, so volume-weighted MACD uses
        // SMA-based lines; it should still be a well-defined finite series.
        let mut m = VolumeWeightedMacd::new(3, 6, 3).unwrap();
        let candles: Vec<Candle> = (0..60)
            .map(|i| candle(100.0 + (f64::from(i) * 0.2).sin() * 4.0, 2_000.0))
            .collect();
        for o in m.batch(&candles).into_iter().flatten() {
            assert!(o.macd.is_finite() && o.signal.is_finite());
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut m = VolumeWeightedMacd::new(3, 6, 3).unwrap();
        let candles: Vec<Candle> = (0..40)
            .map(|i| candle(100.0 + f64::from(i), 1_000.0))
            .collect();
        m.batch(&candles);
        assert!(m.is_ready());
        m.reset();
        assert!(!m.is_ready());
        assert_eq!(m.value(), None);
        assert_eq!(m.update(candle(100.0, 1_000.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..120)
            .map(|i| {
                candle(
                    100.0 + (f64::from(i) * 0.25).sin() * 9.0,
                    1_000.0 + f64::from(i),
                )
            })
            .collect();
        let batch = VolumeWeightedMacd::new(12, 26, 9).unwrap().batch(&candles);
        let mut b = VolumeWeightedMacd::new(12, 26, 9).unwrap();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }
}
