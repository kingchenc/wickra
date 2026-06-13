#![allow(clippy::doc_markdown)]

//! Tom DeMark TD Combo — an aggressive variant of TD Countdown.
//!
//! TD Combo is DeMark's stricter countdown variant. Unlike vanilla TD
//! Sequential (which only requires `close <= low[i - 2]` for a buy
//! countdown), Combo adds two strictness conditions that prevent the
//! countdown from advancing on weak bars:
//!
//! - **Buy combo** bars must satisfy:
//!   1. `close[i] <= low[i - 2]`               (the classic countdown rule)
//!   2. `low[i]   <= low[i - 1]`               (monotone strictly-non-rising lows)
//!   3. `close[i] <  close[i - 1]`             (each combo bar must close strictly lower)
//! - **Sell combo** bars must satisfy the mirror set:
//!   1. `close[i] >= high[i - 2]`
//!   2. `high[i]  >= high[i - 1]`
//!   3. `close[i] >  close[i - 1]`
//!
//! Like vanilla countdown, the combo is *armed* by a completed 9-bar setup
//! (same definition as [`crate::TdSetup`]) in the same direction. The combo
//! count saturates at `target` (DeMark's classic value is `13`).
//!
//! Output is a signed counter: positive for an active buy-combo run,
//! negative for a sell-combo run, `0.0` when no combo is currently armed.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Direction of an active TD Combo run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    None,
    Buy,
    Sell,
}

/// TD Combo — aggressive countdown variant.
/// # Example
///
/// ```
/// use wickra_core::{TdCombo, Candle, Indicator};
///
/// let mut indicator = TdCombo::new(4, 9, 2, 13).unwrap();
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
pub struct TdCombo {
    setup_lookback: usize,
    setup_target: usize,
    countdown_lookback: usize,
    countdown_target: usize,
    candles: VecDeque<Candle>,
    buy_setup: usize,
    sell_setup: usize,
    buy_combo: usize,
    sell_combo: usize,
    direction: Direction,
    ready: bool,
}

impl TdCombo {
    /// Construct a TD Combo with explicit lookbacks and targets. The
    /// canonical DeMark configuration is `setup_lookback = 4`,
    /// `setup_target = 9`, `countdown_lookback = 2`, `countdown_target = 13`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if any argument is zero.
    pub fn new(
        setup_lookback: usize,
        setup_target: usize,
        countdown_lookback: usize,
        countdown_target: usize,
    ) -> Result<Self> {
        if setup_lookback == 0
            || setup_target == 0
            || countdown_lookback == 0
            || countdown_target == 0
        {
            return Err(Error::PeriodZero);
        }
        let cap = setup_lookback.max(countdown_lookback) + 1;
        Ok(Self {
            setup_lookback,
            setup_target,
            countdown_lookback,
            countdown_target,
            candles: VecDeque::with_capacity(cap),
            buy_setup: 0,
            sell_setup: 0,
            buy_combo: 0,
            sell_combo: 0,
            direction: Direction::None,
            ready: false,
        })
    }

    /// DeMark's classic configuration: setup `lookback = 4, target = 9`,
    /// combo `lookback = 2, target = 13`.
    pub fn classic() -> Self {
        Self::new(4, 9, 2, 13).expect("classic TD Combo parameters are valid")
    }

    /// Configured `(setup_lookback, setup_target, countdown_lookback,
    /// countdown_target)`.
    pub const fn params(&self) -> (usize, usize, usize, usize) {
        (
            self.setup_lookback,
            self.setup_target,
            self.countdown_lookback,
            self.countdown_target,
        )
    }
}

impl Indicator for TdCombo {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let need = self.setup_lookback.max(self.countdown_lookback);
        let cap = need + 1;
        if self.candles.len() == cap {
            self.candles.pop_front();
        }
        if self.candles.len() < need {
            self.candles.push_back(candle);
            return None;
        }

        // Setup rule: compare to close[setup_lookback bars ago].
        let setup_ref_idx = need - self.setup_lookback;
        let setup_ref_close = self.candles[setup_ref_idx].close;
        if candle.close < setup_ref_close {
            self.buy_setup = (self.buy_setup + 1).min(self.setup_target);
            self.sell_setup = 0;
        } else if candle.close > setup_ref_close {
            self.sell_setup = (self.sell_setup + 1).min(self.setup_target);
            self.buy_setup = 0;
        } else {
            self.buy_setup = 0;
            self.sell_setup = 0;
        }

        // Combo arming: a completed setup in either direction arms the
        // combo in the same direction (resetting any opposite-direction
        // combo count first).
        if self.buy_setup == self.setup_target {
            if self.direction != Direction::Buy {
                self.buy_combo = 0;
                self.sell_combo = 0;
            }
            self.direction = Direction::Buy;
        } else if self.sell_setup == self.setup_target {
            if self.direction != Direction::Sell {
                self.buy_combo = 0;
                self.sell_combo = 0;
            }
            self.direction = Direction::Sell;
        }

        // Combo rule references the candle `countdown_lookback` bars ago
        // (high / low) and the immediately-prior candle (low / high /
        // close monotone strictness).
        let combo_ref = self.candles[need - self.countdown_lookback];
        let prev = self.candles[need - 1];
        match self.direction {
            Direction::Buy => {
                let cond_classic = candle.close <= combo_ref.low;
                let cond_low = candle.low <= prev.low;
                let cond_close = candle.close < prev.close;
                if cond_classic && cond_low && cond_close && self.buy_combo < self.countdown_target
                {
                    self.buy_combo += 1;
                }
            }
            Direction::Sell => {
                let cond_classic = candle.close >= combo_ref.high;
                let cond_high = candle.high >= prev.high;
                let cond_close = candle.close > prev.close;
                if cond_classic
                    && cond_high
                    && cond_close
                    && self.sell_combo < self.countdown_target
                {
                    self.sell_combo += 1;
                }
            }
            Direction::None => {}
        }

        self.candles.push_back(candle);
        self.ready = true;

        let v = match self.direction {
            Direction::Buy => self.buy_combo as f64,
            Direction::Sell => -(self.sell_combo as f64),
            Direction::None => 0.0,
        };
        Some(v)
    }

    fn reset(&mut self) {
        self.candles.clear();
        self.buy_setup = 0;
        self.sell_setup = 0;
        self.buy_combo = 0;
        self.sell_combo = 0;
        self.direction = Direction::None;
        self.ready = false;
    }

    fn warmup_period(&self) -> usize {
        self.setup_lookback.max(self.countdown_lookback) + 1
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn name(&self) -> &'static str {
        "TDCombo"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new_unchecked(close, high, low, close, 0.0, ts)
    }

    #[test]
    fn pure_uptrend_arms_sell_combo_and_advances() {
        // Strictly increasing closes -> sell setup completes at idx 12,
        // then every subsequent bar satisfies the three sell-combo
        // strictness conditions, so combo advances by one per bar and
        // saturates at -13.
        let candles: Vec<Candle> = (1..=40)
            .map(|i| {
                c(
                    f64::from(i) + 0.5,
                    f64::from(i) - 0.5,
                    f64::from(i),
                    i64::from(i),
                )
            })
            .collect();
        let mut combo = TdCombo::classic();
        let out = combo.batch(&candles);
        // First emit is at index 4 (warmup is 5).
        for v in out.iter().take(4) {
            assert!(v.is_none());
        }
        // At idx 12 the setup completes and combo direction is sell; on
        // the same bar the combo rule fires once because the
        // monotone-strictness conditions hold for a strictly-rising
        // series, so combo == -1.
        let at_12 = out[12].expect("ready");
        assert_eq!(at_12, -1.0);
        // By idx 30 the combo has saturated at -13.
        let later = out[30].expect("ready");
        assert_eq!(later, -13.0);
    }

    #[test]
    fn pure_downtrend_arms_buy_combo_and_advances() {
        // Strictly decreasing closes -> buy setup completes at idx 12,
        // then every subsequent bar satisfies the three buy-combo
        // strictness conditions, so combo advances by one per bar and
        // saturates at +13.
        let candles: Vec<Candle> = (1..=40)
            .rev()
            .enumerate()
            .map(|(k, i)| {
                c(
                    f64::from(i) + 0.5,
                    f64::from(i) - 0.5,
                    f64::from(i),
                    i64::try_from(k).unwrap(),
                )
            })
            .collect();
        let mut combo = TdCombo::classic();
        let out = combo.batch(&candles);
        for v in out.iter().take(4) {
            assert!(v.is_none());
        }
        // At idx 12 the setup completes and combo direction is buy; on
        // the same bar the combo rule fires once because the
        // monotone-strictness conditions hold for a strictly-falling
        // series, so combo == +1.
        let at_12 = out[12].expect("ready");
        assert_eq!(at_12, 1.0);
        // By idx 30 the combo has saturated at +13.
        let later = out[30].expect("ready");
        assert_eq!(later, 13.0);
    }

    #[test]
    fn flat_series_never_arms_combo() {
        // All closes equal -> setup never completes -> combo never arms.
        let candles: Vec<Candle> = (0..40).map(|i| c(10.5, 9.5, 10.0, i64::from(i))).collect();
        let mut combo = TdCombo::classic();
        for v in combo.batch(&candles).into_iter().flatten() {
            assert_eq!(v, 0.0);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.3).sin() * 5.0;
                c(m + 1.0, m - 1.0, m, i64::from(i))
            })
            .collect();
        let mut a = TdCombo::classic();
        let mut b = TdCombo::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn rejects_invalid_params() {
        assert!(matches!(TdCombo::new(0, 9, 2, 13), Err(Error::PeriodZero)));
        assert!(matches!(TdCombo::new(4, 0, 2, 13), Err(Error::PeriodZero)));
        assert!(matches!(TdCombo::new(4, 9, 0, 13), Err(Error::PeriodZero)));
        assert!(matches!(TdCombo::new(4, 9, 2, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (1..=30)
            .map(|i| {
                c(
                    f64::from(i) + 0.5,
                    f64::from(i) - 0.5,
                    f64::from(i),
                    i64::from(i),
                )
            })
            .collect();
        let mut combo = TdCombo::classic();
        combo.batch(&candles);
        assert!(combo.is_ready());
        combo.reset();
        assert!(!combo.is_ready());
        assert_eq!(combo.update(candles[0]), None);
    }

    #[test]
    fn accessors_and_metadata() {
        let combo = TdCombo::classic();
        assert_eq!(combo.params(), (4, 9, 2, 13));
        assert_eq!(combo.warmup_period(), 5);
        assert_eq!(combo.name(), "TDCombo");
    }
}
