//! `DeMark` Pivot Points.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// `DeMark` Pivot Points output: a single resistance, pivot and support.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DemarkPivotsOutput {
    /// Pivot Point: `X / 4` where `X` is the conditional sum (see [`DemarkPivots`]).
    pub pp: f64,
    /// Resistance 1: `X / 2 − L`.
    pub r1: f64,
    /// Support 1: `X / 2 − H`.
    pub s1: f64,
}

/// `DeMark` Pivot Points — Tom `DeMark`'s conditional pivot formulation, derived
/// from a sum `X` that depends on whether the bar closed up, down or flat.
///
/// ```text
/// X = 2·H + L + C   if C  < O   (down bar)
///     H + 2·L + C   if C  > O   (up bar)
///     H + L + 2·C   if C == O   (doji)
///
/// PP = X / 4
/// R1 = X / 2 − L
/// S1 = X / 2 − H
/// ```
///
/// Unlike the classic pivots, only one resistance and one support are
/// produced; `DeMark`'s intent is a tighter, condition-sensitive set rather than
/// a multi-tier fan. The branching means a bar's open carries information that
/// other pivot variants discard.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, DemarkPivots, Indicator};
///
/// // Up bar: O=100, H=120, L=80, C=110 -> X = H + 2·L + C = 390.
/// let up = Candle::new(100.0, 120.0, 80.0, 110.0, 1.0, 0).unwrap();
/// let lv = DemarkPivots::new().update(up).unwrap();
/// assert!((lv.pp - 97.5).abs() < 1e-9);
/// ```
#[derive(Debug, Clone, Default)]
pub struct DemarkPivots {
    ready: bool,
}

impl DemarkPivots {
    /// Construct a new `DeMark` Pivot Points indicator.
    pub const fn new() -> Self {
        Self { ready: false }
    }
}

impl Indicator for DemarkPivots {
    type Input = Candle;
    type Output = DemarkPivotsOutput;

    fn update(&mut self, candle: Candle) -> Option<DemarkPivotsOutput> {
        let open = candle.open;
        let high = candle.high;
        let low = candle.low;
        let close = candle.close;
        let x = if close < open {
            2.0 * high + low + close
        } else if close > open {
            high + 2.0 * low + close
        } else {
            high + low + 2.0 * close
        };
        let pp = x / 4.0;
        let half = x / 2.0;
        let out = DemarkPivotsOutput {
            pp,
            r1: half - low,
            s1: half - high,
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
        "DemarkPivots"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    #[test]
    fn down_bar_uses_2h_plus_l_plus_c() {
        // O=110, H=120, L=80, C=100 (close < open) -> X = 2·120 + 80 + 100 = 420.
        let cd = Candle::new(110.0, 120.0, 80.0, 100.0, 1.0, 0).unwrap();
        let lv = DemarkPivots::new().update(cd).unwrap();
        assert!((lv.pp - 105.0).abs() < 1e-12);
        assert!((lv.r1 - (210.0 - 80.0)).abs() < 1e-12);
        assert!((lv.s1 - (210.0 - 120.0)).abs() < 1e-12);
    }

    #[test]
    fn up_bar_uses_h_plus_2l_plus_c() {
        // O=100, H=120, L=80, C=110 (close > open) -> X = 120 + 160 + 110 = 390.
        let cd = Candle::new(100.0, 120.0, 80.0, 110.0, 1.0, 0).unwrap();
        let lv = DemarkPivots::new().update(cd).unwrap();
        assert!((lv.pp - 97.5).abs() < 1e-12);
        assert!((lv.r1 - (195.0 - 80.0)).abs() < 1e-12);
        assert!((lv.s1 - (195.0 - 120.0)).abs() < 1e-12);
    }

    #[test]
    fn doji_uses_h_plus_l_plus_2c() {
        // O = C = 100, H=120, L=80 -> X = 120 + 80 + 200 = 400.
        let cd = Candle::new(100.0, 120.0, 80.0, 100.0, 1.0, 0).unwrap();
        let lv = DemarkPivots::new().update(cd).unwrap();
        assert!((lv.pp - 100.0).abs() < 1e-12);
    }

    #[test]
    fn ordering_resistance_above_pivot_above_support() {
        let cd = Candle::new(100.0, 120.0, 80.0, 110.0, 1.0, 0).unwrap();
        let lv = DemarkPivots::new().update(cd).unwrap();
        assert!(lv.r1 >= lv.pp);
        assert!(lv.pp >= lv.s1);
    }

    #[test]
    fn constant_series_collapses_levels() {
        let cd = Candle::new(50.0, 50.0, 50.0, 50.0, 1.0, 0).unwrap();
        let lv = DemarkPivots::new().update(cd).unwrap();
        assert_eq!(lv.pp, 50.0);
        assert_eq!(lv.r1, 50.0);
        assert_eq!(lv.s1, 50.0);
    }

    #[test]
    fn warmup_and_ready() {
        let mut p = DemarkPivots::new();
        assert!(!p.is_ready());
        assert_eq!(p.warmup_period(), 1);
        let cd = Candle::new(10.0, 11.0, 9.0, 10.0, 1.0, 0).unwrap();
        p.update(cd);
        assert!(p.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut p = DemarkPivots::new();
        let cd = Candle::new(10.0, 11.0, 9.0, 10.0, 1.0, 0).unwrap();
        p.update(cd);
        p.reset();
        assert!(!p.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = f64::from(i);
                Candle::new(base, base + 2.0, base - 0.5, base + 1.0, 1.0, i64::from(i)).unwrap()
            })
            .collect();
        let mut a = DemarkPivots::new();
        let mut b = DemarkPivots::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn accessors_and_metadata() {
        let p = DemarkPivots::new();
        assert_eq!(p.warmup_period(), 1);
        assert_eq!(p.name(), "DemarkPivots");
    }
}
