//! Hasbrouck Information Share — each venue's contribution to price discovery.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Hasbrouck Information Share — the share of price-discovery attributable to the
/// **first** of two synchronised price series (e.g. the same asset on two venues).
///
/// ```text
/// rx_t = x_t − x_{t−1},  ry_t = y_t − y_{t−1}      (one-step price changes)
/// IS_x = var(rx) / ( var(rx) + var(ry) )           over the window, ∈ [0, 1]
/// ```
///
/// When the same instrument trades on several venues, Joel Hasbrouck's information
/// share measures how much each venue contributes to the common efficient price.
/// The venue whose innovations carry more of the variance leads price discovery.
/// This streaming form uses the **variance-ratio proxy**: the fraction of total
/// return variance contributed by series `x`. A reading above `0.5` means venue
/// `x` is the price leader; below `0.5`, the follower. (The full Hasbrouck measure
/// estimates a vector error-correction model and reports an upper/lower bound from
/// the Cholesky ordering; this proxy captures the leading idea without the VECM.)
///
/// The output is in `[0, 1]`; if both series are flat it reports the neutral `0.5`.
/// The first value lands after `period + 1` inputs. Each `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, HasbrouckInformationShare};
///
/// let mut indicator = HasbrouckInformationShare::new(20).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     // Venue x moves a lot, venue y barely moves -> x leads.
///     let x = (f64::from(i) * 0.5).sin() * 10.0;
///     let y = (f64::from(i) * 0.5).sin() * 1.0;
///     last = indicator.update((x, y));
/// }
/// assert!(last.unwrap() > 0.8);
/// ```
#[derive(Debug, Clone)]
pub struct HasbrouckInformationShare {
    period: usize,
    prev: Option<(f64, f64)>,
    window: VecDeque<(f64, f64)>,
    sum_x: f64,
    sum_y: f64,
    sum_xx: f64,
    sum_yy: f64,
}

impl HasbrouckInformationShare {
    /// Construct a Hasbrouck information share over `period` return pairs.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPeriod`] if `period < 2` (variance needs two
    /// returns).
    pub fn new(period: usize) -> Result<Self> {
        if period < 2 {
            return Err(Error::InvalidPeriod {
                message: "information share needs period >= 2",
            });
        }
        Ok(Self {
            period,
            prev: None,
            window: VecDeque::with_capacity(period),
            sum_x: 0.0,
            sum_y: 0.0,
            sum_xx: 0.0,
            sum_yy: 0.0,
        })
    }

    /// Configured window of return pairs.
    pub const fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for HasbrouckInformationShare {
    type Input = (f64, f64);
    type Output = f64;

    fn update(&mut self, input: (f64, f64)) -> Option<f64> {
        let (x, y) = input;
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        let Some((px, py)) = self.prev else {
            self.prev = Some((x, y));
            return None;
        };
        self.prev = Some((x, y));
        let (rx, ry) = (x - px, y - py);
        if self.window.len() == self.period {
            let (ox, oy) = self.window.pop_front().expect("non-empty");
            self.sum_x -= ox;
            self.sum_y -= oy;
            self.sum_xx -= ox * ox;
            self.sum_yy -= oy * oy;
        }
        self.window.push_back((rx, ry));
        self.sum_x += rx;
        self.sum_y += ry;
        self.sum_xx += rx * rx;
        self.sum_yy += ry * ry;
        if self.window.len() < self.period {
            return None;
        }
        let n = self.period as f64;
        let var_x = (self.sum_xx / n - (self.sum_x / n).powi(2)).max(0.0);
        let var_y = (self.sum_yy / n - (self.sum_y / n).powi(2)).max(0.0);
        let total = var_x + var_y;
        Some(if total > 0.0 { var_x / total } else { 0.5 })
    }

    fn reset(&mut self) {
        self.prev = None;
        self.window.clear();
        self.sum_x = 0.0;
        self.sum_y = 0.0;
        self.sum_xx = 0.0;
        self.sum_yy = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "HasbrouckInformationShare"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_period_below_two() {
        assert!(matches!(
            HasbrouckInformationShare::new(1),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(HasbrouckInformationShare::new(2).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let h = HasbrouckInformationShare::new(20).unwrap();
        assert_eq!(h.period(), 20);
        assert_eq!(h.warmup_period(), 21);
        assert_eq!(h.name(), "HasbrouckInformationShare");
        assert!(!h.is_ready());
    }

    #[test]
    fn warmup_needs_period_plus_one() {
        let mut h = HasbrouckInformationShare::new(3).unwrap();
        assert_eq!(h.update((1.0, 1.0)), None);
        assert_eq!(h.update((2.0, 2.0)), None);
        assert_eq!(h.update((3.0, 2.5)), None);
        assert!(h.update((4.0, 3.0)).is_some());
    }

    #[test]
    fn loud_venue_leads() {
        // x is far more volatile than y -> x holds nearly all the share.
        let pairs: Vec<(f64, f64)> = (0..40)
            .map(|i| {
                (
                    (f64::from(i) * 0.5).sin() * 10.0,
                    (f64::from(i) * 0.5).sin() * 1.0,
                )
            })
            .collect();
        let last = HasbrouckInformationShare::new(20)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!(last > 0.8, "the loud venue should lead, got {last}");
    }

    #[test]
    fn equal_venues_split_evenly() {
        // Independent but equal-variance moves -> share near 0.5.
        let pairs: Vec<(f64, f64)> = (0..200)
            .map(|i| {
                (
                    (f64::from(i) * 0.5).sin() * 5.0,
                    (f64::from(i) * 0.5).cos() * 5.0,
                )
            })
            .collect();
        for v in HasbrouckInformationShare::new(40)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
        {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn flat_series_is_half() {
        let pairs: Vec<(f64, f64)> = (0..20).map(|_| (7.0, 9.0)).collect();
        let last = HasbrouckInformationShare::new(5)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, 0.5, epsilon = 1e-12);
    }

    #[test]
    fn reset_clears_state() {
        let mut h = HasbrouckInformationShare::new(4).unwrap();
        h.batch(&[(1.0, 1.0), (2.0, 2.0), (3.0, 3.0), (4.0, 4.0), (5.0, 5.0)]);
        assert!(h.is_ready());
        h.reset();
        assert!(!h.is_ready());
        assert_eq!(h.update((1.0, 1.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let pairs: Vec<(f64, f64)> = (0..120)
            .map(|i| {
                let t = f64::from(i);
                (t.sin() * 5.0, (t * 0.5).cos() * 3.0)
            })
            .collect();
        let batch = HasbrouckInformationShare::new(20).unwrap().batch(&pairs);
        let mut h = HasbrouckInformationShare::new(20).unwrap();
        let streamed: Vec<_> = pairs.iter().map(|p| h.update(*p)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn non_finite_input_returns_none() {
        let mut h = HasbrouckInformationShare::new(2).unwrap();
        assert_eq!(h.update((f64::NAN, 1.0)), None);
        assert_eq!(h.update((1.0, f64::INFINITY)), None);
        // First finite tick seeds prev; two more returns fill the window.
        assert_eq!(h.update((1.0, 1.0)), None);
        assert_eq!(h.update((2.0, 3.0)), None);
        assert!(h.update((3.0, 4.0)).is_some());
    }
}
