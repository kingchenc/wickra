//! Money Flow Index (MFI).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Money Flow Index: a volume-weighted version of RSI.
///
/// `MFI = 100 - 100 / (1 + positive_money_flow / negative_money_flow)` where
/// money flow is `typical_price * volume`, classified positive when TP increases
/// and negative when it decreases.
#[derive(Debug, Clone)]
pub struct Mfi {
    period: usize,
    prev_tp: Option<f64>,
    pos_window: VecDeque<f64>,
    neg_window: VecDeque<f64>,
    pos_sum: f64,
    neg_sum: f64,
}

impl Mfi {
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            prev_tp: None,
            pos_window: VecDeque::with_capacity(period),
            neg_window: VecDeque::with_capacity(period),
            pos_sum: 0.0,
            neg_sum: 0.0,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for Mfi {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let tp = candle.typical_price();
        let mf = tp * candle.volume;
        let (pos_flow, neg_flow) = match self.prev_tp {
            None => (0.0, 0.0),
            Some(prev) => {
                if tp > prev {
                    (mf, 0.0)
                } else if tp < prev {
                    (0.0, mf)
                } else {
                    (0.0, 0.0)
                }
            }
        };

        if self.pos_window.len() == self.period {
            self.pos_sum -= self.pos_window.pop_front().expect("non-empty");
            self.neg_sum -= self.neg_window.pop_front().expect("non-empty");
        }
        self.pos_window.push_back(pos_flow);
        self.neg_window.push_back(neg_flow);
        self.pos_sum += pos_flow;
        self.neg_sum += neg_flow;

        self.prev_tp = Some(tp);

        // Need period+1 candles total (the first one only gives prev_tp).
        if self.prev_tp.is_none() || self.pos_window.len() < self.period {
            return None;
        }
        // Need at least one comparison-based flow inside the window, otherwise we
        // are still on the very first candle.
        if self.pos_sum == 0.0 && self.neg_sum == 0.0 {
            return Some(50.0);
        }
        if self.neg_sum == 0.0 {
            return Some(100.0);
        }
        let mr = self.pos_sum / self.neg_sum;
        Some(100.0 - 100.0 / (1.0 + mr))
    }

    fn reset(&mut self) {
        self.prev_tp = None;
        self.pos_window.clear();
        self.neg_window.clear();
        self.pos_sum = 0.0;
        self.neg_sum = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.pos_window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "MFI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(price: f64, volume: f64) -> Candle {
        Candle::new(price, price, price, price, volume, 0).unwrap()
    }

    #[test]
    fn pure_uptrend_yields_high_mfi() {
        let candles: Vec<Candle> = (1..30).map(|i| c(f64::from(i), 100.0)).collect();
        let mut mfi = Mfi::new(14).unwrap();
        let last = mfi.batch(&candles).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 100.0, epsilon = 1e-9);
    }

    #[test]
    fn pure_downtrend_yields_low_mfi() {
        let candles: Vec<Candle> = (1..30).rev().map(|i| c(f64::from(i), 100.0)).collect();
        let mut mfi = Mfi::new(14).unwrap();
        let last = mfi.batch(&candles).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40).map(|i| c(f64::from(i) + 10.0, 50.0)).collect();
        let mut a = Mfi::new(14).unwrap();
        let mut b = Mfi::new(14).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let candles: Vec<Candle> = (1..30).map(|i| c(f64::from(i), 100.0)).collect();
        let mut mfi = Mfi::new(14).unwrap();
        mfi.batch(&candles);
        assert!(mfi.is_ready());
        mfi.reset();
        assert!(!mfi.is_ready());
    }

    #[test]
    fn rejects_zero_period() {
        assert!(Mfi::new(0).is_err());
    }
}
