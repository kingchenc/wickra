//! Effective Spread — the realised cost of a single trade in basis points.

use crate::microstructure::TradeQuote;
use crate::traits::Indicator;

/// Effective Spread — twice the signed deviation of an executed trade price
/// from the prevailing mid, expressed in basis points of the mid.
///
/// ```text
/// effectiveSpread = 2 · D · (tradePrice − mid) / mid · 10_000   (bps)
/// ```
///
/// where `D` is the aggressor sign (`+1` for a buy, `−1` for a sell). The
/// factor of two scales the one-sided deviation up to a full round-trip cost so
/// it is directly comparable to the [quoted spread]: a marketable order that
/// fills exactly at the touch of an otherwise quoted-spread book pays an
/// effective spread equal to the quoted spread. Trades that fill *inside* the
/// spread (price improvement) read below the quoted spread; trades that walk
/// the book read above it.
///
/// A buy printed above the mid (`tradePrice > mid`) and a sell printed below it
/// both yield a positive effective spread — the conventional sign, since the
/// aggressor pays in both cases. A trade printed on the wrong side of the mid
/// for its aggressor flag (a buy below the mid) reads negative, the signature of
/// price improvement or a stale/mislabelled quote.
///
/// `Input = TradeQuote`, `Output = f64`. Stateless; ready after the first
/// trade-quote.
///
/// [quoted spread]: crate::QuotedSpread
///
/// # Example
///
/// ```
/// use wickra_core::{EffectiveSpread, Indicator, Side, Trade, TradeQuote};
///
/// let mut es = EffectiveSpread::new();
/// // Buy filled at 100.05 against a mid of 100.0:
/// // 2 · (+1) · (100.05 − 100.0) / 100.0 · 10_000 = 10 bps.
/// let trade = Trade::new(100.05, 1.0, Side::Buy, 0).unwrap();
/// let quote = TradeQuote::new(trade, 100.0).unwrap();
/// assert!((es.update(quote).unwrap() - 10.0).abs() < 1e-9);
/// ```
#[derive(Debug, Clone, Default)]
pub struct EffectiveSpread {
    has_emitted: bool,
}

impl EffectiveSpread {
    /// Construct a new effective-spread indicator.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for EffectiveSpread {
    type Input = TradeQuote;
    type Output = f64;

    fn update(&mut self, quote: TradeQuote) -> Option<f64> {
        self.has_emitted = true;
        let sign = quote.trade.side.sign();
        Some(2.0 * sign * (quote.trade.price - quote.mid) / quote.mid * 10_000.0)
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
        "EffectiveSpread"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::microstructure::{Side, Trade};
    use crate::traits::BatchExt;

    fn quote(price: f64, side: Side, mid: f64) -> TradeQuote {
        TradeQuote::new(Trade::new(price, 1.0, side, 0).unwrap(), mid).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let es = EffectiveSpread::new();
        assert_eq!(es.name(), "EffectiveSpread");
        assert_eq!(es.warmup_period(), 1);
        assert!(!es.is_ready());
    }

    #[test]
    fn buy_above_mid_is_positive() {
        let mut es = EffectiveSpread::new();
        // 2 · (+1) · (100.05 − 100.0) / 100.0 · 10_000 = 10 bps.
        let out = es.update(quote(100.05, Side::Buy, 100.0)).unwrap();
        assert!((out - 10.0).abs() < 1e-9);
        assert!(es.is_ready());
    }

    #[test]
    fn sell_below_mid_is_positive() {
        let mut es = EffectiveSpread::new();
        // 2 · (−1) · (99.95 − 100.0) / 100.0 · 10_000 = 10 bps.
        let out = es.update(quote(99.95, Side::Sell, 100.0)).unwrap();
        assert!((out - 10.0).abs() < 1e-9);
    }

    #[test]
    fn price_improvement_reads_negative() {
        let mut es = EffectiveSpread::new();
        // A buy filled below the mid: price improvement -> negative.
        let out = es.update(quote(99.95, Side::Buy, 100.0)).unwrap();
        assert!(out < 0.0);
    }

    #[test]
    fn trade_at_mid_is_zero() {
        let mut es = EffectiveSpread::new();
        assert_eq!(es.update(quote(100.0, Side::Buy, 100.0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let quotes: Vec<TradeQuote> = (0..20)
            .map(|i| {
                let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
                let price = 100.0 + f64::from(i % 4) * 0.01;
                quote(price, side, 100.0)
            })
            .collect();
        let mut a = EffectiveSpread::new();
        let mut b = EffectiveSpread::new();
        assert_eq!(
            a.batch(&quotes),
            quotes.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut es = EffectiveSpread::new();
        es.update(quote(100.05, Side::Buy, 100.0));
        assert!(es.is_ready());
        es.reset();
        assert!(!es.is_ready());
    }
}
