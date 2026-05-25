//! Schaff Trend Cycle (STC).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::indicators::ema::Ema;
use crate::traits::Indicator;

/// Doug Schaff's Trend Cycle — a doubly-`Stochastic`-smoothed MACD that
/// produces a bounded `[0, 100]` reading reacting faster than `MACD` itself.
///
/// ```text
/// macd_t  = EMA(close, fast)_t − EMA(close, slow)_t
/// %K_t    = 100 · (macd − LL(macd, period)) / (HH(macd, period) − LL(macd, period))
/// %D_t    = %D_{t-1} + factor · (%K_t − %D_{t-1})           // half-EMA when factor = 0.5
/// %K2_t   = 100 · (%D − LL(%D, period)) / (HH(%D, period) − LL(%D, period))
/// STC_t   = STC_{t-1} + factor · (%K2_t − STC_{t-1})
/// ```
///
/// Wickra uses `factor = 0.5` and Schaff's recommended defaults
/// `(fast = 23, slow = 50, period = 10)`. The stochastic stages clamp to `0`
/// when the window range collapses (perfectly flat input), and the smoothing
/// stages hold their previous value if the upstream stage is not yet ready —
/// so a flat input series settles deterministically at `0` after warmup.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Stc};
///
/// let mut stc = Stc::classic();
/// let mut last = None;
/// for i in 0..200 {
///     last = stc.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Stc {
    fast_period: usize,
    slow_period: usize,
    schaff_period: usize,
    factor: f64,
    fast_ema: Ema,
    slow_ema: Ema,
    macd_window: VecDeque<f64>,
    d_window: VecDeque<f64>,
    last_d: Option<f64>,
    last_value: Option<f64>,
}

impl Stc {
    /// # Errors
    /// - [`Error::PeriodZero`] if any period is zero.
    /// - [`Error::InvalidPeriod`] if `fast >= slow` or `factor` is not in `(0, 1]`.
    pub fn new(fast: usize, slow: usize, schaff_period: usize, factor: f64) -> Result<Self> {
        if fast == 0 || slow == 0 || schaff_period == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "STC fast period must be strictly less than slow",
            });
        }
        if !factor.is_finite() || factor <= 0.0 || factor > 1.0 {
            return Err(Error::InvalidPeriod {
                message: "STC factor must be a finite value in (0, 1]",
            });
        }
        Ok(Self {
            fast_period: fast,
            slow_period: slow,
            schaff_period,
            factor,
            fast_ema: Ema::new(fast)?,
            slow_ema: Ema::new(slow)?,
            macd_window: VecDeque::with_capacity(schaff_period),
            d_window: VecDeque::with_capacity(schaff_period),
            last_d: None,
            last_value: None,
        })
    }

    /// Schaff's recommended defaults `(fast = 23, slow = 50, period = 10, factor = 0.5)`.
    pub fn classic() -> Self {
        Self::new(23, 50, 10, 0.5).expect("classic STC parameters are valid")
    }

    /// Configured `(fast, slow, schaff_period, factor)`.
    pub const fn params(&self) -> (usize, usize, usize, f64) {
        (
            self.fast_period,
            self.slow_period,
            self.schaff_period,
            self.factor,
        )
    }
}

fn rolling_minmax(window: &VecDeque<f64>) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &v in window {
        if v < lo {
            lo = v;
        }
        if v > hi {
            hi = v;
        }
    }
    (lo, hi)
}

impl Indicator for Stc {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        let f = self.fast_ema.update(input);
        let s = self.slow_ema.update(input);
        let (f, s) = (f?, s?);
        let macd = f - s;

        if self.macd_window.len() == self.schaff_period {
            self.macd_window.pop_front();
        }
        self.macd_window.push_back(macd);
        if self.macd_window.len() < self.schaff_period {
            return None;
        }

        let (lo, hi) = rolling_minmax(&self.macd_window);
        let k = if hi > lo {
            100.0 * (macd - lo) / (hi - lo)
        } else {
            0.0
        };

        let d = match self.last_d {
            Some(prev) => prev + self.factor * (k - prev),
            None => k,
        };
        self.last_d = Some(d);

        if self.d_window.len() == self.schaff_period {
            self.d_window.pop_front();
        }
        self.d_window.push_back(d);
        if self.d_window.len() < self.schaff_period {
            return None;
        }

        let (lo_d, hi_d) = rolling_minmax(&self.d_window);
        let k2 = if hi_d > lo_d {
            100.0 * (d - lo_d) / (hi_d - lo_d)
        } else {
            0.0
        };

        let stc = match self.last_value {
            Some(prev) => prev + self.factor * (k2 - prev),
            None => k2,
        };
        self.last_value = Some(stc);
        Some(stc.clamp(0.0, 100.0))
    }

    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.macd_window.clear();
        self.d_window.clear();
        self.last_d = None;
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        // Slow EMA emits at `slow` inputs. Then the macd-window needs
        // `schaff_period − 1` more inputs to fill, and the d-window another
        // `schaff_period − 1` after that.
        self.slow_period + 2 * (self.schaff_period - 1)
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some() && self.d_window.len() == self.schaff_period
    }

    fn name(&self) -> &'static str {
        "STC"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(Stc::new(0, 50, 10, 0.5), Err(Error::PeriodZero)));
        assert!(matches!(Stc::new(23, 0, 10, 0.5), Err(Error::PeriodZero)));
        assert!(matches!(Stc::new(23, 50, 0, 0.5), Err(Error::PeriodZero)));
    }

    #[test]
    fn rejects_invalid_params() {
        assert!(matches!(
            Stc::new(50, 23, 10, 0.5),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            Stc::new(23, 50, 10, 0.0),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            Stc::new(23, 50, 10, 1.5),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            Stc::new(23, 50, 10, f64::NAN),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let stc = Stc::classic();
        let (f, s, p, k) = stc.params();
        assert_eq!((f, s, p), (23, 50, 10));
        assert!((k - 0.5).abs() < 1e-12);
        assert_eq!(stc.warmup_period(), 50 + 18);
        assert_eq!(stc.name(), "STC");
    }

    #[test]
    fn classic_factory() {
        let (f, s, p, k) = Stc::classic().params();
        assert_eq!((f, s, p), (23, 50, 10));
        assert!((k - 0.5).abs() < 1e-12);
    }

    #[test]
    fn constant_series_yields_zero() {
        // Flat input -> macd is 0 every bar -> stochastic-on-flat-window
        // returns 0 -> d stays at 0 -> %K2 returns 0 -> STC stays at 0.
        let mut stc = Stc::new(3, 5, 4, 0.5).unwrap();
        let out = stc.batch(&[42.0_f64; 80]);
        for v in out.iter().rev().take(5).flatten() {
            assert_eq!(*v, 0.0);
        }
    }

    #[test]
    fn warmup_emits_first_value_at_warmup_period() {
        let mut stc = Stc::new(2, 4, 3, 0.5).unwrap();
        // slow(4) + 2*(3-1) = 8.
        assert_eq!(stc.warmup_period(), 8);
        let prices: Vec<f64> = (1..=10).map(f64::from).collect();
        let out = stc.batch(&prices);
        for v in out.iter().take(7) {
            assert!(v.is_none());
        }
        assert!(out[7].is_some());
    }

    #[test]
    fn output_is_bounded() {
        let mut stc = Stc::classic();
        let prices: Vec<f64> = (0..400)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 25.0)
            .collect();
        for v in stc.batch(&prices).iter().flatten() {
            assert!((0.0..=100.0).contains(v), "STC out of [0, 100]: {v}");
        }
    }

    #[test]
    fn oscillating_series_visits_full_range() {
        // STC needs a non-degenerate MACD range to exercise the two
        // stochastic stages. A purely monotone series collapses the rolling
        // window (constant MACD) and a purely flat one collapses both
        // stages — in either case both inner ranges become zero and STC
        // sticks at 0. A sinusoidal trend with enough amplitude makes the
        // stages cycle through the full [0, 100] band.
        let mut stc = Stc::classic();
        let prices: Vec<f64> = (0..400)
            .map(|i| 100.0 + (f64::from(i) * 0.15).sin() * 30.0)
            .collect();
        let out = stc.batch(&prices);
        let mut saw_high = false;
        let mut saw_low = false;
        for v in out.iter().flatten() {
            if *v > 80.0 {
                saw_high = true;
            }
            if *v < 20.0 {
                saw_low = true;
            }
        }
        assert!(
            saw_high,
            "STC should reach above 80 on a strong oscillation"
        );
        assert!(saw_low, "STC should reach below 20 on a strong oscillation");
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=200)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 5.0)
            .collect();
        let mut a = Stc::classic();
        let mut b = Stc::classic();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut stc = Stc::classic();
        stc.batch(&(1..=200).map(f64::from).collect::<Vec<_>>());
        assert!(stc.is_ready());
        stc.reset();
        assert!(!stc.is_ready());
        assert!(stc.last_value.is_none());
    }
}
