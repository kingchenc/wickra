//! OHLCV value types: candles and ticks.

use crate::error::{Error, Result};

/// A single OHLCV bar.
///
/// Timestamps are unitless `i64` values so callers can use whatever epoch resolution
/// they prefer (milliseconds, microseconds, seconds…). Wickra never inspects them
/// numerically beyond passing them through.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candle {
    /// Bar open price.
    pub open: f64,
    /// Bar high price.
    pub high: f64,
    /// Bar low price.
    pub low: f64,
    /// Bar close price.
    pub close: f64,
    /// Bar volume.
    pub volume: f64,
    /// Bar timestamp (caller-defined epoch / resolution).
    pub timestamp: i64,
}

impl Candle {
    /// Construct a new candle, validating the OHLC relationships and finiteness.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidCandle`] if any of these invariants are violated:
    /// - `high >= max(open, close, low)`
    /// - `low  <= min(open, close, high)`
    /// - all of `open`, `high`, `low`, `close`, `volume` are finite
    /// - `volume >= 0`
    pub fn new(
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        timestamp: i64,
    ) -> Result<Self> {
        if !(open.is_finite() && high.is_finite() && low.is_finite() && close.is_finite()) {
            return Err(Error::InvalidCandle {
                message: "open, high, low, close must all be finite",
            });
        }
        if !volume.is_finite() {
            return Err(Error::InvalidCandle {
                message: "volume must be finite",
            });
        }
        if volume < 0.0 {
            return Err(Error::InvalidCandle {
                message: "volume must be non-negative",
            });
        }
        if high < low {
            return Err(Error::InvalidCandle {
                message: "high must be >= low",
            });
        }
        if high < open || high < close {
            return Err(Error::InvalidCandle {
                message: "high must be >= open and >= close",
            });
        }
        if low > open || low > close {
            return Err(Error::InvalidCandle {
                message: "low must be <= open and <= close",
            });
        }
        Ok(Self {
            open,
            high,
            low,
            close,
            volume,
            timestamp,
        })
    }

    /// Construct a candle without validation. The caller asserts that all OHLC
    /// invariants hold and that no field is NaN or infinite.
    pub const fn new_unchecked(
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        timestamp: i64,
    ) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
            timestamp,
        }
    }

    /// The typical price `(high + low + close) / 3`. Used by CCI, MFI, VWAP, etc.
    #[inline]
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    /// The mid price `(high + low) / 2`.
    #[inline]
    pub fn median_price(&self) -> f64 {
        f64::midpoint(self.high, self.low)
    }

    /// The weighted close `(high + low + 2*close) / 4`.
    #[inline]
    pub fn weighted_close(&self) -> f64 {
        (self.high + self.low + 2.0 * self.close) / 4.0
    }

    /// The average price `(open + high + low + close) / 4`.
    #[inline]
    pub fn avg_price(&self) -> f64 {
        (self.open + self.high + self.low + self.close) / 4.0
    }

    /// True range of this candle relative to a previous close: `max(H-L, |H-prev|, |L-prev|)`.
    /// If no previous close is supplied, falls back to `high - low`.
    #[inline]
    pub fn true_range(&self, prev_close: Option<f64>) -> f64 {
        let hl = self.high - self.low;
        match prev_close {
            Some(prev) => {
                let hp = (self.high - prev).abs();
                let lp = (self.low - prev).abs();
                hl.max(hp).max(lp)
            }
            None => hl,
        }
    }
}

/// A single trade tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tick {
    /// Trade price.
    pub price: f64,
    /// Trade size.
    pub volume: f64,
    /// Trade timestamp (caller-defined epoch / resolution).
    pub timestamp: i64,
}

impl Tick {
    /// Construct a new tick, validating finiteness and non-negativity of volume.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NonFiniteInput`] if `price` or `volume` is NaN or infinite,
    /// or [`Error::InvalidTick`] for `volume < 0`. (Audit finding R14 — previously
    /// returned [`Error::InvalidCandle`], which is semantically wrong for a tick.)
    pub fn new(price: f64, volume: f64, timestamp: i64) -> Result<Self> {
        if !price.is_finite() || !volume.is_finite() {
            return Err(Error::NonFiniteInput);
        }
        if volume < 0.0 {
            return Err(Error::InvalidTick {
                message: "tick volume must be non-negative",
            });
        }
        Ok(Self {
            price,
            volume,
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candle_new_accepts_valid_ohlc() {
        let c = Candle::new(10.0, 11.0, 9.0, 10.5, 100.0, 1).unwrap();
        assert_eq!(c.open, 10.0);
        assert_eq!(c.high, 11.0);
        assert_eq!(c.low, 9.0);
        assert_eq!(c.close, 10.5);
        assert_eq!(c.volume, 100.0);
        assert_eq!(c.timestamp, 1);
    }

    #[test]
    fn candle_new_rejects_high_below_low() {
        let err = Candle::new(10.0, 9.0, 10.0, 10.0, 1.0, 0).unwrap_err();
        assert!(matches!(err, Error::InvalidCandle { .. }));
    }

    #[test]
    fn candle_new_rejects_high_below_close() {
        let err = Candle::new(10.0, 10.0, 9.0, 11.0, 1.0, 0).unwrap_err();
        assert!(matches!(err, Error::InvalidCandle { .. }));
    }

    #[test]
    fn candle_new_rejects_low_above_open() {
        let err = Candle::new(10.0, 11.0, 10.5, 10.5, 1.0, 0).unwrap_err();
        assert!(matches!(err, Error::InvalidCandle { .. }));
    }

    #[test]
    fn candle_new_rejects_negative_volume() {
        let err = Candle::new(10.0, 11.0, 9.0, 10.5, -1.0, 0).unwrap_err();
        assert!(matches!(err, Error::InvalidCandle { .. }));
    }

    #[test]
    fn candle_new_rejects_nan_price() {
        let err = Candle::new(f64::NAN, 11.0, 9.0, 10.5, 1.0, 0).unwrap_err();
        assert!(matches!(err, Error::InvalidCandle { .. }));
    }

    /// Cover the unchecked constructor `Candle::new_unchecked` (lines 86-102).
    /// Every existing test routes through the validating `Candle::new`, so the
    /// unchecked path is dead.
    ///
    /// The first assertion shows that a valid set of fields round-trips
    /// verbatim. The second feeds `high < low` (which `Candle::new` would
    /// reject with `Error::InvalidCandle`) and asserts the unchecked
    /// constructor still produces the struct as-is — documenting and
    /// enforcing the API contract that the unchecked variant performs no
    /// validation and is the caller's responsibility.
    #[test]
    fn candle_new_unchecked_preserves_fields_verbatim() {
        let c = Candle::new_unchecked(1.0, 2.0, 0.5, 1.5, 100.0, 42);
        assert_eq!(c.open, 1.0);
        assert_eq!(c.high, 2.0);
        assert_eq!(c.low, 0.5);
        assert_eq!(c.close, 1.5);
        assert_eq!(c.volume, 100.0);
        assert_eq!(c.timestamp, 42);

        // Skip-validation contract: an OHLC combination that the checked
        // constructor rejects (high < low) is still built without error.
        assert!(Candle::new(10.0, 9.0, 10.0, 10.0, 1.0, 0).is_err());
        let unchecked = Candle::new_unchecked(10.0, 9.0, 10.0, 10.0, 1.0, 0);
        assert_eq!(unchecked.high, 9.0);
        assert_eq!(unchecked.low, 10.0);
    }

    #[test]
    fn candle_typical_price() {
        let c = Candle::new(10.0, 12.0, 9.0, 11.0, 1.0, 0).unwrap();
        assert_eq!(c.typical_price(), (12.0 + 9.0 + 11.0) / 3.0);
    }

    #[test]
    fn candle_median_price() {
        let c = Candle::new(10.0, 12.0, 8.0, 11.0, 1.0, 0).unwrap();
        assert_eq!(c.median_price(), 10.0);
    }

    #[test]
    fn candle_weighted_close() {
        let c = Candle::new(10.0, 12.0, 8.0, 11.0, 1.0, 0).unwrap();
        assert_eq!(c.weighted_close(), (12.0 + 8.0 + 22.0) / 4.0);
    }

    #[test]
    fn candle_true_range_without_prev() {
        let c = Candle::new(10.0, 12.0, 8.0, 11.0, 1.0, 0).unwrap();
        assert_eq!(c.true_range(None), 4.0);
    }

    #[test]
    fn candle_true_range_with_gap_up() {
        // Previous close 6, today's range 8-12: gap covered by |H-prev|=6
        let c = Candle::new(10.0, 12.0, 8.0, 11.0, 1.0, 0).unwrap();
        assert_eq!(c.true_range(Some(6.0)), 6.0);
    }

    #[test]
    fn candle_true_range_with_gap_down() {
        // Previous close 14, today's range 8-12: gap covered by |L-prev|=6
        let c = Candle::new(10.0, 12.0, 8.0, 11.0, 1.0, 0).unwrap();
        assert_eq!(c.true_range(Some(14.0)), 6.0);
    }

    #[test]
    fn tick_new_accepts_valid() {
        let t = Tick::new(100.5, 0.5, 42).unwrap();
        assert_eq!(t.price, 100.5);
        assert_eq!(t.volume, 0.5);
        assert_eq!(t.timestamp, 42);
    }

    #[test]
    fn tick_new_rejects_nan() {
        assert!(matches!(
            Tick::new(f64::NAN, 1.0, 0),
            Err(Error::NonFiniteInput)
        ));
    }

    #[test]
    fn tick_new_rejects_inf() {
        assert!(matches!(
            Tick::new(f64::INFINITY, 1.0, 0),
            Err(Error::NonFiniteInput)
        ));
    }

    #[test]
    fn tick_new_rejects_negative_volume() {
        // Audit R14: the variant is `InvalidTick`, not `InvalidCandle` — a tick
        // is not a candle, and downstream pipelines should be able to match on
        // the correct semantic.
        let err = Tick::new(100.0, -1.0, 0).unwrap_err();
        assert!(matches!(err, Error::InvalidTick { .. }));
        assert!(
            err.to_string().contains("tick volume"),
            "expected the InvalidTick message in the formatted error, got {err}"
        );
    }
}
