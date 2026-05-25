//! Rolling Gain/Loss Ratio.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Rolling Gain/Loss Ratio.
///
/// Over the trailing window:
///
/// ```text
/// avg_win  = mean(r for r in window if r > 0)
/// avg_loss = mean(−r for r in window if r < 0)
/// GLR      = avg_win / avg_loss
/// ```
///
/// Where Profit Factor sums gains and losses, the Gain/Loss Ratio averages
/// them: it answers "for the typical winning bar, how big is the win
/// compared to the typical losing bar?". If there are no losers the
/// indicator returns `f64::INFINITY`; if there are no winners and no losers
/// it returns `0.0`.
///
/// Each `update` is O(period).
#[derive(Debug, Clone)]
pub struct GainLossRatio {
    period: usize,
    window: VecDeque<f64>,
}

impl GainLossRatio {
    /// Construct a new rolling Gain/Loss Ratio.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
        })
    }

    /// Configured window length.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for GainLossRatio {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(input);
        if self.window.len() < self.period {
            return None;
        }
        let mut sum_win = 0.0_f64;
        let mut n_win = 0_u32;
        let mut sum_loss = 0.0_f64;
        let mut n_loss = 0_u32;
        for &r in &self.window {
            if r > 0.0 {
                sum_win += r;
                n_win += 1;
            } else if r < 0.0 {
                sum_loss += -r;
                n_loss += 1;
            }
        }
        if n_loss == 0 {
            return Some(if n_win == 0 { 0.0 } else { f64::INFINITY });
        }
        let avg_win = if n_win == 0 {
            0.0
        } else {
            sum_win / f64::from(n_win)
        };
        let avg_loss = sum_loss / f64::from(n_loss);
        Some(avg_win / avg_loss)
    }

    fn reset(&mut self) {
        self.window.clear();
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "GainLossRatio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(GainLossRatio::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let g = GainLossRatio::new(10).unwrap();
        assert_eq!(g.period(), 10);
        assert_eq!(g.name(), "GainLossRatio");
        assert_eq!(g.warmup_period(), 10);
    }

    #[test]
    fn reference_value() {
        // returns = [0.02, -0.01, 0.04, -0.03]
        // avg_win = 0.03, avg_loss = 0.02, GLR = 1.5.
        let mut g = GainLossRatio::new(4).unwrap();
        let out = g.batch(&[0.02, -0.01, 0.04, -0.03]);
        assert_relative_eq!(out[3].unwrap(), 1.5, epsilon = 1e-9);
    }

    #[test]
    fn no_losses_yields_infinity() {
        let mut g = GainLossRatio::new(3).unwrap();
        let out = g.batch(&[0.01, 0.02, 0.03]);
        assert!(out[2].unwrap().is_infinite());
    }

    #[test]
    fn flat_window_yields_zero() {
        let mut g = GainLossRatio::new(3).unwrap();
        let out = g.batch(&[0.0_f64; 3]);
        assert_eq!(out[2], Some(0.0));
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut g = GainLossRatio::new(3).unwrap();
        assert_eq!(g.update(f64::NAN), None);
        assert_eq!(g.update(f64::INFINITY), None);
    }

    #[test]
    fn no_wins_but_losses_yields_zero() {
        // Window with only losses: avg_win is 0, GLR = 0.
        let mut g = GainLossRatio::new(3).unwrap();
        let out = g.batch(&[-0.01, -0.02, -0.03]);
        assert_eq!(out[2], Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut g = GainLossRatio::new(3).unwrap();
        g.batch(&[0.01, -0.02, 0.03]);
        assert!(g.is_ready());
        g.reset();
        assert!(!g.is_ready());
        assert_eq!(g.update(0.01), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let returns: Vec<f64> = (0..40).map(|i| (f64::from(i) * 0.3).sin() * 0.01).collect();
        let batch = GainLossRatio::new(10).unwrap().batch(&returns);
        let mut s = GainLossRatio::new(10).unwrap();
        let streamed: Vec<_> = returns.iter().map(|r| s.update(*r)).collect();
        assert_eq!(batch, streamed);
    }
}
