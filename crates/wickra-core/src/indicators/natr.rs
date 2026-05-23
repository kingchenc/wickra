//! Normalized Average True Range.

use crate::error::Result;
use crate::ohlcv::Candle;
use crate::traits::Indicator;

use super::Atr;

/// Normalized Average True Range — [`Atr`] expressed as a percentage of price.
///
/// `Atr` reports volatility in raw price units, which makes its readings
/// impossible to compare across instruments at different price levels. NATR
/// fixes that by dividing by the current close:
///
/// ```text
/// NATR = 100 · ATR / close
/// ```
///
/// A NATR of `2.0` always means "the average true range is 2 % of price",
/// whether the instrument trades at $10 or $10 000 — so NATR values are
/// directly comparable, and stop distances or position sizes expressed as a
/// NATR multiple behave consistently across a portfolio.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, Natr};
///
/// let mut indicator = Natr::new(14).unwrap();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Natr {
    atr: Atr,
    last: Option<f64>,
}

impl Natr {
    /// Construct a new NATR with the given ATR period.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PeriodZero`] if `period == 0`.
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            atr: Atr::new(period)?,
            last: None,
        })
    }

    /// Configured period.
    pub const fn period(&self) -> usize {
        self.atr.period()
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for Natr {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let atr = self.atr.update(candle)?;
        let natr = if candle.close == 0.0 {
            // NATR is undefined against a zero close.
            0.0
        } else {
            100.0 * atr / candle.close
        };
        self.last = Some(natr);
        Some(natr)
    }

    fn reset(&mut self) {
        self.atr.reset();
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.atr.warmup_period()
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "NATR"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(open: f64, high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(open, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn new_rejects_zero_period() {
        assert!(Natr::new(0).is_err());
    }

    #[test]
    fn warmup_period_matches_atr() {
        let natr = Natr::new(14).unwrap();
        assert_eq!(natr.warmup_period(), 14);
    }

    /// Cover the const accessors `period` / `value` (lines 59-66) and the
    /// Indicator-impl `name` body (98-100). `warmup_period` is covered
    /// already by `warmup_period_matches_atr`.
    #[test]
    fn accessors_and_metadata() {
        let mut natr = Natr::new(14).unwrap();
        assert_eq!(natr.period(), 14);
        assert_eq!(natr.name(), "NATR");
        assert_eq!(natr.value(), None);
        let candles: Vec<Candle> = (0..14)
            .map(|i| candle(100.0, 102.0, 98.0, 101.0, i))
            .collect();
        for c in &candles {
            natr.update(*c);
        }
        assert!(natr.value().is_some());
    }

    /// Cover the `candle.close == 0.0` defensive branch (line 77). All
    /// other tests feed candles with close ≈ 100, so the zero-close
    /// fallback never fired. Feed an all-zero candle series — the Candle
    /// validator accepts open == high == low == close == 0 with positive
    /// volume, and ATR is 0 each bar, so the indicator must emit exactly
    /// 0.0 rather than computing 100 * 0 / 0 = NaN.
    #[test]
    fn zero_close_yields_zero_natr() {
        let candles: Vec<Candle> = (0..15).map(|i| candle(0.0, 0.0, 0.0, 0.0, i)).collect();
        let mut natr = Natr::new(5).unwrap();
        let out = natr.batch(&candles);
        let last = out.into_iter().flatten().last().expect("emits");
        assert_eq!(last, 0.0);
    }

    #[test]
    fn natr_is_atr_over_close_as_percent() {
        // NATR must equal 100 * ATR / close, bar for bar.
        let candles: Vec<Candle> = (0..60)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.3).sin() * 10.0;
                candle(mid, mid + 3.0, mid - 3.0, mid + 1.0, i)
            })
            .collect();
        let natr_out = Natr::new(14).unwrap().batch(&candles);
        let atr_out = Atr::new(14).unwrap().batch(&candles);
        for (i, (n, a)) in natr_out.iter().zip(atr_out.iter()).enumerate() {
            // Same warmup period — emission shape must agree at every index.
            assert_eq!(n.is_some(), a.is_some(), "warmup mismatch at index {i}");
            if let (Some(nv), Some(av)) = (n, a) {
                let want = 100.0 * av / candles[i].close;
                assert_relative_eq!(*nv, want, epsilon = 1e-9);
            }
        }
    }

    #[test]
    fn flat_market_yields_zero() {
        // No range -> ATR is 0 -> NATR is 0.
        let mut natr = Natr::new(5).unwrap();
        let candles: Vec<Candle> = (0..30)
            .map(|i| candle(100.0, 100.0, 100.0, 100.0, i))
            .collect();
        for v in natr.batch(&candles).into_iter().flatten() {
            assert_relative_eq!(v, 0.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut natr = Natr::new(5).unwrap();
        let candles: Vec<Candle> = (0..20)
            .map(|i| candle(100.0, 102.0, 98.0, 101.0, i))
            .collect();
        natr.batch(&candles);
        assert!(natr.is_ready());
        natr.reset();
        assert!(!natr.is_ready());
        assert_eq!(natr.update(candles[0]), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.35).sin() * 9.0;
                candle(mid, mid + 2.5, mid - 2.5, mid + 0.5, i)
            })
            .collect();
        let batch = Natr::new(14).unwrap().batch(&candles);
        let mut b = Natr::new(14).unwrap();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }
}
