//! Ultimate Oscillator.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Ultimate Oscillator — Larry Williams' three-timeframe momentum oscillator.
///
/// A single-timeframe oscillator can give false divergence signals when the
/// chosen lookback does not match the swing being measured. The Ultimate
/// Oscillator blends *three* lookbacks into one bounded `[0, 100]` reading,
/// weighting the fastest most heavily:
///
/// ```text
/// true_low_t   = min(low_t, close_{t−1})
/// BP_t         = close_t − true_low_t                       (buying pressure)
/// TR_t         = max(high_t, close_{t−1}) − true_low_t      (true range)
/// avg_n        = Σ BP over n / Σ TR over n
/// UO           = 100 · (4·avg_short + 2·avg_mid + avg_long) / 7
/// ```
///
/// The conventional periods are `7`, `14` and `28`. A fully flat window (zero
/// true range) contributes the neutral ratio `0.5`, so a flat market reads
/// `50`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, UltimateOscillator};
///
/// let mut indicator = UltimateOscillator::new(7, 14, 28).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     let p = 100.0 + f64::from(i);
///     let candle = Candle::new(p, p + 1.0, p - 1.0, p, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct UltimateOscillator {
    short: usize,
    mid: usize,
    long: usize,
    longest: usize,
    prev_close: Option<f64>,
    /// Rolling window of `(buying_pressure, true_range)` pairs.
    window: VecDeque<(f64, f64)>,
    sum_bp_short: f64,
    sum_tr_short: f64,
    sum_bp_mid: f64,
    sum_tr_mid: f64,
    sum_bp_long: f64,
    sum_tr_long: f64,
    pairs: usize,
    last: Option<f64>,
}

impl UltimateOscillator {
    /// Construct a new Ultimate Oscillator with the three lookback periods.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if any period is `0`.
    pub fn new(short: usize, mid: usize, long: usize) -> Result<Self> {
        if short == 0 || mid == 0 || long == 0 {
            return Err(Error::PeriodZero);
        }
        let longest = short.max(mid).max(long);
        Ok(Self {
            short,
            mid,
            long,
            longest,
            prev_close: None,
            window: VecDeque::with_capacity(longest + 1),
            sum_bp_short: 0.0,
            sum_tr_short: 0.0,
            sum_bp_mid: 0.0,
            sum_tr_mid: 0.0,
            sum_bp_long: 0.0,
            sum_tr_long: 0.0,
            pairs: 0,
            last: None,
        })
    }

    /// Classic Ultimate Oscillator: periods `7`, `14`, `28`.
    pub fn classic() -> Self {
        Self::new(7, 14, 28).expect("classic Ultimate Oscillator periods are valid")
    }

    /// The `(short, mid, long)` periods.
    pub const fn periods(&self) -> (usize, usize, usize) {
        (self.short, self.mid, self.long)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for UltimateOscillator {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev_close) = self.prev_close else {
            // The first bar has no previous close, so no BP/TR can be formed.
            self.prev_close = Some(candle.close);
            return None;
        };
        self.prev_close = Some(candle.close);

        let true_low = candle.low.min(prev_close);
        let bp = candle.close - true_low;
        let tr = candle.high.max(prev_close) - true_low;

        self.window.push_back((bp, tr));
        let n = self.window.len();
        self.sum_bp_short += bp;
        self.sum_tr_short += tr;
        self.sum_bp_mid += bp;
        self.sum_tr_mid += tr;
        self.sum_bp_long += bp;
        self.sum_tr_long += tr;
        if n > self.short {
            let (b, t) = self.window[n - 1 - self.short];
            self.sum_bp_short -= b;
            self.sum_tr_short -= t;
        }
        if n > self.mid {
            let (b, t) = self.window[n - 1 - self.mid];
            self.sum_bp_mid -= b;
            self.sum_tr_mid -= t;
        }
        if n > self.long {
            let (b, t) = self.window[n - 1 - self.long];
            self.sum_bp_long -= b;
            self.sum_tr_long -= t;
        }
        if self.window.len() > self.longest {
            self.window.pop_front();
        }

        self.pairs += 1;
        if self.pairs < self.longest {
            return None;
        }

        let avg = |bp_sum: f64, tr_sum: f64| {
            if tr_sum == 0.0 {
                // A fully flat window has no range; contribute the midpoint.
                0.5
            } else {
                bp_sum / tr_sum
            }
        };
        let avg_short = avg(self.sum_bp_short, self.sum_tr_short);
        let avg_mid = avg(self.sum_bp_mid, self.sum_tr_mid);
        let avg_long = avg(self.sum_bp_long, self.sum_tr_long);
        let uo = 100.0 * (4.0 * avg_short + 2.0 * avg_mid + avg_long) / 7.0;
        self.last = Some(uo);
        Some(uo)
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.window.clear();
        self.sum_bp_short = 0.0;
        self.sum_tr_short = 0.0;
        self.sum_bp_mid = 0.0;
        self.sum_tr_mid = 0.0;
        self.sum_bp_long = 0.0;
        self.sum_tr_long = 0.0;
        self.pairs = 0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // The first BP/TR pair needs a previous close, then the longest window
        // must fill.
        self.longest + 1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "UltimateOscillator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    /// Build a flat candle (open = high = low = close).
    fn flat(price: f64, ts: i64) -> Candle {
        Candle::new(price, price, price, price, 1.0, ts).unwrap()
    }

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(
            UltimateOscillator::new(0, 14, 28),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            UltimateOscillator::new(7, 0, 28),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            UltimateOscillator::new(7, 14, 0),
            Err(Error::PeriodZero)
        ));
    }

    /// Cover the const accessors `periods` / `value` (96-103) and the
    /// Indicator-impl `name` body (193-195). `warmup_period` is covered
    /// by `first_emission_at_warmup_period`.
    #[test]
    fn accessors_and_metadata() {
        let mut uo = UltimateOscillator::new(7, 14, 28).unwrap();
        assert_eq!(uo.periods(), (7, 14, 28));
        assert_eq!(uo.name(), "UltimateOscillator");
        assert_eq!(uo.value(), None);
        let warmup = i64::try_from(uo.warmup_period()).unwrap();
        let candles: Vec<Candle> = (0..warmup)
            .map(|i| {
                let p = 100.0 + (i as f64 * 0.3).sin() * 5.0;
                Candle::new(p, p + 1.0, p - 1.0, p, 1.0, i).unwrap()
            })
            .collect();
        for c in &candles {
            uo.update(*c);
        }
        assert!(uo.value().is_some());
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut uo = UltimateOscillator::new(2, 3, 5).unwrap();
        assert_eq!(uo.warmup_period(), 6);
        let candles: Vec<Candle> = (0..20).map(|i| flat(100.0 + i as f64, i)).collect();
        let out = uo.batch(&candles);
        for v in out.iter().take(5) {
            assert!(v.is_none());
        }
        assert!(out[5].is_some());
    }

    #[test]
    fn pure_uptrend_saturates_at_100() {
        // Each flat candle closes higher: BP == TR every bar, so every ratio
        // is 1 and UO is 100.
        let mut uo = UltimateOscillator::new(2, 3, 5).unwrap();
        let candles: Vec<Candle> = (0..30).map(|i| flat(100.0 + i as f64, i)).collect();
        for v in uo.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 100.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn pure_downtrend_saturates_at_0() {
        // Each flat candle closes lower: BP is 0 every bar, so UO is 0.
        let mut uo = UltimateOscillator::new(2, 3, 5).unwrap();
        let candles: Vec<Candle> = (0..30).map(|i| flat(100.0 - i as f64, i)).collect();
        for v in uo.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn flat_market_reads_50() {
        // Every bar identical: zero true range everywhere -> neutral 50.
        let mut uo = UltimateOscillator::new(2, 3, 5).unwrap();
        let candles: Vec<Candle> = (0..30).map(|i| flat(100.0, i)).collect();
        for v in uo.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 50.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn output_stays_within_0_100() {
        let mut uo = UltimateOscillator::classic();
        let candles: Vec<Candle> = (0..200)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.2).sin() * 12.0;
                Candle::new(mid, mid + 3.0, mid - 3.0, mid + 1.0, 10.0, i).unwrap()
            })
            .collect();
        for v in uo.batch(&candles).into_iter().flatten() {
            assert!((0.0..=100.0).contains(&v), "UO out of range: {v}");
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut uo = UltimateOscillator::new(2, 3, 5).unwrap();
        let candles: Vec<Candle> = (0..20).map(|i| flat(100.0 + i as f64, i)).collect();
        uo.batch(&candles);
        assert!(uo.is_ready());
        uo.reset();
        assert!(!uo.is_ready());
        assert_eq!(uo.update(candles[0]), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..120)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.3).sin() * 10.0;
                Candle::new(mid, mid + 2.0, mid - 2.0, mid + 0.5, 10.0, i).unwrap()
            })
            .collect();
        let batch = UltimateOscillator::classic().batch(&candles);
        let mut b = UltimateOscillator::classic();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }
}
