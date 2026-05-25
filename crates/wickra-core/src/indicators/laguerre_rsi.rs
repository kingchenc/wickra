//! Ehlers' Laguerre RSI.

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// John Ehlers' Laguerre RSI — a four-stage Laguerre polynomial filter wrapped
/// in an `RSI`-style up/down accumulator. The single tuning parameter `gamma`
/// in `[0, 1]` trades lag for smoothness: small `gamma` is fast and noisy,
/// large `gamma` is slow and smooth (Ehlers recommends `0.5`).
///
/// ```text
/// alpha = 1 − gamma
/// L0_t  = alpha · price_t + gamma · L0_{t-1}
/// L1_t  = −gamma · L0_t   + L0_{t-1} + gamma · L1_{t-1}
/// L2_t  = −gamma · L1_t   + L1_{t-1} + gamma · L2_{t-1}
/// L3_t  = −gamma · L2_t   + L2_{t-1} + gamma · L3_{t-1}
///
/// cu, cd = 0
/// for each pair (L0, L1), (L1, L2), (L2, L3):
///     if upper ≥ lower: cu += upper − lower
///     else            : cd += lower − upper
///
/// LRSI = 100 · cu / (cu + cd)
/// ```
///
/// The output is bounded in `[0, 100]`. State is seeded by setting all four
/// `L_i` to the first input, so the first emission lands on input #1.
///
/// Reference: John F. Ehlers, *Time Warp — Without Space Travel*, 2002.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, LaguerreRsi};
///
/// let mut lrsi = LaguerreRsi::new(0.5).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = lrsi.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct LaguerreRsi {
    gamma: f64,
    alpha: f64,
    l0: f64,
    l1: f64,
    l2: f64,
    l3: f64,
    seeded: bool,
    current: Option<f64>,
}

impl LaguerreRsi {
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `gamma` is non-finite or outside `[0, 1]`.
    pub fn new(gamma: f64) -> Result<Self> {
        if !gamma.is_finite() || !(0.0..=1.0).contains(&gamma) {
            return Err(Error::InvalidPeriod {
                message: "LaguerreRSI gamma must be a finite value in [0, 1]",
            });
        }
        Ok(Self {
            gamma,
            alpha: 1.0 - gamma,
            l0: 0.0,
            l1: 0.0,
            l2: 0.0,
            l3: 0.0,
            seeded: false,
            current: None,
        })
    }

    /// Ehlers' recommended `gamma = 0.5`.
    pub fn classic() -> Self {
        Self::new(0.5).expect("classic LaguerreRSI gamma is valid")
    }

    /// Configured `gamma`.
    pub const fn gamma(&self) -> f64 {
        self.gamma
    }
}

impl Indicator for LaguerreRsi {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.current;
        }
        if !self.seeded {
            // Seed all four polynomial stages with the first input so a
            // constant series produces zero up/down accumulators (which we
            // map to 50.0 below — the canonical neutral mid-band reading).
            self.l0 = input;
            self.l1 = input;
            self.l2 = input;
            self.l3 = input;
            self.seeded = true;
            self.current = Some(50.0);
            return self.current;
        }
        let (l0_prev, l1_prev, l2_prev) = (self.l0, self.l1, self.l2);
        let l0_new = self.alpha * input + self.gamma * l0_prev;
        let l1_new = -self.gamma * l0_new + l0_prev + self.gamma * self.l1;
        let l2_new = -self.gamma * l1_new + l1_prev + self.gamma * self.l2;
        let l3_new = -self.gamma * l2_new + l2_prev + self.gamma * self.l3;
        self.l0 = l0_new;
        self.l1 = l1_new;
        self.l2 = l2_new;
        self.l3 = l3_new;

        let mut cu = 0.0;
        let mut cd = 0.0;
        let pairs = [(l0_new, l1_new), (l1_new, l2_new), (l2_new, l3_new)];
        for (upper, lower) in pairs {
            if upper >= lower {
                cu += upper - lower;
            } else {
                cd += lower - upper;
            }
        }
        let total = cu + cd;
        let value = if total > 0.0 {
            // Floating-point rounding can push `cu / total` a hair above 1.0;
            // clamp to the algebraic bound to keep the output strictly inside
            // [0, 100].
            (100.0 * cu / total).clamp(0.0, 100.0)
        } else {
            // No up- or down-displacements between stages: stay at the
            // neutral mid-band rather than report 0 / 0.
            50.0
        };
        self.current = Some(value);
        Some(value)
    }

    fn reset(&mut self) {
        self.l0 = 0.0;
        self.l1 = 0.0;
        self.l2 = 0.0;
        self.l3 = 0.0;
        self.seeded = false;
        self.current = None;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.current.is_some()
    }

    fn name(&self) -> &'static str {
        "LaguerreRSI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_invalid_gamma() {
        assert!(matches!(
            LaguerreRsi::new(-0.1),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            LaguerreRsi::new(1.1),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            LaguerreRsi::new(f64::NAN),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let lrsi = LaguerreRsi::new(0.5).unwrap();
        assert_eq!(lrsi.gamma(), 0.5);
        assert_eq!(lrsi.warmup_period(), 1);
        assert_eq!(lrsi.name(), "LaguerreRSI");
    }

    #[test]
    fn classic_factory() {
        assert_eq!(LaguerreRsi::classic().gamma(), 0.5);
    }

    #[test]
    fn constant_series_stays_at_mid_band() {
        // All four L_i seed to the constant; on subsequent flat inputs they
        // stay equal, so cu = cd = 0 and LRSI reports the neutral 50.
        let mut lrsi = LaguerreRsi::classic();
        let out = lrsi.batch(&[42.0_f64; 60]);
        for v in out.iter().flatten() {
            assert_relative_eq!(*v, 50.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn output_is_bounded() {
        let mut lrsi = LaguerreRsi::classic();
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 25.0)
            .collect();
        for v in lrsi.batch(&prices).iter().flatten() {
            assert!(*v >= 0.0 && *v <= 100.0, "out of range: {v}");
        }
    }

    #[test]
    fn pure_uptrend_saturates_high() {
        let mut lrsi = LaguerreRsi::classic();
        for i in 0..200 {
            lrsi.update(100.0 + f64::from(i));
        }
        let v = lrsi.current.unwrap();
        assert!(v > 80.0, "uptrend should drive LRSI well above 50: {v}");
    }

    #[test]
    fn pure_downtrend_saturates_low() {
        let mut lrsi = LaguerreRsi::classic();
        for i in 0..200 {
            lrsi.update(300.0 - f64::from(i));
        }
        let v = lrsi.current.unwrap();
        assert!(v < 20.0, "downtrend should drive LRSI well below 50: {v}");
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 5.0)
            .collect();
        let mut a = LaguerreRsi::classic();
        let mut b = LaguerreRsi::classic();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut lrsi = LaguerreRsi::classic();
        lrsi.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(lrsi.is_ready());
        lrsi.reset();
        assert!(!lrsi.is_ready());
        assert!(!lrsi.seeded);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut lrsi = LaguerreRsi::classic();
        let before = lrsi.update(10.0).unwrap();
        assert_eq!(lrsi.update(f64::NAN), Some(before));
        assert_eq!(lrsi.update(f64::INFINITY), Some(before));
    }

    #[test]
    fn gamma_zero_passes_through_l0() {
        // gamma = 0 -> alpha = 1, so L0 mirrors the input exactly each step.
        // The polynomial chain then lags by one stage; the up/down accumulator
        // still produces a bounded reading and the first non-seed step shifts
        // off 50 as soon as input changes.
        let mut lrsi = LaguerreRsi::new(0.0).unwrap();
        assert_eq!(lrsi.update(10.0), Some(50.0));
        let v = lrsi.update(11.0).unwrap();
        assert!((0.0..=100.0).contains(&v));
    }
}
