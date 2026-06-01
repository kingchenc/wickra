//! Liquidation Features — per-tick long/short liquidation breakdown.

use crate::derivatives::DerivativesTick;
use crate::traits::Indicator;

/// The liquidation feature vector emitted by [`LiquidationFeatures`] for one
/// tick.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LiquidationFeaturesOutput {
    /// Long-side liquidation notional on this tick.
    pub long: f64,
    /// Short-side liquidation notional on this tick.
    pub short: f64,
    /// Net liquidation `long − short` (positive = longs being liquidated).
    pub net: f64,
    /// Total liquidation `long + short`.
    pub total: f64,
    /// Liquidation imbalance `(long − short) / (long + short)`, in `[−1, +1]`;
    /// `0.0` when there is no liquidation.
    pub imbalance: f64,
}

/// Liquidation Features — decomposes the long- and short-side liquidation
/// notional carried by each tick into a small feature vector.
///
/// ```text
/// net       = longLiquidation − shortLiquidation
/// total     = longLiquidation + shortLiquidation
/// imbalance = net / total                      (0 when total == 0)
/// ```
///
/// Liquidation cascades are a perpetual-market-specific tail risk: a wave of
/// long liquidations forces market sells that beget more liquidations. Splitting
/// the flow into net, total and a bounded imbalance turns the raw venue feed
/// into model-ready features — `total` sizes the stress, `imbalance` (and its
/// sign) says which side is being flushed. A positive imbalance means longs are
/// being liquidated (downside cascade), a negative one shorts (upside squeeze).
///
/// `Input = DerivativesTick`, `Output = LiquidationFeaturesOutput`. Stateless;
/// ready after the first tick.
///
/// # Example
///
/// ```
/// use wickra_core::{DerivativesTick, Indicator, LiquidationFeatures};
///
/// fn tick(long_liq: f64, short_liq: f64) -> DerivativesTick {
///     DerivativesTick::new(
///         0.0, 100.0, 100.0, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0, long_liq, short_liq, 0,
///     )
///     .unwrap()
/// }
///
/// let mut liq = LiquidationFeatures::new();
/// // 30 long vs 10 short liquidated: net 20, total 40, imbalance 0.5.
/// let out = liq.update(tick(30.0, 10.0)).unwrap();
/// assert_eq!(out.net, 20.0);
/// assert_eq!(out.total, 40.0);
/// assert_eq!(out.imbalance, 0.5);
/// ```
#[derive(Debug, Clone, Default)]
pub struct LiquidationFeatures {
    has_emitted: bool,
}

impl LiquidationFeatures {
    /// Construct a new liquidation-features indicator.
    #[must_use]
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for LiquidationFeatures {
    type Input = DerivativesTick;
    type Output = LiquidationFeaturesOutput;

    fn update(&mut self, tick: DerivativesTick) -> Option<LiquidationFeaturesOutput> {
        self.has_emitted = true;
        let long = tick.long_liquidation;
        let short = tick.short_liquidation;
        let net = long - short;
        let total = long + short;
        let imbalance = if total == 0.0 { 0.0 } else { net / total };
        Some(LiquidationFeaturesOutput {
            long,
            short,
            net,
            total,
            imbalance,
        })
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
        "LiquidationFeatures"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn tick(long_liq: f64, short_liq: f64) -> DerivativesTick {
        DerivativesTick::new_unchecked(
            0.0, 100.0, 100.0, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0, long_liq, short_liq, 0,
        )
    }

    #[test]
    fn accessors_and_metadata() {
        let liq = LiquidationFeatures::new();
        assert_eq!(liq.name(), "LiquidationFeatures");
        assert_eq!(liq.warmup_period(), 1);
        assert!(!liq.is_ready());
    }

    #[test]
    fn decomposes_liquidations() {
        let mut liq = LiquidationFeatures::new();
        let out = liq.update(tick(30.0, 10.0)).unwrap();
        assert_eq!(out.long, 30.0);
        assert_eq!(out.short, 10.0);
        assert_eq!(out.net, 20.0);
        assert_eq!(out.total, 40.0);
        assert_eq!(out.imbalance, 0.5);
        assert!(liq.is_ready());
    }

    #[test]
    fn short_cascade_is_negative_imbalance() {
        let mut liq = LiquidationFeatures::new();
        let out = liq.update(tick(0.0, 50.0)).unwrap();
        assert_eq!(out.net, -50.0);
        assert_eq!(out.imbalance, -1.0);
    }

    #[test]
    fn no_liquidation_is_zero_imbalance() {
        let mut liq = LiquidationFeatures::new();
        let out = liq.update(tick(0.0, 0.0)).unwrap();
        assert_eq!(out.total, 0.0);
        assert_eq!(out.imbalance, 0.0);
    }

    #[test]
    fn batch_equals_streaming() {
        let ticks: Vec<DerivativesTick> = (0..20)
            .map(|i| tick(f64::from(i % 5) * 10.0, f64::from(i % 3) * 10.0))
            .collect();
        let mut a = LiquidationFeatures::new();
        let mut b = LiquidationFeatures::new();
        assert_eq!(
            a.batch(&ticks),
            ticks.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut liq = LiquidationFeatures::new();
        liq.update(tick(30.0, 10.0));
        assert!(liq.is_ready());
        liq.reset();
        assert!(!liq.is_ready());
    }
}
