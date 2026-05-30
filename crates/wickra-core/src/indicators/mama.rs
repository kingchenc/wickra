//! Ehlers MESA Adaptive Moving Average (MAMA) and its follower (FAMA).
#![allow(
    clippy::doc_markdown,
    clippy::doc_lazy_continuation,
    clippy::struct_field_names,
    clippy::manual_clamp
)]

use std::f64::consts::PI;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// MAMA + FAMA output pair.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MamaOutput {
    /// MESA Adaptive Moving Average.
    pub mama: f64,
    /// Following Adaptive Moving Average (slower companion).
    pub fama: f64,
}

/// Ehlers' MESA Adaptive Moving Average (MAMA).
///
/// MAMA adapts its smoothing constant from the rate-of-change of price phase,
/// derived via a truncated Hilbert transform — full math in "Cycle Analytics
/// for Traders" (Ehlers 2013, ch. 8) and the original 2001 MESA paper.
///
/// The two-parameter `(fast_limit, slow_limit)` is the range over which the
/// adaptive alpha can vary; defaults `(0.5, 0.05)` match the canonical
/// EasyLanguage implementation. The companion FAMA is `mama * 0.5 * fast_limit
/// + fama_prev * (1 - 0.5 * fast_limit)`, lagging MAMA so crossovers signal
/// trend reversals.
///
/// The indicator emits both lines as a [`MamaOutput`]. Use the [`crate::Fama`] wrapper
/// in this module to expose just the slow line if needed (e.g. for chaining).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Mama};
///
/// let mut mama = Mama::new(0.5, 0.05).unwrap();
/// let mut last = None;
/// for i in 0..100 {
///     last = mama.update(100.0 + (f64::from(i) * 0.2).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Mama {
    fast_limit: f64,
    slow_limit: f64,
    smooth_buf: Vec<f64>,
    detrender_buf: Vec<f64>,
    q1_buf: Vec<f64>,
    i1_buf: Vec<f64>,
    prev_i2: f64,
    prev_q2: f64,
    prev_re: f64,
    prev_im: f64,
    prev_period: f64,
    prev_phase: f64,
    prev_mama: f64,
    prev_fama: f64,
    count: usize,
    last_value: Option<MamaOutput>,
}

impl Mama {
    /// Construct with custom `(fast_limit, slow_limit)` adaptive alpha bounds.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPeriod`] if either limit is outside `(0, 1]`
    /// or if `slow_limit > fast_limit`.
    pub fn new(fast_limit: f64, slow_limit: f64) -> Result<Self> {
        if !fast_limit.is_finite()
            || !slow_limit.is_finite()
            || fast_limit <= 0.0
            || fast_limit > 1.0
            || slow_limit <= 0.0
            || slow_limit > 1.0
            || slow_limit > fast_limit
        {
            return Err(Error::InvalidPeriod {
                message: "fast_limit, slow_limit must satisfy 0 < slow_limit <= fast_limit <= 1",
            });
        }
        Ok(Self {
            fast_limit,
            slow_limit,
            smooth_buf: Vec::with_capacity(7),
            detrender_buf: Vec::with_capacity(7),
            q1_buf: Vec::with_capacity(7),
            i1_buf: Vec::with_capacity(7),
            prev_i2: 0.0,
            prev_q2: 0.0,
            prev_re: 0.0,
            prev_im: 0.0,
            prev_period: 0.0,
            prev_phase: 0.0,
            prev_mama: 0.0,
            prev_fama: 0.0,
            count: 0,
            last_value: None,
        })
    }

    /// Default `(0.5, 0.05)` parameters from Ehlers' original publication.
    pub fn classic() -> Self {
        Self::new(0.5, 0.05).expect("classic MAMA limits are valid")
    }

    /// Configured `(fast_limit, slow_limit)`.
    pub const fn limits(&self) -> (f64, f64) {
        (self.fast_limit, self.slow_limit)
    }

    /// Current `(mama, fama)` pair if available.
    pub const fn value(&self) -> Option<MamaOutput> {
        self.last_value
    }

    fn push_front(buf: &mut Vec<f64>, v: f64, cap: usize) {
        buf.insert(0, v);
        if buf.len() > cap {
            buf.truncate(cap);
        }
    }
}

impl Indicator for Mama {
    type Input = f64;
    type Output = MamaOutput;

    fn update(&mut self, input: f64) -> Option<MamaOutput> {
        if !input.is_finite() {
            return self.last_value;
        }
        self.count += 1;

        Self::push_front(&mut self.smooth_buf, input, 7);
        if self.smooth_buf.len() < 4 {
            return None;
        }
        let smooth = (4.0 * self.smooth_buf[0]
            + 3.0 * self.smooth_buf[1]
            + 2.0 * self.smooth_buf[2]
            + self.smooth_buf[3])
            / 10.0;

        let period = self.prev_period.max(6.0).min(50.0);
        let adj = 0.075 * period + 0.54;

        if self.smooth_buf.len() < 7 {
            // Seed the EMA outputs with the smoothed price so early bars are
            // well-behaved without producing a public value.
            self.prev_mama = smooth;
            self.prev_fama = smooth;
            return None;
        }
        let s0 = smooth;
        let s2 = self.smooth_buf[2];
        let s4 = self.smooth_buf[4];
        let s6 = self.smooth_buf[6];
        let detrender = (0.0962 * s0 + 0.5769 * s2 - 0.5769 * s4 - 0.0962 * s6) * adj;
        Self::push_front(&mut self.detrender_buf, detrender, 7);
        if self.detrender_buf.len() < 7 {
            return None;
        }

        let q1 = (0.0962 * self.detrender_buf[0] + 0.5769 * self.detrender_buf[2]
            - 0.5769 * self.detrender_buf[4]
            - 0.0962 * self.detrender_buf[6])
            * adj;
        let i1 = self.detrender_buf[3];
        Self::push_front(&mut self.q1_buf, q1, 7);
        Self::push_front(&mut self.i1_buf, i1, 7);
        if self.q1_buf.len() < 7 || self.i1_buf.len() < 7 {
            return None;
        }

        let ji = (0.0962 * self.i1_buf[0] + 0.5769 * self.i1_buf[2]
            - 0.5769 * self.i1_buf[4]
            - 0.0962 * self.i1_buf[6])
            * adj;
        let jq = (0.0962 * self.q1_buf[0] + 0.5769 * self.q1_buf[2]
            - 0.5769 * self.q1_buf[4]
            - 0.0962 * self.q1_buf[6])
            * adj;

        let mut i2 = i1 - jq;
        let mut q2 = q1 + ji;
        i2 = 0.2 * i2 + 0.8 * self.prev_i2;
        q2 = 0.2 * q2 + 0.8 * self.prev_q2;

        let mut re = i2 * self.prev_i2 + q2 * self.prev_q2;
        let mut im = i2 * self.prev_q2 - q2 * self.prev_i2;
        re = 0.2 * re + 0.8 * self.prev_re;
        im = 0.2 * im + 0.8 * self.prev_im;

        self.prev_i2 = i2;
        self.prev_q2 = q2;
        self.prev_re = re;
        self.prev_im = im;

        let mut new_period = if im.abs() > f64::EPSILON && re.abs() > f64::EPSILON {
            2.0 * PI / im.atan2(re)
        } else {
            self.prev_period
        };
        new_period = new_period.min(1.5 * self.prev_period);
        new_period = new_period.max(0.67 * self.prev_period);
        new_period = new_period.clamp(6.0, 50.0);
        self.prev_period = 0.2 * new_period + 0.8 * self.prev_period;

        // Adaptive alpha derived from phase rate-of-change.
        let phase = if i1.abs() > f64::EPSILON {
            (q1 / i1).atan().to_degrees()
        } else {
            self.prev_phase
        };
        let mut delta_phase = self.prev_phase - phase;
        self.prev_phase = phase;
        if delta_phase < 1.0 {
            delta_phase = 1.0;
        }
        // `delta_phase` is clamped to >= 1.0 above, so `fast_limit / delta_phase`
        // never exceeds `fast_limit`; only the lower bound can bind.
        let mut alpha = self.fast_limit / delta_phase;
        if alpha < self.slow_limit {
            alpha = self.slow_limit;
        }

        self.prev_mama = alpha * input + (1.0 - alpha) * self.prev_mama;
        let fama_alpha = 0.5 * alpha;
        self.prev_fama = fama_alpha * self.prev_mama + (1.0 - fama_alpha) * self.prev_fama;

        if self.count < 33 {
            return None;
        }
        let out = MamaOutput {
            mama: self.prev_mama,
            fama: self.prev_fama,
        };
        self.last_value = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.smooth_buf.clear();
        self.detrender_buf.clear();
        self.q1_buf.clear();
        self.i1_buf.clear();
        self.prev_i2 = 0.0;
        self.prev_q2 = 0.0;
        self.prev_re = 0.0;
        self.prev_im = 0.0;
        self.prev_period = 0.0;
        self.prev_phase = 0.0;
        self.prev_mama = 0.0;
        self.prev_fama = 0.0;
        self.count = 0;
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        33
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "MAMA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    #[test]
    fn rejects_invalid_limits() {
        assert!(matches!(
            Mama::new(0.0, 0.05),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            Mama::new(0.5, 0.0),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            Mama::new(0.05, 0.5),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            Mama::new(1.5, 0.05),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            Mama::new(f64::NAN, 0.05),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let mut mama = Mama::classic();
        assert_eq!(mama.limits(), (0.5, 0.05));
        assert_eq!(mama.warmup_period(), 33);
        assert_eq!(mama.name(), "MAMA");
        assert!(!mama.is_ready());
        for i in 0..60 {
            mama.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
        }
        assert!(mama.is_ready());
        assert!(mama.value().is_some());
    }

    #[test]
    fn fama_lags_or_equals_mama_on_constant_series() {
        let mut mama = Mama::classic();
        let out = mama.batch(&[100.0_f64; 200]);
        let last = out.iter().flatten().last().unwrap();
        // On a flat series both lines converge to the price.
        assert!((last.mama - 100.0).abs() < 1.0);
        assert!((last.fama - 100.0).abs() < 1.0);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 5.0)
            .collect();
        let mut a = Mama::classic();
        let mut b = Mama::classic();
        let batch = a.batch(&prices);
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut mama = Mama::classic();
        let prices: Vec<f64> = (0..100)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        mama.batch(&prices);
        let before = mama.value();
        assert!(before.is_some());
        assert_eq!(mama.update(f64::NAN), before);
    }

    #[test]
    fn reset_clears_state() {
        let mut mama = Mama::classic();
        let prices: Vec<f64> = (0..100)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        mama.batch(&prices);
        assert!(mama.is_ready());
        mama.reset();
        assert!(!mama.is_ready());
    }

    #[test]
    fn flat_input_uses_phase_fallback() {
        // Zero inputs make every smooth/detrender term arithmetically exact
        // zero, so `i1 == 0.0` and the phase calculation takes the
        // `self.prev_phase` fallback rather than `atan(q1/i1)`. A non-zero
        // constant like `50.0` leaves a sub-EPSILON cancellation residue
        // that flips the branch back to the `atan` path on real hardware.
        let mut mama = Mama::classic();
        let out = mama.batch(&[0.0_f64; 200]);
        assert!(out.iter().flatten().count() > 100);
    }
}
