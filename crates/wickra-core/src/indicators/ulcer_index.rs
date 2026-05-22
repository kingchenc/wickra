//! Ulcer Index.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Ulcer Index — Peter Martin's downside-only volatility / risk measure.
///
/// Standard deviation punishes upside and downside moves equally; the Ulcer
/// Index measures only the **pain of drawdowns**. For each bar it computes the
/// percentage drop from the highest price of the trailing window, squares it,
/// and reports the root-mean-square over the window:
///
/// ```text
/// drawdown_t = 100 · (price_t − max(price, period)_t) / max(price, period)_t
/// UlcerIndex = √( mean( drawdown² over period ) )
/// ```
///
/// A pure up-trend never trades below its own running high, so its Ulcer Index
/// is `0`; the deeper and longer the drawdowns, the higher the reading. It is
/// the volatility measure of choice for risk-adjusted return ratios (the
/// "Martin ratio" / UPI).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, UlcerIndex};
///
/// let mut indicator = UlcerIndex::new(14).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.3).sin() * 8.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct UlcerIndex {
    period: usize,
    /// Rolling window of the last `period` prices (for the trailing maximum).
    prices: VecDeque<f64>,
    /// Rolling window of the last `period` squared percentage drawdowns.
    drawdowns_sq: VecDeque<f64>,
    sum_sq: f64,
    last: Option<f64>,
}

impl UlcerIndex {
    /// Construct a new Ulcer Index with the given period.
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
            prices: VecDeque::with_capacity(period),
            drawdowns_sq: VecDeque::with_capacity(period),
            sum_sq: 0.0,
            last: None,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for UlcerIndex {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            // Non-finite input is ignored; state is left untouched.
            return self.last;
        }
        if self.prices.len() == self.period {
            self.prices.pop_front();
        }
        self.prices.push_back(input);
        if self.prices.len() < self.period {
            return None;
        }
        let max_price = self
            .prices
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let drawdown = if max_price == 0.0 {
            0.0
        } else {
            100.0 * (input - max_price) / max_price
        };
        let sq = drawdown * drawdown;

        if self.drawdowns_sq.len() == self.period {
            self.sum_sq -= self.drawdowns_sq.pop_front().expect("window is non-empty");
        }
        self.drawdowns_sq.push_back(sq);
        self.sum_sq += sq;
        if self.drawdowns_sq.len() < self.period {
            return None;
        }
        let ui = (self.sum_sq / self.period as f64).sqrt();
        self.last = Some(ui);
        Some(ui)
    }

    fn reset(&mut self) {
        self.prices.clear();
        self.drawdowns_sq.clear();
        self.sum_sq = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // `period` prices fill the trailing-max window, then `period` squared
        // drawdowns fill the RMS window.
        2 * self.period - 1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "UlcerIndex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(UlcerIndex::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn reference_values() {
        // UlcerIndex(2): warmup = 3.
        // [10, 8, 12, 9]:
        //   bar 3: window [8,12], max 12, drawdown 0; sq window [400, 0]
        //          -> UI = sqrt(200).
        //   bar 4: window [12,9], max 12, drawdown -25, sq 625; sq window [0, 625]
        //          -> UI = sqrt(312.5).
        let mut ui = UlcerIndex::new(2).unwrap();
        let out = ui.batch(&[10.0, 8.0, 12.0, 9.0]);
        assert_eq!(ui.warmup_period(), 3);
        assert_eq!(out[0], None);
        assert_eq!(out[1], None);
        assert_relative_eq!(out[2].unwrap(), 200.0_f64.sqrt(), epsilon = 1e-12);
        assert_relative_eq!(out[3].unwrap(), 312.5_f64.sqrt(), epsilon = 1e-12);
    }

    #[test]
    fn pure_uptrend_yields_zero() {
        // Price never trades below its own running high: no drawdown at all.
        let mut ui = UlcerIndex::new(5).unwrap();
        let out = ui.batch(&(1..=40).map(f64::from).collect::<Vec<_>>());
        for v in out.iter().skip(ui.warmup_period() - 1).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn constant_series_yields_zero() {
        let mut ui = UlcerIndex::new(5).unwrap();
        let out = ui.batch(&[50.0; 30]);
        for v in out.iter().skip(ui.warmup_period() - 1).flatten() {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn output_is_non_negative() {
        let mut ui = UlcerIndex::new(14).unwrap();
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 15.0)
            .collect();
        for v in ui.batch(&prices).into_iter().flatten() {
            assert!(v >= 0.0, "Ulcer Index must be non-negative, got {v}");
        }
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut ui = UlcerIndex::new(2).unwrap();
        let out = ui.batch(&[10.0, 8.0, 12.0, 9.0]);
        let last = *out.last().unwrap();
        assert!(last.is_some());
        assert_eq!(ui.update(f64::NAN), last);
        assert_eq!(ui.update(f64::INFINITY), last);
    }

    #[test]
    fn reset_clears_state() {
        let mut ui = UlcerIndex::new(3).unwrap();
        ui.batch(&[10.0, 8.0, 12.0, 9.0, 11.0, 7.0]);
        assert!(ui.is_ready());
        ui.reset();
        assert!(!ui.is_ready());
        assert_eq!(ui.update(10.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=80)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 10.0)
            .collect();
        let batch = UlcerIndex::new(14).unwrap().batch(&prices);
        let mut b = UlcerIndex::new(14).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
