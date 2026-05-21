//! Average True Range (Wilder).

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Average True Range with Wilder smoothing.
///
/// The first emitted value, by convention, appears after `period` candles: the
/// first `period − 1` true-range values seed the Wilder average alongside the
/// `period`-th, then the smoothed update begins.
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

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(Atr::new(0), Err(Error::PeriodZero)));
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
}
