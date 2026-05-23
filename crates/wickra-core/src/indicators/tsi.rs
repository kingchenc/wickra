//! True Strength Index.

use crate::error::{Error, Result};
use crate::traits::Indicator;

use super::Ema;

/// True Strength Index — William Blau's double-smoothed momentum oscillator.
///
/// The 1-bar momentum `price_t − price_{t−1}` and its absolute value are each
/// smoothed twice — first with an EMA of length `long`, then with an EMA of
/// length `short` — and the indicator reports their ratio scaled to a
/// percentage:
///
/// ```text
/// TSI = 100 · EMA_short(EMA_long(momentum)) / EMA_short(EMA_long(|momentum|))
/// ```
///
/// The double smoothing strips most of the noise while the ratio normalises
/// the result into a roughly `[−100, 100]` oscillator centred on zero:
/// positive means net upward pressure, negative net downward.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Tsi};
///
/// let mut indicator = Tsi::new(25, 13).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert_eq!(last, Some(100.0)); // pure uptrend saturates at +100
/// ```
#[derive(Debug, Clone)]
pub struct Tsi {
    long: usize,
    short: usize,
    prev_price: Option<f64>,
    ema_long_mom: Ema,
    ema_short_mom: Ema,
    ema_long_abs: Ema,
    ema_short_abs: Ema,
    current: Option<f64>,
}

impl Tsi {
    /// Construct a new TSI with the `long` and `short` smoothing periods.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if either period is `0`.
    pub fn new(long: usize, short: usize) -> Result<Self> {
        if long == 0 || short == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            long,
            short,
            prev_price: None,
            ema_long_mom: Ema::new(long)?,
            ema_short_mom: Ema::new(short)?,
            ema_long_abs: Ema::new(long)?,
            ema_short_abs: Ema::new(short)?,
            current: None,
        })
    }

    /// The `(long, short)` smoothing periods.
    pub const fn periods(&self) -> (usize, usize) {
        (self.long, self.short)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.current
    }
}

impl Indicator for Tsi {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            // Non-finite input is ignored; state is left untouched.
            return self.current;
        }
        let Some(prev) = self.prev_price else {
            self.prev_price = Some(input);
            return None;
        };
        self.prev_price = Some(input);

        let momentum = input - prev;
        let ds_mom = self
            .ema_long_mom
            .update(momentum)
            .and_then(|v| self.ema_short_mom.update(v));
        let ds_abs = self
            .ema_long_abs
            .update(momentum.abs())
            .and_then(|v| self.ema_short_abs.update(v));

        match (ds_mom, ds_abs) {
            (Some(m), Some(a)) => {
                let tsi = if a == 0.0 {
                    // Flat double-smoothed range: there is no momentum at all.
                    0.0
                } else {
                    100.0 * m / a
                };
                self.current = Some(tsi);
                Some(tsi)
            }
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.prev_price = None;
        self.ema_long_mom.reset();
        self.ema_short_mom.reset();
        self.ema_long_abs.reset();
        self.ema_short_abs.reset();
        self.current = None;
    }

    fn warmup_period(&self) -> usize {
        self.long + self.short
    }

    fn is_ready(&self) -> bool {
        self.current.is_some()
    }

    fn name(&self) -> &'static str {
        "TSI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(Tsi::new(0, 13), Err(Error::PeriodZero)));
        assert!(matches!(Tsi::new(25, 0), Err(Error::PeriodZero)));
    }

    /// Cover the const accessors `periods` / `value` (70-77) and the
    /// Indicator-impl `name` body (137-139). Existing tests inspect
    /// TSI output but never query the metadata.
    #[test]
    fn accessors_and_metadata() {
        let mut tsi = Tsi::new(25, 13).unwrap();
        assert_eq!(tsi.periods(), (25, 13));
        assert_eq!(tsi.name(), "TSI");
        assert_eq!(tsi.value(), None);
        for i in 1..=tsi.warmup_period() {
            tsi.update(100.0 + f64::from(u32::try_from(i).unwrap()));
        }
        assert!(tsi.value().is_some());
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut tsi = Tsi::new(5, 3).unwrap();
        assert_eq!(tsi.warmup_period(), 8);
        let out = tsi.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        for v in out.iter().take(7) {
            assert!(v.is_none());
        }
        assert!(out[7].is_some());
    }

    #[test]
    fn pure_uptrend_saturates_at_plus_100() {
        // Every momentum is +1, so |momentum| == momentum and the ratio is 1.
        let mut tsi = Tsi::new(5, 3).unwrap();
        let out = tsi.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        for v in out.iter().skip(8).flatten() {
            assert_relative_eq!(*v, 100.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn pure_downtrend_saturates_at_minus_100() {
        let mut tsi = Tsi::new(5, 3).unwrap();
        let out = tsi.batch(&(1..=40).rev().map(f64::from).collect::<Vec<_>>());
        for v in out.iter().skip(8).flatten() {
            assert_relative_eq!(*v, -100.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn constant_series_yields_zero() {
        let mut tsi = Tsi::new(5, 3).unwrap();
        let out = tsi.batch(&[50.0; 40]);
        for v in out.iter().skip(8).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut tsi = Tsi::new(5, 3).unwrap();
        let out = tsi.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        let last = *out.last().unwrap();
        assert!(last.is_some());
        assert_eq!(tsi.update(f64::NAN), last);
        assert_eq!(tsi.update(f64::INFINITY), last);
    }

    #[test]
    fn reset_clears_state() {
        let mut tsi = Tsi::new(5, 3).unwrap();
        tsi.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        assert!(tsi.is_ready());
        tsi.reset();
        assert!(!tsi.is_ready());
        assert_eq!(tsi.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=80)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 9.0)
            .collect();
        let batch = Tsi::new(13, 7).unwrap().batch(&prices);
        let mut b = Tsi::new(13, 7).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
