//! Classic (Floor-Trader) Pivot Points.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Classic Pivot Points output: pivot plus three resistances and three supports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClassicPivotsOutput {
    /// Pivot Point: `(H + L + C) / 3`.
    pub pp: f64,
    /// Resistance 1: `2·PP − L`.
    pub r1: f64,
    /// Resistance 2: `PP + (H − L)`.
    pub r2: f64,
    /// Resistance 3: `H + 2·(PP − L)`.
    pub r3: f64,
    /// Support 1: `2·PP − H`.
    pub s1: f64,
    /// Support 2: `PP − (H − L)`.
    pub s2: f64,
    /// Support 3: `L − 2·(H − PP)`.
    pub s3: f64,
}

/// Classic (Floor-Trader) Pivot Points — the standard pivot/resistance/support
/// levels computed from a completed candle's high, low and close.
///
/// ```text
/// PP = (H + L + C) / 3
/// R1 = 2·PP − L          S1 = 2·PP − H
/// R2 = PP + (H − L)      S2 = PP − (H − L)
/// R3 = H + 2·(PP − L)    S3 = L − 2·(H − PP)
/// ```
///
/// Pivots are typically computed once per session (day, week, month) from the
/// **previous** session's bar and used as fixed reference levels for the next
/// session. The streaming API here simply re-evaluates the formula on every
/// candle it sees, which makes it a one-step transform you can wire to any
/// pre-aggregated session bar. There are no parameters and no warmup — the
/// first candle produces the first set of levels.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, ClassicPivots, Indicator};
///
/// let prev = Candle::new(100.0, 110.0, 90.0, 105.0, 1.0, 0).unwrap();
/// let mut pp = ClassicPivots::new();
/// let levels = pp.update(prev).unwrap();
/// assert!((levels.pp - 101.6666666666).abs() < 1e-9);
/// assert!(levels.r1 > levels.pp);
/// assert!(levels.s1 < levels.pp);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ClassicPivots {
    ready: bool,
}

impl ClassicPivots {
    /// Construct a new Classic Pivot Points indicator. The indicator has no
    /// parameters and no warmup.
    pub const fn new() -> Self {
        Self { ready: false }
    }
}

impl Indicator for ClassicPivots {
    type Input = Candle;
    type Output = ClassicPivotsOutput;

    fn update(&mut self, candle: Candle) -> Option<ClassicPivotsOutput> {
        let (h, l, c) = (candle.high, candle.low, candle.close);
        let pp = (h + l + c) / 3.0;
        let range = h - l;
        let out = ClassicPivotsOutput {
            pp,
            r1: 2.0 * pp - l,
            r2: pp + range,
            r3: h + 2.0 * (pp - l),
            s1: 2.0 * pp - h,
            s2: pp - range,
            s3: l - 2.0 * (h - pp),
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
        "ClassicPivots"
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
        // H=110, L=90, C=105 -> PP = 305/3 ≈ 101.6667.
        let levels = ClassicPivots::new()
            .update(c(110.0, 90.0, 105.0, 0))
            .unwrap();
        let pp = 305.0 / 3.0;
        let range = 20.0;
        assert!((levels.pp - pp).abs() < 1e-12);
        assert!((levels.r1 - (2.0 * pp - 90.0)).abs() < 1e-12);
        assert!((levels.s1 - (2.0 * pp - 110.0)).abs() < 1e-12);
        assert!((levels.r2 - (pp + range)).abs() < 1e-12);
        assert!((levels.s2 - (pp - range)).abs() < 1e-12);
        assert!((levels.r3 - (110.0 + 2.0 * (pp - 90.0))).abs() < 1e-12);
        assert!((levels.s3 - (90.0 - 2.0 * (110.0 - pp))).abs() < 1e-12);
    }

    #[test]
    fn ordering_resistance_above_pivot_above_support() {
        // For any non-degenerate bar with H > L, R-levels exceed PP and S-levels lie below.
        let levels = ClassicPivots::new()
            .update(c(200.0, 100.0, 150.0, 0))
            .unwrap();
        assert!(levels.r3 >= levels.r2);
        assert!(levels.r2 >= levels.r1);
        assert!(levels.r1 >= levels.pp);
        assert!(levels.pp >= levels.s1);
        assert!(levels.s1 >= levels.s2);
        assert!(levels.s2 >= levels.s3);
    }

    #[test]
    fn constant_series_collapses_levels() {
        // H = L = C means range = 0 and every level equals the close.
        let levels = ClassicPivots::new().update(c(50.0, 50.0, 50.0, 0)).unwrap();
        assert_eq!(levels.pp, 50.0);
        assert_eq!(levels.r1, 50.0);
        assert_eq!(levels.s1, 50.0);
        assert_eq!(levels.r2, 50.0);
        assert_eq!(levels.s2, 50.0);
        assert_eq!(levels.r3, 50.0);
        assert_eq!(levels.s3, 50.0);
    }

    #[test]
    fn ready_after_first_update_warmup_is_one() {
        let mut pp = ClassicPivots::new();
        assert!(!pp.is_ready());
        assert_eq!(pp.warmup_period(), 1);
        pp.update(c(11.0, 9.0, 10.0, 0));
        assert!(pp.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut pp = ClassicPivots::new();
        pp.update(c(11.0, 9.0, 10.0, 0));
        assert!(pp.is_ready());
        pp.reset();
        assert!(!pp.is_ready());
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
        let mut a = ClassicPivots::new();
        let mut b = ClassicPivots::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn accessors_and_metadata() {
        let pp = ClassicPivots::new();
        assert_eq!(pp.warmup_period(), 1);
        assert_eq!(pp.name(), "ClassicPivots");
    }
}
