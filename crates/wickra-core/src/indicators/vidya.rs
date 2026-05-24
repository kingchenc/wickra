//! Variable Index Dynamic Average (VIDYA).

use crate::error::{Error, Result};
use crate::indicators::cmo::Cmo;
use crate::traits::Indicator;

/// Tushar Chande's Variable Index Dynamic Average — an EMA whose smoothing
/// factor is scaled by the absolute Chande Momentum Oscillator (`CMO`).
///
/// Strong directional momentum (high `|CMO|`) pushes the effective smoothing
/// constant toward the EMA-of-`period`'s natural rate; flat / choppy windows
/// (`|CMO|` close to zero) shrink it toward zero so VIDYA coasts on its prior
/// value:
///
/// ```text
/// alpha_base = 2 / (period + 1)
/// alpha_t    = alpha_base * |CMO(cmo_period)| / 100
/// VIDYA_t    = alpha_t * price_t + (1 - alpha_t) * VIDYA_{t-1}
/// ```
///
/// The series is seeded with the first price emitted after the `CMO`
/// warm-up (i.e. after `cmo_period + 1` inputs).
///
/// Reference: Tushar Chande, *Stocks & Commodities*, 1992.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Vidya};
///
/// let mut vidya = Vidya::new(14, 9).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = vidya.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Vidya {
    period: usize,
    cmo_period: usize,
    alpha_base: f64,
    cmo: Cmo,
    current: Option<f64>,
}

impl Vidya {
    /// # Errors
    /// Returns [`Error::PeriodZero`] if either period is zero.
    pub fn new(period: usize, cmo_period: usize) -> Result<Self> {
        if period == 0 || cmo_period == 0 {
            return Err(Error::PeriodZero);
        }
        let alpha_base = 2.0 / (period as f64 + 1.0);
        Ok(Self {
            period,
            cmo_period,
            alpha_base,
            cmo: Cmo::new(cmo_period)?,
            current: None,
        })
    }

    /// Configured `(period, cmo_period)`.
    pub const fn periods(&self) -> (usize, usize) {
        (self.period, self.cmo_period)
    }
}

impl Indicator for Vidya {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.current;
        }
        let cmo = self.cmo.update(input)?;
        let alpha = self.alpha_base * (cmo.abs() / 100.0);
        let prev = self.current.unwrap_or(input);
        let next = alpha * input + (1.0 - alpha) * prev;
        self.current = Some(next);
        Some(next)
    }

    fn reset(&mut self) {
        self.cmo.reset();
        self.current = None;
    }

    fn warmup_period(&self) -> usize {
        self.cmo_period + 1
    }

    fn is_ready(&self) -> bool {
        self.current.is_some()
    }

    fn name(&self) -> &'static str {
        "VIDYA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(Vidya::new(0, 9), Err(Error::PeriodZero)));
        assert!(matches!(Vidya::new(14, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let v = Vidya::new(14, 9).unwrap();
        assert_eq!(v.periods(), (14, 9));
        assert_eq!(v.warmup_period(), 10);
        assert_eq!(v.name(), "VIDYA");
    }

    #[test]
    fn constant_series_yields_the_constant() {
        // Flat input -> CMO = 0 -> alpha = 0 -> VIDYA holds its seed value.
        let mut v = Vidya::new(14, 4).unwrap();
        let out = v.batch(&[42.0_f64; 30]);
        for x in out.iter().skip(4).flatten() {
            assert_relative_eq!(*x, 42.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn pure_uptrend_alpha_equals_base() {
        // Monotonic uptrend: CMO saturates at +100, so alpha = alpha_base.
        // After warmup the recurrence is a plain EMA with that alpha; once
        // the series is long enough VIDYA closely tracks the latest input.
        let mut v = Vidya::new(2, 4).unwrap();
        let prices: Vec<f64> = (1..=40).map(f64::from).collect();
        let out = v.batch(&prices);
        let last = out.last().unwrap().unwrap();
        let latest = *prices.last().unwrap();
        // alpha_base = 2/3, EMA(2) tracks close — last value is within 2 of
        // the latest input after this many bars.
        assert!(
            (latest - last).abs() < 2.0,
            "VIDYA should track close on a clean uptrend: {last} vs {latest}"
        );
    }

    #[test]
    fn warmup_emits_first_value_at_cmo_period_plus_one() {
        let mut v = Vidya::new(14, 3).unwrap();
        assert_eq!(v.warmup_period(), 4);
        assert_eq!(v.update(10.0), None);
        assert_eq!(v.update(11.0), None);
        assert_eq!(v.update(12.0), None);
        assert!(v.update(13.0).is_some());
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=60)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 5.0)
            .collect();
        let mut a = Vidya::new(14, 9).unwrap();
        let mut b = Vidya::new(14, 9).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut v = Vidya::new(14, 9).unwrap();
        v.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        assert!(v.is_ready());
        v.reset();
        assert!(!v.is_ready());
        assert_eq!(v.update(1.0), None);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut v = Vidya::new(14, 4).unwrap();
        v.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        let before = v.update(21.0).unwrap();
        assert_eq!(v.update(f64::NAN), Some(before));
        assert_eq!(v.update(f64::INFINITY), Some(before));
    }
}
