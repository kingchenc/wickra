//! Williams A/D Oscillator (ADOSC).

use crate::indicators::sma::Sma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Smoothing window applied to the Williams A/D line to form the oscillator.
const SIGNAL_PERIOD: usize = 13;

/// Williams **A/D Oscillator** — the volume-free Williams Accumulation/
/// Distribution line measured against its own moving average, so it oscillates
/// around zero instead of drifting like the cumulative line.
///
/// The underlying line is Larry Williams' volume-less A/D (1972), which uses a
/// *true* high/low anchored on the prior close; the oscillator subtracts its
/// 13-bar simple moving average:
///
/// ```text
/// TR_h_t = max(close_{t−1}, high_t)
/// TR_l_t = min(close_{t−1}, low_t)
/// WAD_t  = WAD_{t−1} + (close_t − TR_l_t)   if close_t > close_{t−1}
/// WAD_t  = WAD_{t−1} + (close_t − TR_h_t)   if close_t < close_{t−1}
/// WAD_t  = WAD_{t−1}                          if close_t == close_{t−1}
/// ADOSC_t = WAD_t − SMA(WAD, 13)_t
/// ```
///
/// This is distinct from the raw cumulative line, which Wickra ships as
/// [`Wad`](crate::Wad): `Wad` is the drifting line for divergence analysis,
/// while this oscillator is its zero-centred, mean-reverting form (positive
/// when accumulation is running ahead of its recent average, negative when
/// distribution is). The first bar only seeds the previous close; the first
/// oscillator value lands once the 13-bar average of the line is full.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, AdOscillator};
///
/// let mut indicator = AdOscillator::new();
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
pub struct AdOscillator {
    prev_close: Option<f64>,
    line: f64,
    signal: Sma,
    last: Option<f64>,
}

impl Default for AdOscillator {
    fn default() -> Self {
        Self::new()
    }
}

impl AdOscillator {
    /// Construct a new Williams A/D Oscillator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            prev_close: None,
            line: 0.0,
            signal: Sma::new(SIGNAL_PERIOD).expect("SIGNAL_PERIOD is non-zero"),
            last: None,
        }
    }

    /// Current oscillator value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for AdOscillator {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev) = self.prev_close else {
            // The first bar only establishes the previous close anchor.
            self.prev_close = Some(candle.close);
            return None;
        };
        let delta = if candle.close > prev {
            // Accumulation: distance from the true low.
            candle.close - prev.min(candle.low)
        } else if candle.close < prev {
            // Distribution: distance from the true high (negative).
            candle.close - prev.max(candle.high)
        } else {
            0.0
        };
        self.line += delta;
        self.prev_close = Some(candle.close);
        let signal = self.signal.update(self.line)?;
        let osc = self.line - signal;
        self.last = Some(osc);
        Some(osc)
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.line = 0.0;
        self.signal.reset();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // One seed bar establishes the prior close; the line then feeds the
        // 13-bar signal SMA, which is full after `SIGNAL_PERIOD` line values.
        1 + SIGNAL_PERIOD
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "ADOSC"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::wad::Wad;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(open: f64, high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(open, high, low, close, 100.0, ts).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let ad = AdOscillator::new();
        assert_eq!(ad.name(), "ADOSC");
        assert_eq!(ad.warmup_period(), 14);
        assert!(!ad.is_ready());
        assert_eq!(ad.value(), None);
        // `Default` matches `new`.
        assert_eq!(AdOscillator::default().warmup_period(), 14);
    }

    #[test]
    fn seed_bar_returns_none() {
        let mut ad = AdOscillator::new();
        assert_eq!(ad.update(c(100.0, 101.0, 99.0, 100.0, 0)), None);
    }

    #[test]
    fn equals_wad_line_minus_its_sma() {
        // The oscillator is exactly the Williams A/D line minus its 13-SMA, so
        // it must match the standalone `Wad` line passed through an SMA(13).
        let candles: Vec<Candle> = (0..80_i64)
            .map(|i| {
                let base = 100.0 + (i as f64 * 0.3).sin() * 6.0;
                c(
                    base,
                    base + 2.0,
                    base - 2.0,
                    base + (i as f64 * 0.5).cos(),
                    i,
                )
            })
            .collect();
        let osc = AdOscillator::new().batch(&candles);
        // Reconstruct: Wad line, then line − SMA(line, 13).
        let line = Wad::new().batch(&candles);
        let mut sma = Sma::new(SIGNAL_PERIOD).unwrap();
        let expected: Vec<Option<f64>> = line
            .iter()
            .map(|v| v.and_then(|l| sma.update(l).map(|s| l - s)))
            .collect();
        assert_eq!(osc, expected);
    }

    #[test]
    fn flat_market_oscillates_at_zero() {
        // A flat market never accumulates or distributes, so the line is
        // constant and the oscillator sits at zero once warm.
        let mut ad = AdOscillator::new();
        let candles: Vec<Candle> = (0..40).map(|i| c(50.0, 50.0, 50.0, 50.0, i)).collect();
        let out = ad.batch(&candles);
        for v in out.iter().skip(ad.warmup_period() - 1).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn warmup_emits_at_warmup_period() {
        let mut ad = AdOscillator::new();
        let candles: Vec<Candle> = (0..20)
            .map(|i| {
                let close = 100.0 + f64::from(i);
                c(close, close + 2.0, close - 2.0, close, i64::from(i))
            })
            .collect();
        let out = ad.batch(&candles);
        assert_eq!(ad.warmup_period(), 14);
        for v in out.iter().take(13) {
            assert!(v.is_none());
        }
        assert!(out[13].is_some());
    }

    #[test]
    fn reset_clears_state() {
        let mut ad = AdOscillator::new();
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                let close = 100.0 + f64::from(i);
                c(close, close + 2.0, close - 2.0, close, i64::from(i))
            })
            .collect();
        ad.batch(&candles);
        assert!(ad.is_ready());
        ad.reset();
        assert!(!ad.is_ready());
        assert_eq!(ad.value(), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..100_i64)
            .map(|i| {
                let base = 100.0 + (i as f64 * 0.2).sin() * 5.0;
                c(base, base + 1.5, base - 1.5, base + 0.4, i)
            })
            .collect();
        let batch = AdOscillator::new().batch(&candles);
        let mut s = AdOscillator::new();
        let streamed: Vec<_> = candles.iter().map(|x| s.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
