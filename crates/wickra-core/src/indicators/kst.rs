//! Know Sure Thing (KST).

use crate::error::{Error, Result};
use crate::indicators::roc::Roc;
use crate::indicators::sma::Sma;
use crate::traits::Indicator;

/// KST output: the `kst` summed-RCMA line and its `signal` SMA.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KstOutput {
    /// The KST line — Pring's weighted sum of four smoothed ROCs.
    pub kst: f64,
    /// `SMA(kst, signal_period)`.
    pub signal: f64,
}

/// Martin Pring's Know Sure Thing — a long-horizon momentum oscillator built
/// from four smoothed-ROC components plus an SMA signal line.
///
/// For each `i ∈ {1, 2, 3, 4}` an `RCMA_i = SMA(ROC(close, roc_i), sma_i)` is
/// computed, then combined with Pring's fixed weights `(1, 2, 3, 4)`:
///
/// ```text
/// RCMA_i = SMA(ROC(close, roc_i), sma_i)
/// KST    = 1 · RCMA_1 + 2 · RCMA_2 + 3 · RCMA_3 + 4 · RCMA_4
/// Signal = SMA(KST, signal_period)
/// ```
///
/// Pring's classic parameter set — exposed via [`Kst::classic`] — is
/// `(roc = (10, 15, 20, 30), sma = (10, 10, 10, 15), signal = 9)`. A KST
/// crossing above its signal is a bullish trigger; below, bearish.
///
/// Warmup is `max_i(roc_i + sma_i) + signal_period − 1` — the slowest branch
/// has to fully warm before the signal SMA can start filling.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Kst};
///
/// let mut indicator = Kst::classic().unwrap();
/// let mut last = None;
/// for i in 0..100 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone)]
pub struct Kst {
    roc_periods: [usize; 4],
    sma_periods: [usize; 4],
    signal_period: usize,
    rocs: [Roc; 4],
    rcmas: [Sma; 4],
    signal_sma: Sma,
    last: Option<KstOutput>,
}

impl Kst {
    /// Construct a new KST with explicit ROC periods, SMA periods, and signal
    /// SMA period.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if any of the nine periods is `0`.
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
        signal_period: usize,
    ) -> Result<Self> {
        if roc1 == 0
            || roc2 == 0
            || roc3 == 0
            || roc4 == 0
            || sma1 == 0
            || sma2 == 0
            || sma3 == 0
            || sma4 == 0
            || signal_period == 0
        {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            roc_periods: [roc1, roc2, roc3, roc4],
            sma_periods: [sma1, sma2, sma3, sma4],
            signal_period,
            rocs: [
                Roc::new(roc1)?,
                Roc::new(roc2)?,
                Roc::new(roc3)?,
                Roc::new(roc4)?,
            ],
            rcmas: [
                Sma::new(sma1)?,
                Sma::new(sma2)?,
                Sma::new(sma3)?,
                Sma::new(sma4)?,
            ],
            signal_sma: Sma::new(signal_period)?,
            last: None,
        })
    }

    /// Pring's classic KST: `roc = (10, 15, 20, 30)`, `sma = (10, 10, 10, 15)`,
    /// `signal_period = 9`.
    ///
    /// # Errors
    ///
    /// None in practice — all periods are non-zero.
    pub fn classic() -> Result<Self> {
        Self::new(10, 15, 20, 30, 10, 10, 10, 15, 9)
    }

    /// Configured `(roc1, roc2, roc3, roc4)`.
    pub const fn roc_periods(&self) -> (usize, usize, usize, usize) {
        (
            self.roc_periods[0],
            self.roc_periods[1],
            self.roc_periods[2],
            self.roc_periods[3],
        )
    }

    /// Configured `(sma1, sma2, sma3, sma4)`.
    pub const fn sma_periods(&self) -> (usize, usize, usize, usize) {
        (
            self.sma_periods[0],
            self.sma_periods[1],
            self.sma_periods[2],
            self.sma_periods[3],
        )
    }

    /// Configured signal SMA period.
    pub const fn signal_period(&self) -> usize {
        self.signal_period
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<KstOutput> {
        self.last
    }
}

impl Indicator for Kst {
    type Input = f64;
    type Output = KstOutput;

    fn update(&mut self, input: f64) -> Option<KstOutput> {
        // Always feed every parallel branch unconditionally so they warm in
        // lock-step; `KST` only emits once all four `RCMA`s are ready.
        let r0 = self.rocs[0].update(input);
        let r1 = self.rocs[1].update(input);
        let r2 = self.rocs[2].update(input);
        let r3 = self.rocs[3].update(input);

        let rcma0 = r0.and_then(|v| self.rcmas[0].update(v));
        let rcma1 = r1.and_then(|v| self.rcmas[1].update(v));
        let rcma2 = r2.and_then(|v| self.rcmas[2].update(v));
        let rcma3 = r3.and_then(|v| self.rcmas[3].update(v));

        let (Some(a), Some(b), Some(c), Some(d)) = (rcma0, rcma1, rcma2, rcma3) else {
            return None;
        };
        let kst = 1.0 * a + 2.0 * b + 3.0 * c + 4.0 * d;
        let signal = self.signal_sma.update(kst)?;
        let out = KstOutput { kst, signal };
        self.last = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        for r in &mut self.rocs {
            r.reset();
        }
        for s in &mut self.rcmas {
            s.reset();
        }
        self.signal_sma.reset();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // Each branch i needs roc_i inputs for the first ROC value, then
        // sma_i - 1 more inputs to fill the SMA window → branch i first
        // emits at input roc_i + sma_i (1-based). The signal SMA then needs
        // signal_period inputs of KST values, so first KST output lands at
        // max_branch + signal_period - 1.
        let max_branch = (0..4)
            .map(|i| self.roc_periods[i] + self.sma_periods[i])
            .max()
            .expect("array length 4");
        max_branch + self.signal_period - 1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
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
        // All nine slots fail on zero.
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
        let mut k = Kst::classic().unwrap();
        assert_eq!(k.roc_periods(), (10, 15, 20, 30));
        assert_eq!(k.sma_periods(), (10, 10, 10, 15));
        assert_eq!(k.signal_period(), 9);
        assert_eq!(k.name(), "KST");
        // Slowest branch is roc=30 + sma=15 = 45; signal adds 9 - 1 = 8 → 53.
        assert_eq!(k.warmup_period(), 53);
        assert!(k.value().is_none());
        for i in 1..=80 {
            k.update(100.0 + f64::from(i));
        }
        assert!(k.value().is_some());
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let prices: Vec<f64> = (1..=120).map(f64::from).collect();
        let mut k = Kst::classic().unwrap();
        let out = k.batch(&prices);
        let warmup = 53;
        for v in out.iter().take(warmup - 1) {
            assert!(v.is_none());
        }
        assert!(out[warmup - 1].is_some());
    }

    #[test]
    fn constant_series_yields_zero_lines() {
        // Flat prices: every ROC is 0, every RCMA is 0, KST = 0, signal = 0.
        let mut k = Kst::new(3, 4, 5, 6, 2, 2, 2, 2, 3).unwrap();
        let out = k.batch(&[100.0_f64; 40]);
        for v in out.iter().rev().flatten().take(3) {
            assert_relative_eq!(v.kst, 0.0, epsilon = 1e-12);
            assert_relative_eq!(v.signal, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn pure_uptrend_is_positive() {
        let prices: Vec<f64> = (1..=120).map(|i| 100.0 * 1.01_f64.powi(i)).collect();
        let mut k = Kst::classic().unwrap();
        let last = k.batch(&prices).into_iter().flatten().last().unwrap();
        assert!(last.kst > 0.0, "uptrend KST positive, got {}", last.kst);
        assert!(last.signal > 0.0);
    }

    #[test]
    fn signal_equals_sma_of_kst() {
        // The signal line is by definition the SMA(KST, signal_period); verify
        // against a separate SMA fed every KST value (including the warmup
        // values before the indicator-level signal is itself defined).
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 5.0)
            .collect();
        let signal_period = 4;
        let mut k = Kst::new(3, 5, 7, 9, 2, 2, 2, 3, signal_period).unwrap();
        let out = k.batch(&prices);

        // The indicator only emits KstOutput once the signal SMA is warm,
        // i.e. after signal_period KST values have been observed. To verify
        // we replay all KST values through an SMA(signal_period) ourselves
        // and compare every emitted signal entry.
        let kst_series: Vec<f64> = (0..prices.len())
            .filter_map(|i| {
                // Walk a parallel KST recomputation to extract kst before the
                // signal-warmup gating. Easier: re-derive kst series from the
                // emitted outputs (works because every emitted output exposes
                // its `kst`), then prepend the missing pre-signal kst values
                // by running a second indicator without the signal SMA — but
                // that's overkill. Instead, re-run a second KST with signal
                // period 1 (which gates only on RCMA readiness, so the
                // emitted kst is identical at every index).
                let _ = i;
                None::<f64>
            })
            .collect();
        let _ = kst_series;

        let mut k2 = Kst::new(3, 5, 7, 9, 2, 2, 2, 3, 1).unwrap();
        let out2 = k2.batch(&prices);
        let kst_full: Vec<f64> = out2.iter().filter_map(|v| v.map(|x| x.kst)).collect();

        let mut signal_check = Sma::new(signal_period).unwrap();
        let kst_emitted: Vec<f64> = kst_full
            .iter()
            .map(|v| signal_check.update(*v).unwrap_or(f64::NAN))
            .collect();
        // The outputs from `out` only start at the first index where the
        // signal SMA is warm, which is the (signal_period - 1)-th kst value.
        let emitted_signals: Vec<f64> = out.iter().filter_map(|v| v.map(|x| x.signal)).collect();
        let want: Vec<f64> = kst_emitted
            .iter()
            .filter(|v| !v.is_nan())
            .copied()
            .collect();
        assert_eq!(emitted_signals.len(), want.len());
        for (got, exp) in emitted_signals.iter().zip(want.iter()) {
            assert_relative_eq!(got, exp, epsilon = 1e-9);
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..120)
            .map(|i| 100.0 + (f64::from(i) * 0.2).sin() * 10.0)
            .collect();
        let batch = Kst::classic().unwrap().batch(&prices);
        let mut b = Kst::classic().unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn reset_clears_state() {
        let mut k = Kst::classic().unwrap();
        let prices: Vec<f64> = (1..=80).map(f64::from).collect();
        k.batch(&prices);
        assert!(k.is_ready());
        k.reset();
        assert!(!k.is_ready());
        assert_eq!(k.update(100.0), None);
    }
}
