//! Volume-Weighted Average Price (VWAP).
//!
//! Two variants are offered: a cumulative `Vwap` that runs forever (the
//! intraday convention), and a rolling-window `RollingVwap` for streaming bots
//! that need a finite-memory price benchmark.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Cumulative session VWAP. Call [`Indicator::reset`] at the start of each
/// session (e.g. trading-day boundary) to restart the accumulation.
#[derive(Debug, Clone, Default)]
pub struct Vwap {
    sum_pv: f64,
    sum_v: f64,
    has_emitted: bool,
}

impl Vwap {
    /// Construct a fresh cumulative VWAP.
    pub const fn new() -> Self {
        Self {
            sum_pv: 0.0,
            sum_v: 0.0,
            has_emitted: false,
        }
    }

    /// Current VWAP if at least one candle with non-zero volume has been observed.
    pub fn value(&self) -> Option<f64> {
        if self.sum_v == 0.0 {
            None
        } else {
            Some(self.sum_pv / self.sum_v)
        }
    }
}

impl Indicator for Vwap {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let tp = candle.typical_price();
        self.sum_pv += tp * candle.volume;
        self.sum_v += candle.volume;
        if self.sum_v == 0.0 {
            return None;
        }
        self.has_emitted = true;
        Some(self.sum_pv / self.sum_v)
    }

    fn reset(&mut self) {
        self.sum_pv = 0.0;
        self.sum_v = 0.0;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "VWAP"
    }
}

/// Rolling-window VWAP: a finite-memory variant for bots that don't want
/// unbounded accumulation.
#[derive(Debug, Clone)]
pub struct RollingVwap {
    period: usize,
    window: VecDeque<(f64, f64)>, // (typical_price * volume, volume)
    sum_pv: f64,
    sum_v: f64,
}

impl RollingVwap {
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
            sum_pv: 0.0,
            sum_v: 0.0,
        })
    }

    /// Configured rolling window length.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for RollingVwap {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let pv = candle.typical_price() * candle.volume;
        if self.window.len() == self.period {
            let (old_pv, old_v) = self.window.pop_front().expect("non-empty");
            self.sum_pv -= old_pv;
            self.sum_v -= old_v;
        }
        self.window.push_back((pv, candle.volume));
        self.sum_pv += pv;
        self.sum_v += candle.volume;
        if self.window.len() < self.period || self.sum_v == 0.0 {
            return None;
        }
        Some(self.sum_pv / self.sum_v)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum_pv = 0.0;
        self.sum_v = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period && self.sum_v > 0.0
    }

    fn name(&self) -> &'static str {
        "RollingVWAP"
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
    fn cumulative_vwap_equal_volumes_equals_mean() {
        let candles = vec![c(10.0, 1.0), c(20.0, 1.0), c(30.0, 1.0)];
        let mut v = Vwap::new();
        let out = v.batch(&candles);
        assert_relative_eq!(out[2].unwrap(), 20.0, epsilon = 1e-12);
    }

    #[test]
    fn cumulative_vwap_weighted() {
        // Two candles: 10@1 and 20@3 -> (10*1 + 20*3) / (1+3) = 70/4 = 17.5
        let candles = vec![c(10.0, 1.0), c(20.0, 3.0)];
        let mut v = Vwap::new();
        let out = v.batch(&candles);
        assert_relative_eq!(out[1].unwrap(), 17.5, epsilon = 1e-12);
    }

    #[test]
    fn rolling_vwap_window_slides() {
        let candles = vec![c(10.0, 1.0), c(20.0, 1.0), c(30.0, 1.0), c(40.0, 1.0)];
        let mut v = RollingVwap::new(3).unwrap();
        let out = v.batch(&candles);
        assert!(out[1].is_none());
        // index 2 -> (10+20+30)/3 = 20
        assert_relative_eq!(out[2].unwrap(), 20.0, epsilon = 1e-12);
        // index 3 -> (20+30+40)/3 = 30
        assert_relative_eq!(out[3].unwrap(), 30.0, epsilon = 1e-12);
    }

    #[test]
    fn batch_equals_streaming_cumulative() {
        let candles: Vec<Candle> = (1..20).map(|i| c(f64::from(i), 1.0)).collect();
        let mut a = Vwap::new();
        let mut b = Vwap::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn batch_equals_streaming_rolling() {
        let candles: Vec<Candle> = (1..30)
            .map(|i| c(f64::from(i), f64::from(i % 5 + 1)))
            .collect();
        let mut a = RollingVwap::new(10).unwrap();
        let mut b = RollingVwap::new(10).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn rolling_rejects_zero_period() {
        assert!(RollingVwap::new(0).is_err());
    }
}
