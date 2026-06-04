//! Stochastic CCI — a stochastic oscillator applied to the CCI.

use std::collections::VecDeque;

use crate::error::Result;
use crate::indicators::cci::Cci;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Stochastic CCI — the stochastic oscillator computed over the
/// [`Cci`](crate::Cci) instead of price.
///
/// The CCI is unbounded and spends most of its time inside `±100`, which makes
/// fixed overbought/oversold lines awkward. Running a stochastic over the CCI
/// re-scales it to `[0, 100]` relative to its own recent range, turning it into
/// a bounded, self-normalising momentum oscillator:
///
/// ```text
/// cci = CCI(typical price, period)
/// %K  = 100 * (cci - lowest(cci, period)) / (highest(cci, period) - lowest(cci, period))
/// ```
///
/// The same `period` is used for the CCI and the stochastic lookback. When the
/// CCI range over the window is zero (a flat market, where the CCI is pinned at
/// `0`) the oscillator returns the neutral `50`. The first value lands after
/// `2·period − 1` bars: `period` to seed the CCI, then `period` CCI values to
/// fill the stochastic window.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, StochasticCci, Indicator};
///
/// let mut sc = StochasticCci::new(14).unwrap();
/// let mut last = None;
/// for i in 0..60 {
///     let base = 100.0 + (f64::from(i) * 0.3).sin() * 10.0;
///     let c = Candle::new(base, base + 1.0, base - 1.0, base, 1.0, i64::from(i)).unwrap();
///     last = sc.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct StochasticCci {
    period: usize,
    cci: Cci,
    /// The last `period` CCI values.
    window: VecDeque<f64>,
}

impl StochasticCci {
    /// Construct a Stochastic CCI with the given period (shared by the CCI and
    /// the stochastic lookback).
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            period,
            cci: Cci::new(period)?,
            window: VecDeque::with_capacity(period),
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for StochasticCci {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let cci = self.cci.update(candle)?;
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(cci);
        if self.window.len() < self.period {
            return None;
        }
        let mut lo = f64::MAX;
        let mut hi = f64::MIN;
        for &v in &self.window {
            if v < lo {
                lo = v;
            }
            if v > hi {
                hi = v;
            }
        }
        let range = hi - lo;
        if range == 0.0 {
            return Some(50.0);
        }
        // Ratio first, then scale: `100 * x / x` can round to 100.0000…1.
        Some(100.0 * ((cci - lo) / range))
    }

    fn reset(&mut self) {
        self.cci.reset();
        self.window.clear();
    }

    fn warmup_period(&self) -> usize {
        // CCI seeds at `period`, then `period` CCI values fill the stochastic window.
        2 * self.period - 1
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "StochasticCCI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(high: f64, low: f64, close: f64) -> Candle {
        Candle::new(close, high, low, close, 1.0, 0).unwrap()
    }

    #[test]
    fn rejects_zero_period() {
        assert!(StochasticCci::new(0).is_err());
    }

    /// Cover the const accessor `period` and the Indicator-impl `warmup_period`
    /// + `name`.
    #[test]
    fn accessors_and_metadata() {
        let sc = StochasticCci::new(14).unwrap();
        assert_eq!(sc.period(), 14);
        assert_eq!(sc.warmup_period(), 27);
        assert_eq!(sc.name(), "StochasticCCI");
    }

    #[test]
    fn first_emission_matches_warmup_period() {
        let bars: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.4).sin() * 8.0;
                candle(base + 1.0, base - 1.0, base)
            })
            .collect();
        let mut sc = StochasticCci::new(5).unwrap();
        let out = sc.batch(&bars);
        let warmup = sc.warmup_period();
        assert_eq!(warmup, 9);
        for (i, v) in out.iter().enumerate().take(warmup - 1) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(out[warmup - 1].is_some());
    }

    #[test]
    fn bounded_zero_to_hundred() {
        let bars: Vec<Candle> = (0..80)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.35).sin() * 12.0;
                candle(base + 2.0, base - 2.0, base)
            })
            .collect();
        let mut sc = StochasticCci::new(9).unwrap();
        for v in sc.batch(&bars).into_iter().flatten() {
            assert!((0.0..=100.0).contains(&v), "%K {v} left [0, 100]");
        }
    }

    #[test]
    fn flat_market_is_neutral() {
        // Constant candles -> CCI pinned at 0 -> zero range -> neutral 50.
        let mut sc = StochasticCci::new(4).unwrap();
        let bars = vec![candle(10.0, 10.0, 10.0); 20];
        let last = sc.batch(&bars).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 50.0, epsilon = 1e-12);
    }

    #[test]
    fn highest_cci_in_window_is_hundred() {
        // When the latest CCI is the window maximum, %K must be 100.
        // A long rise then makes the final CCI the highest in its window.
        let mut bars: Vec<Candle> = (0..20)
            .map(|i| candle(f64::from(i) + 1.0, f64::from(i) - 1.0, f64::from(i)))
            .collect();
        // Strong final push so the last CCI tops its window.
        bars.push(candle(100.0, 98.0, 100.0));
        let mut sc = StochasticCci::new(5).unwrap();
        let last = sc.batch(&bars).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 100.0, epsilon = 1e-9);
    }

    #[test]
    fn reset_clears_state() {
        let mut sc = StochasticCci::new(5).unwrap();
        sc.batch(
            &(0..30)
                .map(|i| candle(f64::from(i) + 1.0, f64::from(i) - 1.0, f64::from(i)))
                .collect::<Vec<_>>(),
        );
        assert!(sc.is_ready());
        sc.reset();
        assert!(!sc.is_ready());
        assert_eq!(sc.update(candle(2.0, 0.0, 1.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let bars: Vec<Candle> = (0..60)
            .map(|i| {
                let base = 50.0 + (f64::from(i) * 0.5).sin() * 10.0;
                candle(base + 1.5, base - 1.5, base)
            })
            .collect();
        let mut a = StochasticCci::new(9).unwrap();
        let mut b = StochasticCci::new(9).unwrap();
        assert_eq!(
            a.batch(&bars),
            bars.iter().map(|c| b.update(*c)).collect::<Vec<_>>()
        );
    }
}
