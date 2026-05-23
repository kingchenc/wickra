//! Average True Range (Wilder).

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Average True Range with Wilder smoothing.
///
/// The first emitted value, by convention, appears after `period` candles: the
/// first `period − 1` true-range values seed the Wilder average alongside the
/// `period`-th, then the smoothed update begins.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Atr};
///
/// let mut indicator = Atr::new(5).unwrap();
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
pub struct Atr {
    period: usize,
    prev_close: Option<f64>,
    seed_buf: Vec<f64>,
    avg: Option<f64>,
}

impl Atr {
    /// Construct an ATR with the given Wilder period.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            prev_close: None,
            seed_buf: Vec::with_capacity(period),
            avg: None,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.avg
    }
}

impl Indicator for Atr {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let tr = candle.true_range(self.prev_close);
        self.prev_close = Some(candle.close);

        if let Some(avg) = self.avg {
            let n = self.period as f64;
            let new_avg = avg.mul_add(n - 1.0, tr) / n;
            self.avg = Some(new_avg);
            return Some(new_avg);
        }

        self.seed_buf.push(tr);
        if self.seed_buf.len() == self.period {
            let seed = self.seed_buf.iter().copied().sum::<f64>() / self.period as f64;
            self.avg = Some(seed);
            return Some(seed);
        }
        None
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.seed_buf.clear();
        self.avg = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.avg.is_some()
    }

    fn name(&self) -> &'static str {
        "ATR"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(h: f64, l: f64, cl: f64) -> Candle {
        // ts/open/volume don't affect ATR; use safe placeholders.
        Candle::new(cl, h, l, cl, 1.0, 0).unwrap()
    }

    /// Independent reference: Wilder ATR computed straight from the definition.
    fn atr_naive(hlc: &[(f64, f64, f64)], period: usize) -> Vec<Option<f64>> {
        let n = period as f64;
        let mut out = Vec::with_capacity(hlc.len());
        let mut trs: Vec<f64> = Vec::new();
        let mut avg: Option<f64> = None;
        let mut prev_close: Option<f64> = None;
        for &(h, l, cl) in hlc {
            let tr = match prev_close {
                None => h - l,
                Some(pc) => (h - l).max((h - pc).abs()).max((l - pc).abs()),
            };
            prev_close = Some(cl);
            if let Some(a) = avg {
                let na = (a * (n - 1.0) + tr) / n;
                avg = Some(na);
                out.push(Some(na));
            } else {
                trs.push(tr);
                if trs.len() == period {
                    avg = Some(trs.iter().sum::<f64>() / n);
                    out.push(avg);
                } else {
                    out.push(None);
                }
            }
        }
        out
    }

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(Atr::new(0), Err(Error::PeriodZero)));
    }

    /// Cover the const accessors `period` / `value` (54-62) and the
    /// Indicator-impl `name` body (103-105). Existing tests inspect
    /// numeric ATR output but never query the metadata.
    #[test]
    fn accessors_and_metadata() {
        let mut atr = Atr::new(14).unwrap();
        assert_eq!(atr.period(), 14);
        assert_eq!(atr.name(), "ATR");
        assert_eq!(atr.value(), None);
        for _ in 0..14 {
            atr.update(c(11.0, 9.0, 10.0));
        }
        assert!(atr.value().is_some());
    }

    #[test]
    fn warmup_emits_on_period_th_candle() {
        let candles = vec![
            c(2.0, 1.0, 1.5),
            c(3.0, 2.0, 2.5),
            c(4.0, 3.0, 3.5),
            c(5.0, 4.0, 4.5),
            c(6.0, 5.0, 5.5),
        ];
        let mut atr = Atr::new(3).unwrap();
        let out = atr.batch(&candles);
        assert!(out[0].is_none());
        assert!(out[1].is_none());
        assert!(out[2].is_some());
        assert!(out[3].is_some());
    }

    #[test]
    fn constant_range_yields_constant_atr() {
        // Every candle has H=11, L=9, C=10 -> TR=2 (no gaps).
        let candles: Vec<Candle> = (0..30).map(|_| c(11.0, 9.0, 10.0)).collect();
        let mut atr = Atr::new(14).unwrap();
        let out = atr.batch(&candles);
        for v in out.iter().skip(13).flatten() {
            assert_relative_eq!(*v, 2.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn gap_up_uses_high_minus_prev_close() {
        // Previous close 5, current candle H=10 L=9 C=9.5 -> TR = max(1, 5, 4) = 5.
        let candles = vec![
            c(6.0, 4.0, 5.0),  // prev close = 5
            c(10.0, 9.0, 9.5), // TR = 5
        ];
        let mut atr = Atr::new(2).unwrap();
        let out = atr.batch(&candles);
        // Seed window covers TR_1 and TR_2. TR_1 = H1-L1 = 2 (no prev close). TR_2 = 5.
        // Seed = (2+5)/2 = 3.5
        assert_relative_eq!(out[1].unwrap(), 3.5, epsilon = 1e-12);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let mid = f64::from(i) + 10.0;
                c(mid + 0.5, mid - 0.5, mid)
            })
            .collect();
        let mut a = Atr::new(14).unwrap();
        let mut b = Atr::new(14).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (0..20).map(|_| c(11.0, 9.0, 10.0)).collect();
        let mut atr = Atr::new(5).unwrap();
        atr.batch(&candles);
        assert!(atr.is_ready());
        atr.reset();
        assert!(!atr.is_ready());
        assert_eq!(atr.update(candles[0]), None);
    }

    #[test]
    fn never_negative() {
        let candles: Vec<Candle> = (0..200)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.3).sin() * 5.0;
                c(base + 1.0, base - 1.0, base)
            })
            .collect();
        let mut atr = Atr::new(14).unwrap();
        for v in atr.batch(&candles).into_iter().flatten() {
            assert!(v >= 0.0, "ATR must be non-negative: {v}");
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(48))]
        #[test]
        fn atr_matches_naive(
            period in 1usize..15,
            bars in proptest::collection::vec(
                (10.0_f64..1000.0, 0.0_f64..50.0, 0.0_f64..1.0),
                0..120,
            ),
        ) {
            // bars: (low, range, close_fraction) -> a valid OHLC candle.
            let hlc: Vec<(f64, f64, f64)> = bars
                .iter()
                .map(|&(low, range, frac)| (low + range, low, low + range * frac))
                .collect();
            let candles: Vec<Candle> = hlc.iter().map(|&(h, l, cl)| c(h, l, cl)).collect();
            let mut atr = Atr::new(period).unwrap();
            let got = atr.batch(&candles);
            let want = atr_naive(&hlc, period);
            proptest::prop_assert_eq!(got.len(), want.len());
            for (g, w) in got.iter().zip(want.iter()) {
                match (g, w) {
                    (None, None) => {}
                    (Some(a), Some(b)) => proptest::prop_assert!(
                        (a - b).abs() <= 1e-9 * a.abs().max(1.0),
                        "got={a} want={b}"
                    ),
                    _ => proptest::prop_assert!(false, "warmup mismatch"),
                }
            }
        }
    }
}
