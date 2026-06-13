#![allow(clippy::doc_markdown)]

//! Tom DeMark TD Sequential (Setup + Countdown).
//!
//! TD Sequential is DeMark's flagship two-phase exhaustion pattern:
//!
//! 1. **Setup phase** — 9 consecutive bars whose close is less-than (buy
//!    setup) or greater-than (sell setup) the close 4 bars earlier. The
//!    setup *completes* on the 9th bar.
//! 2. **Countdown phase** — after a completed setup, count up to 13 bars
//!    that satisfy the countdown comparison (buy countdown: `close <= low`
//!    two bars earlier; sell countdown: `close >= high` two bars earlier).
//!    Countdown bars do not need to be consecutive.
//!
//! A completed countdown (13) signals exhaustion in the direction of the
//! original setup and is the canonical DeMark reversal signal.
//!
//! Output struct `TdSequentialOutput`:
//!
//! - `setup`: signed setup count (positive for buy setup, negative for sell
//!   setup, 0 when no streak is active; capped at ±9).
//! - `countdown`: signed countdown count (positive for buy countdown, negative
//!   for sell countdown, 0 when no countdown is active; capped at ±13).
//! - `direction`: `+1.0` if a buy countdown is currently active, `-1.0` if a
//!   sell countdown is active, `0.0` otherwise. The countdown direction is
//!   set when the originating setup completes and stays valid until the
//!   countdown finishes or is invalidated by an opposite-direction setup.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Direction of an active TD Sequential countdown phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    None,
    Buy,
    Sell,
}

/// Output of [`TdSequential`]: setup count, countdown count, and active
/// countdown direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TdSequentialOutput {
    /// Signed setup count: +N for an active buy setup of length `N`, −N for
    /// a sell setup of length `N`, 0 if neither streak is active. Capped at
    /// ±9 (the canonical setup target).
    pub setup: f64,
    /// Signed countdown count: +N for an active buy countdown of length `N`,
    /// −N for a sell countdown of length `N`, 0 if no countdown is active.
    /// Capped at ±13.
    pub countdown: f64,
    /// Direction of the active countdown: `+1.0` for buy, `−1.0` for sell,
    /// `0.0` if no countdown is currently active.
    pub direction: f64,
}

/// TD Sequential state machine: combined Setup (1-9) + Countdown (1-13).
/// # Example
///
/// ```
/// use wickra_core::{TdSequential, Candle, Indicator};
///
/// let mut indicator = TdSequential::new(4, 9, 2, 13).unwrap();
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
pub struct TdSequential {
    // Rolling window of recent candles. We need up to 5 closes back (for the
    // setup rule which compares close[i] vs close[i-4]) and the high/low from
    // 2 bars ago (for the countdown rule).
    candles: VecDeque<Candle>,
    setup_lookback: usize,
    setup_target: usize,
    countdown_lookback: usize,
    countdown_target: usize,
    buy_setup: usize,
    sell_setup: usize,
    buy_countdown: usize,
    sell_countdown: usize,
    countdown_dir: Direction,
    ready: bool,
}

impl TdSequential {
    /// Construct a TD Sequential with explicit lookbacks and targets. The
    /// canonical DeMark configuration is `setup_lookback = 4`, `setup_target =
    /// 9`, `countdown_lookback = 2`, `countdown_target = 13`.
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
        // Need to keep enough candles for both rules: setup uses close[-N];
        // countdown uses high/low[-M]. Reserve `max(N, M) + 1` slots.
        let cap = setup_lookback.max(countdown_lookback) + 1;
        Ok(Self {
            candles: VecDeque::with_capacity(cap),
            setup_lookback,
            setup_target,
            countdown_lookback,
            countdown_target,
            buy_setup: 0,
            sell_setup: 0,
            buy_countdown: 0,
            sell_countdown: 0,
            countdown_dir: Direction::None,
            ready: false,
        })
    }

    /// DeMark's classic configuration: setup `lookback = 4, target = 9`,
    /// countdown `lookback = 2, target = 13`.
    pub fn classic() -> Self {
        Self::new(4, 9, 2, 13).expect("classic TD Sequential parameters are valid")
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

impl Indicator for TdSequential {
    type Input = Candle;
    type Output = TdSequentialOutput;

    fn update(&mut self, candle: Candle) -> Option<TdSequentialOutput> {
        let cap = self.setup_lookback.max(self.countdown_lookback) + 1;
        if self.candles.len() == cap {
            self.candles.pop_front();
        }
        // The required minimum history is `max(setup_lookback,
        // countdown_lookback)` previous bars. Once we have that many, we can
        // evaluate both rules.
        let need = self.setup_lookback.max(self.countdown_lookback);
        if self.candles.len() < need {
            self.candles.push_back(candle);
            return None;
        }

        // --- Setup rule: compare to close[setup_lookback bars ago] ---
        // After `need` candles are buffered, the candle at offset `need - L`
        // from the front is the one `L` bars before the new candle (0-based
        // count: `front()` is `need` bars ago).
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

        // --- Countdown activation: when a setup completes, arm the countdown
        // in the same direction; an opposite-direction setup invalidates any
        // active countdown.
        if self.buy_setup == self.setup_target {
            if self.countdown_dir != Direction::Buy {
                self.buy_countdown = 0;
                self.sell_countdown = 0;
            }
            self.countdown_dir = Direction::Buy;
        } else if self.sell_setup == self.setup_target {
            if self.countdown_dir != Direction::Sell {
                self.buy_countdown = 0;
                self.sell_countdown = 0;
            }
            self.countdown_dir = Direction::Sell;
        }

        // --- Countdown rule: compare close to high/low `countdown_lookback`
        // bars ago. Only the active direction advances. Once a countdown
        // reaches `countdown_target`, the strict `< countdown_target` guard
        // keeps it pinned so the caller can detect the "13" signal on this
        // bar and any subsequent bar until a new setup arms a fresh run.
        let cd_ref_idx = need - self.countdown_lookback;
        let cd_ref = &self.candles[cd_ref_idx];
        match self.countdown_dir {
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

        let setup = if self.buy_setup > 0 {
            self.buy_setup as f64
        } else if self.sell_setup > 0 {
            -(self.sell_setup as f64)
        } else {
            0.0
        };
        let (countdown, direction) = match self.countdown_dir {
            Direction::Buy => (self.buy_countdown as f64, 1.0),
            Direction::Sell => (-(self.sell_countdown as f64), -1.0),
            Direction::None => (0.0, 0.0),
        };

        Some(TdSequentialOutput {
            setup,
            countdown,
            direction,
        })
    }

    fn reset(&mut self) {
        self.candles.clear();
        self.buy_setup = 0;
        self.sell_setup = 0;
        self.buy_countdown = 0;
        self.sell_countdown = 0;
        self.countdown_dir = Direction::None;
        self.ready = false;
    }

    fn warmup_period(&self) -> usize {
        self.setup_lookback.max(self.countdown_lookback) + 1
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn name(&self) -> &'static str {
        "TDSequential"
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
    fn pure_uptrend_completes_sell_setup_then_progresses_countdown() {
        // Strictly increasing closes -> sell setup increments every bar past
        // warmup, reaching -9 by index 12 (warmup is 4 + 1). After that,
        // every bar continues to make a higher close, so each subsequent bar
        // also makes a higher close than the high 2 bars ago — the sell
        // countdown increments on each bar after activation.
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
        let mut td = TdSequential::classic();
        let out = td.batch(&candles);

        // Warmup: indices 0..3 yield None (need=4 prior closes).
        for v in out.iter().take(4) {
            assert!(v.is_none());
        }
        // After index 12, setup reaches -9 (completed). From the next bar on,
        // countdown begins to increment.
        let at_12 = out[12].expect("setup ready");
        assert_eq!(at_12.setup, -9.0);
        assert_eq!(at_12.direction, -1.0); // countdown direction armed

        // Each subsequent bar makes close > high[i-2], so the sell countdown
        // advances by one per bar; by some later index it caps at -13.
        let later = out[30].expect("ready");
        assert_eq!(later.direction, -1.0);
        assert_eq!(later.countdown, -13.0);
    }

    #[test]
    fn pure_downtrend_completes_buy_setup_then_progresses_countdown() {
        // Strictly decreasing closes -> buy setup increments every bar past
        // warmup, reaching 9 by index 12. After activation, every subsequent
        // bar satisfies close <= low[i-2], so the buy countdown advances by
        // one per bar and pins at +13.
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
        let mut td = TdSequential::classic();
        let out = td.batch(&candles);

        // Warmup: indices 0..3 yield None.
        for v in out.iter().take(4) {
            assert!(v.is_none());
        }
        let at_12 = out[12].expect("setup ready");
        assert_eq!(at_12.setup, 9.0);
        assert_eq!(at_12.direction, 1.0); // buy direction armed

        // By idx 30 the buy countdown has saturated at +13.
        let later = out[30].expect("ready");
        assert_eq!(later.direction, 1.0);
        assert_eq!(later.countdown, 13.0);
    }

    #[test]
    fn flat_series_emits_zero_setup_and_no_countdown() {
        // All closes equal -> never completes any setup; countdown never
        // activates; setup, countdown, direction all stay at 0.
        let candles: Vec<Candle> = (0..30).map(|i| c(10.5, 9.5, 10.0, i64::from(i))).collect();
        let mut td = TdSequential::classic();
        let out = td.batch(&candles);
        for v in out.iter().skip(5) {
            let o = v.expect("ready post-warmup");
            assert_eq!(o.setup, 0.0);
            assert_eq!(o.countdown, 0.0);
            assert_eq!(o.direction, 0.0);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                let m = 100.0 + (f64::from(i) * 0.3).sin() * 5.0;
                c(m + 1.0, m - 1.0, m, i64::from(i))
            })
            .collect();
        let mut a = TdSequential::classic();
        let mut b = TdSequential::classic();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn rejects_invalid_params() {
        assert!(matches!(
            TdSequential::new(0, 9, 2, 13),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            TdSequential::new(4, 0, 2, 13),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            TdSequential::new(4, 9, 0, 13),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            TdSequential::new(4, 9, 2, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (1..=20)
            .map(|i| {
                c(
                    f64::from(i) + 0.5,
                    f64::from(i) - 0.5,
                    f64::from(i),
                    i64::from(i),
                )
            })
            .collect();
        let mut td = TdSequential::classic();
        td.batch(&candles);
        assert!(td.is_ready());
        td.reset();
        assert!(!td.is_ready());
        assert_eq!(td.update(candles[0]), None);
    }

    #[test]
    fn accessors_and_metadata() {
        let td = TdSequential::classic();
        assert_eq!(td.params(), (4, 9, 2, 13));
        assert_eq!(td.warmup_period(), 5);
        assert_eq!(td.name(), "TDSequential");
    }
}
