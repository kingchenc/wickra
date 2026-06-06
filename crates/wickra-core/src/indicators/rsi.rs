//! Relative Strength Index using Wilder's smoothing.

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Relative Strength Index (Wilder, 1978).
///
/// Uses Wilder's smoothing (an EMA with `alpha = 1 / period`). The first output
/// is produced after `period + 1` inputs: the seed averages the first `period`
/// gains and losses, and the first emitted RSI corresponds to the input at
/// index `period`.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Rsi};
///
/// let mut indicator = Rsi::new(3).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Rsi {
    period: usize,
    /// `period - 1` as `f64`, precomputed for the Wilder smoothing step.
    n_minus_1: f64,
    /// `1 / period`, precomputed so the per-tick smoothing multiplies instead of
    /// divides (a reciprocal is hoisted out of the hot path).
    inv_period: f64,
    /// Previous close, valid once `has_prev` is set. Bare `f64` + flag instead of
    /// `Option<f64>` to avoid an enum-tag read on every tick.
    prev_close: f64,
    has_prev: bool,
    // Wilder seeds with the simple average of the first `period` gains/losses,
    // then transitions to recursive smoothing.
    seed_buf_gains: Vec<f64>,
    seed_buf_losses: Vec<f64>,
    /// Smoothed average gain / loss, valid once `avgs_seeded` is set. Bare `f64`s
    /// + flag so the hot recurrence avoids reading two `Option<f64>` tags per tick.
    avg_gain: f64,
    avg_loss: f64,
    avgs_seeded: bool,
    last_value: Option<f64>,
}

impl Rsi {
    /// Construct an RSI with the given Wilder period.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            period,
            n_minus_1: (period - 1) as f64,
            inv_period: 1.0 / period as f64,
            prev_close: 0.0,
            has_prev: false,
            seed_buf_gains: Vec::with_capacity(period),
            seed_buf_losses: Vec::with_capacity(period),
            avg_gain: 0.0,
            avg_loss: 0.0,
            avgs_seeded: false,
            last_value: None,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }

    fn rsi_from_avgs(avg_gain: f64, avg_loss: f64) -> f64 {
        // Algebraically `100 - 100/(1 + ag/al)` collapses to `100·ag/(ag+al)`,
        // which needs a single division instead of two and removes the separate
        // `rs` step. Edge cases stay exact: `al == 0, ag > 0` gives `100·ag/ag =
        // 100`; `ag == 0, al > 0` gives `0`; both zero (no movement) is the
        // undefined case and returns the neutral 50.
        let denom = avg_gain + avg_loss;
        if denom == 0.0 {
            50.0
        } else {
            100.0 * avg_gain / denom
        }
    }
}

impl Indicator for Rsi {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.last_value;
        }

        if !self.has_prev {
            self.prev_close = input;
            self.has_prev = true;
            return None;
        }
        let prev = self.prev_close;
        self.prev_close = input;

        let diff = input - prev;
        let gain = if diff > 0.0 { diff } else { 0.0 };
        let loss = if diff < 0.0 { -diff } else { 0.0 };

        if self.avgs_seeded {
            // Wilder smoothing `(prev·(n-1) + x) / n` with the reciprocal hoisted:
            // a fused multiply-add then a multiply by `1/n`, no per-tick division.
            let new_ag = self.avg_gain.mul_add(self.n_minus_1, gain) * self.inv_period;
            let new_al = self.avg_loss.mul_add(self.n_minus_1, loss) * self.inv_period;
            self.avg_gain = new_ag;
            self.avg_loss = new_al;
            let v = Self::rsi_from_avgs(new_ag, new_al);
            self.last_value = Some(v);
            return Some(v);
        }

        self.seed_buf_gains.push(gain);
        self.seed_buf_losses.push(loss);
        if self.seed_buf_gains.len() == self.period {
            let ag = self.seed_buf_gains.iter().sum::<f64>() / self.period as f64;
            let al = self.seed_buf_losses.iter().sum::<f64>() / self.period as f64;
            self.avg_gain = ag;
            self.avg_loss = al;
            self.avgs_seeded = true;
            let v = Self::rsi_from_avgs(ag, al);
            self.last_value = Some(v);
            return Some(v);
        }
        None
    }

    fn reset(&mut self) {
        self.prev_close = 0.0;
        self.has_prev = false;
        self.seed_buf_gains.clear();
        self.seed_buf_losses.clear();
        self.avg_gain = 0.0;
        self.avg_loss = 0.0;
        self.avgs_seeded = false;
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "RSI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    /// Independent reference: Wilder RSI computed straight from the definition.
    fn rsi_naive(prices: &[f64], period: usize) -> Vec<Option<f64>> {
        let n = period as f64;
        let mut out = vec![None; prices.len()];
        let mut gains: Vec<f64> = Vec::new();
        let mut losses: Vec<f64> = Vec::new();
        let mut avg_gain: Option<f64> = None;
        let mut avg_loss: Option<f64> = None;
        let rsi_val = |ag: f64, al: f64| -> f64 {
            if al == 0.0 {
                if ag == 0.0 {
                    50.0
                } else {
                    100.0
                }
            } else {
                100.0 - 100.0 / (1.0 + ag / al)
            }
        };
        for i in 1..prices.len() {
            let diff = prices[i] - prices[i - 1];
            let gain = if diff > 0.0 { diff } else { 0.0 };
            let loss = if diff < 0.0 { -diff } else { 0.0 };
            if let (Some(ag), Some(al)) = (avg_gain, avg_loss) {
                let nag = (ag * (n - 1.0) + gain) / n;
                let nal = (al * (n - 1.0) + loss) / n;
                avg_gain = Some(nag);
                avg_loss = Some(nal);
                out[i] = Some(rsi_val(nag, nal));
            } else {
                gains.push(gain);
                losses.push(loss);
                if gains.len() == period {
                    let ag = gains.iter().sum::<f64>() / n;
                    let al = losses.iter().sum::<f64>() / n;
                    avg_gain = Some(ag);
                    avg_loss = Some(al);
                    out[i] = Some(rsi_val(ag, al));
                }
            }
        }
        out
    }

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(Rsi::new(0), Err(Error::PeriodZero)));
    }

    /// Cover the const accessors `period` / `value` (60-67) and the
    /// Indicator-impl `name` body (145-147). `warmup_period` is covered
    /// already by `warmup_period_is_period_plus_one`.
    #[test]
    fn accessors_and_metadata() {
        let mut rsi = Rsi::new(14).unwrap();
        assert_eq!(rsi.period(), 14);
        assert_eq!(rsi.name(), "RSI");
        assert_eq!(rsi.value(), None);
        for i in 1..=15 {
            rsi.update(100.0 + f64::from(i));
        }
        assert!(rsi.value().is_some());
    }

    /// Cover the `ag == 0` branch (line 167) of the test-helper `rsi_naive`:
    /// when both `avg_gain` and `avg_loss` are 0 (a perfectly flat series),
    /// the helper must return the neutral 50.0. The proptest reference uses
    /// random inputs that essentially never hit zero gains AND zero losses
    /// simultaneously, leaving this branch dead in the helper.
    #[test]
    fn naive_helper_flat_series_yields_50() {
        let ks = rsi_naive(&[42.0; 20], 5);
        for r in ks.into_iter().skip(5) {
            assert_eq!(r.expect("ready after period+1 inputs"), 50.0);
        }
    }

    /// Cover the `100.0` branch (line 169) of the test-helper `rsi_naive`:
    /// strictly increasing prices give `avg_loss == 0` while `avg_gain > 0`,
    /// the textbook overbought saturation case. Random proptest inputs
    /// virtually never satisfy `al == 0 && ag != 0`, so this needs an
    /// explicit monotone series.
    #[test]
    fn naive_helper_monotone_up_yields_100() {
        let prices: Vec<f64> = (1..=20).map(f64::from).collect();
        let ks = rsi_naive(&prices, 5);
        for r in ks.into_iter().skip(5) {
            assert_eq!(r.expect("ready after period+1 inputs"), 100.0);
        }
    }

    #[test]
    fn warmup_period_is_period_plus_one() {
        let rsi = Rsi::new(14).unwrap();
        assert_eq!(rsi.warmup_period(), 15);
    }

    #[test]
    fn first_emission_at_index_period() {
        // RSI(14) needs 14 diffs => 15 inputs before first value.
        let prices: Vec<f64> = (1..=20).map(f64::from).collect();
        let mut rsi = Rsi::new(14).unwrap();
        let out = rsi.batch(&prices);
        // indices 0..14 -> None, index 14 -> first Some
        for x in &out[..14] {
            assert!(x.is_none());
        }
        assert!(out[14].is_some());
    }

    #[test]
    fn pure_uptrend_yields_rsi_100() {
        let prices: Vec<f64> = (1..=20).map(f64::from).collect();
        let mut rsi = Rsi::new(14).unwrap();
        let out = rsi.batch(&prices);
        // All diffs are positive => avg_loss == 0 => RSI == 100
        for v in out.iter().filter_map(|x| x.as_ref()) {
            assert_relative_eq!(*v, 100.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn pure_downtrend_yields_rsi_0() {
        let prices: Vec<f64> = (1..=20).rev().map(f64::from).collect();
        let mut rsi = Rsi::new(14).unwrap();
        let out = rsi.batch(&prices);
        for v in out.iter().filter_map(|x| x.as_ref()) {
            assert_relative_eq!(*v, 0.0, epsilon = 1e-9);
        }
    }

    #[test]
    fn flat_series_yields_rsi_50() {
        let prices = [10.0_f64; 30];
        let mut rsi = Rsi::new(14).unwrap();
        let out = rsi.batch(&prices);
        for v in out.iter().filter_map(|x| x.as_ref()) {
            assert_relative_eq!(*v, 50.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn classic_wilder_textbook_values() {
        // Wilder's original example from "New Concepts in Technical Trading Systems",
        // 14-period RSI. We compute the first value at index 14 and compare to the
        // value Wilder publishes (~70.46).
        // Source: classic textbook table, reproduced in many references (e.g. Investopedia).
        let prices = [
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03,
            45.61, 46.28, 46.28,
        ];
        let mut rsi = Rsi::new(14).unwrap();
        let out = rsi.batch(&prices);
        let first = out[14].expect("first RSI emitted at index period");
        assert_relative_eq!(first, 70.464, epsilon = 0.05);
    }

    #[test]
    fn rsi_stays_in_0_100_range() {
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.7).sin() * 10.0)
            .collect();
        let mut rsi = Rsi::new(14).unwrap();
        for x in rsi.batch(&prices).into_iter().flatten() {
            assert!((0.0..=100.0).contains(&x), "RSI out of range: {x}");
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut rsi = Rsi::new(5).unwrap();
        rsi.batch(&[1.0, 2.0, 3.0, 2.0, 4.0, 5.0, 6.0]);
        assert!(rsi.is_ready());
        rsi.reset();
        assert!(!rsi.is_ready());
        assert_eq!(rsi.update(1.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=40)
            .map(|i| (f64::from(i) * 0.3).sin() * 5.0 + f64::from(i))
            .collect();
        let mut a = Rsi::new(7).unwrap();
        let mut b = Rsi::new(7).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut rsi = Rsi::new(3).unwrap();
        rsi.batch(&[1.0, 2.0, 3.0, 4.0]);
        let before = rsi.value();
        assert!(before.is_some());
        assert_eq!(rsi.update(f64::NAN), before);
        assert_eq!(rsi.update(f64::INFINITY), before);
        assert_eq!(rsi.value(), before);
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(48))]
        #[test]
        fn rsi_matches_naive(
            period in 1usize..20,
            prices in proptest::collection::vec(1.0_f64..1000.0, 0..150),
        ) {
            let mut rsi = Rsi::new(period).unwrap();
            let got = rsi.batch(&prices);
            let want = rsi_naive(&prices, period);
            proptest::prop_assert_eq!(got.len(), want.len());
            for (g, w) in got.iter().zip(want.iter()) {
                match (g, w) {
                    (None, None) => {}
                    (Some(a), Some(b)) => proptest::prop_assert!(
                        (a - b).abs() < 1e-7,
                        "got={a} want={b}"
                    ),
                    _ => proptest::prop_assert!(false, "warmup mismatch"),
                }
            }
        }
    }
}
