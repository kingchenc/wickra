//! Simple Moving Average.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Simple Moving Average over a fixed window.
///
/// Maintains a rolling sum so each update is O(1). Output equals
/// `sum(last `period` prices) / period` once the window is full; `None` before.
#[derive(Debug, Clone)]
pub struct Sma {
    period: usize,
    window: VecDeque<f64>,
    sum: f64,
}

impl Sma {
    /// Construct a new SMA with the given window length.
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
            window: VecDeque::with_capacity(period),
            sum: 0.0,
        })
    }

    /// Configured window length.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Current value if available.
    pub fn value(&self) -> Option<f64> {
        if self.window.len() == self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }
}

impl Indicator for Sma {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, input: f64) -> Option<f64> {
        if !input.is_finite() {
            return self.value();
        }
        if self.window.len() == self.period {
            // Drop the oldest from the sum to keep numerical drift bounded by recomputing
            // the sum after each pop; a single subtract works in O(1) and is acceptable
            // here because we use f64 throughout.
            let old = self.window.pop_front().expect("window non-empty");
            self.sum -= old;
        }
        self.window.push_back(input);
        self.sum += input;
        self.value()
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "SMA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn new_rejects_zero_period() {
        assert!(matches!(Sma::new(0), Err(Error::PeriodZero)));
    }

    #[test]
    fn warmup_returns_none() {
        let mut sma = Sma::new(3).unwrap();
        assert_eq!(sma.update(1.0), None);
        assert_eq!(sma.update(2.0), None);
        assert_eq!(sma.update(3.0), Some(2.0));
    }

    #[test]
    fn rolls_window_after_full() {
        let mut sma = Sma::new(3).unwrap();
        let out: Vec<_> = [1.0, 2.0, 3.0, 4.0, 5.0]
            .iter()
            .map(|p| sma.update(*p))
            .collect();
        assert_eq!(out, vec![None, None, Some(2.0), Some(3.0), Some(4.0)]);
    }

    #[test]
    fn period_one_is_pass_through() {
        let mut sma = Sma::new(1).unwrap();
        assert_eq!(sma.update(5.0), Some(5.0));
        assert_eq!(sma.update(10.0), Some(10.0));
    }

    #[test]
    fn ignores_non_finite_input_but_keeps_state() {
        let mut sma = Sma::new(3).unwrap();
        sma.update(1.0);
        sma.update(2.0);
        sma.update(3.0);
        assert_eq!(sma.update(f64::NAN), Some(2.0));
        assert_eq!(sma.update(f64::INFINITY), Some(2.0));
        // Non-finite inputs were not pushed; window still holds 1,2,3.
        assert_eq!(sma.update(6.0), Some((2.0 + 3.0 + 6.0) / 3.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut sma = Sma::new(3).unwrap();
        sma.batch(&[1.0, 2.0, 3.0]);
        assert!(sma.is_ready());
        sma.reset();
        assert!(!sma.is_ready());
        assert_eq!(sma.update(10.0), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (1..=20).map(f64::from).collect();
        let mut a = Sma::new(5).unwrap();
        let batch = a.batch(&prices);
        let mut b = Sma::new(5).unwrap();
        let streamed: Vec<_> = prices.iter().map(|p| b.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn known_reference_values() {
        // SMA(3) of [2, 4, 6, 8, 10] -> [_, _, 4, 6, 8]
        let mut sma = Sma::new(3).unwrap();
        let out = sma.batch(&[2.0, 4.0, 6.0, 8.0, 10.0]);
        assert_eq!(out[2], Some(4.0));
        assert_eq!(out[3], Some(6.0));
        assert_eq!(out[4], Some(8.0));
    }

    #[test]
    fn constant_series_yields_constant_sma() {
        let mut sma = Sma::new(5).unwrap();
        let v = sma.batch(&[7.0; 10]);
        for x in v.iter().skip(4) {
            assert_relative_eq!(x.unwrap(), 7.0, epsilon = 1e-12);
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(64))]
        #[test]
        fn sma_matches_naive_definition(
            period in 1usize..20,
            prices in proptest::collection::vec(-1000.0_f64..1000.0, 0..200),
        ) {
            let mut sma = Sma::new(period).unwrap();
            let stream: Vec<_> = prices.iter().map(|p| sma.update(*p)).collect();
            for (i, got) in stream.iter().enumerate() {
                if i + 1 < period {
                    proptest::prop_assert!(got.is_none());
                } else {
                    let window = &prices[i + 1 - period..=i];
                    let expected = window.iter().sum::<f64>() / period as f64;
                    let actual = got.expect("ready");
                    proptest::prop_assert!(
                        (actual - expected).abs() < 1e-9,
                        "i={i} actual={actual} expected={expected}"
                    );
                }
            }
        }
    }
}
