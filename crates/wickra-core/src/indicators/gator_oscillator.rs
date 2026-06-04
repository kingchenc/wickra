//! Bill Williams' Gator Oscillator (derived from the Alligator).

use crate::error::Result;
use crate::indicators::alligator::Alligator;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Gator Oscillator output: the two histogram bars drawn above and below the
/// zero line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GatorOscillatorOutput {
    /// Upper histogram `|jaw - teeth|`, always `>= 0`.
    pub upper: f64,
    /// Lower histogram `-|teeth - lips|`, always `<= 0`.
    pub lower: f64,
}

/// Bill Williams' Gator Oscillator: a convergence/divergence view of the
/// [`Alligator`] lines. The upper bar is the absolute gap between Jaw and
/// Teeth; the lower bar is the negated absolute gap between Teeth and Lips.
///
/// ```text
/// upper =  |jaw   - teeth|
/// lower = -|teeth - lips |
/// ```
///
/// Widening bars mean the Alligator's mouth is opening (a trending market);
/// shrinking bars mean it is closing (consolidation). Warmup matches the
/// underlying Alligator — the first value appears once the slowest line (Jaw)
/// has warmed up.
///
/// Reference: Bill Williams, *Trading Chaos*, 1995.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, GatorOscillator, Indicator};
///
/// let mut indicator = GatorOscillator::classic();
/// let mut last = None;
/// for i in 0..40 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 1.0, base - 1.0, base, 1.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct GatorOscillator {
    alligator: Alligator,
}

impl GatorOscillator {
    /// Construct a Gator Oscillator from explicit Alligator periods
    /// `(jaw, teeth, lips)`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`](crate::error::Error::PeriodZero) if any period is zero.
    pub fn new(jaw_period: usize, teeth_period: usize, lips_period: usize) -> Result<Self> {
        Ok(Self {
            alligator: Alligator::new(jaw_period, teeth_period, lips_period)?,
        })
    }

    /// Bill Williams' classic parameters: `(jaw = 13, teeth = 8, lips = 5)`.
    pub fn classic() -> Self {
        Self {
            alligator: Alligator::classic(),
        }
    }

    /// Configured `(jaw_period, teeth_period, lips_period)`.
    pub const fn periods(&self) -> (usize, usize, usize) {
        self.alligator.periods()
    }
}

impl Indicator for GatorOscillator {
    type Input = Candle;
    type Output = GatorOscillatorOutput;

    fn update(&mut self, candle: Candle) -> Option<GatorOscillatorOutput> {
        let lines = self.alligator.update(candle)?;
        Some(GatorOscillatorOutput {
            upper: (lines.jaw - lines.teeth).abs(),
            lower: -(lines.teeth - lines.lips).abs(),
        })
    }

    fn reset(&mut self) {
        self.alligator.reset();
    }

    fn warmup_period(&self) -> usize {
        self.alligator.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.alligator.is_ready()
    }

    fn name(&self) -> &'static str {
        "GatorOscillator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(high: f64, low: f64, ts: i64) -> Candle {
        let close = f64::midpoint(high, low);
        Candle::new(close, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(
            GatorOscillator::new(0, 8, 5),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            GatorOscillator::new(13, 0, 5),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            GatorOscillator::new(13, 8, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let g = GatorOscillator::classic();
        assert_eq!(g.periods(), (13, 8, 5));
        assert_eq!(g.warmup_period(), 13);
        assert_eq!(g.name(), "GatorOscillator");
        assert!(!g.is_ready());
    }

    #[test]
    fn constant_series_collapses_both_bars() {
        // All three Alligator lines equal the constant median -> zero spread.
        let mut g = GatorOscillator::classic();
        let candles: Vec<Candle> = (0..40).map(|i| candle(11.0, 9.0, i)).collect();
        let out = g.batch(&candles);
        let last = out.last().unwrap().unwrap();
        assert_relative_eq!(last.upper, 0.0, epsilon = 1e-12);
        assert_relative_eq!(last.lower, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn trending_series_opens_the_mouth() {
        // On a clean trend the lines separate -> upper > 0, lower < 0.
        let mut g = GatorOscillator::classic();
        let candles: Vec<Candle> = (0_i64..80)
            .map(|i| candle(10.0 + i as f64, 9.0 + i as f64, i))
            .collect();
        let last = g.batch(&candles).last().unwrap().unwrap();
        assert!(last.upper > 0.0, "upper {} should be positive", last.upper);
        assert!(last.lower < 0.0, "lower {} should be negative", last.lower);
    }

    #[test]
    fn warmup_emits_first_value_at_longest_period() {
        let mut g = GatorOscillator::new(5, 3, 2).unwrap();
        let candles: Vec<Candle> = (0..6).map(|i| candle(11.0, 9.0, i)).collect();
        let out = g.batch(&candles);
        for v in out.iter().take(4) {
            assert!(v.is_none());
        }
        assert!(out[4].is_some());
    }

    #[test]
    fn reset_clears_state() {
        let mut g = GatorOscillator::classic();
        let candles: Vec<Candle> = (0..40).map(|i| candle(11.0, 9.0, i)).collect();
        g.batch(&candles);
        assert!(g.is_ready());
        g.reset();
        assert!(!g.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80_i64)
            .map(|i| {
                let base = 100.0 + (i as f64 * 0.2).sin() * 5.0;
                candle(base + 1.0, base - 1.0, i)
            })
            .collect();
        let mut a = GatorOscillator::classic();
        let mut b = GatorOscillator::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|c| b.update(*c)).collect::<Vec<_>>()
        );
    }
}
