//! Ehlers Empirical Mode Decomposition (bandpass + envelope).

use std::collections::VecDeque;
use std::f64::consts::PI;

use crate::error::{Error, Result};
use crate::indicators::super_smoother::SuperSmoother;
use crate::traits::Indicator;

/// Ehlers' adaptation of Empirical Mode Decomposition (EMD).
///
/// Implementation per *Cycle Analytics for Traders* (Ehlers 2013, ch. 14).
/// The procedure is:
///
/// 1. Apply a bandpass filter centred on `period` to the price.
/// 2. Detect peaks and valleys of the bandpassed signal over a `fraction`
///    of the period.
/// 3. Average the peaks and valleys separately to form an upper / lower
///    envelope, then return the centred bandpass minus the envelope mean
///    (the "EMD" line).
///
/// The output crosses zero at trend changes and stays near zero in
/// non-trending markets — the classic visual cue Ehlers documents.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, EmpiricalModeDecomposition};
///
/// let mut emd = EmpiricalModeDecomposition::new(20, 0.5).unwrap();
/// let mut last = None;
/// for i in 0..200 {
///     last = emd.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct EmpiricalModeDecomposition {
    period: usize,
    fraction: f64,
    bandpass: f64,
    prev_bp_1: f64,
    prev_bp_2: f64,
    prev_in_1: Option<f64>,
    prev_in_2: Option<f64>,
    beta: f64,
    alpha: f64,
    smoother: SuperSmoother,
    peak_smoother: SuperSmoother,
    valley_smoother: SuperSmoother,
    bp_buf: VecDeque<f64>,
    bp_history_len: usize,
    last_value: Option<f64>,
}

impl EmpiricalModeDecomposition {
    /// Construct with the bandpass centre period and the peak-detection
    /// window fraction.
    ///
    /// `fraction` is multiplied by `period` to size the rolling peak/valley
    /// window; Ehlers recommends `0.5`. Both must be positive.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`, and
    /// [`Error::InvalidPeriod`] if `fraction <= 0` or non-finite.
    pub fn new(period: usize, fraction: f64) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        if !fraction.is_finite() || fraction <= 0.0 || fraction > 1.0 {
            return Err(Error::InvalidPeriod {
                message: "fraction must be in (0, 1]",
            });
        }
        let beta = (2.0 * PI / period as f64).cos();
        let gamma = 1.0 / (2.0 * PI * 0.25 / period as f64).cos();
        let alpha = gamma - (gamma * gamma - 1.0).sqrt();
        let history = (period as f64 * fraction).round().max(1.0) as usize;
        Ok(Self {
            period,
            fraction,
            bandpass: 0.0,
            prev_bp_1: 0.0,
            prev_bp_2: 0.0,
            prev_in_1: None,
            prev_in_2: None,
            beta,
            alpha,
            smoother: SuperSmoother::new(period.max(2))?,
            peak_smoother: SuperSmoother::new(period.max(2))?,
            valley_smoother: SuperSmoother::new(period.max(2))?,
            bp_buf: VecDeque::with_capacity(history),
            bp_history_len: history,
            last_value: None,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Configured fraction.
    pub const fn fraction(&self) -> f64 {
        self.fraction
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for EmpiricalModeDecomposition {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.last_value;
        }
        // 2nd-order resonant bandpass per Ehlers ch. 6.
        let bp = if let (Some(_x1), Some(x2)) = (self.prev_in_1, self.prev_in_2) {
            0.5 * (1.0 - self.alpha) * (input - x2)
                + self.beta * (1.0 + self.alpha) * self.prev_bp_1
                - self.alpha * self.prev_bp_2
        } else {
            0.0
        };
        self.prev_bp_2 = self.prev_bp_1;
        self.prev_bp_1 = bp;
        self.bandpass = bp;
        self.prev_in_2 = self.prev_in_1;
        self.prev_in_1 = Some(input);

        if self.bp_buf.len() == self.bp_history_len {
            self.bp_buf.pop_front();
        }
        self.bp_buf.push_back(bp);
        if self.bp_buf.len() < self.bp_history_len {
            return None;
        }

        // Identify the current peak (largest), valley (smallest) within the window.
        let peak = self
            .bp_buf
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let valley = self.bp_buf.iter().copied().fold(f64::INFINITY, f64::min);

        let avg_peak = self.peak_smoother.update(peak)?;
        let avg_valley = self.valley_smoother.update(valley)?;

        // The EMD line is the bandpass minus the smoothed mean envelope.
        let mean = 0.5 * (avg_peak + avg_valley);
        let raw = bp - mean;
        let v = self.smoother.update(raw)?;
        self.last_value = Some(v);
        Some(v)
    }

    fn reset(&mut self) {
        self.bandpass = 0.0;
        self.prev_bp_1 = 0.0;
        self.prev_bp_2 = 0.0;
        self.prev_in_1 = None;
        self.prev_in_2 = None;
        self.smoother.reset();
        self.peak_smoother.reset();
        self.valley_smoother.reset();
        self.bp_buf.clear();
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        self.bp_history_len
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "EmpiricalModeDecomposition"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    #[test]
    fn new_rejects_invalid_params() {
        assert!(matches!(
            EmpiricalModeDecomposition::new(0, 0.5),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            EmpiricalModeDecomposition::new(20, 0.0),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            EmpiricalModeDecomposition::new(20, 1.5),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            EmpiricalModeDecomposition::new(20, f64::NAN),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let mut emd = EmpiricalModeDecomposition::new(20, 0.5).unwrap();
        assert_eq!(emd.period(), 20);
        assert!((emd.fraction() - 0.5).abs() < 1e-15);
        assert_eq!(emd.name(), "EmpiricalModeDecomposition");
        assert!(emd.warmup_period() >= 1);
        assert!(!emd.is_ready());
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        emd.batch(&prices);
        assert!(emd.is_ready());
        assert!(emd.value().is_some());
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.2).cos() * 5.0)
            .collect();
        let mut a = EmpiricalModeDecomposition::new(20, 0.5).unwrap();
        let mut b = EmpiricalModeDecomposition::new(20, 0.5).unwrap();
        let batch = a.batch(&prices);
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut emd = EmpiricalModeDecomposition::new(20, 0.5).unwrap();
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        emd.batch(&prices);
        let before = emd.value();
        assert!(before.is_some());
        assert_eq!(emd.update(f64::NAN), before);
    }

    #[test]
    fn reset_clears_state() {
        let mut emd = EmpiricalModeDecomposition::new(20, 0.5).unwrap();
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        emd.batch(&prices);
        assert!(emd.is_ready());
        emd.reset();
        assert!(!emd.is_ready());
    }
}
