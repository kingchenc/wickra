//! Sample Entropy (`SampEn`) — the regularity / predictability of a window.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Population standard deviation of a slice (used for the matching tolerance).
fn population_stddev(window: &[f64]) -> f64 {
    let n = window.len() as f64;
    let mean = window.iter().sum::<f64>() / n;
    let var = window.iter().map(|&v| (v - mean) * (v - mean)).sum::<f64>() / n;
    var.max(0.0).sqrt()
}

/// Whether two length-`len` templates starting at `i` and `j` match within the
/// Chebyshev tolerance `tol`.
fn templates_match(window: &[f64], i: usize, j: usize, len: usize, tol: f64) -> bool {
    for k in 0..len {
        if (window[i + k] - window[j + k]).abs() > tol {
            return false;
        }
    }
    true
}

/// Sample Entropy (`SampEn`) — Richman & Moorman's measure of how *regular* (i.e.
/// predictable) a series is: the negative log conditional probability that two
/// sub-sequences similar for `m` points stay similar at the next point.
///
/// ```text
/// tol = r_factor · stddev(window)
/// B   = # template pairs of length m   within tol   (i < j)
/// A   = # template pairs of length m+1 within tol   (i < j)
/// `SampEn` = − ln(A / B)
/// ```
///
/// Low `SampEn` means the window is **regular** — patterns of length `m` reliably
/// extend to length `m + 1`, the fingerprint of a trending or cyclic market. High
/// `SampEn` means the series is **irregular** — knowing the last `m` points tells
/// you little about the next, the fingerprint of noise. Unlike the older
/// approximate entropy (`ApEn`), `SampEn` excludes self-matches, so it is far less
/// biased on short windows.
///
/// The tolerance is `r_factor` times the window's standard deviation, so the
/// measure self-scales. A perfectly flat window (`stddev == 0`) is maximally
/// regular and returns `0`. If no length-`m` pairs match, the entropy is
/// undefined and `0` is returned; if length-`m` pairs match but none extend, the
/// estimator falls back to treating the unseen count as one (`−ln(1/B) = ln(B)`).
/// The first value lands after `period` inputs; each `update` is O(`period²`).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, SampleEntropy};
///
/// let mut indicator = SampleEntropy::new(50, 2, 0.2).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update((f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct SampleEntropy {
    period: usize,
    emb_dim: usize,
    r_factor: f64,
    window: VecDeque<f64>,
    last: Option<f64>,
}

impl SampleEntropy {
    /// Construct a Sample Entropy over `period` values with embedding dimension
    /// `m` and tolerance factor `r_factor`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period` or `m` is `0`,
    /// [`Error::InvalidPeriod`] if `period < m + 2` (no length-`m+1` template
    /// pairs otherwise), and [`Error::InvalidParameter`] if `r_factor` is not
    /// finite and positive.
    pub fn new(period: usize, m: usize, r_factor: f64) -> Result<Self> {
        if period == 0 || m == 0 {
            return Err(Error::PeriodZero);
        }
        if period < m + 2 {
            return Err(Error::InvalidPeriod {
                message: "sample entropy needs period >= m + 2",
            });
        }
        if !r_factor.is_finite() || r_factor <= 0.0 {
            return Err(Error::InvalidParameter {
                message: "sample entropy r_factor must be finite and positive",
            });
        }
        Ok(Self {
            period,
            emb_dim: m,
            r_factor,
            window: VecDeque::with_capacity(period),
            last: None,
        })
    }

    /// Configured `(period, m, r_factor)`.
    pub const fn params(&self) -> (usize, usize, f64) {
        (self.period, self.emb_dim, self.r_factor)
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }

    fn compute(&self) -> f64 {
        let window: Vec<f64> = self.window.iter().copied().collect();
        let std = population_stddev(&window);
        if std == 0.0 {
            return 0.0;
        }
        let tol = self.r_factor * std;
        let m = self.emb_dim;
        // Restrict both template lengths to the same index range so A and B share
        // their candidate pairs: there are `period − m` length-(m+1) templates.
        let count = self.period - m;
        let mut matches_m = 0u64;
        let mut matches_m1 = 0u64;
        for i in 0..count {
            for j in (i + 1)..count {
                if templates_match(&window, i, j, m, tol) {
                    matches_m += 1;
                    if templates_match(&window, i, j, m + 1, tol) {
                        matches_m1 += 1;
                    }
                }
            }
        }
        if matches_m == 0 {
            return 0.0;
        }
        if matches_m1 == 0 {
            // No length-(m+1) matches: fall back to one unseen count.
            return (matches_m as f64).ln();
        }
        -((matches_m1 as f64) / (matches_m as f64)).ln()
    }
}

impl Indicator for SampleEntropy {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.last;
        }
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(input);
        if self.window.len() < self.period {
            return None;
        }
        let out = self.compute();
        self.last = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "SampleEntropy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_invalid_params() {
        assert!(matches!(
            SampleEntropy::new(0, 2, 0.2),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            SampleEntropy::new(50, 0, 0.2),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            SampleEntropy::new(3, 2, 0.2),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            SampleEntropy::new(50, 2, 0.0),
            Err(Error::InvalidParameter { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let s = SampleEntropy::new(50, 2, 0.2).unwrap();
        assert_eq!(s.params(), (50, 2, 0.2));
        assert_eq!(s.warmup_period(), 50);
        assert_eq!(s.name(), "SampleEntropy");
        assert!(!s.is_ready());
        assert_eq!(s.value(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut s = SampleEntropy::new(10, 2, 0.2).unwrap();
        let xs: Vec<f64> = (0..14).map(|i| (f64::from(i) * 0.5).sin()).collect();
        let out = s.batch(&xs);
        for v in out.iter().take(9) {
            assert!(v.is_none());
        }
        assert!(out[9].is_some());
    }

    #[test]
    fn constant_window_is_zero() {
        let mut s = SampleEntropy::new(20, 2, 0.2).unwrap();
        let last = s.batch(&[5.0; 30]).into_iter().flatten().last().unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn output_is_non_negative() {
        let mut s = SampleEntropy::new(40, 2, 0.2).unwrap();
        for v in s
            .batch(
                &(0..200)
                    .map(|i| (f64::from(i) * 0.3).sin() * 5.0)
                    .collect::<Vec<_>>(),
            )
            .into_iter()
            .flatten()
        {
            assert!(v >= 0.0, "sample entropy must be non-negative, got {v}");
        }
    }

    #[test]
    fn regular_below_irregular() {
        // A smooth sine is far more regular (lower `SampEn`) than a chaotic
        // logistic-map series. (An *alternating* series would be periodic, hence
        // regular too -- chaos is what makes the window genuinely unpredictable.)
        let smooth: Vec<f64> = (0..60).map(|i| (f64::from(i) * 0.2).sin() * 5.0).collect();
        let mut x = 0.37_f64;
        let chaotic: Vec<f64> = (0..60)
            .map(|_| {
                x = 3.99 * x * (1.0 - x);
                x * 5.0
            })
            .collect();
        let s_smooth = SampleEntropy::new(50, 2, 0.2)
            .unwrap()
            .batch(&smooth)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        let s_chaotic = SampleEntropy::new(50, 2, 0.2)
            .unwrap()
            .batch(&chaotic)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!(
            s_smooth <= s_chaotic,
            "smooth ({s_smooth}) should be <= chaotic ({s_chaotic})"
        );
    }

    #[test]
    fn ignores_non_finite() {
        let mut s = SampleEntropy::new(10, 2, 0.2).unwrap();
        let xs: Vec<f64> = (0..10).map(|i| (f64::from(i) * 0.5).sin()).collect();
        let ready = s.batch(&xs).into_iter().flatten().last().unwrap();
        assert_eq!(s.update(f64::NAN), Some(ready));
    }

    #[test]
    fn reset_clears_state() {
        let mut s = SampleEntropy::new(10, 2, 0.2).unwrap();
        let xs: Vec<f64> = (0..10).map(|i| (f64::from(i) * 0.5).sin()).collect();
        s.batch(&xs);
        assert!(s.is_ready());
        s.reset();
        assert!(!s.is_ready());
        assert_eq!(s.value(), None);
        assert_eq!(s.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let xs: Vec<f64> = (0..120)
            .map(|i| (f64::from(i) * 0.25).sin() * 9.0)
            .collect();
        let batch = SampleEntropy::new(40, 2, 0.2).unwrap().batch(&xs);
        let mut b = SampleEntropy::new(40, 2, 0.2).unwrap();
        let streamed: Vec<_> = xs.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn falls_back_when_no_m_plus_one_matches() {
        // `[1, 1, 1, 5]` with m = 2: the length-2 template `(1, 1)` repeats
        // (matches_m > 0) but no length-3 template repeats (matches_m1 == 0),
        // so SampEn takes the `ln(matches_m)` fallback branch.
        let xs = [1.0, 1.0, 1.0, 5.0];
        let v = SampleEntropy::new(4, 2, 0.2)
            .unwrap()
            .batch(&xs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!(v.is_finite() && v >= 0.0, "got {v}");
    }
}
