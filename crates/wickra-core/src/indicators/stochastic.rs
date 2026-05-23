//! Stochastic Oscillator (%K and %D).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::indicators::sma::Sma;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Stochastic Oscillator output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StochasticOutput {
    /// Raw %K: `100 * (close - LL) / (HH - LL)` over the lookback.
    pub k: f64,
    /// %D: SMA of %K over the smoothing period.
    pub d: f64,
}

/// Fast Stochastic Oscillator.
///
/// Maintains rolling highest-high and lowest-low over the lookback period via a
/// monotonic deque, giving O(1) amortized updates. %D is an SMA of the %K series.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Stochastic};
///
/// let mut indicator = Stochastic::new(5, 3).unwrap();
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
pub struct Stochastic {
    k_period: usize,
    d_period: usize,
    candles: VecDeque<Candle>,
    // Monotonic deques over candle indices in the rolling window.
    hh_idx: VecDeque<usize>, // indices of candidates for highest high (front = current max)
    ll_idx: VecDeque<usize>, // indices of candidates for lowest low (front = current min)
    // Absolute count of candles ever ingested. Used so monotonic-deque indices stay unique.
    count: usize,
    d_sma: Sma,
    last_k: Option<f64>,
}

impl Stochastic {
    /// Construct a stochastic with %K lookback and %D smoothing periods.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if either period is zero.
    pub fn new(k_period: usize, d_period: usize) -> Result<Self> {
        if k_period == 0 || d_period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            k_period,
            d_period,
            candles: VecDeque::with_capacity(k_period),
            hh_idx: VecDeque::with_capacity(k_period),
            ll_idx: VecDeque::with_capacity(k_period),
            count: 0,
            d_sma: Sma::new(d_period)?,
            last_k: None,
        })
    }

    /// Classic fast stochastic: `%K = 14`, `%D = 3`.
    pub fn classic() -> Self {
        Self::new(14, 3).expect("classic stochastic periods are valid")
    }

    /// Configured `(k_period, d_period)`.
    pub const fn periods(&self) -> (usize, usize) {
        (self.k_period, self.d_period)
    }

    fn push_window(&mut self, candle: Candle) {
        let idx = self.count;
        self.count += 1;
        // Drop deque entries that are outside the window.
        let oldest_keep_idx = idx.saturating_sub(self.k_period - 1);
        while let Some(&front) = self.hh_idx.front() {
            if front < oldest_keep_idx {
                self.hh_idx.pop_front();
            } else {
                break;
            }
        }
        while let Some(&front) = self.ll_idx.front() {
            if front < oldest_keep_idx {
                self.ll_idx.pop_front();
            } else {
                break;
            }
        }
        // Maintain monotonic-decreasing deque for highs.
        while let Some(&back) = self.hh_idx.back() {
            let back_off = back - idx.saturating_sub(self.candles.len());
            if self.candles[back_off].high <= candle.high {
                self.hh_idx.pop_back();
            } else {
                break;
            }
        }
        self.hh_idx.push_back(idx);
        // Maintain monotonic-increasing deque for lows.
        while let Some(&back) = self.ll_idx.back() {
            let back_off = back - idx.saturating_sub(self.candles.len());
            if self.candles[back_off].low >= candle.low {
                self.ll_idx.pop_back();
            } else {
                break;
            }
        }
        self.ll_idx.push_back(idx);

        if self.candles.len() == self.k_period {
            self.candles.pop_front();
        }
        self.candles.push_back(candle);
    }

    fn current_extremes(&self) -> (f64, f64) {
        let base = self.count - self.candles.len();
        let hi = self.candles[self.hh_idx[0] - base].high;
        let lo = self.candles[self.ll_idx[0] - base].low;
        (hi, lo)
    }
}

impl Indicator for Stochastic {
    type Input = Candle;
    type Output = StochasticOutput;

    fn update(&mut self, candle: Candle) -> Option<StochasticOutput> {
        self.push_window(candle);
        if self.candles.len() < self.k_period {
            return None;
        }
        let (hh, ll) = self.current_extremes();
        let range = hh - ll;
        let k = if range == 0.0 {
            // Flat range; convention: 50 (neutral, like RSI on flat input).
            50.0
        } else {
            100.0 * (candle.close - ll) / range
        };
        self.last_k = Some(k);
        let d = self.d_sma.update(k)?;
        Some(StochasticOutput { k, d })
    }

    fn reset(&mut self) {
        self.candles.clear();
        self.hh_idx.clear();
        self.ll_idx.clear();
        self.count = 0;
        self.d_sma.reset();
        self.last_k = None;
    }

    fn warmup_period(&self) -> usize {
        self.k_period + self.d_period - 1
    }

    fn is_ready(&self) -> bool {
        self.d_sma.is_ready()
    }

    fn name(&self) -> &'static str {
        "Stochastic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(h: f64, l: f64, cl: f64) -> Candle {
        Candle::new(cl, h, l, cl, 1.0, 0).unwrap()
    }

    /// Naive %K computation for cross-checks.
    fn naive_k(candles: &[Candle], k_period: usize) -> Vec<Option<f64>> {
        candles
            .iter()
            .enumerate()
            .map(|(i, _)| {
                if i + 1 < k_period {
                    None
                } else {
                    let w = &candles[i + 1 - k_period..=i];
                    let hh = w.iter().map(|x| x.high).fold(f64::NEG_INFINITY, f64::max);
                    let ll = w.iter().map(|x| x.low).fold(f64::INFINITY, f64::min);
                    let range = hh - ll;
                    let cl = candles[i].close;
                    Some(if range == 0.0 {
                        50.0
                    } else {
                        100.0 * (cl - ll) / range
                    })
                }
            })
            .collect()
    }

    #[test]
    fn rejects_zero_periods() {
        assert!(matches!(Stochastic::new(0, 3), Err(Error::PeriodZero)));
        assert!(matches!(Stochastic::new(14, 0), Err(Error::PeriodZero)));
    }

    /// Cover the `Stochastic::classic()` convenience constructor plus the
    /// `periods()` const accessor and the Indicator-impl `warmup_period`
    /// / `name` methods. Existing tests called `Stochastic::new(_, _)`
    /// directly and never asked for the configured periods, warmup
    /// length, or name.
    #[test]
    fn classic_periods_and_metadata() {
        let s = Stochastic::classic();
        assert_eq!(s.periods(), (14, 3));
        // Warmup for the classic config: k_period + d_period - 1 = 14 + 3 - 1 = 16.
        assert_eq!(s.warmup_period(), 16);
        assert_eq!(s.name(), "Stochastic");
    }

    #[test]
    fn close_at_high_yields_k_100() {
        let candles = vec![
            c(10.0, 8.0, 9.0),
            c(11.0, 9.0, 10.0),
            c(12.0, 10.0, 12.0), // close == high == HH
        ];
        let mut s = Stochastic::new(3, 1).unwrap();
        let out = s.batch(&candles);
        assert_relative_eq!(out[2].unwrap().k, 100.0, epsilon = 1e-12);
    }

    #[test]
    fn close_at_low_yields_k_0() {
        let candles = vec![
            c(10.0, 8.0, 9.0),
            c(11.0, 9.0, 10.0),
            c(12.0, 8.0, 8.0), // close == LL
        ];
        let mut s = Stochastic::new(3, 1).unwrap();
        let out = s.batch(&candles);
        assert_relative_eq!(out[2].unwrap().k, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn flat_range_yields_k_50() {
        let candles: Vec<Candle> = (0..20).map(|_| c(10.0, 10.0, 10.0)).collect();
        let mut s = Stochastic::new(14, 3).unwrap();
        for o in s.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(o.k, 50.0, epsilon = 1e-12);
            assert_relative_eq!(o.d, 50.0, epsilon = 1e-12);
        }
        // Cross-check: the naive_k test helper must agree on the flat-range
        // convention. The k_matches_naive test only feeds oscillating prices,
        // so the helper's flat-range branch was never exercised.
        let ks = naive_k(&candles, 14);
        for k in ks.into_iter().skip(13) {
            assert_relative_eq!(k.expect("ready after 14 inputs"), 50.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn k_matches_naive() {
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                let mid = 50.0 + (f64::from(i) * 0.4).sin() * 10.0;
                c(mid + 2.0, mid - 2.0, mid + (f64::from(i) * 0.7).cos())
            })
            .collect();
        let mut s = Stochastic::new(14, 3).unwrap();
        let out = s.batch(&candles);
        let naive = naive_k(&candles, 14);
        for (i, got) in out.iter().enumerate() {
            if let Some(o) = got {
                let n = naive[i].expect("naive ready");
                assert_relative_eq!(o.k, n, epsilon = 1e-9);
            }
        }
    }

    #[test]
    fn d_is_sma_of_k() {
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                let mid = 50.0 + f64::from(i).sin() * 5.0;
                c(mid + 1.5, mid - 1.5, mid)
            })
            .collect();
        let mut s = Stochastic::new(14, 3).unwrap();
        let out = s.batch(&candles);
        // The naive %K series gives us the ground-truth values that %D should average.
        let naive_ks = naive_k(&candles, 14);
        // The first emitted %D corresponds to the SMA of the first three valid %K values
        // (i.e. those at indices 13, 14, 15). At that point %D becomes ready, and the
        // first `Some(_)` output appears at index 15.
        let first_emit_idx = out
            .iter()
            .position(Option::is_some)
            .expect("d eventually emits");
        let first_d = out[first_emit_idx].unwrap().d;
        let k_window = &naive_ks[first_emit_idx - 2..=first_emit_idx];
        let want = k_window
            .iter()
            .map(|v| v.expect("naive K ready inside window"))
            .sum::<f64>()
            / 3.0;
        assert_relative_eq!(first_d, want, epsilon = 1e-9);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..50)
            .map(|i| {
                let mid = 100.0 + f64::from(i) * 0.5;
                c(mid + 2.0, mid - 2.0, mid)
            })
            .collect();
        let mut a = Stochastic::new(14, 3).unwrap();
        let mut b = Stochastic::new(14, 3).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut s = Stochastic::new(5, 3).unwrap();
        let candles: Vec<Candle> = (0..10).map(|i| c(10.0 + f64::from(i), 5.0, 7.0)).collect();
        s.batch(&candles);
        assert!(s.is_ready());
        s.reset();
        assert!(!s.is_ready());
        assert_eq!(s.update(candles[0]), None);
    }
}
