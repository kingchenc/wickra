//! Bill Williams' Alligator indicator.

use crate::error::{Error, Result};
use crate::indicators::smma::Smma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Alligator output: three smoothed moving averages of the median price
/// `(high + low) / 2`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlligatorOutput {
    /// `Jaw` — the slowest line (default period 13).
    pub jaw: f64,
    /// `Teeth` — the middle line (default period 8).
    pub teeth: f64,
    /// `Lips` — the fastest line (default period 5).
    pub lips: f64,
}

/// Bill Williams' Alligator: three `SMMA`s of the median price `(high + low) / 2`
/// with different periods. Classic parameters are `(jaw = 13, teeth = 8, lips = 5)`.
///
/// The original chart variant additionally shifts each line forward by a fixed
/// number of bars for display (Jaw +8, Teeth +5, Lips +3). Wickra publishes the
/// *unshifted* `SMMA` values — the consumer can apply the visual shift on the
/// chart side. The indicator emits values once all three `SMMA`s have warmed
/// up, i.e. after `max(jaw, teeth, lips) = jaw` candles.
///
/// Reference: Bill Williams, *Trading Chaos*, 1995.
///
/// # Example
///
/// ```
/// use wickra_core::{Alligator, Candle, Indicator};
///
/// let mut alligator = Alligator::classic();
/// let mut last = None;
/// for i in 0..40 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 1.0, base - 1.0, base, 1.0, i64::from(i)).unwrap();
///     last = alligator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Alligator {
    jaw_period: usize,
    teeth_period: usize,
    lips_period: usize,
    jaw: Smma,
    teeth: Smma,
    lips: Smma,
}

impl Alligator {
    /// # Errors
    /// Returns [`Error::PeriodZero`] if any period is zero.
    pub fn new(jaw_period: usize, teeth_period: usize, lips_period: usize) -> Result<Self> {
        if jaw_period == 0 || teeth_period == 0 || lips_period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            jaw_period,
            teeth_period,
            lips_period,
            jaw: Smma::new(jaw_period)?,
            teeth: Smma::new(teeth_period)?,
            lips: Smma::new(lips_period)?,
        })
    }

    /// Bill Williams' classic parameters: `(jaw = 13, teeth = 8, lips = 5)`.
    pub fn classic() -> Self {
        Self::new(13, 8, 5).expect("classic Alligator parameters are valid")
    }

    /// Configured `(jaw_period, teeth_period, lips_period)`.
    pub const fn periods(&self) -> (usize, usize, usize) {
        (self.jaw_period, self.teeth_period, self.lips_period)
    }
}

impl Indicator for Alligator {
    type Input = Candle;
    type Output = AlligatorOutput;

    fn update(&mut self, candle: Candle) -> Option<AlligatorOutput> {
        let median = f64::midpoint(candle.high, candle.low);
        // Feed every `SMMA` on every bar so they warm up in parallel; gating
        // the longer lines behind the shorter ones would starve them during
        // their own warmup.
        let lips = self.lips.update(median);
        let teeth = self.teeth.update(median);
        let jaw = self.jaw.update(median);
        Some(AlligatorOutput {
            jaw: jaw?,
            teeth: teeth?,
            lips: lips?,
        })
    }

    fn reset(&mut self) {
        self.jaw.reset();
        self.teeth.reset();
        self.lips.reset();
    }

    fn warmup_period(&self) -> usize {
        // All three SMMAs run on every bar, so readiness is gated by the
        // longest period — the Jaw with the default parameters.
        self.jaw_period.max(self.teeth_period).max(self.lips_period)
    }

    fn is_ready(&self) -> bool {
        self.jaw.is_ready() && self.teeth.is_ready() && self.lips.is_ready()
    }

    fn name(&self) -> &'static str {
        "Alligator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(high: f64, low: f64, ts: i64) -> Candle {
        let close = f64::midpoint(high, low);
        Candle::new(close, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(Alligator::new(0, 8, 5), Err(Error::PeriodZero)));
        assert!(matches!(Alligator::new(13, 0, 5), Err(Error::PeriodZero)));
        assert!(matches!(Alligator::new(13, 8, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let alligator = Alligator::classic();
        assert_eq!(alligator.periods(), (13, 8, 5));
        assert_eq!(alligator.warmup_period(), 13);
        assert_eq!(alligator.name(), "Alligator");
    }

    #[test]
    fn constant_series_yields_the_constant() {
        // Median price = 10 for every bar, so each SMMA seeds to 10 and stays.
        let mut alligator = Alligator::classic();
        let candles: Vec<Candle> = (0..40).map(|i| candle(11.0, 9.0, i)).collect();
        let out = alligator.batch(&candles);
        for v in out.iter().skip(12).flatten() {
            assert_relative_eq!(v.jaw, 10.0, epsilon = 1e-12);
            assert_relative_eq!(v.teeth, 10.0, epsilon = 1e-12);
            assert_relative_eq!(v.lips, 10.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn warmup_emits_first_value_at_longest_period() {
        let mut alligator = Alligator::new(5, 3, 2).unwrap();
        let candles: Vec<Candle> = (0..6).map(|i| candle(11.0, 9.0, i)).collect();
        let out = alligator.batch(&candles);
        for v in out.iter().take(4) {
            assert!(v.is_none());
        }
        assert!(out[4].is_some());
    }

    #[test]
    fn pure_uptrend_ordering() {
        // On a clean uptrend the fastest line (Lips, smallest SMMA) leads the
        // slowest line (Jaw) — lips > teeth > jaw at the latest bar.
        let mut alligator = Alligator::classic();
        let candles: Vec<Candle> = (0_i64..80)
            .map(|i| candle(10.0 + i as f64, 9.0 + i as f64, i))
            .collect();
        let out = alligator.batch(&candles);
        let last = out.last().unwrap().unwrap();
        assert!(
            last.lips > last.teeth,
            "lips {} > teeth {}",
            last.lips,
            last.teeth
        );
        assert!(
            last.teeth > last.jaw,
            "teeth {} > jaw {}",
            last.teeth,
            last.jaw
        );
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80_i64)
            .map(|i| {
                let base = 100.0 + (i as f64 * 0.2).sin() * 5.0;
                candle(base + 1.0, base - 1.0, i)
            })
            .collect();
        let mut a = Alligator::classic();
        let mut b = Alligator::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|c| b.update(*c)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut alligator = Alligator::classic();
        let candles: Vec<Candle> = (0..40).map(|i| candle(11.0, 9.0, i)).collect();
        alligator.batch(&candles);
        assert!(alligator.is_ready());
        alligator.reset();
        assert!(!alligator.is_ready());
    }
}
