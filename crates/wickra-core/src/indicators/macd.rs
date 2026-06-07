//! Moving Average Convergence Divergence (MACD).

use crate::error::{Error, Result};
use crate::indicators::ema::Ema;
use crate::traits::Indicator;

/// MACD output: the three classic series at a given step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacdOutput {
    /// Fast EMA − slow EMA.
    pub macd: f64,
    /// EMA of `macd` over the signal period.
    pub signal: f64,
    /// `macd − signal`.
    pub histogram: f64,
}

/// MACD = EMA(fast) − EMA(slow), with a signal EMA on top.
///
/// Standard parameters are `fast = 12`, `slow = 26`, `signal = 9`. The signal EMA
/// is seeded from the first `signal` raw MACD values, so the first full
/// [`MacdOutput`] is emitted after `slow + signal − 1` inputs (assuming the
/// slow EMA seeded by then).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, MacdIndicator};
///
/// let mut indicator = MacdIndicator::new(3, 6, 3).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct MacdIndicator {
    fast: Ema,
    slow: Ema,
    signal_ema: Ema,
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    last: Option<MacdOutput>,
}

impl MacdIndicator {
    /// Construct a MACD with the given periods.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if any period is zero, and
    /// [`Error::InvalidPeriod`] if `fast >= slow`.
    pub fn new(fast: usize, slow: usize, signal: usize) -> Result<Self> {
        if fast == 0 || slow == 0 || signal == 0 {
            return Err(Error::PeriodZero);
        }
        if fast >= slow {
            return Err(Error::InvalidPeriod {
                message: "fast period must be strictly less than slow period",
            });
        }
        Ok(Self {
            fast: Ema::new(fast)?,
            slow: Ema::new(slow)?,
            signal_ema: Ema::new(signal)?,
            fast_period: fast,
            slow_period: slow,
            signal_period: signal,
            last: None,
        })
    }

    /// Default `(12, 26, 9)` configuration, matching every classical chart package.
    pub fn classic() -> Self {
        Self::new(12, 26, 9).expect("classic MACD periods are valid")
    }

    /// Configured periods as `(fast, slow, signal)`.
    pub const fn periods(&self) -> (usize, usize, usize) {
        (self.fast_period, self.slow_period, self.signal_period)
    }

    /// Most recent fully-computed output if available.
    pub const fn value(&self) -> Option<MacdOutput> {
        self.last
    }

    /// Vectorized flat batch for bindings: `n * 3` values laid out as
    /// `[macd, signal, histogram]` per input row, warmup rows all `NaN`.
    ///
    /// For a fresh, all-finite slice long enough for a full output it runs the
    /// fast EMA, slow EMA and signal EMA as three recurrences fused into a single
    /// pass with one allocation — no `Option` per tick, no per-EMA intermediate
    /// buffers, identical SMA-mean seeds (division) and `mul_add` recurrences. The
    /// result is *bit-for-bit* equal to replaying `update`. Anything else (not
    /// fresh, non-finite, or too short to emit) defers to the exact `update`
    /// replay.
    ///
    /// Separate from the trait [`batch`](crate::BatchExt::batch), which stays a
    /// bit-identical `update` replay; only the bindings call this.
    pub fn batch_macd(&mut self, inputs: &[f64]) -> Vec<f64> {
        let n = inputs.len();
        let (fp, sp, gp) = (self.fast_period, self.slow_period, self.signal_period);
        // First full output needs the slow EMA seeded (index sp-1) plus gp signal
        // values: index sp + gp - 2. Below that, or non-fresh/non-finite, replay.
        if self.last.is_some()
            || !self.fast.is_fresh()
            || !self.slow.is_fresh()
            || !self.signal_ema.is_fresh()
            || n < sp + gp - 1
            || !inputs.iter().all(|x| x.is_finite())
        {
            let mut out = vec![f64::NAN; n * 3];
            for (i, &x) in inputs.iter().enumerate() {
                if let Some(o) = self.update(x) {
                    out[i * 3] = o.macd;
                    out[i * 3 + 1] = o.signal;
                    out[i * 3 + 2] = o.histogram;
                }
            }
            return out;
        }

        // Pre-sized output: warmup rows stay NaN, full-output rows are written in
        // place by index — no per-row `push` length/capacity check.
        let mut out = vec![f64::NAN; n * 3];
        let (fa, fo) = (self.fast.alpha(), 1.0 - self.fast.alpha());
        let (sa, so) = (self.slow.alpha(), 1.0 - self.slow.alpha());
        let (ga, go) = (self.signal_ema.alpha(), 1.0 - self.signal_ema.alpha());
        let (fp_f, sp_f, gp_f) = (fp as f64, sp as f64, gp as f64);

        let (mut fast_val, mut slow_val, mut sig) = (0.0_f64, 0.0_f64, 0.0_f64);
        let (mut fsum, mut ssum, mut gsum) = (0.0_f64, 0.0_f64, 0.0_f64);
        let mut sig_count = 0usize; // signal-EMA seed progress (raw MACD values seen)
        let mut sig_seeded = false;
        let mut last = MacdOutput {
            macd: 0.0,
            signal: 0.0,
            histogram: 0.0,
        };

        for (i, &x) in inputs.iter().enumerate() {
            // Fast EMA: SMA-seeded at index fp-1, then recurrence.
            if i < fp {
                fsum += x;
                if i == fp - 1 {
                    fast_val = fsum / fp_f;
                }
            } else {
                fast_val = fa.mul_add(x, fo * fast_val);
            }
            // Slow EMA: SMA-seeded at index sp-1, then recurrence.
            if i < sp {
                ssum += x;
                if i == sp - 1 {
                    slow_val = ssum / sp_f;
                }
            } else {
                slow_val = sa.mul_add(x, so * slow_val);
            }
            if i + 1 < sp {
                continue; // slow EMA not seeded yet → no raw MACD line
            }
            let macd = fast_val - slow_val;
            // Signal EMA over the MACD line: SMA-seeded over its first gp values.
            let signal = if sig_seeded {
                sig = ga.mul_add(macd, go * sig);
                sig
            } else {
                gsum += macd;
                sig_count += 1;
                if sig_count < gp {
                    continue; // signal EMA still seeding → no full output
                }
                sig = gsum / gp_f;
                sig_seeded = true;
                sig
            };
            let histogram = macd - signal;
            out[i * 3] = macd;
            out[i * 3 + 1] = signal;
            out[i * 3 + 2] = histogram;
            last = MacdOutput {
                macd,
                signal,
                histogram,
            };
        }

        // Leave every sub-EMA and `last` where a full `update` replay would.
        self.fast.seed_to(fast_val);
        self.slow.seed_to(slow_val);
        self.signal_ema.seed_to(sig);
        self.last = Some(last);
        out
    }
}

impl Indicator for MacdIndicator {
    type Input = f64;
    type Output = MacdOutput;

    fn update(&mut self, input: f64) -> Option<MacdOutput> {
        if !input.is_finite() {
            return self.last;
        }

        let fast = self.fast.update(input);
        let slow = self.slow.update(input);

        match (fast, slow) {
            (Some(f), Some(s)) => {
                let macd = f - s;
                let signal = self.signal_ema.update(macd)?;
                let out = MacdOutput {
                    macd,
                    signal,
                    histogram: macd - signal,
                };
                self.last = Some(out);
                Some(out)
            }
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.fast.reset();
        self.slow.reset();
        self.signal_ema.reset();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        // Slow EMA needs `slow` inputs to seed; signal EMA needs another `signal - 1`.
        self.slow_period + self.signal_period - 1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "MACD"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_fast_geq_slow() {
        assert!(matches!(
            MacdIndicator::new(26, 12, 9),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(matches!(
            MacdIndicator::new(12, 12, 9),
            Err(Error::InvalidPeriod { .. })
        ));
    }

    /// Cover the const accessors `periods` / `value` (81-88) and the
    /// Indicator-impl `name` body (135-137). `warmup_period` is exercised
    /// elsewhere.
    #[test]
    fn accessors_and_metadata() {
        let mut m = MacdIndicator::new(12, 26, 9).unwrap();
        assert_eq!(m.periods(), (12, 26, 9));
        assert_eq!(m.name(), "MACD");
        assert!(m.value().is_none());
        for i in 1..=m.warmup_period() {
            m.update(100.0 + f64::from(u32::try_from(i).unwrap()));
        }
        assert!(m.value().is_some());
    }

    #[test]
    fn rejects_zero_periods() {
        assert!(matches!(
            MacdIndicator::new(0, 26, 9),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            MacdIndicator::new(12, 0, 9),
            Err(Error::PeriodZero)
        ));
        assert!(matches!(
            MacdIndicator::new(12, 26, 0),
            Err(Error::PeriodZero)
        ));
    }

    #[test]
    fn first_emission_matches_warmup_period() {
        let prices: Vec<f64> = (1..=60).map(f64::from).collect();
        let mut macd = MacdIndicator::classic();
        let out = macd.batch(&prices);
        let warmup = macd.warmup_period();
        // Indices 0..warmup-1 are None, index warmup-1 might be Some or might still need
        // the signal EMA's seeding. Our warmup_period is the index at which the first
        // signal value appears: slow + signal - 1.
        for x in out.iter().take(warmup - 1) {
            assert!(x.is_none(), "expected None within warmup");
        }
        assert!(
            out[warmup - 1].is_some(),
            "expected first emission at warmup_period - 1 ({warmup} idx)"
        );
    }

    #[test]
    fn histogram_equals_macd_minus_signal() {
        let prices: Vec<f64> = (1..=80).map(|i| f64::from(i) * 0.5).collect();
        let mut macd = MacdIndicator::classic();
        for v in macd.batch(&prices).into_iter().flatten() {
            assert_relative_eq!(v.histogram, v.macd - v.signal, epsilon = 1e-12);
        }
    }

    #[test]
    fn constant_series_yields_zero_macd_eventually() {
        let mut macd = MacdIndicator::classic();
        let out = macd.batch(&[100.0_f64; 200]);
        // Both EMAs converge to 100, so MACD must approach 0.
        let last = out.iter().rev().flatten().next().expect("emits a value");
        assert_relative_eq!(last.macd, 0.0, epsilon = 1e-9);
        assert_relative_eq!(last.signal, 0.0, epsilon = 1e-9);
        assert_relative_eq!(last.histogram, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn rising_series_macd_positive_then_signal_catches_up() {
        let prices: Vec<f64> = (1..=200).map(f64::from).collect();
        let mut macd = MacdIndicator::classic();
        let out = macd.batch(&prices);
        let last = out.iter().rev().flatten().next().unwrap();
        assert!(last.macd > 0.0, "rising series must yield positive MACD");
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=100)
            .map(|i| (f64::from(i) * 0.4).cos() * 10.0)
            .collect();
        let mut a = MacdIndicator::classic();
        let mut b = MacdIndicator::classic();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut macd = MacdIndicator::classic();
        macd.batch(&(1..=80).map(f64::from).collect::<Vec<_>>());
        assert!(macd.is_ready());
        macd.reset();
        assert!(!macd.is_ready());
        assert_eq!(macd.update(1.0), None);
    }

    fn bits_eq(a: &[f64], b: &[f64]) -> bool {
        a.len() == b.len()
            && a.iter()
                .zip(b)
                .all(|(x, y)| x == y || (x.is_nan() && y.is_nan()))
    }

    /// Flat `n*3` `[macd, signal, histogram]` replay of `update`.
    fn macd_replay(series: &[f64]) -> Vec<f64> {
        let mut m = MacdIndicator::classic();
        let mut out = Vec::with_capacity(series.len() * 3);
        for &x in series {
            match m.update(x) {
                Some(o) => out.extend_from_slice(&[o.macd, o.signal, o.histogram]),
                None => out.extend_from_slice(&[f64::NAN; 3]),
            }
        }
        out
    }

    #[test]
    fn batch_macd_fast_path_is_bit_identical() {
        let series: Vec<f64> = (0..300)
            .map(|i| (f64::from(i) * 0.4).cos() * 10.0 + 100.0)
            .collect();
        let mut macd = MacdIndicator::classic();
        let got = macd.batch_macd(&series);
        assert!(bits_eq(&got, &macd_replay(&series)));
        // Sub-EMA + last state left where the replay would: continued update agrees.
        let mut ref_macd = MacdIndicator::classic();
        for &x in &series {
            ref_macd.update(x);
        }
        let (a, b) = (macd.update(101.0), ref_macd.update(101.0));
        assert_eq!(a.is_some(), b.is_some());
        assert_relative_eq!(a.unwrap().macd, b.unwrap().macd, epsilon = 1e-12);
    }

    #[test]
    fn batch_macd_falls_back_on_non_finite() {
        let mut series: Vec<f64> = (0..60).map(|i| f64::from(i) + 100.0).collect();
        series[40] = f64::NAN;
        let mut macd = MacdIndicator::classic();
        assert!(bits_eq(&macd.batch_macd(&series), &macd_replay(&series)));
    }

    #[test]
    fn batch_macd_falls_back_when_not_fresh() {
        let series: Vec<f64> = (0..60).map(|i| f64::from(i) + 100.0).collect();
        let mut macd = MacdIndicator::classic();
        macd.update(50.0);
        let mut ref_macd = MacdIndicator::classic();
        ref_macd.update(50.0);
        let mut want = Vec::new();
        for &x in &series {
            match ref_macd.update(x) {
                Some(o) => want.extend_from_slice(&[o.macd, o.signal, o.histogram]),
                None => want.extend_from_slice(&[f64::NAN; 3]),
            }
        }
        assert!(bits_eq(&macd.batch_macd(&series), &want));
    }

    #[test]
    fn batch_macd_too_short_for_output_falls_back() {
        // n < slow + signal - 1 (= 34): no full output, routed to the replay.
        let series: Vec<f64> = (0..20).map(|i| f64::from(i) + 100.0).collect();
        let mut macd = MacdIndicator::classic();
        let got = macd.batch_macd(&series);
        assert!(bits_eq(&got, &macd_replay(&series)));
        assert!(got.iter().all(|x| x.is_nan()));
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut macd = MacdIndicator::classic();
        macd.batch(&(1..=80).map(f64::from).collect::<Vec<_>>());
        let before = macd.value();
        assert!(before.is_some());
        // Non-finite inputs return the last value without advancing any EMA.
        assert_eq!(macd.update(f64::NAN), before);
        assert_eq!(macd.update(f64::INFINITY), before);
        assert_eq!(macd.value(), before);
    }
}
