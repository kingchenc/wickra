//! Aroon Oscillator.

use crate::error::Result;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

use super::Aroon;

/// Aroon Oscillator — the single-line difference `AroonUp − AroonDown`.
///
/// The [`Aroon`] indicator reports two `[0, 100]` lines; the Aroon Oscillator
/// collapses them into one value in `[−100, 100]`:
///
/// ```text
/// AroonOscillator = AroonUp − AroonDown
/// ```
///
/// Strongly positive means the most recent high is much fresher than the most
/// recent low (an up-trend); strongly negative is the mirror image. Readings
/// near zero mean neither extreme is recent — a range.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, AroonOscillator};
///
/// let mut indicator = AroonOscillator::new(5).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + i as f64;
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert_eq!(last, Some(100.0)); // pure uptrend
/// ```
#[derive(Debug, Clone)]
pub struct AroonOscillator {
    aroon: Aroon,
    last: Option<f64>,
}

impl AroonOscillator {
    /// Construct a new Aroon Oscillator with the given period.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            aroon: Aroon::new(period)?,
            last: None,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.aroon.period()
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for AroonOscillator {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let osc = self.aroon.update(candle).map(|o| o.up - o.down)?;
        self.last = Some(osc);
        Some(osc)
    }

    fn reset(&mut self) {
        self.aroon.reset();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.aroon.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "AroonOscillator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(close, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn new_rejects_zero_period() {
        assert!(AroonOscillator::new(0).is_err());
    }

    /// Cover the const accessors `period` / `value` (57-64) and the
    /// Indicator-impl `name` body (90-92). `warmup_period` is covered
    /// already by `warmup_period_matches_aroon`.
    #[test]
    fn accessors_and_metadata() {
        let mut osc = AroonOscillator::new(7).unwrap();
        assert_eq!(osc.period(), 7);
        assert_eq!(osc.name(), "AroonOscillator");
        assert_eq!(osc.value(), None);
        for i in 0..8 {
            osc.update(candle(100.0 + f64::from(i), 90.0, 95.0, i64::from(i)));
        }
        assert!(osc.value().is_some());
    }

    #[test]
    fn pure_uptrend_yields_plus_100() {
        // Every bar a fresh high, no fresh low: AroonUp = 100, AroonDown = 0.
        let mut osc = AroonOscillator::new(5).unwrap();
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                let p = 100.0 + i as f64;
                candle(p + 1.0, p - 1.0, p, i)
            })
            .collect();
        for v in osc.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 100.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn pure_downtrend_yields_minus_100() {
        let mut osc = AroonOscillator::new(5).unwrap();
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                let p = 100.0 - i as f64;
                candle(p + 1.0, p - 1.0, p, i)
            })
            .collect();
        for v in osc.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, -100.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn output_stays_within_minus_100_and_100() {
        let mut osc = AroonOscillator::new(14).unwrap();
        let candles: Vec<Candle> = (0..200)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.25).sin() * 12.0;
                candle(mid + 2.0, mid - 2.0, mid, i)
            })
            .collect();
        for v in osc.batch(&candles).into_iter().flatten() {
            assert!((-100.0..=100.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn warmup_period_matches_aroon() {
        let osc = AroonOscillator::new(7).unwrap();
        assert_eq!(osc.warmup_period(), 8);
    }

    #[test]
    fn reset_clears_state() {
        let mut osc = AroonOscillator::new(5).unwrap();
        let candles: Vec<Candle> = (0..20)
            .map(|i| candle(100.0 + i as f64, 90.0, 95.0, i))
            .collect();
        osc.batch(&candles);
        assert!(osc.is_ready());
        osc.reset();
        assert!(!osc.is_ready());
        assert_eq!(osc.update(candles[0]), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.3).sin() * 8.0;
                candle(mid + 2.0, mid - 2.0, mid, i)
            })
            .collect();
        let batch = AroonOscillator::new(14).unwrap().batch(&candles);
        let mut b = AroonOscillator::new(14).unwrap();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }
}
