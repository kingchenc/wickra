//! Woodie Pivot Points (Tom Williams).

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Woodie Pivot Points output: two resistances, pivot, two supports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WoodiePivotsOutput {
    /// Pivot Point: `(H + L + 2·C) / 4`.
    pub pp: f64,
    /// Resistance 1: `2·PP − L`.
    pub r1: f64,
    /// Resistance 2: `PP + (H − L)`.
    pub r2: f64,
    /// Support 1: `2·PP − H`.
    pub s1: f64,
    /// Support 2: `PP − (H − L)`.
    pub s2: f64,
}

/// Woodie Pivot Points — Tom Williams' close-weighted pivot variant.
///
/// ```text
/// PP = (H + L + 2·C) / 4
/// R1 = 2·PP − L        S1 = 2·PP − H
/// R2 = PP + (H − L)    S2 = PP − (H − L)
/// ```
///
/// The double-weighted close shifts the pivot toward where most of the
/// session's activity actually settled — useful in trending markets where the
/// close is more meaningful than the midpoint.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, WoodiePivots};
///
/// let prev = Candle::new(100.0, 110.0, 90.0, 108.0, 1.0, 0).unwrap();
/// let levels = WoodiePivots::new().update(prev).unwrap();
/// // Close-weighted PP = (110 + 90 + 2·108)/4 = 104.
/// assert!((levels.pp - 104.0).abs() < 1e-9);
/// ```
#[derive(Debug, Clone, Default)]
pub struct WoodiePivots {
    ready: bool,
}

impl WoodiePivots {
    /// Construct a new Woodie Pivot Points indicator.
    pub const fn new() -> Self {
        Self { ready: false }
    }
}

impl Indicator for WoodiePivots {
    type Input = Candle;
    type Output = WoodiePivotsOutput;

    fn update(&mut self, candle: Candle) -> Option<WoodiePivotsOutput> {
        let (h, l, c) = (candle.high, candle.low, candle.close);
        let pp = (h + l + 2.0 * c) / 4.0;
        let range = h - l;
        let out = WoodiePivotsOutput {
            pp,
            r1: 2.0 * pp - l,
            r2: pp + range,
            s1: 2.0 * pp - h,
            s2: pp - range,
        };
        self.ready = true;
        Some(out)
    }

    fn reset(&mut self) {
        self.ready = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn name(&self) -> &'static str {
        "WoodiePivots"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(h: f64, l: f64, close: f64, ts: i64) -> Candle {
        Candle::new(close, h, l, close, 1.0, ts).unwrap()
    }

    #[test]
    fn formula_reference_values() {
        // H=110, L=90, C=108 -> PP = (110+90+216)/4 = 104.
        let levels = WoodiePivots::new()
            .update(c(110.0, 90.0, 108.0, 0))
            .unwrap();
        assert!((levels.pp - 104.0).abs() < 1e-12);
        assert!((levels.r1 - (2.0 * 104.0 - 90.0)).abs() < 1e-12);
        assert!((levels.s1 - (2.0 * 104.0 - 110.0)).abs() < 1e-12);
        assert!((levels.r2 - (104.0 + 20.0)).abs() < 1e-12);
        assert!((levels.s2 - (104.0 - 20.0)).abs() < 1e-12);
    }

    #[test]
    fn pp_differs_from_classic_when_close_is_skewed() {
        // Classic PP = (H+L+C)/3; Woodie PP weights close twice. They agree
        // only when C equals (H+L)/2.
        let levels = WoodiePivots::new()
            .update(c(120.0, 80.0, 110.0, 0))
            .unwrap();
        let classic_pp = (120.0 + 80.0 + 110.0) / 3.0;
        assert!((levels.pp - classic_pp).abs() > 1e-6);
        // Equal when close = midpoint.
        let mid = WoodiePivots::new()
            .update(c(120.0, 80.0, 100.0, 0))
            .unwrap();
        let classic_mid = (120.0 + 80.0 + 100.0) / 3.0;
        assert!((mid.pp - classic_mid).abs() < 1e-9);
    }

    #[test]
    fn ordering_resistance_above_pivot_above_support() {
        let levels = WoodiePivots::new()
            .update(c(120.0, 80.0, 110.0, 0))
            .unwrap();
        assert!(levels.r2 >= levels.r1);
        assert!(levels.r1 >= levels.pp);
        assert!(levels.pp >= levels.s1);
        assert!(levels.s1 >= levels.s2);
    }

    #[test]
    fn constant_series_collapses_levels() {
        let levels = WoodiePivots::new().update(c(50.0, 50.0, 50.0, 0)).unwrap();
        assert_eq!(levels.pp, 50.0);
        assert_eq!(levels.r2, 50.0);
        assert_eq!(levels.s2, 50.0);
    }

    #[test]
    fn warmup_and_ready() {
        let mut p = WoodiePivots::new();
        assert!(!p.is_ready());
        assert_eq!(p.warmup_period(), 1);
        p.update(c(11.0, 9.0, 10.0, 0));
        assert!(p.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut p = WoodiePivots::new();
        p.update(c(11.0, 9.0, 10.0, 0));
        p.reset();
        assert!(!p.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0_i32..40)
            .map(|i| {
                c(
                    f64::from(i) + 2.0,
                    f64::from(i),
                    f64::from(i) + 1.0,
                    i.into(),
                )
            })
            .collect();
        let mut a = WoodiePivots::new();
        let mut b = WoodiePivots::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn accessors_and_metadata() {
        let p = WoodiePivots::new();
        assert_eq!(p.warmup_period(), 1);
        assert_eq!(p.name(), "WoodiePivots");
    }
}
