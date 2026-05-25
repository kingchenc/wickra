//! Stochastic Momentum Index (SMI).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::indicators::ema::Ema;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// William Blau's Stochastic Momentum Index — a doubly-smoothed,
/// `±100`-bounded oscillator built from the close's distance to the centre
/// of the recent high-low range.
///
/// Over the lookback `period`, let `HH = max(high)`, `LL = min(low)`,
/// `C = (HH + LL) / 2` and `R = HH - LL`. The raw displacement is
/// `d_t = close_t - C_t`. Both `d` and `R` are smoothed twice with `EMA`s,
/// then combined into the bounded reading:
///
/// ```text
/// D_smoothed  = EMA(EMA(d, d_period), d2_period)
/// HL_smoothed = EMA(EMA(R, d_period), d2_period)
/// SMI         = 100 · D_smoothed / (HL_smoothed / 2)
/// ```
///
/// Blau's recommended defaults are `(period = 5, d = 3, d2 = 3)`. Wickra
/// publishes the SMI value only; the optional signal `EMA(SMI, k)` is left
/// to the consumer via `Chain` / their own `Ema`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Smi};
///
/// let mut smi = Smi::new(5, 3, 3).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let p = 100.0 + f64::from(i);
///     let candle = Candle::new(p, p + 1.0, p - 1.0, p, 1.0, i64::from(i)).unwrap();
///     last = smi.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Smi {
    period: usize,
    d_period: usize,
    d2_period: usize,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    ema_d1: Ema,
    ema_d2: Ema,
    ema_r1: Ema,
    ema_r2: Ema,
    current: Option<f64>,
}

impl Smi {
    /// # Errors
    /// Returns [`Error::PeriodZero`] if any period is zero.
    pub fn new(period: usize, d_period: usize, d2_period: usize) -> Result<Self> {
        if period == 0 || d_period == 0 || d2_period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            d_period,
            d2_period,
            highs: VecDeque::with_capacity(period),
            lows: VecDeque::with_capacity(period),
            ema_d1: Ema::new(d_period)?,
            ema_d2: Ema::new(d2_period)?,
            ema_r1: Ema::new(d_period)?,
            ema_r2: Ema::new(d2_period)?,
            current: None,
        })
    }

    /// Blau's recommended defaults `(period = 5, d = 3, d2 = 3)`.
    pub fn classic() -> Self {
        Self::new(5, 3, 3).expect("classic SMI parameters are valid")
    }

    /// Configured `(period, d_period, d2_period)`.
    pub const fn periods(&self) -> (usize, usize, usize) {
        (self.period, self.d_period, self.d2_period)
    }
}

impl Indicator for Smi {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        if self.highs.len() == self.period {
            self.highs.pop_front();
            self.lows.pop_front();
        }
        self.highs.push_back(candle.high);
        self.lows.push_back(candle.low);
        if self.highs.len() < self.period {
            return None;
        }
        let hh = self.highs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let ll = self.lows.iter().copied().fold(f64::INFINITY, f64::min);
        let center = f64::midpoint(hh, ll);
        let displacement = candle.close - center;
        let range = hh - ll;

        // Feed every EMA on every candle so both stacks warm in parallel —
        // gating the range stack behind the displacement stack would starve
        // it by one input.
        let d1 = self.ema_d1.update(displacement);
        let r1 = self.ema_r1.update(range);
        let d2 = d1.and_then(|x| self.ema_d2.update(x));
        let r2 = r1.and_then(|x| self.ema_r2.update(x));
        let (d2, r2) = (d2?, r2?);

        if r2 <= 0.0 {
            // Window where the smoothed range collapses to zero: the formula
            // is undefined. Hold the previous reading rather than emit inf.
            return self.current;
        }
        let value = 100.0 * d2 / (r2 / 2.0);
        self.current = Some(value);
        Some(value)
    }

    fn reset(&mut self) {
        self.highs.clear();
        self.lows.clear();
        self.ema_d1.reset();
        self.ema_d2.reset();
        self.ema_r1.reset();
        self.ema_r2.reset();
        self.current = None;
    }

    fn warmup_period(&self) -> usize {
        // The high-low window needs `period` candles; then both EMA stacks
        // need `d_period + d2_period - 1` more values to fully warm up.
        self.period + self.d_period + self.d2_period - 2
    }

    fn is_ready(&self) -> bool {
        self.current.is_some()
    }

    fn name(&self) -> &'static str {
        "SMI"
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
    fn rejects_zero_period() {
        assert!(matches!(Smi::new(0, 3, 3), Err(Error::PeriodZero)));
        assert!(matches!(Smi::new(5, 0, 3), Err(Error::PeriodZero)));
        assert!(matches!(Smi::new(5, 3, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let smi = Smi::new(5, 3, 3).unwrap();
        assert_eq!(smi.periods(), (5, 3, 3));
        assert_eq!(smi.warmup_period(), 9);
        assert_eq!(smi.name(), "SMI");
    }

    #[test]
    fn classic_factory() {
        let smi = Smi::classic();
        assert_eq!(smi.periods(), (5, 3, 3));
    }

    #[test]
    fn close_at_high_pushes_toward_plus_100() {
        // Every candle's close equals its high in a rising series: the
        // displacement is at the top of the range every bar, so SMI sits in
        // the strongly positive region. After enough double-smoothing it
        // approaches the upper bound.
        let mut smi = Smi::classic();
        let mut last = None;
        for i in 0..80 {
            let h = 100.0 + f64::from(i);
            let l = h - 2.0;
            last = smi.update(candle(h, l, h, i64::from(i)));
        }
        let v = last.expect("SMI is warm");
        assert!(
            v > 50.0,
            "close-at-high series should drive SMI well above 0: {v}"
        );
    }

    #[test]
    fn close_at_low_pushes_toward_minus_100() {
        let mut smi = Smi::classic();
        let mut last = None;
        for i in 0..80 {
            let h = 100.0 - f64::from(i);
            let l = h - 2.0;
            last = smi.update(candle(h, l, l, i64::from(i)));
        }
        let v = last.expect("SMI is warm");
        assert!(
            v < -50.0,
            "close-at-low series should drive SMI well below 0: {v}"
        );
    }

    #[test]
    fn warmup_emits_first_value_at_warmup_period() {
        let mut smi = Smi::new(3, 2, 2).unwrap();
        // period 3 + d 2 + d2 2 - 2 = 5.
        assert_eq!(smi.warmup_period(), 5);
        let mut got = None;
        for i in 0..5 {
            got = smi.update(candle(11.0, 9.0, 10.0, i));
        }
        assert!(got.is_some());
    }

    #[test]
    fn flat_close_yields_zero_displacement() {
        // Every close is exactly at the centre of the range -> displacement
        // is 0 every bar -> SMI converges to 0.
        let mut smi = Smi::classic();
        let mut last = None;
        for i in 0..60 {
            // High and low straddle a constant close.
            last = smi.update(candle(11.0, 9.0, 10.0, i));
        }
        let v = last.unwrap();
        assert_relative_eq!(v, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80_i64)
            .map(|i| {
                let c = 100.0 + (i as f64 * 0.3).sin() * 8.0;
                candle(c + 1.0, c - 1.0, c, i)
            })
            .collect();
        let batch = Smi::classic().batch(&candles);
        let mut b = Smi::classic();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn reset_clears_state() {
        let mut smi = Smi::classic();
        for i in 0..40 {
            smi.update(candle(11.0, 9.0, 10.0, i));
        }
        assert!(smi.is_ready());
        smi.reset();
        assert!(!smi.is_ready());
    }

    #[test]
    fn zero_range_holds_previous_value() {
        // High == low on every bar -> instantaneous range is zero, the
        // EMA of (range / 2) settles to zero, so `r2 <= 0.0` after warmup
        // and the indicator must hold its previous value (None here, since
        // r2 was zero from the very first warm bar) rather than divide by
        // zero.
        let mut smi = Smi::new(3, 2, 2).unwrap();
        // warmup_period = 3 + 2 + 2 - 2 = 5; feed warmup + 2 extra bars.
        for i in 0..7 {
            let v = smi.update(candle(10.0, 10.0, 10.0, i));
            assert_eq!(v, None, "zero-range SMI must hold None, got {v:?}");
        }
    }
}
