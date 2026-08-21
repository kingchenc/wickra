//! Numerically stable rolling first and second moments.
//!
//! The textbook incremental variance `E[x²] − E[x]²` is cheap and O(1), but it
//! cancels catastrophically once the values are large relative to their spread —
//! which is exactly the shape of a price series. Measured against a two-pass
//! reference over a 20-bar window of `level + sin(i · 0.7) · spread`:
//!
//! | level | spread | `E[x²] − E[x]²` | shifted |
//! |-------|--------|-----------------|---------|
//! | 1e2   | 1      | 3.9e-12         | 3.3e-16 |
//! | 1e2   | 0.01   | 9.0e-08         | 5.1e-16 |
//! | 1e5   | 1      | 4.3e-06         | 1.6e-16 |
//! | 1e5   | 0.01   | **9.4e-02**     | 1.3e-16 |
//! | 1e8   | 1      | **1.0**         | 4.9e-16 |
//!
//! At 1e8 the naive form does not merely lose digits: `sum_sq / n` and
//! `mean · mean` agree to every bit, the difference is exactly zero, and the
//! `.max(0.0)` clamp that was meant to absorb rounding noise hides the collapse
//! completely. A squeeze indicator then reports a permanent squeeze and a
//! z-score divides by zero dispersion. Bitcoin at 100 000 on one-second bars sits
//! in the 1e5 / 0.01 row.
//!
//! [`ShiftedMoments`] fixes this by accumulating the moments of `x − offset`
//! instead of `x`, where `offset` is a value from inside the window. The
//! subtraction is exact in the common case and near-exact otherwise, so the
//! cancellation that remains is between quantities of order `spread²` rather
//! than `level²`. Cost is unchanged: one add and one subtract per bar, plus an
//! amortised reseed.
//!
//! It deliberately does **not** own the window. Every indicator that needs this
//! already keeps its own values (most in a `VecDeque`, some in a ring buffer),
//! often for other purposes too, so the accumulator attaches to whatever the
//! caller already has and is driven by `push` / `evict`.

/// Rolling `Σ(x − offset)` and `Σ(x − offset)²` for a fixed-length window.
///
/// The caller owns the window and is responsible for calling [`Self::evict`]
/// with a value that was previously [`pushed`](Self::push), and for passing a
/// consistent `n` to the query methods.
#[derive(Debug, Clone)]
pub(crate) struct ShiftedMoments {
    /// Reference point the accumulated moments are relative to. Chosen from
    /// inside the window so `x − offset` stays on the order of the spread.
    offset: f64,
    /// Whether `offset` has been chosen yet. The first pushed value seeds it.
    seeded: bool,
    /// `Σ(x − offset)` over the live window.
    sum: f64,
    /// `Σ(x − offset)²` over the live window.
    sum_sq: f64,
    /// Pushes since the last reseed, used to bound both accumulated drift and
    /// how far `offset` may have wandered from the live window.
    pushes_since_reseed: usize,
}

impl ShiftedMoments {
    /// A fresh accumulator with no reference point yet.
    pub(crate) const fn new() -> Self {
        Self {
            offset: 0.0,
            seeded: false,
            sum: 0.0,
            sum_sq: 0.0,
            pushes_since_reseed: 0,
        }
    }

    /// Add `value` to the accumulated moments.
    ///
    /// The first value seeds the reference point, which makes its own
    /// contribution exactly zero.
    pub(crate) fn push(&mut self, value: f64) {
        if !self.seeded {
            self.offset = value;
            self.seeded = true;
        }
        let d = value - self.offset;
        self.sum += d;
        self.sum_sq += d * d;
        self.pushes_since_reseed += 1;
    }

    /// Remove `value` from the accumulated moments.
    ///
    /// `value` must be one previously passed to [`Self::push`] and not yet
    /// evicted; passing anything else silently corrupts the moments, exactly as
    /// a hand-rolled `sum -= old` would.
    pub(crate) fn evict(&mut self, value: f64) {
        let d = value - self.offset;
        self.sum -= d;
        self.sum_sq -= d * d;
    }

    /// Mean of the `n` values currently in the window.
    pub(crate) fn mean(&self, n: usize) -> f64 {
        self.offset + self.sum / n as f64
    }

    /// Population variance (divisor `n`) of the `n` values in the window.
    ///
    /// Clamped at zero: the residual cancellation is now on the order of
    /// `spread² · ε`, so a negative result is pure rounding noise rather than
    /// the sign of a collapsed computation.
    pub(crate) fn variance(&self, n: usize) -> f64 {
        let n = n as f64;
        let mean_shifted = self.sum / n;
        (self.sum_sq / n - mean_shifted * mean_shifted).max(0.0)
    }

    /// Population standard deviation of the `n` values in the window.
    pub(crate) fn std_dev(&self, n: usize) -> f64 {
        self.variance(n).sqrt()
    }

    /// Whether enough pushes have accumulated to justify a reseed.
    ///
    /// Reseeding once per window keeps the reference point at most one window
    /// old, so `offset` can never drift further from the live values than the
    /// price moved across that window — the same order as the spread the
    /// accumulator is measuring. The cost is `O(period)` once every `period`
    /// pushes, i.e. amortised `O(1)`.
    pub(crate) const fn needs_reseed(&self, period: usize) -> bool {
        self.pushes_since_reseed >= period
    }

    /// Recompute both moments from the live window, re-centring `offset` on its
    /// mean.
    ///
    /// This is the only place either accumulator is derived from scratch, so it
    /// simultaneously bounds the drift that repeated add/subtract accumulates
    /// and re-anchors the reference point. `values` must yield exactly the live
    /// window.
    pub(crate) fn reseed<I>(&mut self, values: I)
    where
        I: IntoIterator<Item = f64> + Clone,
    {
        let mut count = 0_usize;
        let mut total = 0.0;
        for v in values.clone() {
            total += v;
            count += 1;
        }
        if count == 0 {
            self.reset();
            return;
        }
        let mean = total / count as f64;
        self.offset = mean;
        self.seeded = true;
        self.sum = 0.0;
        self.sum_sq = 0.0;
        for v in values {
            let d = v - mean;
            self.sum += d;
            self.sum_sq += d * d;
        }
        self.pushes_since_reseed = 0;
    }

    /// Drop every accumulated value and the reference point.
    pub(crate) fn reset(&mut self) {
        self.offset = 0.0;
        self.seeded = false;
        self.sum = 0.0;
        self.sum_sq = 0.0;
        self.pushes_since_reseed = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// Two-pass population variance — the numerically stable reference.
    fn reference_variance(window: &[f64]) -> f64 {
        let n = window.len() as f64;
        let mean = window.iter().sum::<f64>() / n;
        window.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / n
    }

    /// Drive a rolling window of `period` through `values`, returning the final
    /// accumulator and the live window.
    fn roll(values: &[f64], period: usize, reseed: bool) -> (ShiftedMoments, Vec<f64>) {
        let mut moments = ShiftedMoments::new();
        let mut window: Vec<f64> = Vec::with_capacity(period);
        for &v in values {
            if window.len() == period {
                moments.evict(window.remove(0));
            }
            window.push(v);
            moments.push(v);
            if reseed && moments.needs_reseed(period) {
                moments.reseed(window.iter().copied());
            }
        }
        (moments, window)
    }

    #[test]
    fn matches_the_two_pass_reference_at_extreme_price_levels() {
        for level in [1.0e2_f64, 1.0e5, 1.0e8] {
            for spread in [1.0_f64, 0.01] {
                let values: Vec<f64> = (0..60)
                    .map(|i| level + (f64::from(i) * 0.7).sin() * spread)
                    .collect();
                let (moments, window) = roll(&values, 20, true);
                let want = reference_variance(&window);
                assert_relative_eq!(moments.variance(20), want, max_relative = 1e-9);
            }
        }
    }

    #[test]
    fn mean_matches_the_window_mean() {
        let values: Vec<f64> = (0..40).map(|i| 1.0e6 + f64::from(i) * 0.25).collect();
        let (moments, window) = roll(&values, 12, true);
        let want = window.iter().sum::<f64>() / 12.0;
        assert_relative_eq!(moments.mean(12), want, max_relative = 1e-12);
    }

    #[test]
    fn variance_matches_a_known_data_set() {
        let values = [2.0_f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let (moments, _) = roll(&values, 8, false);
        // Population variance of this classic set is exactly 4.
        assert_relative_eq!(moments.variance(8), 4.0, epsilon = 1e-12);
    }

    #[test]
    fn std_dev_is_the_root_of_the_variance() {
        let values: Vec<f64> = (0..30)
            .map(|i| 100.0 + (f64::from(i) * 0.5).sin())
            .collect();
        let (moments, _) = roll(&values, 10, true);
        assert_relative_eq!(
            moments.std_dev(10),
            moments.variance(10).sqrt(),
            epsilon = 1e-15
        );
    }

    #[test]
    fn a_constant_window_has_zero_variance() {
        let (moments, _) = roll(&[1.0e8_f64; 40], 16, true);
        assert_relative_eq!(moments.variance(16), 0.0, epsilon = 1e-12);
    }

    #[test]
    fn reseeding_bounds_drift_over_a_long_stream() {
        // Without the periodic reseed the incremental add/subtract accumulates
        // rounding noise; with it the result must still track the reference.
        let values: Vec<f64> = (0..200_000)
            .map(|i| 1.0e5 + (f64::from(i % 1000) * 0.01).sin())
            .collect();
        let (moments, window) = roll(&values, 50, true);
        assert_relative_eq!(
            moments.variance(50),
            reference_variance(&window),
            max_relative = 1e-9
        );
    }

    #[test]
    fn reseed_on_an_empty_window_clears_the_accumulator() {
        let mut moments = ShiftedMoments::new();
        moments.push(5.0);
        moments.reseed(std::iter::empty());
        assert_relative_eq!(moments.variance(1), 0.0, epsilon = 1e-15);
        assert_relative_eq!(moments.mean(1), 0.0, epsilon = 1e-15);
    }

    #[test]
    fn reset_clears_the_reference_point() {
        let mut moments = ShiftedMoments::new();
        moments.push(1.0e8);
        moments.reset();
        // A fresh reference point must be taken from the next value, not kept
        // from before the reset.
        moments.push(1.0);
        moments.push(3.0);
        assert_relative_eq!(moments.mean(2), 2.0, epsilon = 1e-15);
        assert_relative_eq!(moments.variance(2), 1.0, epsilon = 1e-15);
    }

    #[test]
    fn needs_reseed_triggers_once_per_window() {
        let mut moments = ShiftedMoments::new();
        for _ in 0..4 {
            moments.push(1.0);
        }
        assert!(!moments.needs_reseed(5));
        moments.push(1.0);
        assert!(moments.needs_reseed(5));
    }
}
