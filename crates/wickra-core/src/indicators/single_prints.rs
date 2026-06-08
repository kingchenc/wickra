//! Single Prints — count of price levels touched by exactly one bar (low acceptance).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Single Prints — the number of price levels (bins) in the rolling profile that
/// were touched by **exactly one** bar, marking zones of low acceptance / fast
/// movement.
///
/// ```text
/// for each of `bins` price levels over the last `period` candles:
///   touches = number of bars whose high-low range covers that level
/// SinglePrints = count of levels with touches == 1
/// ```
///
/// In Market Profile a "single print" is a price the market traded through so
/// quickly that only one time-period printed there — a footprint of an aggressive,
/// one-sided move with little two-way trade. Single prints often act as support or
/// resistance on a retest (the imbalance gets "repaired") and mark the edges of
/// rapid moves. Counting them per profile gives a streaming gauge of how much of
/// the recent range was traversed without acceptance: a high count means a fast,
/// trending, low-rotation market; a low count means a balanced, well-traded range.
///
/// The output is a non-negative count. The first value lands after `period`
/// candles; each `update` rebuilds the touch histogram in O(`period · bins`).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, SinglePrints};
///
/// let mut indicator = SinglePrints::new(20, 24).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let base = 100.0 + f64::from(i); // a one-directional ramp -> many single prints
///     let c = Candle::new(base, base + 0.5, base - 0.5, base, 1_000.0, 0).unwrap();
///     last = indicator.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct SinglePrints {
    period: usize,
    bins: usize,
    window: VecDeque<Candle>,
    last: Option<f64>,
}

impl SinglePrints {
    /// Construct a Single Prints counter.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period` or `bins` is zero.
    pub fn new(period: usize, bins: usize) -> Result<Self> {
        if period == 0 || bins == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            bins,
            window: VecDeque::with_capacity(period),
            last: None,
        })
    }

    /// Configured `(period, bins)`.
    pub const fn params(&self) -> (usize, usize) {
        (self.period, self.bins)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn count_single_prints(&self) -> usize {
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for c in &self.window {
            low = low.min(c.low);
            high = high.max(c.high);
        }
        let span = high - low;
        if span <= 0.0 {
            return 0;
        }
        let width = span / self.bins as f64;
        let mut touches = vec![0u32; self.bins];
        for c in &self.window {
            let lo_idx = (((c.low - low) / width).floor() as usize).min(self.bins - 1);
            let hi_idx = (((c.high - low) / width).floor() as usize).min(self.bins - 1);
            for t in touches.iter_mut().take(hi_idx + 1).skip(lo_idx) {
                *t += 1;
            }
        }
        touches.iter().filter(|&&t| t == 1).count()
    }
}

impl Indicator for SinglePrints {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(candle);
        if self.window.len() < self.period {
            return None;
        }
        let count = self.count_single_prints() as f64;
        self.last = Some(count);
        Some(count)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "SinglePrints"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(high: f64, low: f64) -> Candle {
        Candle::new_unchecked(
            f64::midpoint(high, low),
            high,
            low,
            f64::midpoint(high, low),
            1_000.0,
            0,
        )
    }

    #[test]
    fn rejects_zero_params() {
        assert!(matches!(SinglePrints::new(0, 24), Err(Error::PeriodZero)));
        assert!(matches!(SinglePrints::new(20, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let s = SinglePrints::new(20, 24).unwrap();
        assert_eq!(s.params(), (20, 24));
        assert_eq!(s.warmup_period(), 20);
        assert_eq!(s.name(), "SinglePrints");
        assert!(!s.is_ready());
        assert_eq!(s.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut s = SinglePrints::new(4, 8).unwrap();
        let candles: Vec<Candle> = (0..6)
            .map(|i| c(101.0 + f64::from(i), 99.0 + f64::from(i)))
            .collect();
        let out = s.batch(&candles);
        for v in out.iter().take(3) {
            assert!(v.is_none());
        }
        assert!(out[3].is_some());
    }

    #[test]
    fn flat_range_has_no_single_prints() {
        // Every bar covers the same single price -> zero span -> 0.
        let mut s = SinglePrints::new(4, 8).unwrap();
        let last = s
            .batch(&[c(100.0, 100.0); 6])
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_eq!(last, 0.0);
    }

    #[test]
    fn ramp_has_many_single_prints() {
        // A one-directional ramp visits most levels exactly once.
        let mut s = SinglePrints::new(10, 24).unwrap();
        let candles: Vec<Candle> = (0..10)
            .map(|i| c(100.5 + f64::from(i), 99.5 + f64::from(i)))
            .collect();
        let last = s.batch(&candles).into_iter().flatten().last().unwrap();
        assert!(
            last > 0.0,
            "a ramp should produce single prints, got {last}"
        );
    }

    #[test]
    fn output_non_negative() {
        let mut s = SinglePrints::new(14, 24).unwrap();
        for v in s
            .batch(
                &(0..60)
                    .map(|i| c(110.0 + (f64::from(i) * 0.3).sin() * 8.0, 90.0))
                    .collect::<Vec<_>>(),
            )
            .into_iter()
            .flatten()
        {
            assert!(v >= 0.0);
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut s = SinglePrints::new(4, 8).unwrap();
        s.batch(
            &(0..6)
                .map(|i| c(101.0 + f64::from(i), 99.0 + f64::from(i)))
                .collect::<Vec<_>>(),
        );
        assert!(s.is_ready());
        s.reset();
        assert!(!s.is_ready());
        assert_eq!(s.value(), None);
        assert_eq!(s.update(c(101.0, 99.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| c(110.0 + (f64::from(i) * 0.25).sin() * 9.0, 90.0))
            .collect();
        let batch = SinglePrints::new(20, 24).unwrap().batch(&candles);
        let mut b = SinglePrints::new(20, 24).unwrap();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
