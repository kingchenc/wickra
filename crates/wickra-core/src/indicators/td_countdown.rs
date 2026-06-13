#![allow(clippy::doc_markdown)]

//! Tom DeMark TD Countdown (standalone 13-bar countdown).
//!
//! The Countdown is the second half of DeMark's TD Sequential, packaged
//! here as a standalone indicator that runs the setup-detection phase
//! internally and then exposes only the countdown count (and direction)
//! to callers who don't need the running setup state.
//!
//! - **Setup detection** (internal): 9 consecutive bars whose close is
//!   less-than (buy setup) or greater-than (sell setup) the close
//!   `setup_lookback` bars earlier.
//! - **Buy countdown** advances on bars where `close[i] <= low[i -
//!   countdown_lookback]` (need not be consecutive). Saturates at
//!   `countdown_target` (13 in DeMark's classic configuration).
//! - **Sell countdown** advances on bars where `close[i] >= high[i -
//!   countdown_lookback]`.
//! - An opposite-direction setup completion invalidates the active
//!   countdown (count resets to zero in the new direction).
//!
//! Output is a signed counter: positive for an active buy countdown,
//! negative for an active sell countdown, and `0.0` when no countdown is
//! currently armed.
//!
//! This indicator differs from [`crate::TdSequential`] only in its
//! output shape: callers who only need the countdown value (and not the
//! running setup count) can use this for a smaller streaming payload.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Direction of an active TD Countdown phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    None,
    Buy,
    Sell,
}

/// TD Countdown — standalone 13-bar countdown.
/// # Example
///
/// ```
/// use wickra_core::{TdCountdown, Candle, Indicator};
///
/// let mut indicator = TdCountdown::new(4, 9, 2, 13).unwrap();
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
pub struct TdCountdown {
    setup_lookback: usize,
    setup_target: usize,
    countdown_lookback: usize,
    countdown_target: usize,
    candles: VecDeque<Candle>,
    buy_setup: usize,
    sell_setup: usize,
    buy_countdown: usize,
    sell_countdown: usize,
    direction: Direction,
    ready: bool,
}

impl TdCountdown {
    /// Construct a TD Countdown with explicit lookbacks and targets. The
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
            buy_countdown: 0,
            sell_countdown: 0,
            direction: Direction::None,
            ready: false,
        })
    }

    /// DeMark's classic configuration: setup `lookback = 4, target = 9`,
    /// countdown `lookback = 2, target = 13`.
    pub fn classic() -> Self {
        Self::new(4, 9, 2, 13).expect("classic TD Countdown parameters are valid")
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

impl Indicator for TdCountdown {
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

        if self.buy_setup == self.setup_target {
            if self.direction != Direction::Buy {
                self.buy_countdown = 0;
                self.sell_countdown = 0;
            }
            self.direction = Direction::Buy;
        } else if self.sell_setup == self.setup_target {
            if self.direction != Direction::Sell {
                self.buy_countdown = 0;
                self.sell_countdown = 0;
            }
            self.direction = Direction::Sell;
        }

        let cd_ref = self.candles[need - self.countdown_lookback];
        match self.direction {
            Direction::Buy => {
                if candle.close <= cd_ref.low && self.buy_countdown < self.countdown_target {
                    self.buy_countdown += 1;
                }
            }
            Direction::Sell => {
                if candle.close >= cd_ref.high && self.sell_countdown < self.countdown_target {
                    self.sell_countdown += 1;
                }
            }
            Direction::None => {}
        }

        self.candles.push_back(candle);
        self.ready = true;

        let v = match self.direction {
            Direction::Buy => self.buy_countdown as f64,
            Direction::Sell => -(self.sell_countdown as f64),
            Direction::None => 0.0,
        };
        Some(v)
    }

    fn reset(&mut self) {
        self.candles.clear();
        self.buy_setup = 0;
        self.sell_setup = 0;
        self.buy_countdown = 0;
        self.sell_countdown = 0;
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
        "TDCountdown"
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
    fn pure_uptrend_completes_setup_then_runs_sell_countdown_to_minus_13() {
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
        let mut td = TdCountdown::classic();
        let out = td.batch(&candles);
        // Warmup: 4 None values.
        for v in out.iter().take(4) {
            assert!(v.is_none());
        }
        // At idx 12 the sell setup completes; on the same bar the
        // countdown rule fires once because close > high[i-2] for a
        // strictly-rising series, so countdown == -1.
        assert_eq!(out[12].expect("ready"), -1.0);
        // After enough bars the countdown saturates at -13.
        assert_eq!(out[30].expect("ready"), -13.0);
    }

    #[test]
    fn pure_downtrend_completes_setup_then_runs_buy_countdown_to_plus_13() {
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
        let mut td = TdCountdown::classic();
        let out = td.batch(&candles);
        for v in out.iter().take(4) {
            assert!(v.is_none());
        }
        // At idx 12 the buy setup completes; on the same bar the
        // countdown rule fires once because close < low[i-2] for a
        // strictly-falling series, so countdown == +1.
        assert_eq!(out[12].expect("ready"), 1.0);
        // After enough bars the countdown saturates at +13.
        assert_eq!(out[30].expect("ready"), 13.0);
    }

    #[test]
    fn flat_series_never_arms_countdown() {
        let candles: Vec<Candle> = (0..30).map(|i| c(10.5, 9.5, 10.0, i64::from(i))).collect();
        let mut td = TdCountdown::classic();
        for v in td.batch(&candles).into_iter().flatten() {
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
        let mut a = TdCountdown::classic();
        let mut b = TdCountdown::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn rejects_invalid_params() {
        assert!(matches!(
            TdCountdown::new(0, 9, 2, 13),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            TdCountdown::new(4, 0, 2, 13),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            TdCountdown::new(4, 9, 0, 13),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            TdCountdown::new(4, 9, 2, 0),
            Err(Error::PeriodZero)
        ));
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
        let mut td = TdCountdown::classic();
        td.batch(&candles);
        assert!(td.is_ready());
        td.reset();
        assert!(!td.is_ready());
        assert_eq!(td.update(candles[0]), None);
    }

    #[test]
    fn accessors_and_metadata() {
        let td = TdCountdown::classic();
        assert_eq!(td.params(), (4, 9, 2, 13));
        assert_eq!(td.warmup_period(), 5);
        assert_eq!(td.name(), "TDCountdown");
    }
}
