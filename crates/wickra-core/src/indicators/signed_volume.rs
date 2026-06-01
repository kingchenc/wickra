//! Signed Volume — per-trade volume signed by aggressor side.

use crate::microstructure::Trade;
use crate::traits::Indicator;

/// Signed Volume — the size of each trade signed by its aggressor side.
///
/// ```text
/// signedVolume = size · (+1 if buy, −1 if sell)
/// ```
///
/// A positive value is buyer-initiated flow, a negative value seller-initiated.
/// It is the per-trade building block of [`crate::CumulativeVolumeDelta`] and
/// trade-flow imbalance.
///
/// `Input = Trade`, `Output = f64`. Stateless; ready after the first trade.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, SignedVolume, Side, Trade};
///
/// let mut sv = SignedVolume::new();
/// let buy = Trade::new(100.0, 2.0, Side::Buy, 0).unwrap();
/// assert_eq!(sv.update(buy), Some(2.0));
/// let sell = Trade::new(100.0, 3.0, Side::Sell, 1).unwrap();
/// assert_eq!(sv.update(sell), Some(-3.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct SignedVolume {
    has_emitted: bool,
}

impl SignedVolume {
    /// Construct a new signed-volume indicator.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for SignedVolume {
    type Input = Trade;
    type Output = f64;

    fn update(&mut self, trade: Trade) -> Option<f64> {
        self.has_emitted = true;
        Some(trade.size * trade.side.sign())
    }

    fn reset(&mut self) {
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "SignedVolume"
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
        let sv = SignedVolume::new();
        assert_eq!(sv.name(), "SignedVolume");
        assert_eq!(sv.warmup_period(), 1);
        assert!(!sv.is_ready());
    }

    #[test]
    fn buy_is_positive() {
        let mut sv = SignedVolume::new();
        assert_eq!(sv.update(trade(2.0, Side::Buy, 0)), Some(2.0));
        assert!(sv.is_ready());
    }

    #[test]
    fn sell_is_negative() {
        let mut sv = SignedVolume::new();
        assert_eq!(sv.update(trade(3.0, Side::Sell, 0)), Some(-3.0));
    }

    #[test]
    fn zero_size_is_zero() {
        let mut sv = SignedVolume::new();
        assert_eq!(sv.update(trade(0.0, Side::Buy, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let trades: Vec<Trade> = (0..20)
            .map(|i| {
                let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
                trade(1.0 + (i % 4) as f64, side, i)
            })
            .collect();
        let mut a = SignedVolume::new();
        let mut b = SignedVolume::new();
        assert_eq!(
            a.batch(&trades),
            trades.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut sv = SignedVolume::new();
        sv.update(trade(1.0, Side::Buy, 0));
        assert!(sv.is_ready());
        sv.reset();
        assert!(!sv.is_ready());
    }
}
