//! Anchored Relative Strength Index.

use crate::traits::Indicator;

/// Anchored RSI — a cumulative Relative Strength Index whose averaging begins at
/// a user-chosen anchor bar rather than over a fixed Wilder period.
///
/// Where [`crate::Rsi`] uses Wilder's `period`-length smoothing, Anchored RSI
/// accumulates *every* up- and down-move since the anchor with equal weight, so
/// it answers "what is the RSI of the entire move since the anchor point?". The
/// running relative strength is `Σ gains / Σ losses` over all bars in the
/// current anchor window (the bar count cancels, so this equals
/// `avg_gain / avg_loss`):
///
/// ```text
/// RSI_t = 100 - 100 / (1 + Σ_{i ≥ anchor} gain_i / Σ_{i ≥ anchor} loss_i)
/// ```
///
/// As with [`crate::AnchoredVwap`], the anchor is chosen at runtime:
/// [`AnchoredRsi::set_anchor`] re-anchors at the **next** bar that arrives,
/// clearing the running sums. Because RSI needs a price *change*, the first bar
/// of a fresh anchor window only seeds the previous close and emits `None`; the
/// first value follows on the second bar (warmup period 2).
///
/// Saturation follows the standard convention: a window with no losses yet (and
/// at least one gain) reads 100, no gains yet reads 0, and a perfectly flat
/// window reads the neutral 50. Non-finite inputs are ignored, leaving the last
/// value unchanged.
///
/// # Example
///
/// ```
/// use wickra_core::{AnchoredRsi, Indicator};
///
/// let mut indicator = AnchoredRsi::new();
/// let mut last = None;
/// for i in 0..80 {
///     let price = 100.0 + (f64::from(i) * 0.5).sin() * 5.0;
///     // Re-anchor at bar 40 (e.g. a major swing low).
///     if i == 40 {
///         indicator.set_anchor();
///     }
///     last = indicator.update(price);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct AnchoredRsi {
    prev_close: Option<f64>,
    sum_gain: f64,
    sum_loss: f64,
    last_value: Option<f64>,
    pending_anchor: bool,
}

impl AnchoredRsi {
    /// Construct a fresh Anchored RSI. The first bar to arrive is the anchor.
    pub const fn new() -> Self {
        Self {
            prev_close: None,
            sum_gain: 0.0,
            sum_loss: 0.0,
            last_value: None,
            pending_anchor: false,
        }
    }

    /// Mark a re-anchor: the **next** [`Indicator::update`] call clears the
    /// running sums and previous close before folding in its own bar, starting
    /// a fresh anchored window.
    pub fn set_anchor(&mut self) {
        self.pending_anchor = true;
    }

    /// Current anchored RSI value if at least one price change has been
    /// observed in the current anchor window.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }

    fn rsi_from_sums(sum_gain: f64, sum_loss: f64) -> f64 {
        if sum_loss == 0.0 {
            if sum_gain == 0.0 {
                // No movement at all -> RSI undefined; standard convention returns 50.
                50.0
            } else {
                100.0
            }
        } else {
            let rs = sum_gain / sum_loss;
            100.0 - 100.0 / (1.0 + rs)
        }
    }
}

impl Indicator for AnchoredRsi {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.last_value;
        }

        if self.pending_anchor {
            self.prev_close = None;
            self.sum_gain = 0.0;
            self.sum_loss = 0.0;
            self.last_value = None;
            self.pending_anchor = false;
        }

        let Some(prev) = self.prev_close else {
            self.prev_close = Some(input);
            return None;
        };
        self.prev_close = Some(input);

        let diff = input - prev;
        if diff > 0.0 {
            self.sum_gain += diff;
        } else if diff < 0.0 {
            self.sum_loss -= diff;
        }

        let value = Self::rsi_from_sums(self.sum_gain, self.sum_loss);
        self.last_value = Some(value);
        Some(value)
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.sum_gain = 0.0;
        self.sum_loss = 0.0;
        self.last_value = None;
        self.pending_anchor = false;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "AnchoredRSI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn accessors_and_metadata() {
        let indicator = AnchoredRsi::new();
        assert_eq!(indicator.name(), "AnchoredRSI");
        assert_eq!(indicator.warmup_period(), 2);
        assert_eq!(indicator.value(), None);
        assert!(!indicator.is_ready());
    }

    #[test]
    fn first_bar_seeds_and_returns_none() {
        let mut indicator = AnchoredRsi::new();
        assert_eq!(indicator.update(100.0), None);
        assert!(!indicator.is_ready());
        // Second bar produces the first value.
        assert!(indicator.update(101.0).is_some());
        assert!(indicator.is_ready());
    }

    #[test]
    fn pure_uptrend_saturates_at_100() {
        let mut indicator = AnchoredRsi::new();
        let out = indicator.batch(&[10.0, 11.0, 12.0, 13.0]);
        assert_relative_eq!(out[3].unwrap(), 100.0, epsilon = 1e-12);
    }

    #[test]
    fn pure_downtrend_saturates_at_0() {
        let mut indicator = AnchoredRsi::new();
        let out = indicator.batch(&[13.0, 12.0, 11.0, 10.0]);
        assert_relative_eq!(out[3].unwrap(), 0.0, epsilon = 1e-12);
    }

    #[test]
    fn flat_window_reads_50() {
        let mut indicator = AnchoredRsi::new();
        let out = indicator.batch(&[42.0, 42.0, 42.0]);
        assert_relative_eq!(out[2].unwrap(), 50.0, epsilon = 1e-12);
    }

    #[test]
    fn cumulative_reference_values() {
        // prices 10 -> 11 (+1) -> 9 (-2) -> 12 (+3)
        // after bar2: sum_gain=1, sum_loss=2 -> rs=0.5 -> 100 - 100/1.5 = 33.3333
        // after bar3: sum_gain=4, sum_loss=2 -> rs=2.0 -> 100 - 100/3   = 66.6667
        let mut indicator = AnchoredRsi::new();
        let out = indicator.batch(&[10.0, 11.0, 9.0, 12.0]);
        assert_relative_eq!(out[1].unwrap(), 100.0, epsilon = 1e-9);
        assert_relative_eq!(out[2].unwrap(), 33.333_333_333, epsilon = 1e-6);
        assert_relative_eq!(out[3].unwrap(), 66.666_666_666, epsilon = 1e-6);
    }

    #[test]
    fn set_anchor_clears_old_window() {
        // Downtrend, then re-anchor and pump an uptrend: the new window must
        // read 100, not the blended value.
        let mut indicator = AnchoredRsi::new();
        indicator.batch(&[20.0, 19.0, 18.0, 17.0]);
        assert_relative_eq!(indicator.value().unwrap(), 0.0, epsilon = 1e-12);
        indicator.set_anchor();
        // First bar after anchor re-seeds (None), second bar emits.
        assert_eq!(indicator.update(50.0), None);
        let after = indicator.update(51.0).unwrap();
        assert_relative_eq!(after, 100.0, epsilon = 1e-12);
    }

    #[test]
    fn set_anchor_before_first_bar_acts_as_normal_start() {
        let mut indicator = AnchoredRsi::new();
        indicator.set_anchor();
        assert_eq!(indicator.update(10.0), None);
        assert_relative_eq!(indicator.update(11.0).unwrap(), 100.0, epsilon = 1e-12);
    }

    #[test]
    fn ignores_non_finite_input() {
        let mut indicator = AnchoredRsi::new();
        indicator.batch(&[10.0, 11.0, 12.0]);
        let before = indicator.value();
        assert!(before.is_some());
        assert_eq!(indicator.update(f64::NAN), before);
        assert_eq!(indicator.update(f64::INFINITY), before);
        assert_eq!(indicator.value(), before);
    }

    #[test]
    fn non_finite_before_any_bar_returns_none() {
        let mut indicator = AnchoredRsi::new();
        assert_eq!(indicator.update(f64::NAN), None);
        assert!(!indicator.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = AnchoredRsi::new();
        indicator.batch(&[10.0, 11.0, 12.0]);
        assert!(indicator.is_ready());
        indicator.reset();
        assert!(!indicator.is_ready());
        assert_eq!(indicator.value(), None);
        assert_eq!(indicator.update(50.0), None);
    }

    #[test]
    fn stays_in_0_100_range() {
        let prices: Vec<f64> = (0..200)
            .map(|i| 100.0 + (f64::from(i) * 0.7).sin() * 10.0)
            .collect();
        let mut indicator = AnchoredRsi::new();
        for value in indicator.batch(&prices).into_iter().flatten() {
            assert!((0.0..=100.0).contains(&value), "RSI out of range: {value}");
        }
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=40)
            .map(|i| (f64::from(i) * 0.3).sin() * 5.0 + f64::from(i))
            .collect();
        let mut a = AnchoredRsi::new();
        let mut b = AnchoredRsi::new();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }
}
