//! Camarilla Pivot Points (Nick Stott).

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Camarilla Pivot Points output: four resistances, the pivot, four supports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamarillaPivotsOutput {
    /// Pivot Point: `(H + L + C) / 3` (informational, not in the Camarilla R/S formulas).
    pub pp: f64,
    /// Resistance 1: `C + (H − L)·1.1/12`.
    pub r1: f64,
    /// Resistance 2: `C + (H − L)·1.1/6`.
    pub r2: f64,
    /// Resistance 3: `C + (H − L)·1.1/4`.
    pub r3: f64,
    /// Resistance 4: `C + (H − L)·1.1/2`.
    pub r4: f64,
    /// Support 1: `C − (H − L)·1.1/12`.
    pub s1: f64,
    /// Support 2: `C − (H − L)·1.1/6`.
    pub s2: f64,
    /// Support 3: `C − (H − L)·1.1/4`.
    pub s3: f64,
    /// Support 4: `C − (H − L)·1.1/2`.
    pub s4: f64,
}

/// Camarilla Pivot Points — Nick Stott's four-tier range-based level set.
/// Anchored on the prior close rather than the typical price, with widths
/// scaled by the constant `1.1` divided by `{12, 6, 4, 2}`.
///
/// ```text
/// PP = (H + L + C) / 3
/// R_n = C + (H − L) · 1.1 / d_n     S_n = C − (H − L) · 1.1 / d_n
///   where d_1 = 12, d_2 = 6, d_3 = 4, d_4 = 2
/// ```
///
/// R3/S3 are typically used as reversal levels; R4/S4 as breakout levels. As
/// with the other pivot variants there are no parameters and no warmup — the
/// first candle produces the first set of levels.
///
/// # Example
///
/// ```
/// use wickra_core::{Camarilla, Candle, Indicator};
///
/// let prev = Candle::new(100.0, 110.0, 90.0, 105.0, 1.0, 0).unwrap();
/// let levels = Camarilla::new().update(prev).unwrap();
/// assert!(levels.r4 > levels.r3);
/// assert!(levels.s4 < levels.s3);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Camarilla {
    ready: bool,
}

impl Camarilla {
    /// Construct a new Camarilla Pivot Points indicator.
    pub const fn new() -> Self {
        Self { ready: false }
    }
}

const CAM: f64 = 1.1;

impl Indicator for Camarilla {
    type Input = Candle;
    type Output = CamarillaPivotsOutput;

    fn update(&mut self, candle: Candle) -> Option<CamarillaPivotsOutput> {
        let (h, l, c) = (candle.high, candle.low, candle.close);
        let range = h - l;
        let pp = (h + l + c) / 3.0;
        let w1 = range * CAM / 12.0;
        let w2 = range * CAM / 6.0;
        let w3 = range * CAM / 4.0;
        let w4 = range * CAM / 2.0;
        let out = CamarillaPivotsOutput {
            pp,
            r1: c + w1,
            r2: c + w2,
            r3: c + w3,
            r4: c + w4,
            s1: c - w1,
            s2: c - w2,
            s3: c - w3,
            s4: c - w4,
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
        "Camarilla"
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
        // H=110, L=90, C=105, range=20.
        let levels = Camarilla::new().update(c(110.0, 90.0, 105.0, 0)).unwrap();
        let range = 20.0;
        assert!((levels.r1 - (105.0 + range * 1.1 / 12.0)).abs() < 1e-12);
        assert!((levels.r2 - (105.0 + range * 1.1 / 6.0)).abs() < 1e-12);
        assert!((levels.r3 - (105.0 + range * 1.1 / 4.0)).abs() < 1e-12);
        assert!((levels.r4 - (105.0 + range * 1.1 / 2.0)).abs() < 1e-12);
        assert!((levels.s1 - (105.0 - range * 1.1 / 12.0)).abs() < 1e-12);
        assert!((levels.s4 - (105.0 - range * 1.1 / 2.0)).abs() < 1e-12);
    }

    #[test]
    fn resistance_strictly_widens_with_index() {
        let levels = Camarilla::new().update(c(120.0, 80.0, 110.0, 0)).unwrap();
        assert!(levels.r4 > levels.r3);
        assert!(levels.r3 > levels.r2);
        assert!(levels.r2 > levels.r1);
        assert!(levels.r1 > 110.0);
        assert!(levels.s1 < 110.0);
        assert!(levels.s2 < levels.s1);
        assert!(levels.s3 < levels.s2);
        assert!(levels.s4 < levels.s3);
    }

    #[test]
    fn constant_series_collapses_levels() {
        let levels = Camarilla::new().update(c(50.0, 50.0, 50.0, 0)).unwrap();
        assert_eq!(levels.r4, 50.0);
        assert_eq!(levels.s4, 50.0);
        assert_eq!(levels.pp, 50.0);
    }

    #[test]
    fn warmup_and_ready() {
        let mut p = Camarilla::new();
        assert!(!p.is_ready());
        assert_eq!(p.warmup_period(), 1);
        p.update(c(11.0, 9.0, 10.0, 0));
        assert!(p.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut p = Camarilla::new();
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
        let mut a = Camarilla::new();
        let mut b = Camarilla::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn accessors_and_metadata() {
        let p = Camarilla::new();
        assert_eq!(p.warmup_period(), 1);
        assert_eq!(p.name(), "Camarilla");
    }
}
