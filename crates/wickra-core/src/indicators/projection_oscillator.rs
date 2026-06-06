//! Projection Oscillator (Mel Widner) — the close's position inside the
//! [`ProjectionBands`](crate::ProjectionBands).

use crate::error::Result;
use crate::indicators::projection_bands::ProjectionBands;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Projection Oscillator: where the close sits inside the projection bands,
/// scaled to `0..100`.
///
/// The companion to [`ProjectionBands`](crate::ProjectionBands) from Mel
/// Widner's May 1995 *Stocks & Commodities* article. It maps the close onto the
/// `[lower, upper]` projection envelope:
///
/// ```text
/// PO = 100 · (close − lower) / (upper − lower)
/// ```
///
/// `PO = 0` means the close is sitting on the lower band, `PO = 100` on the
/// upper band, and `PO = 50` at the midline. Because the bands by construction
/// bracket every projected high and low, the close almost always falls inside
/// them and the oscillator stays in `0..100` — readings near the extremes flag
/// an overbought/oversold position *relative to the trend-tilted channel*
/// rather than to a horizontal level. When the bands collapse (a zero-range
/// window, `upper == lower`) the position is undefined and the oscillator
/// returns the neutral `50.0`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, ProjectionOscillator};
///
/// let mut indicator = ProjectionOscillator::new(14).unwrap();
/// let mut last = None;
/// for i in 0..30 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct ProjectionOscillator {
    bands: ProjectionBands,
}

impl ProjectionOscillator {
    /// Construct a new Projection Oscillator.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`](crate::Error::InvalidPeriod) if
    /// `period < 2`.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            bands: ProjectionBands::new(period)?,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.bands.period()
    }
}

impl Indicator for ProjectionOscillator {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let bands = self.bands.update(candle)?;
        let width = bands.upper - bands.lower;
        if width == 0.0 {
            return Some(50.0);
        }
        Some(100.0 * (candle.close - bands.lower) / width)
    }

    fn reset(&mut self) {
        self.bands.reset();
    }

    fn warmup_period(&self) -> usize {
        self.bands.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.bands.is_ready()
    }

    fn name(&self) -> &'static str {
        "ProjectionOscillator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use approx::assert_relative_eq;

    fn candle(high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(low, high, low, close, 10.0, ts).unwrap()
    }

    #[test]
    fn rejects_period_below_two() {
        assert!(matches!(
            ProjectionOscillator::new(1),
            Err(Error::InvalidPeriod { .. })
        ));
        assert!(ProjectionOscillator::new(2).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let po = ProjectionOscillator::new(14).unwrap();
        assert_eq!(po.period(), 14);
        assert_eq!(po.warmup_period(), 14);
        assert_eq!(po.name(), "ProjectionOscillator");
        assert!(!po.is_ready());
    }

    #[test]
    fn warms_up_then_emits() {
        let mut po = ProjectionOscillator::new(3).unwrap();
        assert!(po.update(candle(10.0, 8.0, 9.0, 0)).is_none());
        assert!(po.update(candle(12.0, 9.0, 11.0, 1)).is_none());
        assert!(po.update(candle(11.0, 10.0, 11.0, 2)).is_some());
        assert!(po.is_ready());
    }

    #[test]
    fn known_position() {
        // Same window as ProjectionBands::known_projection: upper 12.5, lower 10.
        // close 11 -> 100 * (11 - 10) / (12.5 - 10) = 40.
        let mut po = ProjectionOscillator::new(3).unwrap();
        po.update(candle(10.0, 8.0, 9.0, 0));
        po.update(candle(12.0, 9.0, 11.0, 1));
        let out = po.update(candle(11.0, 10.0, 11.0, 2)).unwrap();
        assert_relative_eq!(out, 40.0, epsilon = 1e-9);
    }

    #[test]
    fn collapsed_bands_return_neutral() {
        // Zero-range, perfectly trending candles: upper == lower every bar.
        let mut po = ProjectionOscillator::new(3).unwrap();
        let mut last = None;
        for i in 0..6 {
            let v = 100.0 + f64::from(i);
            last = po.update(candle(v, v, v, i64::from(i)));
        }
        assert_relative_eq!(last.unwrap(), 50.0, epsilon = 1e-12);
    }

    #[test]
    fn reset_clears_state() {
        let mut po = ProjectionOscillator::new(3).unwrap();
        po.update(candle(10.0, 8.0, 9.0, 0));
        po.update(candle(12.0, 9.0, 11.0, 1));
        po.update(candle(11.0, 10.0, 11.0, 2));
        assert!(po.is_ready());
        po.reset();
        assert!(!po.is_ready());
        assert!(po.update(candle(10.0, 8.0, 9.0, 3)).is_none());
    }
}
