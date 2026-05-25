//! Know Sure Thing (KST).

use crate::error::{Error, Result};
use crate::indicators::roc::Roc;
use crate::indicators::sma::Sma;
use crate::traits::Indicator;

/// `KST` output: the indicator line and its `SMA` signal line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KstOutput {
    /// Weighted sum of four smoothed `ROC` series.
    pub kst: f64,
    /// `SMA` of `kst` over the signal period.
    pub signal: f64,
}

/// Pring's Know Sure Thing — a long-horizon momentum oscillator that combines
/// four `ROC` series at different lookbacks, each smoothed by its own `SMA`,
/// summed with Pring's fixed weights `1, 2, 3, 4`:
///
/// ```text
/// RCMA_i = SMA(ROC(close, roc_i), sma_i)        for i = 1..=4
/// KST    = 1·RCMA_1 + 2·RCMA_2 + 3·RCMA_3 + 4·RCMA_4
/// Signal = SMA(KST, signal_period)
/// ```
///
/// Pring's recommended defaults are
/// `(roc1, roc2, roc3, roc4) = (10, 15, 20, 30)`,
/// `(sma1, sma2, sma3, sma4) = (10, 10, 10, 15)`,
/// `signal_period = 9`. `Kst::classic()` constructs that configuration.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Kst};
///
/// let mut kst = Kst::classic();
/// let mut last = None;
/// for i in 0..200 {
///     last = kst.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Kst {
    roc1_period: usize,
    roc2_period: usize,
    roc3_period: usize,
    roc4_period: usize,
    sma1_period: usize,
    sma2_period: usize,
    sma3_period: usize,
    sma4_period: usize,
    signal_period: usize,
    roc1: Roc,
    roc2: Roc,
    roc3: Roc,
    roc4: Roc,
    sma1: Sma,
    sma2: Sma,
    sma3: Sma,
    sma4: Sma,
    signal_sma: Sma,
    last_line: Option<f64>,
    last_signal: Option<f64>,
}

impl Kst {
    /// # Errors
    /// Returns [`Error::PeriodZero`] if any of the nine periods is zero.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        roc1: usize,
        roc2: usize,
        roc3: usize,
        roc4: usize,
        sma1: usize,
        sma2: usize,
        sma3: usize,
        sma4: usize,
        signal: usize,
    ) -> Result<Self> {
        if [roc1, roc2, roc3, roc4, sma1, sma2, sma3, sma4, signal].contains(&0) {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            roc1_period: roc1,
            roc2_period: roc2,
            roc3_period: roc3,
            roc4_period: roc4,
            sma1_period: sma1,
            sma2_period: sma2,
            sma3_period: sma3,
            sma4_period: sma4,
            signal_period: signal,
            roc1: Roc::new(roc1)?,
            roc2: Roc::new(roc2)?,
            roc3: Roc::new(roc3)?,
            roc4: Roc::new(roc4)?,
            sma1: Sma::new(sma1)?,
            sma2: Sma::new(sma2)?,
            sma3: Sma::new(sma3)?,
            sma4: Sma::new(sma4)?,
            signal_sma: Sma::new(signal)?,
            last_line: None,
            last_signal: None,
        })
    }

    /// Pring's recommended defaults: `KST(10, 15, 20, 30, 10, 10, 10, 15, 9)`.
    pub fn classic() -> Self {
        Self::new(10, 15, 20, 30, 10, 10, 10, 15, 9).expect("classic KST parameters are valid")
    }

    /// Configured `(roc1, roc2, roc3, roc4, sma1, sma2, sma3, sma4, signal)`.
    pub const fn periods(
        &self,
    ) -> (
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    ) {
        (
            self.roc1_period,
            self.roc2_period,
            self.roc3_period,
            self.roc4_period,
            self.sma1_period,
            self.sma2_period,
            self.sma3_period,
            self.sma4_period,
            self.signal_period,
        )
    }
}

impl Indicator for Kst {
    type Input = f64;
    type Output = KstOutput;

    fn update(&mut self, input: f64) -> Option<KstOutput> {
        // Feed every inner state machine on every input so they warm up in
        // parallel. The KST line waits for all four RCMA branches; the signal
        // line additionally waits for its own SMA to fill.
        let r1 = self.roc1.update(input);
        let r2 = self.roc2.update(input);
        let r3 = self.roc3.update(input);
        let r4 = self.roc4.update(input);
        let rcma1 = r1.and_then(|x| self.sma1.update(x));
        let rcma2 = r2.and_then(|x| self.sma2.update(x));
        let rcma3 = r3.and_then(|x| self.sma3.update(x));
        let rcma4 = r4.and_then(|x| self.sma4.update(x));
        let (rcma1, rcma2, rcma3, rcma4) = (rcma1?, rcma2?, rcma3?, rcma4?);
        let kst = rcma1 + 2.0 * rcma2 + 3.0 * rcma3 + 4.0 * rcma4;
        self.last_line = Some(kst);
        let signal = self.signal_sma.update(kst);
        let signal = signal?;
        self.last_signal = Some(signal);
        Some(KstOutput { kst, signal })
    }

    fn reset(&mut self) {
        self.roc1.reset();
        self.roc2.reset();
        self.roc3.reset();
        self.roc4.reset();
        self.sma1.reset();
        self.sma2.reset();
        self.sma3.reset();
        self.sma4.reset();
        self.signal_sma.reset();
        self.last_line = None;
        self.last_signal = None;
    }

    fn warmup_period(&self) -> usize {
        // Each RCMA_i emits once the inner ROC has warmed up (roc_i + 1
        // inputs) AND the SMA has filled (sma_i inputs through it). All four
        // run in parallel so the slowest branch dominates, and the signal SMA
        // adds signal_period − 1 inputs on top of the slowest branch.
        let branch = |roc: usize, sma: usize| roc + sma;
        let slowest = branch(self.roc1_period, self.sma1_period)
            .max(branch(self.roc2_period, self.sma2_period))
            .max(branch(self.roc3_period, self.sma3_period))
            .max(branch(self.roc4_period, self.sma4_period));
        slowest + self.signal_period - 1
    }

    fn is_ready(&self) -> bool {
        self.last_signal.is_some()
    }

    fn name(&self) -> &'static str {
        "KST"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(
            Kst::new(0, 15, 20, 30, 10, 10, 10, 15, 9),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            Kst::new(10, 15, 20, 30, 10, 10, 10, 15, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let kst = Kst::classic();
        assert_eq!(kst.periods(), (10, 15, 20, 30, 10, 10, 10, 15, 9));
        assert_eq!(kst.name(), "KST");
        // The slowest branch is ROC(30) + SMA(15) = 45; signal_period - 1 = 8.
        assert_eq!(kst.warmup_period(), 53);
    }

    #[test]
    fn classic_factory_matches_pring_defaults() {
        let kst = Kst::classic();
        let (r1, r2, r3, r4, s1, s2, s3, s4, sig) = kst.periods();
        assert_eq!((r1, r2, r3, r4), (10, 15, 20, 30));
        assert_eq!((s1, s2, s3, s4), (10, 10, 10, 15));
        assert_eq!(sig, 9);
    }

    #[test]
    fn constant_series_yields_zero() {
        // ROC is zero on a flat series, so every RCMA collapses to zero and
        // KST itself is zero. The signal SMA inherits that.
        let mut kst = Kst::classic();
        let prices = vec![42.0_f64; 80];
        let out = kst.batch(&prices);
        for v in out.iter().skip(kst.warmup_period() - 1).flatten() {
            assert_relative_eq!(v.kst, 0.0, epsilon = 1e-12);
            assert_relative_eq!(v.signal, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn warmup_emits_first_value_at_warmup_period() {
        let mut kst = Kst::new(2, 3, 4, 5, 2, 2, 2, 3, 2).unwrap();
        // Slowest branch is ROC(5) + SMA(3) = 8; signal − 1 = 1; total 9.
        assert_eq!(kst.warmup_period(), 9);
        let prices: Vec<f64> = (1..=15).map(f64::from).collect();
        let out = kst.batch(&prices);
        for v in out.iter().take(8) {
            assert!(v.is_none());
        }
        assert!(out[8].is_some());
    }

    #[test]
    fn pure_uptrend_is_positive() {
        // Monotonic uptrend -> every ROC > 0 -> every RCMA > 0 -> KST > 0.
        let mut kst = Kst::classic();
        let prices: Vec<f64> = (1..=120).map(|i| f64::from(i) * 2.0).collect();
        let out = kst.batch(&prices);
        let last = out.iter().rev().flatten().next().unwrap();
        assert!(
            last.kst > 0.0,
            "KST on a clean uptrend should be positive: {}",
            last.kst
        );
        assert!(last.signal > 0.0);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=120)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 5.0 + f64::from(i) * 0.1)
            .collect();
        let mut a = Kst::classic();
        let mut b = Kst::classic();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut kst = Kst::classic();
        let prices: Vec<f64> = (1..=120).map(f64::from).collect();
        kst.batch(&prices);
        assert!(kst.is_ready());
        kst.reset();
        assert!(!kst.is_ready());
    }
}
