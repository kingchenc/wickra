//! Bomar Bands — adaptive percentage bands that contain a target fraction of
//! recent price.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::indicators::rolling_quantile::quantile_sorted;
use crate::traits::Indicator;

/// Bomar Bands output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BomarBandsOutput {
    /// Upper band: `middle + |middle| · p`.
    pub upper: f64,
    /// Middle line: the simple moving average over the window.
    pub middle: f64,
    /// Lower band: `middle − |middle| · p`.
    pub lower: f64,
}

/// Bomar Bands: percentage bands whose width adapts so that a fixed `coverage`
/// fraction of recent closes falls inside them.
///
/// The Bomar Bands predate Bollinger Bands; John Bollinger cites them as an
/// inspiration — percentage bands around a moving average, with the percentage
/// tuned so a fixed share (classically ~85%) of price stayed within. Wickra
/// realises that idea deterministically: the half-width is the `coverage`
/// quantile of the relative deviations from the midline, so by construction
/// `coverage` of the window's closes lie inside the bands.
///
/// ```text
/// middle = SMA(close, period)
/// dev_i  = | close_i / middle − 1 |          // relative distance from midline
/// p      = coverage-quantile of { dev_i }     // type-7 interpolation
/// upper  = middle + |middle| · p
/// lower  = middle − |middle| · p
/// ```
///
/// Unlike the fixed-percentage [`MaEnvelope`](crate::MaEnvelope), the offset
/// here is data-driven: the bands widen in turbulent regimes and tighten in
/// quiet ones without a volatility input. Unlike Bollinger Bands, the width is
/// an order statistic of the actual deviations rather than a multiple of the
/// standard deviation, so it is unaffected by the shape of the tails beyond the
/// `coverage` rank. When the midline is zero the relative deviation is
/// undefined and the bands collapse onto the midline.
///
/// # Example
///
/// ```
/// use wickra_core::{BomarBands, Indicator};
///
/// let mut indicator = BomarBands::new(20, 0.85).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     last = indicator.update(100.0 + f64::from(i % 7));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct BomarBands {
    period: usize,
    coverage: f64,
    window: VecDeque<f64>,
    scratch: Vec<f64>,
}

impl BomarBands {
    /// Construct new Bomar Bands.
    ///
    /// `coverage` is the target fraction of closes to contain, in `(0.0, 1.0]`.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`] if `period == 0`, or
    /// [`Error::InvalidParameter`] if `coverage` is not a finite value in
    /// `(0.0, 1.0]`.
    pub fn new(period: usize, coverage: f64) -> Result<Self> {
        if period == 0 {
            return Err(Error::PeriodZero);
        }
        if !coverage.is_finite() || coverage <= 0.0 || coverage > 1.0 {
            return Err(Error::InvalidParameter {
                message: "bomar bands coverage must be a finite value in (0.0, 1.0]",
            });
        }
        Ok(Self {
            period,
            coverage,
            window: VecDeque::with_capacity(period),
            scratch: Vec::with_capacity(period),
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.period
    }

    /// Configured coverage fraction.
    pub const fn coverage(&self) -> f64 {
        self.coverage
    }
}

impl Indicator for BomarBands {
    type Input = f64;
    type Output = BomarBandsOutput;

    fn update(&mut self, value: f64) -> Option<BomarBandsOutput> {
        if !value.is_finite() {
            return None;
        }
        if self.window.len() == self.period {
            self.window.pop_front();
        }
        self.window.push_back(value);
        if self.window.len() < self.period {
            return None;
        }
        let sum: f64 = self.window.iter().sum();
        let middle = sum / (self.period as f64);
        let denom = middle.abs();

        self.scratch.clear();
        for &v in &self.window {
            let dev = if denom == 0.0 {
                0.0
            } else {
                ((v - middle) / denom).abs()
            };
            self.scratch.push(dev);
        }
        self.scratch.sort_by(f64::total_cmp);
        let p = quantile_sorted(&self.scratch, self.coverage);
        let offset = denom * p;

        Some(BomarBandsOutput {
            upper: middle + offset,
            middle,
            lower: middle - offset,
        })
    }

    fn reset(&mut self) {
        self.window.clear();
        self.scratch.clear();
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.window.len() == self.period
    }

    fn name(&self) -> &'static str {
        "BomarBands"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_period() {
        assert!(matches!(BomarBands::new(0, 0.85), Err(Error::PeriodZero)));
        assert!(BomarBands::new(1, 0.85).is_ok());
    }

    #[test]
    fn rejects_out_of_range_coverage() {
        assert!(matches!(
            BomarBands::new(20, 0.0),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            BomarBands::new(20, 1.1),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            BomarBands::new(20, -0.5),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            BomarBands::new(20, f64::NAN),
            Err(Error::InvalidParameter { .. })
        ));
    }

    #[test]
    fn accessors_and_metadata() {
        let bb = BomarBands::new(20, 0.85).unwrap();
        assert_eq!(bb.period(), 20);
        assert_relative_eq!(bb.coverage(), 0.85, epsilon = 1e-12);
        assert_eq!(bb.warmup_period(), 20);
        assert_eq!(bb.name(), "BomarBands");
        assert!(!bb.is_ready());
    }

    #[test]
    fn warms_up_then_emits() {
        let mut bb = BomarBands::new(4, 0.85).unwrap();
        assert!(bb.update(100.0).is_none());
        assert!(bb.update(102.0).is_none());
        assert!(bb.update(98.0).is_none());
        assert!(bb.update(104.0).is_some());
        assert!(bb.is_ready());
    }

    #[test]
    fn known_bands() {
        // mean=101; |dev| = {1,1,3,3}/101; coverage 0.85 quantile -> 3/101.
        // offset = 101 * 3/101 = 3 -> upper 104, lower 98.
        let mut bb = BomarBands::new(4, 0.85).unwrap();
        let out = bb.batch(&[100.0, 102.0, 98.0, 104.0]);
        let last = out[3].unwrap();
        assert_relative_eq!(last.middle, 101.0, epsilon = 1e-9);
        assert_relative_eq!(last.upper, 104.0, epsilon = 1e-9);
        assert_relative_eq!(last.lower, 98.0, epsilon = 1e-9);
    }

    #[test]
    fn zero_midline_collapses_bands() {
        // Window mean exactly zero -> relative deviation undefined -> collapse.
        let mut bb = BomarBands::new(2, 0.85).unwrap();
        let out = bb.batch(&[3.0, -3.0]);
        let last = out[1].unwrap();
        assert_relative_eq!(last.middle, 0.0, epsilon = 1e-12);
        assert_relative_eq!(last.upper, 0.0, epsilon = 1e-12);
        assert_relative_eq!(last.lower, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn rolling_window_evicts_oldest() {
        // Eight values through a period-4 window: only the last four survive,
        // reproducing the `known_bands` window.
        let mut bb = BomarBands::new(4, 0.85).unwrap();
        let out = bb.batch(&[50.0, 50.0, 50.0, 50.0, 100.0, 102.0, 98.0, 104.0]);
        let last = out[7].unwrap();
        assert_relative_eq!(last.middle, 101.0, epsilon = 1e-9);
        assert_relative_eq!(last.upper, 104.0, epsilon = 1e-9);
        assert_relative_eq!(last.lower, 98.0, epsilon = 1e-9);
    }

    #[test]
    fn reset_clears_state() {
        let mut bb = BomarBands::new(4, 0.85).unwrap();
        for v in [100.0, 102.0, 98.0, 104.0] {
            bb.update(v);
        }
        assert!(bb.is_ready());
        bb.reset();
        assert!(!bb.is_ready());
        assert!(bb.update(100.0).is_none());
    }
}
