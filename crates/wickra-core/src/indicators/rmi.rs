//! Relative Momentum Index (RMI).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Relative Momentum Index — RSI generalised to a multi-bar momentum lookback.
///
/// Wilder's [`Rsi`](crate::Rsi) compares each close to the *previous* close.
/// The RMI (Roger Altman, 1993) compares it to the close `momentum` bars ago,
/// then applies the same Wilder-smoothed up/down accumulator over `period`:
///
/// ```text
/// change_t = close_t - close_{t-momentum}
/// gain     = max(change, 0),  loss = max(-change, 0)
/// avg_gain, avg_loss = Wilder-smoothed over `period`
/// RMI      = 100 * avg_gain / (avg_gain + avg_loss)
/// ```
///
/// `momentum = 1` reduces the RMI exactly to the RSI. Larger `momentum` makes
/// the oscillator smoother and slower to flip, holding overbought/oversold
/// readings longer in a trend. Output is bounded in `[0, 100]`; a flat market
/// (no gains and no losses) returns the neutral `50`.
///
/// The first value lands after `momentum + period` inputs: `momentum` to fill
/// the lookback, then `period` changes to seed Wilder's averages.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Rmi};
///
/// let mut indicator = Rmi::new(14, 5).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.2).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Rmi {
    period: usize,
    momentum: usize,
    /// The last `momentum` prices, oldest at the front.
    window: VecDeque<f64>,
    seed_gains: Vec<f64>,
    seed_losses: Vec<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
    last_value: Option<f64>,
}

impl Rmi {
    /// Construct an RMI with the given smoothing `period` and `momentum`
    /// lookback.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if either `period` or `momentum` is `0`.
    pub fn new(period: usize, momentum: usize) -> Result<Self> {
        if period == 0 || momentum == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            momentum,
            window: VecDeque::with_capacity(momentum),
            seed_gains: Vec::with_capacity(period),
            seed_losses: Vec::with_capacity(period),
            avg_gain: None,
            avg_loss: None,
            last_value: None,
        })
    }

    /// Configured smoothing period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Configured momentum lookback.
    pub const fn momentum(&self) -> usize {
        self.momentum
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }

    fn rmi_from_avgs(avg_gain: f64, avg_loss: f64) -> f64 {
        let denom = avg_gain + avg_loss;
        if denom == 0.0 {
            50.0
        } else {
            // Ratio first, then scale, so `100 * g / g` cannot round above 100.
            100.0 * (avg_gain / denom)
        }
    }
}

impl Indicator for Rmi {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.last_value;
        }
        if self.window.len() < self.momentum {
            // Still filling the momentum lookback; no change to measure yet.
            self.window.push_back(input);
            return None;
        }
        let past = self.window.pop_front().expect("window full");
        self.window.push_back(input);

        let change = input - past;
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { -change } else { 0.0 };

        if let (Some(ag), Some(al)) = (self.avg_gain, self.avg_loss) {
            let n = self.period as f64;
            let new_ag = (ag * (n - 1.0) + gain) / n;
            let new_al = (al * (n - 1.0) + loss) / n;
            self.avg_gain = Some(new_ag);
            self.avg_loss = Some(new_al);
            let v = Self::rmi_from_avgs(new_ag, new_al);
            self.last_value = Some(v);
            return Some(v);
        }

        self.seed_gains.push(gain);
        self.seed_losses.push(loss);
        if self.seed_gains.len() == self.period {
            let ag = self.seed_gains.iter().sum::<f64>() / self.period as f64;
            let al = self.seed_losses.iter().sum::<f64>() / self.period as f64;
            self.avg_gain = Some(ag);
            self.avg_loss = Some(al);
            let v = Self::rmi_from_avgs(ag, al);
            self.last_value = Some(v);
            return Some(v);
        }
        None
    }

    fn reset(&mut self) {
        self.window.clear();
        self.seed_gains.clear();
        self.seed_losses.clear();
        self.avg_gain = None;
        self.avg_loss = None;
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        self.momentum + self.period
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "RMI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::Rsi;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_params() {
        assert!(matches!(Rmi::new(0, 5), Err(Error::PeriodZero)));
        assert!(matches!(Rmi::new(14, 0), Err(Error::PeriodZero)));
    }

    /// Cover the const accessors `period` + `momentum` + `value` and the
    /// Indicator-impl `warmup_period` + `name`.
    #[test]
    fn accessors_and_metadata() {
        let rmi = Rmi::new(14, 5).unwrap();
        assert_eq!(rmi.period(), 14);
        assert_eq!(rmi.momentum(), 5);
        assert_eq!(rmi.value(), None);
        assert_eq!(rmi.warmup_period(), 19);
        assert_eq!(rmi.name(), "RMI");
    }

    #[test]
    fn momentum_one_equals_rsi() {
        // With momentum = 1 the RMI is exactly Wilder's RSI.
        let prices: Vec<f64> = (0..60)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 8.0)
            .collect();
        let mut rmi = Rmi::new(14, 1).unwrap();
        let mut rsi = Rsi::new(14).unwrap();
        for (i, &p) in prices.iter().enumerate() {
            let got = rmi.update(p);
            let want = rsi.update(p);
            assert_eq!(got.is_some(), want.is_some(), "readiness mismatch at {i}");
            if let (Some(a), Some(b)) = (got, want) {
                assert_relative_eq!(a, b, epsilon = 1e-9);
            }
        }
    }

    #[test]
    fn warmup_then_emits() {
        // momentum + period = 3 + 2 = 5 inputs before the first value.
        let mut rmi = Rmi::new(2, 3).unwrap();
        let out = rmi.batch(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        for (i, v) in out.iter().enumerate().take(4) {
            assert!(v.is_none(), "index {i} must be None during warmup");
        }
        assert!(out[4].is_some(), "first value at warmup_period - 1");
    }

    #[test]
    fn pure_uptrend_is_one_hundred() {
        // Every momentum-spaced change is positive -> avg_loss 0 -> RMI 100.
        let prices: Vec<f64> = (1..=40).map(f64::from).collect();
        let mut rmi = Rmi::new(5, 3).unwrap();
        let last = rmi.batch(&prices).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 100.0, epsilon = 1e-9);
    }

    #[test]
    fn flat_market_is_neutral() {
        // No change -> no gains and no losses -> neutral 50.
        let mut rmi = Rmi::new(3, 2).unwrap();
        let last = rmi.batch(&[7.0; 20]).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 50.0, epsilon = 1e-12);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut rmi = Rmi::new(2, 2).unwrap();
        let ready = rmi
            .batch(&[1.0, 2.0, 3.0, 4.0, 5.0])
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_eq!(rmi.update(f64::NAN), Some(ready));
        assert_eq!(rmi.update(f64::INFINITY), Some(ready));
    }

    #[test]
    fn reset_clears_state() {
        let mut rmi = Rmi::new(3, 2).unwrap();
        rmi.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        assert!(rmi.is_ready());
        rmi.reset();
        assert!(!rmi.is_ready());
        assert_eq!(rmi.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=40)
            .map(|i| 50.0 + (f64::from(i) * 0.5).sin() * 10.0)
            .collect();
        let mut a = Rmi::new(14, 5).unwrap();
        let mut b = Rmi::new(14, 5).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }
}
