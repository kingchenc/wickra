//! Ehlers Even Better Sinewave (EBSW) — a normalised cycle oscillator in [-1, 1].
#![allow(clippy::doc_markdown)]

use std::f64::consts::PI;

use crate::error::{Error, Result};
use crate::indicators::super_smoother::SuperSmoother;
use crate::traits::Indicator;

/// Ehlers' **Even Better Sinewave** (EBSW) — a self-normalising cycle oscillator
/// that swings cleanly in `[−1, +1]` regardless of price amplitude.
///
/// From John Ehlers' *Cycle Analytics for Traders* (2013, ch. 12):
///
/// ```text
/// alpha1 = (1 − sin(2π/hp_period)) / cos(2π/hp_period)
/// HP_t   = 0.5·(1 + alpha1)·(price_t − price_{t−1}) + alpha1·HP_{t−1}   (one-pole highpass)
/// Filt   = SuperSmoother(HP, ssf_length)
/// Wave   = (Filt_t + Filt_{t−1} + Filt_{t−2}) / 3
/// Pwr    = (Filt_t² + Filt_{t−1}² + Filt_{t−2}²) / 3
/// EBSW   = Wave / sqrt(Pwr)
/// ```
///
/// The price is first highpass-filtered to remove the trend, then SuperSmoothed to
/// remove noise, leaving the dominant cycle. Dividing a 3-bar average of that
/// cycle by its RMS power normalises the amplitude, so the output reads like a
/// clean sine wave bounded in `[−1, +1]` whatever the instrument. Unlike the
/// classic [`SineWave`](crate::SineWave) (which derives in-phase/quadrature
/// components from the Hilbert transform and can whip in trends), the EBSW stays
/// well-behaved and is read directly: crossing up through `0`/`−0.9` is a buy
/// cue, crossing down through `0`/`+0.9` a sell cue.
///
/// The first value lands once three SuperSmoothed samples exist
/// (`warmup_period == 3`). Each `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, EvenBetterSinewave};
///
/// let mut indicator = EvenBetterSinewave::new(40, 10).unwrap();
/// let mut last = None;
/// for i in 0..120 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct EvenBetterSinewave {
    hp_period: usize,
    ssf_length: usize,
    alpha1: f64,
    smoother: SuperSmoother,
    prev_price: Option<f64>,
    hp: f64,
    filt1: Option<f64>,
    filt2: Option<f64>,
    filt3: Option<f64>,
    last: Option<f64>,
}

impl EvenBetterSinewave {
    /// Construct an EBSW with the given highpass `hp_period` and SuperSmoother
    /// `ssf_length`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if either argument is `0`.
    pub fn new(hp_period: usize, ssf_length: usize) -> Result<Self> {
        if hp_period == 0 || ssf_length == 0 {
            return Err(Error::PeriodZero);
        }
        let w = 2.0 * PI / hp_period as f64;
        let alpha1 = (1.0 - w.sin()) / w.cos();
        Ok(Self {
            hp_period,
            ssf_length,
            alpha1,
            smoother: SuperSmoother::new(ssf_length)?,
            prev_price: None,
            hp: 0.0,
            filt1: None,
            filt2: None,
            filt3: None,
            last: None,
        })
    }

    /// Configured `(hp_period, ssf_length)`.
    pub const fn params(&self) -> (usize, usize) {
        (self.hp_period, self.ssf_length)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for EvenBetterSinewave {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, price: f64) -> Option<f64> {
        if !price.is_finite() {
            return self.last;
        }
        let hp = match self.prev_price {
            Some(prev) => 0.5 * (1.0 + self.alpha1) * (price - prev) + self.alpha1 * self.hp,
            None => 0.0,
        };
        self.prev_price = Some(price);
        self.hp = hp;
        let filt = self.smoother.update(hp)?;
        // Shift the three-deep filter buffer.
        self.filt3 = self.filt2;
        self.filt2 = self.filt1;
        self.filt1 = Some(filt);
        let (Some(f1), Some(f2), Some(f3)) = (self.filt1, self.filt2, self.filt3) else {
            return None;
        };
        let wave = (f1 + f2 + f3) / 3.0;
        let pwr = (f1 * f1 + f2 * f2 + f3 * f3) / 3.0;
        let ebsw = if pwr > 0.0 {
            (wave / pwr.sqrt()).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        self.last = Some(ebsw);
        Some(ebsw)
    }

    fn reset(&mut self) {
        self.smoother.reset();
        self.prev_price = None;
        self.hp = 0.0;
        self.filt1 = None;
        self.filt2 = None;
        self.filt3 = None;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        3
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "EvenBetterSinewave"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    #[test]
    fn rejects_zero_params() {
        assert!(matches!(
            EvenBetterSinewave::new(0, 10),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            EvenBetterSinewave::new(40, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let e = EvenBetterSinewave::new(40, 10).unwrap();
        assert_eq!(e.params(), (40, 10));
        assert_eq!(e.warmup_period(), 3);
        assert_eq!(e.name(), "EvenBetterSinewave");
        assert!(!e.is_ready());
        assert_eq!(e.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut e = EvenBetterSinewave::new(40, 10).unwrap();
        let xs: Vec<f64> = (0..12)
            .map(|i| 100.0 + (f64::from(i) * 0.5).sin() * 3.0)
            .collect();
        let out = e.batch(&xs);
        for v in out.iter().take(2) {
            assert!(v.is_none());
        }
        assert!(out[2].is_some());
    }

    #[test]
    fn output_in_range() {
        let mut e = EvenBetterSinewave::new(40, 10).unwrap();
        let xs: Vec<f64> = (0..400)
            .map(|i| 100.0 + (std::f64::consts::TAU * f64::from(i) / 30.0).sin() * 5.0)
            .collect();
        for v in e.batch(&xs).into_iter().flatten() {
            assert!((-1.0..=1.0).contains(&v), "EBSW out of range: {v}");
        }
    }

    #[test]
    fn cyclic_input_swings_both_signs() {
        let mut e = EvenBetterSinewave::new(30, 8).unwrap();
        let xs: Vec<f64> = (0..400)
            .map(|i| 100.0 + (std::f64::consts::TAU * f64::from(i) / 30.0).sin() * 5.0)
            .collect();
        let out: Vec<f64> = e.batch(&xs).into_iter().flatten().skip(100).collect();
        assert!(out.iter().any(|&v| v > 0.5));
        assert!(out.iter().any(|&v| v < -0.5));
    }

    #[test]
    fn ignores_non_finite() {
        let mut e = EvenBetterSinewave::new(40, 10).unwrap();
        e.batch(
            &(0..40)
                .map(|i| 100.0 + (f64::from(i) * 0.3).sin())
                .collect::<Vec<_>>(),
        );
        let before = e.value();
        assert_eq!(e.update(f64::NAN), before);
    }

    #[test]
    fn reset_clears_state() {
        let mut e = EvenBetterSinewave::new(40, 10).unwrap();
        e.batch(
            &(0..40)
                .map(|i| 100.0 + (f64::from(i) * 0.3).sin())
                .collect::<Vec<_>>(),
        );
        assert!(e.is_ready());
        e.reset();
        assert!(!e.is_ready());
        assert_eq!(e.value(), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let xs: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.25).sin() * 9.0)
            .collect();
        let batch = EvenBetterSinewave::new(40, 10).unwrap().batch(&xs);
        let mut b = EvenBetterSinewave::new(40, 10).unwrap();
        let streamed: Vec<_> = xs.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn flat_input_yields_zero_power() {
        // A constant series drives the highpass/smoother outputs to zero, so the
        // signal power is zero and the oscillator reports 0.0 (the `pwr == 0` arm).
        let flat = [100.0_f64; 200];
        let last = EvenBetterSinewave::new(40, 10)
            .unwrap()
            .batch(&flat)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_eq!(last, 0.0);
    }
}
