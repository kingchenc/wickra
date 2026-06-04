//! Fisher-transformed RSI.

use crate::error::Result;
use crate::indicators::rsi::Rsi;
use crate::traits::Indicator;

/// Fisher RSI — the Fisher transform applied to a normalised [`Rsi`](crate::Rsi).
///
/// The RSI is bounded in `[0, 100]` and its distribution piles up near the
/// middle, which blurs turning points. The Fisher transform reshapes a bounded
/// input toward a Gaussian, sharpening the extremes into clear, near-symmetric
/// peaks:
///
/// ```text
/// rsi   = RSI(price, period)            in [0, 100]
/// x     = clamp((rsi - 50) / 50, ±0.999)   normalise to (-1, 1)
/// Fisher = 0.5 * ln((1 + x) / (1 - x))
/// ```
///
/// The clamp keeps the logarithm finite when the RSI pins at `0` or `100`. The
/// output is unbounded but in practice oscillates in roughly `[-3, 3]`, with
/// sharp excursions marking momentum extremes. The first value lands with the
/// inner RSI, after `period + 1` inputs.
///
/// # Example
///
/// ```
/// use wickra_core::{FisherRsi, Indicator};
///
/// let mut indicator = FisherRsi::new(9).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + (f64::from(i) * 0.3).sin() * 5.0);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct FisherRsi {
    period: usize,
    rsi: Rsi,
}

impl FisherRsi {
    /// Construct a Fisher RSI with the given RSI period.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            period,
            rsi: Rsi::new(period)?,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for FisherRsi {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        let rsi = self.rsi.update(input)?;
        let x = ((rsi - 50.0) / 50.0).clamp(-0.999, 0.999);
        Some(0.5 * ((1.0 + x) / (1.0 - x)).ln())
    }

    fn reset(&mut self) {
        self.rsi.reset();
    }

    fn warmup_period(&self) -> usize {
        self.rsi.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.rsi.is_ready()
    }

    fn name(&self) -> &'static str {
        "FisherRSI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(FisherRsi::new(0).is_err());
    }

    /// Cover the const accessor `period` and the Indicator-impl `warmup_period`
    /// + `name`.
    #[test]
    fn accessors_and_metadata() {
        let f = FisherRsi::new(9).unwrap();
        assert_eq!(f.period(), 9);
        // RSI warmup is period + 1.
        assert_eq!(f.warmup_period(), 10);
        assert_eq!(f.name(), "FisherRSI");
    }

    #[test]
    fn warmup_matches_rsi() {
        let mut f = FisherRsi::new(3).unwrap();
        // RSI(3) needs 4 inputs; the first three return None.
        assert_eq!(f.update(1.0), None);
        assert_eq!(f.update(2.0), None);
        assert_eq!(f.update(3.0), None);
        assert!(f.update(4.0).is_some());
    }

    #[test]
    fn matches_fisher_of_rsi() {
        // Fisher RSI must equal the Fisher transform of the standalone RSI.
        let prices: Vec<f64> = (0..60)
            .map(|i| 100.0 + (f64::from(i) * 0.4).sin() * 8.0)
            .collect();
        let mut fr = FisherRsi::new(9).unwrap();
        let mut rsi = Rsi::new(9).unwrap();
        for (i, &p) in prices.iter().enumerate() {
            let got = fr.update(p);
            let want = rsi.update(p).map(|r| {
                let x = ((r - 50.0) / 50.0).clamp(-0.999, 0.999);
                0.5 * ((1.0 + x) / (1.0 - x)).ln()
            });
            assert_eq!(got.is_some(), want.is_some(), "readiness mismatch at {i}");
            if let (Some(a), Some(b)) = (got, want) {
                assert_relative_eq!(a, b, epsilon = 1e-12);
            }
        }
    }

    #[test]
    fn strong_uptrend_is_positive() {
        // A pure uptrend pins RSI near 100 -> x near +1 -> large positive Fisher.
        let prices: Vec<f64> = (1..=40).map(f64::from).collect();
        let mut f = FisherRsi::new(9).unwrap();
        let last = f.batch(&prices).into_iter().flatten().last().unwrap();
        assert!(
            last > 1.0,
            "strong uptrend should give a large positive value, got {last}"
        );
    }

    #[test]
    fn clamp_keeps_output_finite_at_extremes() {
        // Monotonic rise pins RSI at 100; the clamp must keep Fisher finite.
        let prices: Vec<f64> = (1..=30).map(f64::from).collect();
        let mut f = FisherRsi::new(5).unwrap();
        for v in f.batch(&prices).into_iter().flatten() {
            assert!(v.is_finite(), "Fisher RSI must stay finite, got {v}");
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut f = FisherRsi::new(5).unwrap();
        f.batch(&(1..=20).map(f64::from).collect::<Vec<_>>());
        assert!(f.is_ready());
        f.reset();
        assert!(!f.is_ready());
        assert_eq!(f.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=40)
            .map(|i| 50.0 + (f64::from(i) * 0.5).sin() * 10.0)
            .collect();
        let mut a = FisherRsi::new(9).unwrap();
        let mut b = FisherRsi::new(9).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }
}
