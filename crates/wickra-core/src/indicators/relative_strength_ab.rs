//! Relative Strength A-vs-B — the price ratio of two assets, plus its MA and RSI.

use crate::error::Result;
use crate::indicators::{Rsi, Sma};
use crate::traits::Indicator;

/// Output of [`RelativeStrengthAB`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelativeStrengthOutput {
    /// The raw relative-strength ratio `a / b`.
    pub ratio: f64,
    /// Simple moving average of the ratio over `ma_period`.
    pub ratio_ma: f64,
    /// Relative Strength Index of the ratio over `rsi_period`.
    pub ratio_rsi: f64,
}

/// Comparative relative strength of asset `a` against asset `b`.
///
/// Each `update` receives one `(a, b)` price pair and forms the **ratio line**
/// `a / b`. The ratio is then smoothed with a simple moving average and run
/// through an RSI, so a single indicator gives you the relative-strength level,
/// its trend, and whether that trend is overbought or oversold:
///
/// ```text
/// ratio     = a / b
/// ratio_ma  = SMA(ratio, ma_period)
/// ratio_rsi = RSI(ratio, rsi_period)
/// ```
///
/// A rising ratio means `a` is outperforming `b`; `ratio_ma` shows the trend of
/// that outperformance and `ratio_rsi` flags exhaustion (e.g. `> 70` after a
/// strong run of `a` over `b`). This is the classic "asset-vs-asset" or
/// "asset-vs-index" rotation screen.
///
/// The first output appears once both the moving average and the RSI have
/// warmed up; the ratio itself is computed from the first valid pair. A
/// non-finite price or a zero denominator (`b == 0`) makes the ratio undefined
/// and is skipped, leaving the internal averages untouched.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, RelativeStrengthAB};
///
/// let mut rs = RelativeStrengthAB::new(5, 5).unwrap();
/// let mut last = None;
/// for _ in 0..20 {
///     last = rs.update((200.0, 100.0)); // ratio is a constant 2.0
/// }
/// let out = last.unwrap();
/// assert!((out.ratio - 2.0).abs() < 1e-12);
/// assert!((out.ratio_ma - 2.0).abs() < 1e-12);
/// // A flat ratio has no gains or losses, so its RSI sits at the neutral 50.
/// assert!((out.ratio_rsi - 50.0).abs() < 1e-9);
/// ```
#[derive(Debug, Clone)]
pub struct RelativeStrengthAB {
    ma_period: usize,
    rsi_period: usize,
    ma: Sma,
    rsi: Rsi,
}

impl RelativeStrengthAB {
    /// Construct a new comparative relative-strength indicator.
    ///
    /// `ma_period` is the moving-average look-back of the ratio; `rsi_period`
    /// is the RSI look-back of the ratio.
    ///
    /// # Errors
    /// Returns [`Error::PeriodZero`](crate::Error::PeriodZero) if either period
    /// is zero.
    pub fn new(ma_period: usize, rsi_period: usize) -> Result<Self> {
        Ok(Self {
            ma_period,
            rsi_period,
            ma: Sma::new(ma_period)?,
            rsi: Rsi::new(rsi_period)?,
        })
    }

    /// Moving-average look-back of the ratio.
    pub const fn ma_period(&self) -> usize {
        self.ma_period
    }

    /// RSI look-back of the ratio.
    pub const fn rsi_period(&self) -> usize {
        self.rsi_period
    }
}

impl Indicator for RelativeStrengthAB {
    /// `(a, b)` price pair.
    type Input = (f64, f64);
    type Output = RelativeStrengthOutput;

    fn update(&mut self, input: (f64, f64)) -> Option<RelativeStrengthOutput> {
        let (a, b) = input;
        if b == 0.0 || !a.is_finite() || !b.is_finite() {
            // Undefined ratio: skip without disturbing the internal averages.
            return None;
        }
        let ratio = a / b;
        let ma = self.ma.update(ratio);
        let rsi = self.rsi.update(ratio);
        match (ma, rsi) {
            (Some(ratio_ma), Some(ratio_rsi)) => Some(RelativeStrengthOutput {
                ratio,
                ratio_ma,
                ratio_rsi,
            }),
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.ma.reset();
        self.rsi.reset();
    }

    fn warmup_period(&self) -> usize {
        self.ma.warmup_period().max(self.rsi.warmup_period())
    }

    fn is_ready(&self) -> bool {
        self.ma.is_ready() && self.rsi.is_ready()
    }

    fn name(&self) -> &'static str {
        "RelativeStrengthAB"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_zero_periods() {
        assert!(RelativeStrengthAB::new(0, 5).is_err());
        assert!(RelativeStrengthAB::new(5, 0).is_err());
        assert!(RelativeStrengthAB::new(5, 5).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let rs = RelativeStrengthAB::new(10, 14).unwrap();
        assert_eq!(rs.ma_period(), 10);
        assert_eq!(rs.rsi_period(), 14);
        // SMA warmup = 10, RSI warmup = 15 ⇒ combined = 15.
        assert_eq!(rs.warmup_period(), 15);
        assert_eq!(rs.name(), "RelativeStrengthAB");
    }

    #[test]
    fn constant_ratio_is_flat() {
        // a = 2·b ⇒ ratio is a constant 2 ⇒ MA = 2, RSI = neutral 50.
        let pairs: Vec<(f64, f64)> = (0..20).map(|_| (200.0, 100.0)).collect();
        let out = RelativeStrengthAB::new(5, 5)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(out.ratio, 2.0, epsilon = 1e-12);
        assert_relative_eq!(out.ratio_ma, 2.0, epsilon = 1e-12);
        assert_relative_eq!(out.ratio_rsi, 50.0, epsilon = 1e-9);
    }

    #[test]
    fn rising_ratio_is_overbought() {
        // a grows while b is flat ⇒ ratio strictly rises ⇒ RSI saturates at 100.
        let pairs: Vec<(f64, f64)> = (0..20)
            .map(|t| (100.0 + 2.0 * f64::from(t), 100.0))
            .collect();
        let out = RelativeStrengthAB::new(5, 5)
            .unwrap()
            .batch(&pairs)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert!(out.ratio > 1.0);
        assert_relative_eq!(out.ratio_rsi, 100.0, epsilon = 1e-9);
    }

    #[test]
    fn zero_denominator_is_skipped() {
        let mut rs = RelativeStrengthAB::new(3, 3).unwrap();
        // b == 0 and non-finite inputs never reach the internal averages.
        assert_eq!(rs.update((100.0, 0.0)), None);
        assert_eq!(rs.update((f64::NAN, 100.0)), None);
        assert!(!rs.is_ready());
        for _ in 0..8 {
            rs.update((150.0, 100.0));
        }
        assert!(rs.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut rs = RelativeStrengthAB::new(3, 3).unwrap();
        for t in 0..10 {
            rs.update((100.0 + f64::from(t), 100.0));
        }
        assert!(rs.is_ready());
        rs.reset();
        assert!(!rs.is_ready());
        assert_eq!(rs.update((100.0, 100.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let pairs: Vec<(f64, f64)> = (0..60)
            .map(|t| {
                let tt = f64::from(t);
                (
                    100.0 + 5.0 * (tt * 0.3).sin(),
                    100.0 + 2.0 * (tt * 0.2).cos(),
                )
            })
            .collect();
        let batch = RelativeStrengthAB::new(10, 14).unwrap().batch(&pairs);
        let mut rs = RelativeStrengthAB::new(10, 14).unwrap();
        let streamed: Vec<_> = pairs.iter().map(|p| rs.update(*p)).collect();
        assert_eq!(batch, streamed);
    }
}
