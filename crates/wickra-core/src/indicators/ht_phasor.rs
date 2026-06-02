//! Ehlers Hilbert Transform Phasor components (`HT_PHASOR`).
#![allow(clippy::manual_clamp)]

use std::f64::consts::PI;

use crate::traits::Indicator;

/// In-phase and quadrature components of the Hilbert transform phasor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HtPhasorOutput {
    /// In-phase component (`I1`).
    pub inphase: f64,
    /// Quadrature component (`Q1`).
    pub quadrature: f64,
}

/// Ehlers' Hilbert Transform Phasor (`HT_PHASOR`).
///
/// Runs the same adaptive Hilbert-transform engine as
/// [`HilbertDominantCycle`](crate::HilbertDominantCycle) but reports the raw
/// in-phase (`I1`) and quadrature (`Q1`) components of the analytic signal rather
/// than the recovered cycle period. The two components are 90° out of phase, so
/// their ratio tracks the instantaneous phase of the dominant cycle.
///
/// From *Rocket Science for Traders* (Ehlers 2001), aligned with TA-Lib's
/// `HT_PHASOR`. The first value is emitted once the transform's tap buffers fill.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, HtPhasor};
///
/// let mut ht = HtPhasor::new();
/// let mut last = None;
/// for i in 0..120 {
///     last = ht.update(100.0 + (f64::from(i) * 0.4).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct HtPhasor {
    smooth_buf: Vec<f64>,
    detrender_buf: Vec<f64>,
    q1_buf: Vec<f64>,
    i1_buf: Vec<f64>,
    prev_i2: f64,
    prev_q2: f64,
    prev_re: f64,
    prev_im: f64,
    prev_period: f64,
    ready: bool,
}

impl HtPhasor {
    /// Construct a new Hilbert transform phasor.
    pub fn new() -> Self {
        Self::default()
    }

    fn push_front(buf: &mut Vec<f64>, v: f64, cap: usize) {
        buf.insert(0, v);
        if buf.len() > cap {
            buf.truncate(cap);
        }
    }
}

impl Indicator for HtPhasor {
    type Input = f64;
    type Output = HtPhasorOutput;

    fn update(&mut self, input: f64) -> Option<HtPhasorOutput> {
        if !input.is_finite() {
            return None;
        }

        Self::push_front(&mut self.smooth_buf, input, 7);
        if self.smooth_buf.len() < 7 {
            return None;
        }
        let smooth = (4.0 * self.smooth_buf[0]
            + 3.0 * self.smooth_buf[1]
            + 2.0 * self.smooth_buf[2]
            + self.smooth_buf[3])
            / 10.0;

        let period = self.prev_period.max(6.0).min(50.0);
        let adj = 0.075 * period + 0.54;

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

        // Continue the dominant-cycle period adaptation so the next bar's `adj`
        // coefficient tracks the cycle, exactly as TA-Lib's HT_PHASOR does.
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

        self.ready = true;
        Some(HtPhasorOutput {
            inphase: i1,
            quadrature: q1,
        })
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
        self.ready = false;
    }

    fn warmup_period(&self) -> usize {
        19
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn name(&self) -> &'static str {
        "HT_PHASOR"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn sine_prices(n: usize) -> Vec<f64> {
        (0..n)
            .map(|i| 100.0 + (i as f64 * 0.4).sin() * 5.0)
            .collect()
    }

    #[test]
    fn accessors_and_metadata() {
        let ht = HtPhasor::new();
        assert_eq!(ht.warmup_period(), 19);
        assert_eq!(ht.name(), "HT_PHASOR");
        assert!(!ht.is_ready());
    }

    #[test]
    fn emits_after_warmup_and_stays_finite() {
        let mut ht = HtPhasor::new();
        let out: Vec<Option<HtPhasorOutput>> = ht.batch(&sine_prices(120));
        assert_eq!(out[0], None);
        let first = out.iter().position(Option::is_some).expect("emits");
        assert!(first <= 19, "first phasor at index {first}");
        for o in out.into_iter().flatten() {
            assert!(o.inphase.is_finite() && o.quadrature.is_finite());
        }
        assert!(ht.is_ready());
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut ht = HtPhasor::new();
        let _ = ht.batch(&sine_prices(120));
        // A non-finite input is skipped and produces no value.
        assert_eq!(ht.update(f64::NAN), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices = sine_prices(150);
        let mut a = HtPhasor::new();
        let mut b = HtPhasor::new();
        let batch = a.batch(&prices);
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn reset_clears_state() {
        let mut ht = HtPhasor::new();
        let _ = ht.batch(&sine_prices(120));
        assert!(ht.is_ready());
        ht.reset();
        assert!(!ht.is_ready());
        assert_eq!(ht.update(100.0), None);
    }
}
