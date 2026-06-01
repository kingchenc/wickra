//! Cumulative Volume Delta — running sum of signed trade volume.

use crate::microstructure::Trade;
use crate::traits::Indicator;

/// Cumulative Volume Delta (CVD) — the running sum of [signed volume].
///
/// ```text
/// CVDₜ = CVDₜ₋₁ + sizeₜ · (+1 if buy, −1 if sell)
/// ```
///
/// CVD is an unbounded running total: a rising line signals net buying pressure
/// over the session, a falling line net selling. Divergence between CVD and
/// price is a classic absorption / exhaustion signal. Call [`reset`] at the
/// start of each session to re-anchor the cumulative total at zero.
///
/// `Input = Trade`, `Output = f64`. Ready after the first trade.
///
/// [signed volume]: crate::SignedVolume
/// [`reset`]: crate::Indicator::reset
///
/// # Example
///
/// ```
/// use wickra_core::{CumulativeVolumeDelta, Indicator, Side, Trade};
///
/// let mut cvd = CumulativeVolumeDelta::new();
/// assert_eq!(cvd.update(Trade::new(100.0, 5.0, Side::Buy, 0).unwrap()), Some(5.0));
/// assert_eq!(cvd.update(Trade::new(100.0, 2.0, Side::Sell, 1).unwrap()), Some(3.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct CumulativeVolumeDelta {
    cumulative: f64,
    has_emitted: bool,
}

impl CumulativeVolumeDelta {
    /// Construct a new CVD indicator with a zero running total.
    pub const fn new() -> Self {
        Self {
            cumulative: 0.0,
            has_emitted: false,
        }
    }
}

impl Indicator for CumulativeVolumeDelta {
    type Input = Trade;
    type Output = f64;

    fn update(&mut self, trade: Trade) -> Option<f64> {
        self.has_emitted = true;
        self.cumulative += trade.size * trade.side.sign();
        Some(self.cumulative)
    }

    fn reset(&mut self) {
        self.cumulative = 0.0;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "CumulativeVolumeDelta"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::microstructure::Side;
    use crate::traits::BatchExt;

    fn trade(size: f64, side: Side, ts: i64) -> Trade {
        Trade::new(100.0, size, side, ts).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let cvd = CumulativeVolumeDelta::new();
        assert_eq!(cvd.name(), "CumulativeVolumeDelta");
        assert_eq!(cvd.warmup_period(), 1);
        assert!(!cvd.is_ready());
    }

    #[test]
    fn accumulates_signed_volume() {
        let mut cvd = CumulativeVolumeDelta::new();
        assert_eq!(cvd.update(trade(5.0, Side::Buy, 0)), Some(5.0));
        assert_eq!(cvd.update(trade(2.0, Side::Sell, 1)), Some(3.0));
        assert_eq!(cvd.update(trade(4.0, Side::Sell, 2)), Some(-1.0));
        assert!(cvd.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let trades: Vec<Trade> = (0..20)
            .map(|i| {
                let side = if i % 3 == 0 { Side::Sell } else { Side::Buy };
                trade(1.0 + (i % 4) as f64, side, i)
            })
            .collect();
        let mut a = CumulativeVolumeDelta::new();
        let mut b = CumulativeVolumeDelta::new();
        assert_eq!(
            a.batch(&trades),
            trades.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_re_anchors_at_zero() {
        let mut cvd = CumulativeVolumeDelta::new();
        cvd.update(trade(5.0, Side::Buy, 0));
        cvd.reset();
        assert!(!cvd.is_ready());
        // After reset the running total starts again from zero.
        assert_eq!(cvd.update(trade(2.0, Side::Buy, 1)), Some(2.0));
    }
}
