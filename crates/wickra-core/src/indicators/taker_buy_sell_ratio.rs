//! Taker Buy/Sell Ratio — aggressive buy volume relative to aggressive sell
//! volume.

use crate::derivatives::DerivativesTick;
use crate::traits::Indicator;

/// Taker Buy/Sell Ratio — the taker (market-order) buy volume divided by the
/// taker sell volume carried by each tick.
///
/// ```text
/// takerBuySellRatio = takerBuyVolume / takerSellVolume
/// ```
///
/// Taker volume is the volume that crossed the spread — the aggressive flow that
/// moves price. A ratio above `1` means buyers are lifting offers faster than
/// sellers are hitting bids (net aggressive buying); below `1` the reverse. It
/// is the perpetual-feed analogue of [trade imbalance], read straight off the
/// venue's taker-volume fields. When taker sell volume is zero the ratio is
/// undefined and the indicator reports `0.0`.
///
/// `Input = DerivativesTick`, `Output = f64`. Stateless; ready after the first
/// tick.
///
/// [trade imbalance]: crate::TradeImbalance
///
/// # Example
///
/// ```
/// use wickra_core::{DerivativesTick, Indicator, TakerBuySellRatio};
///
/// fn tick(buy: f64, sell: f64) -> DerivativesTick {
///     DerivativesTick::new(0.0, 100.0, 100.0, 100.0, 0.0, 0.0, 0.0, buy, sell, 0.0, 0.0, 0)
///         .unwrap()
/// }
///
/// let mut tbs = TakerBuySellRatio::new();
/// // 60 taker buys vs 40 taker sells -> 1.5.
/// assert_eq!(tbs.update(tick(60.0, 40.0)), Some(1.5));
/// ```
#[derive(Debug, Clone, Default)]
pub struct TakerBuySellRatio {
    has_emitted: bool,
}

impl TakerBuySellRatio {
    /// Construct a new taker buy/sell ratio indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for TakerBuySellRatio {
    type Input = DerivativesTick;
    type Output = f64;

    fn update(&mut self, tick: DerivativesTick) -> Option<f64> {
        self.has_emitted = true;
        if tick.taker_sell_volume == 0.0 {
            // No taker sell volume to divide by: the ratio is undefined.
            return Some(0.0);
        }
        Some(tick.taker_buy_volume / tick.taker_sell_volume)
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
        "TakerBuySellRatio"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn tick(buy: f64, sell: f64) -> DerivativesTick {
        DerivativesTick::new_unchecked(
            0.0, 100.0, 100.0, 100.0, 0.0, 0.0, 0.0, buy, sell, 0.0, 0.0, 0,
        )
    }

    #[test]
    fn accessors_and_metadata() {
        let tbs = TakerBuySellRatio::new();
        assert_eq!(tbs.name(), "TakerBuySellRatio");
        assert_eq!(tbs.warmup_period(), 1);
        assert!(!tbs.is_ready());
    }

    #[test]
    fn divides_buy_by_sell() {
        let mut tbs = TakerBuySellRatio::new();
        assert_eq!(tbs.update(tick(60.0, 40.0)), Some(1.5));
        assert_eq!(tbs.update(tick(20.0, 80.0)), Some(0.25));
        assert!(tbs.is_ready());
    }

    #[test]
    fn zero_sell_is_zero() {
        let mut tbs = TakerBuySellRatio::new();
        assert_eq!(tbs.update(tick(60.0, 0.0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let ticks: Vec<DerivativesTick> = (0..20)
            .map(|i| tick(50.0 + f64::from(i % 5) * 5.0, 40.0 + f64::from(i % 3) * 5.0))
            .collect();
        let mut a = TakerBuySellRatio::new();
        let mut b = TakerBuySellRatio::new();
        assert_eq!(
            a.batch(&ticks),
            ticks.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut tbs = TakerBuySellRatio::new();
        tbs.update(tick(60.0, 40.0));
        assert!(tbs.is_ready());
        tbs.reset();
        assert!(!tbs.is_ready());
    }
}
