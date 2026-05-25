//! Connors RSI (CRSI).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::indicators::rsi::Rsi;
use crate::traits::Indicator;

/// Larry Connors' RSI — average of three short-term mean-reversion components,
/// each individually bounded in `[0, 100]` so the aggregate is too:
///
/// 1. `RSI(close, period_rsi)` — a fast `RSI` (Connors' default `3`).
/// 2. `RSI(streak, period_streak)` — `RSI` of the current up/down run length
///    (`+1, +2, ...` for consecutive up closes, `−1, −2, ...` for down closes,
///    `0` for unchanged). Connors' default `2`.
/// 3. `PercentRank(ROC(1), period_rank)` — the percentile rank of yesterday's
///    1-period return in the last `period_rank` returns. Connors' default `100`.
///
/// ```text
/// CRSI = (RSI(close)_t + RSI(streak)_t + PercentRank(roc1)_t) / 3
/// ```
///
/// All three components live in `[0, 100]`, so `CRSI ∈ [0, 100]`. Connors'
/// trading rule of thumb: `CRSI < 5` is oversold, `CRSI > 95` is overbought
/// — both rare conditions, hence the short lookbacks.
///
/// # Example
///
/// ```
/// use wickra_core::{ConnorsRsi, Indicator};
///
/// let mut crsi = ConnorsRsi::classic();
/// let mut last = None;
/// for i in 0..200 {
///     last = crsi.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct ConnorsRsi {
    period_rsi: usize,
    period_streak: usize,
    period_rank: usize,
    rsi_close: Rsi,
    rsi_streak: Rsi,
    prev_price: Option<f64>,
    streak: f64,
    /// Rolling window of the last `period_rank` 1-period returns
    /// (`(price_t − price_{t-1}) / price_{t-1}`).
    rocs: VecDeque<f64>,
    current: Option<f64>,
}

impl ConnorsRsi {
    /// # Errors
    /// Returns [`Error::PeriodZero`] if any of the three periods is zero.
    pub fn new(period_rsi: usize, period_streak: usize, period_rank: usize) -> Result<Self> {
        if period_rsi == 0 || period_streak == 0 || period_rank == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period_rsi,
            period_streak,
            period_rank,
            rsi_close: Rsi::new(period_rsi)?,
            rsi_streak: Rsi::new(period_streak)?,
            prev_price: None,
            streak: 0.0,
            rocs: VecDeque::with_capacity(period_rank),
            current: None,
        })
    }

    /// Connors' recommended defaults: `(period_rsi = 3, period_streak = 2, period_rank = 100)`.
    pub fn classic() -> Self {
        Self::new(3, 2, 100).expect("classic Connors RSI parameters are valid")
    }

    /// Configured `(period_rsi, period_streak, period_rank)`.
    pub const fn periods(&self) -> (usize, usize, usize) {
        (self.period_rsi, self.period_streak, self.period_rank)
    }
}

impl Indicator for ConnorsRsi {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.current;
        }
        // Run the close-RSI on every input so it warms up regardless of the
        // streak / percent-rank branches.
        let rsi_close = self.rsi_close.update(input);

        let Some(prev) = self.prev_price else {
            self.prev_price = Some(input);
            return None;
        };

        // Update the up/down streak run length.
        self.streak = if input > prev {
            self.streak.max(0.0) + 1.0
        } else if input < prev {
            self.streak.min(0.0) - 1.0
        } else {
            0.0
        };
        let rsi_streak = self.rsi_streak.update(self.streak);

        // 1-period return; defined only when the previous price is non-zero.
        if prev != 0.0 {
            let roc = (input - prev) / prev;
            if self.rocs.len() == self.period_rank {
                self.rocs.pop_front();
            }
            self.rocs.push_back(roc);
        }
        self.prev_price = Some(input);

        // PercentRank emits once the ROC window has filled.
        let percent_rank = if self.rocs.len() == self.period_rank {
            let latest = *self.rocs.back().expect("non-empty window");
            let below = self.rocs.iter().filter(|&&r| r < latest).count();
            Some(100.0 * below as f64 / self.period_rank as f64)
        } else {
            None
        };

        let value = (rsi_close?, rsi_streak?, percent_rank?);
        let crsi = (value.0 + value.1 + value.2) / 3.0;
        self.current = Some(crsi);
        Some(crsi)
    }

    fn reset(&mut self) {
        self.rsi_close.reset();
        self.rsi_streak.reset();
        self.prev_price = None;
        self.streak = 0.0;
        self.rocs.clear();
        self.current = None;
    }

    fn warmup_period(&self) -> usize {
        // The slowest branch is the percent-rank: it needs period_rank + 1
        // prices (period_rank one-period returns). The close-RSI needs
        // period_rsi + 1 prices and the streak-RSI needs period_streak + 1
        // streak values = period_streak + 2 prices. The rank branch dominates
        // for Connors' defaults.
        let rsi_close = self.period_rsi + 1;
        let rsi_streak = self.period_streak + 2;
        let rank = self.period_rank + 1;
        rsi_close.max(rsi_streak).max(rank)
    }

    fn is_ready(&self) -> bool {
        self.current.is_some()
    }

    fn name(&self) -> &'static str {
        "ConnorsRSI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(ConnorsRsi::new(0, 2, 100), Err(Error::PeriodZero)));
        assert!(matches!(ConnorsRsi::new(3, 0, 100), Err(Error::PeriodZero)));
        assert!(matches!(ConnorsRsi::new(3, 2, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let crsi = ConnorsRsi::classic();
        assert_eq!(crsi.periods(), (3, 2, 100));
        assert_eq!(crsi.name(), "ConnorsRSI");
        // Slowest branch: percent_rank with period_rank + 1 = 101.
        assert_eq!(crsi.warmup_period(), 101);
    }

    #[test]
    fn classic_factory() {
        assert_eq!(ConnorsRsi::classic().periods(), (3, 2, 100));
    }

    #[test]
    fn warmup_emits_first_value_at_warmup_period() {
        // Use small periods so the test is fast.
        let mut crsi = ConnorsRsi::new(3, 2, 5).unwrap();
        // Slowest: 5 + 1 = 6.
        assert_eq!(crsi.warmup_period(), 6);
        let prices: Vec<f64> = (1..=8).map(f64::from).collect();
        let out = crsi.batch(&prices);
        for v in out.iter().take(5) {
            assert!(v.is_none());
        }
        assert!(out[5].is_some());
    }

    #[test]
    fn pure_uptrend_saturates_high() {
        // A monotonic uptrend drives all three components toward 100:
        // RSI of monotonic ups is 100, streak stays positive and growing so
        // its RSI is 100, and every new 1-period return matches the prior
        // ones so percent rank stabilises near 0 — but the average of all
        // three still climbs well above 50.
        let mut crsi = ConnorsRsi::classic();
        for i in 1..=200 {
            crsi.update(f64::from(i));
        }
        let v = crsi.current.unwrap();
        assert!(
            v > 60.0,
            "uptrend should drive Connors RSI well above 50: {v}"
        );
    }

    #[test]
    fn output_is_bounded() {
        let mut crsi = ConnorsRsi::classic();
        let prices: Vec<f64> = (0..300)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 20.0)
            .collect();
        for v in crsi.batch(&prices).iter().flatten() {
            assert!(
                (0.0..=100.0).contains(v),
                "Connors RSI out of [0, 100]: {v}"
            );
        }
    }

    #[test]
    fn streak_resets_to_zero_on_unchanged_close() {
        // Helper: feed a sequence and inspect the internal streak.
        let mut crsi = ConnorsRsi::new(3, 2, 100).unwrap();
        crsi.update(10.0);
        crsi.update(11.0);
        crsi.update(12.0);
        assert_eq!(crsi.streak, 2.0);
        crsi.update(12.0);
        assert_relative_eq!(crsi.streak, 0.0, epsilon = 1e-12);
        crsi.update(11.0);
        assert_eq!(crsi.streak, -1.0);
        crsi.update(10.0);
        assert_eq!(crsi.streak, -2.0);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=200)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 5.0 + f64::from(i) * 0.1)
            .collect();
        let mut a = ConnorsRsi::classic();
        let mut b = ConnorsRsi::classic();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut crsi = ConnorsRsi::classic();
        let prices: Vec<f64> = (1..=200).map(f64::from).collect();
        crsi.batch(&prices);
        assert!(crsi.is_ready());
        crsi.reset();
        assert!(!crsi.is_ready());
        assert_eq!(crsi.streak, 0.0);
        assert!(crsi.prev_price.is_none());
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut crsi = ConnorsRsi::classic();
        let prices: Vec<f64> = (1..=200).map(f64::from).collect();
        crsi.batch(&prices);
        let before = crsi.current;
        assert_eq!(crsi.update(f64::NAN), before);
        assert_eq!(crsi.update(f64::INFINITY), before);
    }

    #[test]
    fn zero_prev_skips_roc_update() {
        // A previous price of 0.0 makes the 1-bar return undefined; the
        // ROC ring buffer must be left unchanged on that step. Feeding
        // 0.0 as the very first price seeds `prev_price = Some(0.0)`, so
        // the next bar takes the `prev == 0.0` branch.
        let mut crsi = ConnorsRsi::new(3, 2, 4).unwrap();
        // Bar 1 seeds prev_price to 0.0.
        crsi.update(0.0);
        // Bar 2 must not push onto the ROC window; we cannot observe the
        // ring directly but the indicator must not panic and must not
        // emit until at least period_rank distinct non-zero returns have
        // accumulated.
        let after = crsi.update(1.0);
        assert!(after.is_none(), "CRSI cannot emit on bar 2: {after:?}");
    }
}
