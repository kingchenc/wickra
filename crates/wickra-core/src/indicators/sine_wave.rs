//! Ehlers Sine Wave indicator.
#![allow(clippy::manual_clamp)]

use std::f64::consts::PI;

use crate::indicators::hilbert_dominant_cycle::HilbertDominantCycle;
use crate::traits::Indicator;

/// Ehlers' Sine Wave indicator (sine + leadsine).
///
/// Implementation from *Rocket Science for Traders* (Ehlers 2001, ch. 9). Uses
/// the same Hilbert-transform machinery as [`HilbertDominantCycle`] to derive
/// the instantaneous phase, then returns `sin(phase)` and the 45° lead
/// `sin(phase + 45°)`. The two lines cross deep in trends but oscillate
/// rapidly during cycles, providing a visual lead/lag signal.
///
/// Only the primary `sine` line is exposed as the scalar output to match the
/// crate's standard scalar-indicator surface; the lead is accessible via the
/// [`SineWave::lead`] accessor after each update.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, SineWave};
///
/// let mut sw = SineWave::new();
/// let mut last = None;
/// for i in 0..200 {
///     last = sw.update(100.0 + (f64::from(i) * 0.4).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct SineWave {
    cycle: HilbertDominantCycle,
    smooth_buf: Vec<f64>,
    detrender_buf: Vec<f64>,
    last_phase: f64,
    last_sine: Option<f64>,
    last_lead: f64,
    count: usize,
}

impl SineWave {
    /// Construct a new Sine Wave indicator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Most recent lead (45°-ahead) value. `0.0` until the indicator is ready.
    pub const fn lead(&self) -> f64 {
        self.last_lead
    }

    /// Current sine value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_sine
    }

    fn push_front(buf: &mut Vec<f64>, v: f64, cap: usize) {
        buf.insert(0, v);
        if buf.len() > cap {
            buf.truncate(cap);
        }
    }
}

impl Indicator for SineWave {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.last_sine;
        }
        self.count += 1;
        // Drive the dominant-cycle estimator first; its smoothing state is
        // independent from ours so the two share input but not buffers.
        let _ = self.cycle.update(input);

        Self::push_front(&mut self.smooth_buf, input, 7);
        if self.smooth_buf.len() < 4 {
            return None;
        }
        let smooth = (4.0 * self.smooth_buf[0]
            + 3.0 * self.smooth_buf[1]
            + 2.0 * self.smooth_buf[2]
            + self.smooth_buf[3])
            / 10.0;
        if self.smooth_buf.len() < 7 {
            return None;
        }
        let period = self.cycle.value().unwrap_or(15.0).max(6.0).min(50.0);
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
        let phase = if i1.abs() > f64::EPSILON {
            (q1 / i1).atan()
        } else {
            self.last_phase
        };
        self.last_phase = phase;
        let sine = phase.sin();
        let lead = (phase + PI / 4.0).sin();

        if self.count < 50 {
            return None;
        }
        self.last_sine = Some(sine);
        self.last_lead = lead;
        Some(sine)
    }

    fn reset(&mut self) {
        self.cycle.reset();
        self.smooth_buf.clear();
        self.detrender_buf.clear();
        self.last_phase = 0.0;
        self.last_sine = None;
        self.last_lead = 0.0;
        self.count = 0;
    }

    fn warmup_period(&self) -> usize {
        50
    }

    fn is_ready(&self) -> bool {
        self.last_sine.is_some()
    }

    fn name(&self) -> &'static str {
        "SineWave"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    #[test]
    fn accessors_and_metadata() {
        let mut sw = SineWave::new();
        assert_eq!(sw.warmup_period(), 50);
        assert_eq!(sw.name(), "SineWave");
        assert!(!sw.is_ready());
        assert!(sw.value().is_none());
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 5.0)
            .collect();
        sw.batch(&prices);
        assert!(sw.is_ready());
        assert!(sw.value().is_some());
    }

    #[test]
    fn output_bounded() {
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.3).cos() * 5.0)
            .collect();
        let mut sw = SineWave::new();
        for v in sw.batch(&prices).into_iter().flatten() {
            assert!((-1.0..=1.0).contains(&v), "sine out of bounds: {v}");
        }
        // Lead value also bounded after warmup.
        assert!(sw.lead() >= -1.0 && sw.lead() <= 1.0);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        let mut a = SineWave::new();
        let mut b = SineWave::new();
        let batch = a.batch(&prices);
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut sw = SineWave::new();
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 5.0)
            .collect();
        sw.batch(&prices);
        let before = sw.value();
        assert!(before.is_some());
        assert_eq!(sw.update(f64::NAN), before);
    }

    #[test]
    fn reset_clears_state() {
        let mut sw = SineWave::new();
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 5.0)
            .collect();
        sw.batch(&prices);
        assert!(sw.is_ready());
        sw.reset();
        assert!(!sw.is_ready());
        assert!(sw.value().is_none());
    }

    #[test]
    fn flat_input_uses_phase_fallback() {
        // Zero inputs make every smooth/detrender term arithmetically exact
        // zero (no IEEE-754 cancellation residue), so `i1 == 0.0` and the
        // phase calculation deterministically takes the `self.last_phase`
        // fallback rather than `atan(q1/i1)`. A non-zero constant like
        // `100.0` leaves a sub-EPSILON residue that flips the branch back.
        let mut sw = SineWave::new();
        let _ = sw.batch(&[0.0_f64; 120]);
        assert!(sw.value().is_some());
    }
}
