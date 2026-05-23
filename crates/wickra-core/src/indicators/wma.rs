//! Weighted Moving Average (linear weights).

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Weighted Moving Average with linear weights `1, 2, ..., period`.
///
/// Output is `sum(weight_i * price_i) / sum(weights)`. Maintained incrementally in
/// O(1) by keeping the rolling sum of values and the rolling weighted sum.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, Wma};
///
/// let mut indicator = Wma::new(3).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Wma {
    period: usize,
    window: VecDeque<f64>,
    weight_sum: f64, // sum_i (weight_i * value_i)
    value_sum: f64,  // sum_i (value_i)
    weights_total: f64,
}

impl Wma {
    /// Construct a new WMA with the given window length.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        let n = period as f64;
        let weights_total = n * (n + 1.0) / 2.0;
        Ok(Self {
            period,
            window: VecDeque::with_capacity(period),
            weight_sum: 0.0,
            value_sum: 0.0,
            weights_total,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if available.
    pub fn value(&self) -> Option<f64> {
        if self.window.len() == self.period {
            Some(self.weight_sum / self.weights_total)
        } else {
            None
        }
    }
}

impl Indicator for Wma {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.value();
        }
        if self.window.len() < self.period {
            // Warmup. Just accumulate; compute weight_sum once when the window first
            // becomes full to avoid having to track changing weights during warmup.
            self.window.push_back(input);
            self.value_sum += input;
            if self.window.len() == self.period {
                self.weight_sum = self
                    .window
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (i as f64 + 1.0) * v)
                    .sum();
            }
            return self.value();
        }
        // Steady state: slide the window. With weights [1, 2, ..., period],
        //   new_weight_sum = old_weight_sum - old_value_sum + period * new_input
        // because every retained element's weight drops by one and the newcomer
        // enters at weight = period. Order matters: subtract `value_sum` BEFORE
        // updating it.
        let oldest = self.window.pop_front().expect("window non-empty");
        self.weight_sum = self.weight_sum - self.value_sum + self.period as f64 * input;
        self.value_sum = self.value_sum - oldest + input;
        self.window.push_back(input);
        self.value()
    }

    fn reset(&mut self) {
        self.window.clear();
        self.weight_sum = 0.0;
        self.value_sum = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "WMA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    /// Reference implementation: explicit weighted average over a window.
    fn wma_naive(prices: &[f64], period: usize) -> Vec<Option<f64>> {
        let weights_total = (period as f64) * (period as f64 + 1.0) / 2.0;
        prices
            .iter()
            .enumerate()
            .map(|(i, _)| {
                if i + 1 < period {
                    None
                } else {
                    let window = &prices[i + 1 - period..=i];
                    let s: f64 = window
                        .iter()
                        .enumerate()
                        .map(|(j, p)| (j as f64 + 1.0) * p)
                        .sum();
                    Some(s / weights_total)
                }
            })
            .collect()
    }

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(Wma::new(0), Err(Error::PeriodZero)));
    }

    /// Cover the const accessor `period` (56-58) and the Indicator-impl
    /// `warmup_period` (111-113) + `name` (119-121). Existing tests never
    /// inspect these metadata methods.
    #[test]
    fn accessors_and_metadata() {
        let wma = Wma::new(7).unwrap();
        assert_eq!(wma.period(), 7);
        assert_eq!(wma.warmup_period(), 7);
        assert_eq!(wma.name(), "WMA");
    }

    #[test]
    fn warmup_returns_none() {
        let mut wma = Wma::new(3).unwrap();
        assert_eq!(wma.update(1.0), None);
        assert_eq!(wma.update(2.0), None);
        // WMA(3) of [1,2,3]: oldest = 1 (weight 1), middle = 2 (weight 2), newest = 3 (weight 3)
        // -> (1*1 + 2*2 + 3*3) / (1+2+3) = 14/6
        assert_relative_eq!(wma.update(3.0).unwrap(), 14.0 / 6.0, epsilon = 1e-12);
    }

    #[test]
    fn known_values_period_4() {
        // WMA(4) weights 1,2,3,4 (total 10); inputs [1,2,3,4]:
        // (1*1 + 2*2 + 3*3 + 4*4) / 10 = (1+4+9+16)/10 = 30/10 = 3.0
        let mut wma = Wma::new(4).unwrap();
        let v = wma.batch(&[1.0, 2.0, 3.0, 4.0]);
        assert_relative_eq!(v[3].unwrap(), 3.0, epsilon = 1e-12);
    }

    #[test]
    fn matches_naive_over_random_inputs() {
        let prices: Vec<f64> = (1..=30).map(|i| f64::from(i) * 1.7 - 5.0).collect();
        let mut wma = Wma::new(7).unwrap();
        let got = wma.batch(&prices);
        let want = wma_naive(&prices, 7);
        for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
            // Same warmup — emission shape must agree at every index.
            assert_eq!(g.is_some(), w.is_some(), "warmup mismatch at index {i}");
            if let (Some(a), Some(b)) = (g, w) {
                assert_relative_eq!(*a, *b, epsilon = 1e-9);
            }
        }
    }

    #[test]
    fn period_one_is_pass_through() {
        let mut wma = Wma::new(1).unwrap();
        assert_relative_eq!(wma.update(5.5).unwrap(), 5.5, epsilon = 1e-12);
        assert_relative_eq!(wma.update(7.5).unwrap(), 7.5, epsilon = 1e-12);
    }

    #[test]
    fn reset_clears_state() {
        let mut wma = Wma::new(4).unwrap();
        wma.batch(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(wma.is_ready());
        wma.reset();
        assert!(!wma.is_ready());
        assert_eq!(wma.update(10.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=20).map(|i| f64::from(i) * 0.5).collect();
        let mut a = Wma::new(5).unwrap();
        let mut b = Wma::new(5).unwrap();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|p| b.update(*p)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn ignores_non_finite_input_but_keeps_state() {
        let mut wma = Wma::new(3).unwrap();
        wma.update(1.0);
        wma.update(2.0);
        let ready = wma.update(3.0).expect("WMA(3) ready after three inputs");
        // Non-finite inputs return the last value without mutating the window.
        assert_eq!(wma.update(f64::NAN), Some(ready));
        assert_eq!(wma.update(f64::INFINITY), Some(ready));
        // The window still holds 1, 2, 3 -> next real input slides it to 2, 3, 4.
        assert_relative_eq!(
            wma.update(4.0).unwrap(),
            (2.0 * 1.0 + 3.0 * 2.0 + 4.0 * 3.0) / 6.0,
            epsilon = 1e-12
        );
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(48))]
        #[test]
        fn proptest_matches_naive(
            period in 1usize..15,
            prices in proptest::collection::vec(-500.0_f64..500.0, 0..120),
        ) {
            let mut wma = Wma::new(period).unwrap();
            let got = wma.batch(&prices);
            let want = wma_naive(&prices, period);
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
