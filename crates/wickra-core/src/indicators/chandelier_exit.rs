//! Chandelier Exit.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::indicators::atr::Atr;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Chandelier Exit output: the long-side and short-side trailing stops.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChandelierExitOutput {
    /// Long-position stop: `highest_high − multiplier · ATR`.
    pub long_stop: f64,
    /// Short-position stop: `lowest_low + multiplier · ATR`.
    pub short_stop: f64,
}

/// Chandelier Exit — Chuck LeBeau's ATR trailing stop, hung from the highest
/// high (for longs) or the lowest low (for shorts) of the lookback window.
///
/// ```text
/// long_stop  = highest_high(period) − multiplier · ATR(period)
/// short_stop = lowest_low(period)   + multiplier · ATR(period)
/// ```
///
/// A long position is exited when price closes below `long_stop`; a short
/// when it closes above `short_stop`. Because the stop hangs a fixed number
/// of ATRs off the extreme of the window — like a chandelier off a ceiling —
/// it follows price up but never loosens. LeBeau's classic configuration is a
/// `22`-bar window with a `3.0` multiplier.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, ChandelierExit};
///
/// let mut indicator = ChandelierExit::new(22, 3.0).unwrap();
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
pub struct ChandelierExit {
    period: usize,
    multiplier: f64,
    atr: Atr,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
}

impl ChandelierExit {
    /// Construct a Chandelier Exit with an explicit window and band multiplier.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0` and
    /// [`Error::NonPositiveMultiplier`] if `multiplier` is not strictly
    /// positive and finite.
    pub fn new(period: usize, multiplier: f64) -> Result<Self> {
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            period,
            multiplier,
            atr: Atr::new(period)?,
            highs: VecDeque::with_capacity(period),
            lows: VecDeque::with_capacity(period),
        })
    }

    /// LeBeau's classic configuration: a `22`-bar window, `3.0` multiplier.
    pub fn classic() -> Self {
        Self::new(22, 3.0).expect("classic Chandelier Exit params are valid")
    }

    /// Configured `(period, multiplier)`.
    pub const fn params(&self) -> (usize, f64) {
        (self.period, self.multiplier)
    }
}

impl Indicator for ChandelierExit {
    type Input = Candle;
    type Output = ChandelierExitOutput;

    fn update(&mut self, candle: Candle) -> Option<ChandelierExitOutput> {
        let atr = self.atr.update(candle);
        if self.highs.len() == self.period {
            self.highs.pop_front();
            self.lows.pop_front();
        }
        self.highs.push_back(candle.high);
        self.lows.push_back(candle.low);
        if self.highs.len() < self.period {
            return None;
        }
        // ATR(period) becomes ready on exactly the candle that fills the
        // highest-high / lowest-low window, so this never discards a value.
        let atr = atr?;
        let highest = self.highs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let lowest = self.lows.iter().copied().fold(f64::INFINITY, f64::min);
        Some(ChandelierExitOutput {
            long_stop: highest - self.multiplier * atr,
            short_stop: lowest + self.multiplier * atr,
        })
    }

    fn reset(&mut self) {
        self.atr.reset();
        self.highs.clear();
        self.lows.clear();
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.highs.len() == self.period
    }

    fn name(&self) -> &'static str {
        "ChandelierExit"
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
    fn reference_values_flat_market() {
        // Flat candles H=11, L=9, C=10 -> TR=2 -> ATR=2; HH=11, LL=9.
        // long_stop  = 11 - 3·2 = 5;  short_stop = 9 + 3·2 = 15.
        let candles: Vec<Candle> = (0..20).map(|i| c(11.0, 9.0, 10.0, i)).collect();
        let mut ce = ChandelierExit::new(5, 3.0).unwrap();
        let last = ce.batch(&candles).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last.long_stop, 5.0, epsilon = 1e-12);
        assert_relative_eq!(last.short_stop, 15.0, epsilon = 1e-12);
    }

    #[test]
    fn long_stop_below_highest_short_stop_above_lowest() {
        let candles: Vec<Candle> = (0..120)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.2).sin() * 9.0;
                c(mid + 1.5, mid - 1.5, mid + 0.4, i)
            })
            .collect();
        let mut ce = ChandelierExit::classic();
        for (i, o) in ce.batch(&candles).into_iter().enumerate() {
            if let Some(o) = o {
                // The window's extremes bound the stops from one side.
                let win = &candles[i + 1 - 22..=i];
                let hh = win.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
                let ll = win.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
                assert!(o.long_stop <= hh + 1e-9);
                assert!(o.short_stop >= ll - 1e-9);
            }
        }
    }

    #[test]
    fn first_emission_matches_warmup_period() {
        let candles: Vec<Candle> = (0..20)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut ce = ChandelierExit::new(8, 3.0).unwrap();
        let out = ce.batch(&candles);
        assert_eq!(ce.warmup_period(), 8);
        for (i, v) in out.iter().enumerate().take(7) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(out[7].is_some(), "first value lands at warmup_period - 1");
    }

    #[test]
    fn rejects_invalid_params() {
        assert!(ChandelierExit::new(0, 3.0).is_err());
        assert!(ChandelierExit::new(22, 0.0).is_err());
        assert!(ChandelierExit::new(22, -1.0).is_err());
        assert!(ChandelierExit::new(22, f64::NAN).is_err());
    }

    /// Cover the const accessor `params` (83-85) and the Indicator-impl
    /// `name` body (128-130). `warmup_period` is exercised elsewhere.
    #[test]
    fn accessors_and_metadata() {
        let ce = ChandelierExit::new(22, 3.0).unwrap();
        let (p, m) = ce.params();
        assert_eq!(p, 22);
        assert!((m - 3.0).abs() < 1e-12);
        assert_eq!(ce.name(), "ChandelierExit");
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base + 1.0, base - 1.0, base, i)
            })
            .collect();
        let mut ce = ChandelierExit::classic();
        ce.batch(&candles);
        assert!(ce.is_ready());
        ce.reset();
        assert!(!ce.is_ready());
        assert_eq!(ce.update(candles[0]), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.3).sin() * 8.0;
                c(mid + 1.5, mid - 1.5, mid + 0.5, i)
            })
            .collect();
        let mut a = ChandelierExit::classic();
        let mut b = ChandelierExit::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
