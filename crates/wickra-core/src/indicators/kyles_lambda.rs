//! Kyle's Lambda — rolling price impact per unit of signed order flow.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::microstructure::TradeQuote;
use crate::traits::Indicator;

/// Kyle's Lambda — the rolling ordinary-least-squares slope of mid-price changes
/// on signed trade volume, the canonical measure of market depth / price
/// impact.
///
/// Each `update` receives a [`TradeQuote`] — a trade plus the mid prevailing at
/// execution. Internally the indicator forms, per trade, the mid change since
/// the previous trade (`Δmid = midₜ − midₜ₋₁`) and the signed volume
/// (`q = size · D`, with `D` the aggressor sign), then runs a rolling OLS
/// regression of `Δmid` on `q` over the trailing window of `window` trades:
///
/// ```text
/// cov = (1/n) · Σ q·Δmid − q̄·Δ̄mid
/// var = (1/n) · Σ q²      − q̄²
/// λ   = cov / var
/// ```
///
/// `λ` is the estimated price move per unit of signed volume: a deep, liquid
/// book absorbs flow with little movement and reads a small `λ`; a thin book
/// moves sharply per unit traded and reads a large `λ`. It is a direct,
/// model-light proxy for the slope of the demand curve in Kyle's microstructure
/// model.
///
/// Each `update` is O(1): four running sums (`Σq`, `ΣΔmid`, `Σq²`, `Σq·Δmid`)
/// are maintained as the window slides. A window of constant signed volume has
/// zero variance and `λ` is undefined; the indicator returns `0` in that case
/// rather than producing `NaN`.
///
/// `Input = TradeQuote`, `Output = f64`. It warms up for `window + 1`
/// trade-quotes: one to seed the previous mid, then `window` paired
/// observations.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, KylesLambda, Side, Trade, TradeQuote};
///
/// // A book where each trade moves the mid by exactly 0.5 per unit of signed
/// // volume gives λ = 0.5.
/// let mut lambda = KylesLambda::new(8).unwrap();
/// let mut mid = 100.0;
/// let mut last = None;
/// for i in 0..20 {
///     let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
///     let size = 1.0 + f64::from(i % 3);
///     let signed = size * side.sign();
///     mid += 0.5 * signed;
///     let trade = Trade::new(mid, size, side, 0).unwrap();
///     last = lambda.update(TradeQuote::new(trade, mid).unwrap());
/// }
/// assert!((last.unwrap() - 0.5).abs() < 1e-9);
/// ```
#[derive(Debug, Clone)]
pub struct KylesLambda {
    window: usize,
    prev_mid: Option<f64>,
    pairs: VecDeque<(f64, f64)>,
    sum_q: f64,
    sum_dm: f64,
    sum_qq: f64,
    sum_qdm: f64,
}

impl KylesLambda {
    /// Construct a rolling Kyle's lambda over `window` paired observations.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPeriod`] if `window < 2` (the regression
    /// variance needs at least two observations).
    pub fn new(window: usize) -> Result<Self> {
        if window < 2 {
            return Err(Error::InvalidPeriod {
                message: "kyle's lambda needs window >= 2",
            });
        }
        Ok(Self {
            window,
            prev_mid: None,
            pairs: VecDeque::with_capacity(window),
            sum_q: 0.0,
            sum_dm: 0.0,
            sum_qq: 0.0,
            sum_qdm: 0.0,
        })
    }

    /// The configured window length, in paired observations.
    pub const fn window(&self) -> usize {
        self.window
    }

    fn push_pair(&mut self, signed_vol: f64, delta_mid: f64) -> Option<f64> {
        if self.pairs.len() == self.window {
            let (old_q, old_dm) = self.pairs.pop_front().expect("non-empty");
            self.sum_q -= old_q;
            self.sum_dm -= old_dm;
            self.sum_qq -= old_q * old_q;
            self.sum_qdm -= old_q * old_dm;
        }
        self.pairs.push_back((signed_vol, delta_mid));
        self.sum_q += signed_vol;
        self.sum_dm += delta_mid;
        self.sum_qq += signed_vol * signed_vol;
        self.sum_qdm += signed_vol * delta_mid;
        if self.pairs.len() < self.window {
            return None;
        }
        let n = self.window as f64;
        let mean_q = self.sum_q / n;
        let mean_dm = self.sum_dm / n;
        let var_q = (self.sum_qq / n - mean_q * mean_q).max(0.0);
        let cov = self.sum_qdm / n - mean_q * mean_dm;
        if var_q == 0.0 {
            // Constant signed-volume window has no defined slope.
            return Some(0.0);
        }
        Some(cov / var_q)
    }
}

impl Indicator for KylesLambda {
    type Input = TradeQuote;
    type Output = f64;

    fn update(&mut self, quote: TradeQuote) -> Option<f64> {
        let mid = quote.mid;
        let signed_vol = quote.trade.size * quote.trade.side.sign();
        let Some(prev) = self.prev_mid else {
            self.prev_mid = Some(mid);
            return None;
        };
        self.prev_mid = Some(mid);
        self.push_pair(signed_vol, mid - prev)
    }

    fn reset(&mut self) {
        self.prev_mid = None;
        self.pairs.clear();
        self.sum_q = 0.0;
        self.sum_dm = 0.0;
        self.sum_qq = 0.0;
        self.sum_qdm = 0.0;
    }

    fn warmup_period(&self) -> usize {
        self.window + 1
    }

    fn is_ready(&self) -> bool {
        self.pairs.len() == self.window
    }

    fn name(&self) -> &'static str {
        "KylesLambda"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::microstructure::{Side, Trade};
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn quotes_with_impact(n: usize, impact: f64) -> Vec<TradeQuote> {
        let mut mid = 100.0;
        (0..n)
            .map(|i| {
                let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
                let size = 1.0 + (i % 3) as f64;
                let signed = size * side.sign();
                mid += impact * signed;
                let trade = Trade::new(mid, size, side, 0).unwrap();
                TradeQuote::new(trade, mid).unwrap()
            })
            .collect()
    }

    #[test]
    fn rejects_window_below_two() {
        assert!(KylesLambda::new(0).is_err());
        assert!(KylesLambda::new(1).is_err());
        assert!(KylesLambda::new(2).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let kl = KylesLambda::new(14).unwrap();
        assert_eq!(kl.name(), "KylesLambda");
        assert_eq!(kl.window(), 14);
        assert_eq!(kl.warmup_period(), 15);
        assert!(!kl.is_ready());
    }

    #[test]
    fn recovers_constant_impact_slope() {
        // mid moves exactly 0.5 per unit signed volume -> lambda = 0.5.
        let last = KylesLambda::new(6)
            .unwrap()
            .batch(&quotes_with_impact(20, 0.5))
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, 0.5, epsilon = 1e-9);
    }

    #[test]
    fn negative_impact_reads_negative() {
        let last = KylesLambda::new(6)
            .unwrap()
            .batch(&quotes_with_impact(20, -0.3))
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, -0.3, epsilon = 1e-9);
    }

    #[test]
    fn constant_signed_volume_is_zero() {
        // Every trade is a buy of size 1: signed volume is constant -> var 0 -> 0.
        let mut mid = 100.0;
        let quotes: Vec<TradeQuote> = (0..10)
            .map(|_| {
                mid += 0.01;
                let trade = Trade::new(mid, 1.0, Side::Buy, 0).unwrap();
                TradeQuote::new(trade, mid).unwrap()
            })
            .collect();
        let last = KylesLambda::new(5)
            .unwrap()
            .batch(&quotes)
            .into_iter()
            .flatten()
            .last()
            .unwrap();
        assert_relative_eq!(last, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn warms_up_after_window_plus_one() {
        let mut kl = KylesLambda::new(3).unwrap();
        let quotes = quotes_with_impact(4, 0.2);
        assert_eq!(kl.update(quotes[0]), None); // seeds prev mid
        assert_eq!(kl.update(quotes[1]), None);
        assert_eq!(kl.update(quotes[2]), None);
        assert!(!kl.is_ready());
        assert!(kl.update(quotes[3]).is_some());
        assert!(kl.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let quotes = quotes_with_impact(40, 0.15);
        let batch = KylesLambda::new(10).unwrap().batch(&quotes);
        let mut kl = KylesLambda::new(10).unwrap();
        let streamed: Vec<_> = quotes.iter().map(|q| kl.update(*q)).collect();
        assert_eq!(batch, streamed);
    }

    #[test]
    fn reset_clears_state() {
        let mut kl = KylesLambda::new(3).unwrap();
        for q in quotes_with_impact(6, 0.2) {
            kl.update(q);
        }
        assert!(kl.is_ready());
        kl.reset();
        assert!(!kl.is_ready());
        assert_eq!(kl.update(quotes_with_impact(1, 0.2)[0]), None);
    }
}
