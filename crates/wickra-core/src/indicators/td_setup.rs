#![allow(clippy::doc_markdown)]

//! Tom DeMark TD Setup (9-bar buy / sell setup).
//!
//! The TD Setup is the first half of DeMark's TD Sequential. It counts how many
//! consecutive bars satisfy a fixed price-comparison rule relative to the close
//! `lookback` bars earlier (the canonical lookback is 4 — i.e. compare `close[i]`
//! to `close[i-4]`).
//!
//! - A **buy setup** advances by one for each bar whose close is *less than* the
//!   close `lookback` bars earlier. The streak resets to zero as soon as the
//!   condition fails. A "completed" buy setup is a streak of 9 (DeMark's
//!   default `target`).
//! - A **sell setup** advances symmetrically when the close is *greater than*
//!   the close `lookback` bars earlier.
//!
//! Only one direction can be active on a given bar: the same bar cannot satisfy
//! both `close < close[-4]` and `close > close[-4]`. If neither condition
//! holds (equality with the lookback close) both streaks reset.
//!
//! This indicator emits a signed count: positive values mean the buy-setup
//! streak is active, negative values mean the sell-setup streak is active,
//! and `0` means neither streak is active on the current bar. The magnitude is
//! the current run length, capped at `target` once the setup completes — the
//! caller can detect "perfected" setups by waiting for `value.abs() ==
//! target`.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// TD Setup state machine: counts consecutive bars meeting DeMark's setup
/// comparison rule against the close `lookback` bars earlier.
/// # Example
///
/// ```
/// use wickra_core::{TdSetup, Candle, Indicator};
///
/// let mut indicator = TdSetup::new(4, 9).unwrap();
/// // `None` during warmup, then `Some(_)` once enough bars are seen.
/// let mut out = None;
/// for i in 0..40i64 {
///     let p = 100.0 + (i as f64 * 0.4).sin() * 5.0;
///     let candle = Candle::new(p, p + 1.5, p - 1.5, p + 0.3, 1_000.0, i).unwrap();
///     out = indicator.update(candle);
/// }
/// let _ = out;
/// ```
#[derive(Debug, Clone)]
pub struct TdSetup {
    lookback: usize,
    target: usize,
    closes: VecDeque<f64>,
    buy_count: usize,
    sell_count: usize,
    last_value: Option<f64>,
}

impl TdSetup {
    /// Construct a TD Setup with an explicit lookback and target count.
    ///
    /// The classic DeMark configuration is `lookback = 4` and `target = 9`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if either argument is zero.
    pub fn new(lookback: usize, target: usize) -> Result<Self> {
        if lookback == 0 || target == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            lookback,
            target,
            closes: VecDeque::with_capacity(lookback + 1),
            buy_count: 0,
            sell_count: 0,
            last_value: None,
        })
    }

    /// DeMark's classic configuration: `lookback = 4`, `target = 9`.
    pub fn classic() -> Self {
        Self::new(4, 9).expect("classic TD Setup parameters are valid")
    }

    /// Configured `(lookback, target)`.
    pub const fn params(&self) -> (usize, usize) {
        (self.lookback, self.target)
    }

    /// Current signed setup value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for TdSetup {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        // Maintain a rolling window of the last `lookback + 1` closes so the
        // oldest entry (front) is exactly the close `lookback` bars ago.
        if self.closes.len() > self.lookback {
            self.closes.pop_front();
        }
        if self.closes.len() < self.lookback {
            self.closes.push_back(candle.close);
            return None;
        }
        // We now have exactly `lookback` historical closes buffered; the oldest
        // is the comparison reference.
        let reference = *self.closes.front().expect("non-empty after the guard");
        self.closes.push_back(candle.close);

        if candle.close < reference {
            self.buy_count = (self.buy_count + 1).min(self.target);
            self.sell_count = 0;
            let v = self.buy_count as f64;
            self.last_value = Some(v);
            Some(v)
        } else if candle.close > reference {
            self.sell_count = (self.sell_count + 1).min(self.target);
            self.buy_count = 0;
            let v = -(self.sell_count as f64);
            self.last_value = Some(v);
            Some(v)
        } else {
            // Equality breaks both streaks; the bar emits zero.
            self.buy_count = 0;
            self.sell_count = 0;
            self.last_value = Some(0.0);
            Some(0.0)
        }
    }

    fn reset(&mut self) {
        self.closes.clear();
        self.buy_count = 0;
        self.sell_count = 0;
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        self.lookback + 1
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "TDSetup"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(close: f64, ts: i64) -> Candle {
        Candle::new_unchecked(close, close, close, close, 0.0, ts)
    }

    #[test]
    fn pure_uptrend_reaches_sell_setup_9() {
        // Every close is strictly greater than four bars ago, so the sell
        // streak advances by one per bar from the moment lookback is filled.
        let candles: Vec<Candle> = (1..=20).map(|i| c(f64::from(i), i64::from(i))).collect();
        let mut setup = TdSetup::classic();
        let out = setup.batch(&candles);
        // Indices 0..4 are warmup. Index 4 is the first bar with a reference.
        // Sell-setup advances each bar: -1 at idx 4, -2 at idx 5, …, -9 at
        // idx 12; from there it caps at -9 because target is 9.
        for (i, v) in out.iter().enumerate().take(4) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert_eq!(out[4], Some(-1.0));
        assert_eq!(out[5], Some(-2.0));
        assert_eq!(out[12], Some(-9.0));
        assert_eq!(out[13], Some(-9.0));
        assert_eq!(out[19], Some(-9.0));
    }

    #[test]
    fn pure_downtrend_reaches_buy_setup_9() {
        let candles: Vec<Candle> = (1..=20)
            .rev()
            .enumerate()
            .map(|(i, v)| c(f64::from(v), i64::try_from(i).unwrap()))
            .collect();
        let mut setup = TdSetup::classic();
        let out = setup.batch(&candles);
        // Buy streak should mirror the sell case: +1 at idx 4, capping at +9.
        assert_eq!(out[4], Some(1.0));
        assert_eq!(out[12], Some(9.0));
        assert_eq!(out[19], Some(9.0));
    }

    #[test]
    fn flat_series_emits_zero_after_warmup() {
        // Every close equals the reference close (lookback bars earlier), so
        // neither streak ever advances; the indicator emits 0 every bar.
        let candles: Vec<Candle> = (0..20).map(|i| c(42.0, i)).collect();
        let mut setup = TdSetup::classic();
        let out = setup.batch(&candles);
        for v in out.iter().skip(4) {
            assert_eq!(*v, Some(0.0));
        }
    }

    #[test]
    fn streak_resets_on_direction_flip() {
        // First 4 closes are warmup. Then 4 strictly-lower closes -> buy
        // streak 1..=4. The next close is higher than its reference -> the
        // buy streak resets and the sell streak starts at 1.
        let candles = [
            c(10.0, 0),
            c(10.0, 1),
            c(10.0, 2),
            c(10.0, 3),
            c(9.0, 4),
            c(8.0, 5),
            c(7.0, 6),
            c(6.0, 7),
            c(11.0, 8),
        ];
        let mut setup = TdSetup::classic();
        let out = setup.batch(&candles);
        assert_eq!(out[4], Some(1.0));
        assert_eq!(out[7], Some(4.0));
        assert_eq!(out[8], Some(-1.0));
    }

    #[test]
    fn rejects_zero_arguments() {
        assert!(matches!(TdSetup::new(0, 9), Err(Error::PeriodZero)));
        assert!(matches!(TdSetup::new(4, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| c(100.0 + (f64::from(i) * 0.3).sin() * 5.0, i64::from(i)))
            .collect();
        let mut a = TdSetup::classic();
        let mut b = TdSetup::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (1..=20).map(|i| c(f64::from(i), i64::from(i))).collect();
        let mut setup = TdSetup::classic();
        setup.batch(&candles);
        assert!(setup.is_ready());
        setup.reset();
        assert!(!setup.is_ready());
        assert_eq!(setup.update(candles[0]), None);
        assert_eq!(setup.value(), None);
    }

    #[test]
    fn accessors_and_metadata() {
        let setup = TdSetup::new(4, 9).unwrap();
        assert_eq!(setup.params(), (4, 9));
        assert_eq!(setup.warmup_period(), 5);
        assert_eq!(setup.name(), "TDSetup");
        assert_eq!(setup.value(), None);
    }
}
