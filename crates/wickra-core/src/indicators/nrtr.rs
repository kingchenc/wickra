//! NRTR — Nick Rypock Trailing Reverse, a percentage trailing-reverse stop.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Output of [`Nrtr`]: the trailing-reverse line and the trend direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NrtrOutput {
    /// The NRTR line — below price in an uptrend, above price in a downtrend.
    pub value: f64,
    /// Trend direction: `+1.0` up (line below price), `-1.0` down.
    pub direction: f64,
}

/// NRTR (Nick Rypock Trailing Reverse) — a **percentage** trailing-reverse stop
/// that follows the trend extreme and flips when price retraces by a fixed
/// percentage.
///
/// ```text
/// uptrend:   high_water = max(high_water, close)
///            line       = high_water · (1 − pct/100)
///            flip down when close < line  (reseed low_water = close)
/// downtrend: low_water  = min(low_water, close)
///            line       = low_water · (1 + pct/100)
///            flip up   when close > line  (reseed high_water = close)
/// ```
///
/// Unlike volatility stops (ATR, σ-of-range), NRTR uses a pure **percentage**
/// retracement: the line trails the highest close reached in the up-leg at a
/// fixed `pct` below it, and a close that gives back that percentage reverses the
/// trend, handing the line to the opposite extreme. This makes it scale-free and
/// trivially tunable — one number sets how much retracement you tolerate. It
/// differs from a fixed percentage *stop-loss* in that it **reverses** (tracks
/// both directions) rather than just exiting.
///
/// The first bar seeds the up-trend and emits a line immediately. Each `update` is
/// O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Nrtr};
///
/// let mut indicator = Nrtr::new(2.0).unwrap();
/// let mut last = None;
/// for i in 0..40 {
///     let close = 100.0 + f64::from(i);
///     let c = Candle::new(close, close + 0.5, close - 0.5, close, 1_000.0, 0).unwrap();
///     last = indicator.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Nrtr {
    pct: f64,
    direction: f64,
    water: f64,
    last: Option<NrtrOutput>,
}

impl Nrtr {
    /// Construct an NRTR with the given trailing percentage (e.g. `2.0` for 2%).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if `pct` is not finite or is outside
    /// `(0, 100)`.
    pub fn new(pct: f64) -> Result<Self> {
        if !pct.is_finite() || pct <= 0.0 || pct >= 100.0 {
            return Err(Error::InvalidParameter {
                message: "NRTR percentage must be in (0, 100)",
            });
        }
        Ok(Self {
            pct,
            direction: 0.0,
            water: 0.0,
            last: None,
        })
    }

    /// Configured trailing percentage.
    pub const fn pct(&self) -> f64 {
        self.pct
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<NrtrOutput> {
        self.last
    }
}

impl Indicator for Nrtr {
    type Input = Candle;
    type Output = NrtrOutput;

    fn update(&mut self, candle: Candle) -> Option<NrtrOutput> {
        let close = candle.close;
        let down = self.pct / 100.0;
        let up = self.pct / 100.0;

        if self.direction == 0.0 {
            self.direction = 1.0;
            self.water = close;
        } else if self.direction > 0.0 {
            self.water = self.water.max(close);
            let line = self.water * (1.0 - down);
            if close < line {
                self.direction = -1.0;
                self.water = close;
            }
        } else {
            self.water = self.water.min(close);
            let line = self.water * (1.0 + up);
            if close > line {
                self.direction = 1.0;
                self.water = close;
            }
        }

        let line = if self.direction > 0.0 {
            self.water * (1.0 - down)
        } else {
            self.water * (1.0 + up)
        };
        let out = NrtrOutput {
            value: line,
            direction: self.direction,
        };
        self.last = Some(out);
        Some(out)
    }

    fn reset(&mut self) {
        self.direction = 0.0;
        self.water = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "Nrtr"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(close: f64) -> Candle {
        Candle::new_unchecked(close, close, close, close, 1_000.0, 0)
    }

    #[test]
    fn rejects_invalid_pct() {
        assert!(matches!(
            Nrtr::new(0.0),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            Nrtr::new(100.0),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            Nrtr::new(f64::NAN),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(Nrtr::new(2.0).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let n = Nrtr::new(2.0).unwrap();
        assert_eq!(n.pct(), 2.0);
        assert_eq!(n.warmup_period(), 1);
        assert_eq!(n.name(), "Nrtr");
        assert!(!n.is_ready());
        assert_eq!(n.value(), None);
    }

    #[test]
    fn first_bar_emits_up_line() {
        let mut n = Nrtr::new(10.0).unwrap();
        let o = n.update(c(100.0)).unwrap();
        assert_eq!(o.direction, 1.0);
        // line = 100 * (1 - 0.10) = 90.
        assert!((o.value - 90.0).abs() < 1e-9);
    }

    #[test]
    fn uptrend_keeps_line_below_price() {
        let mut n = Nrtr::new(5.0).unwrap();
        let candles: Vec<Candle> = (0..40).map(|i| c(100.0 + f64::from(i))).collect();
        for (o, candle) in n.batch(&candles).into_iter().zip(candles.iter()) {
            let o = o.unwrap();
            assert_eq!(o.direction, 1.0);
            assert!(o.value < candle.close);
        }
    }

    #[test]
    fn reverses_on_retracement() {
        let mut n = Nrtr::new(5.0).unwrap();
        // Rise to 120, then drop sharply -> a >5% retracement reverses the trend.
        let mut candles: Vec<Candle> = (0..20).map(|i| c(100.0 + f64::from(i))).collect();
        candles.extend((0..10).map(|i| c(119.0 - 3.0 * f64::from(i))));
        let dirs: Vec<f64> = n
            .batch(&candles)
            .into_iter()
            .flatten()
            .map(|o| o.direction)
            .collect();
        assert!(dirs.iter().any(|&d| d > 0.0));
        assert!(dirs.iter().any(|&d| d < 0.0));
    }

    #[test]
    fn downtrend_keeps_line_above_price() {
        let mut n = Nrtr::new(5.0).unwrap();
        // Establish a downtrend after an initial bar.
        let mut candles = vec![c(100.0)];
        candles.extend((0..30).map(|i| c(80.0 - f64::from(i))));
        let out = n.batch(&candles);
        let o = out.last().unwrap().unwrap();
        let candle = candles.last().unwrap();
        assert_eq!(o.direction, -1.0);
        assert!(o.value > candle.close);
    }

    #[test]
    fn reset_clears_state() {
        let mut n = Nrtr::new(2.0).unwrap();
        n.batch(&(0..20).map(|i| c(100.0 + f64::from(i))).collect::<Vec<_>>());
        assert!(n.is_ready());
        n.reset();
        assert!(!n.is_ready());
        assert_eq!(n.value(), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..120)
            .map(|i| c(100.0 + (f64::from(i) * 0.25).sin() * 15.0))
            .collect();
        let batch = Nrtr::new(3.0).unwrap().batch(&candles);
        let mut b = Nrtr::new(3.0).unwrap();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }
}
