//! Fibonacci Pivot Points.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Fibonacci Pivot Points output: pivot plus three Fib-spaced resistances and
/// supports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FibonacciPivotsOutput {
    /// Pivot Point: `(H + L + C) / 3`.
    pub pp: f64,
    /// Resistance 1: `PP + 0.382·(H − L)`.
    pub r1: f64,
    /// Resistance 2: `PP + 0.618·(H − L)`.
    pub r2: f64,
    /// Resistance 3: `PP + 1.000·(H − L)`.
    pub r3: f64,
    /// Support 1: `PP − 0.382·(H − L)`.
    pub s1: f64,
    /// Support 2: `PP − 0.618·(H − L)`.
    pub s2: f64,
    /// Support 3: `PP − 1.000·(H − L)`.
    pub s3: f64,
}

/// Fibonacci Pivot Points — the classic pivot plus three resistances and
/// supports spaced by the Fibonacci ratios 0.382 / 0.618 / 1.000 applied to
/// the prior bar's range.
///
/// ```text
/// PP = (H + L + C) / 3
/// R1 = PP + 0.382·(H − L)   S1 = PP − 0.382·(H − L)
/// R2 = PP + 0.618·(H − L)   S2 = PP − 0.618·(H − L)
/// R3 = PP + 1.000·(H − L)   S3 = PP − 1.000·(H − L)
/// ```
///
/// As with [`crate::ClassicPivots`], levels are typically built from the
/// previous session's bar. There are no parameters and no warmup.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, FibonacciPivots, Indicator};
///
/// let prev = Candle::new(100.0, 110.0, 90.0, 105.0, 1.0, 0).unwrap();
/// let levels = FibonacciPivots::new().update(prev).unwrap();
/// assert!(levels.r3 > levels.r2);
/// assert!(levels.r2 > levels.r1);
/// assert!(levels.s1 > levels.s2);
/// ```
#[derive(Debug, Clone, Default)]
pub struct FibonacciPivots {
    ready: bool,
}

impl FibonacciPivots {
    /// Construct a new Fibonacci Pivot Points indicator.
    pub const fn new() -> Self {
        Self { ready: false }
    }
}

const FIB1: f64 = 0.382;
const FIB2: f64 = 0.618;
const FIB3: f64 = 1.000;

impl Indicator for FibonacciPivots {
    type Input = Candle;
    type Output = FibonacciPivotsOutput;

    fn update(&mut self, candle: Candle) -> Option<FibonacciPivotsOutput> {
        let (h, l, c) = (candle.high, candle.low, candle.close);
        let pp = (h + l + c) / 3.0;
        let range = h - l;
        let out = FibonacciPivotsOutput {
            pp,
            r1: pp + FIB1 * range,
            r2: pp + FIB2 * range,
            r3: pp + FIB3 * range,
            s1: pp - FIB1 * range,
            s2: pp - FIB2 * range,
            s3: pp - FIB3 * range,
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
        "FibonacciPivots"
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
        // H=110, L=90, range=20, PP = (110+90+100)/3 = 100.
        let levels = FibonacciPivots::new()
            .update(c(110.0, 90.0, 100.0, 0))
            .unwrap();
        assert!((levels.pp - 100.0).abs() < 1e-12);
        assert!((levels.r1 - (100.0 + 0.382 * 20.0)).abs() < 1e-12);
        assert!((levels.r2 - (100.0 + 0.618 * 20.0)).abs() < 1e-12);
        assert!((levels.r3 - (100.0 + 20.0)).abs() < 1e-12);
        assert!((levels.s1 - (100.0 - 0.382 * 20.0)).abs() < 1e-12);
        assert!((levels.s2 - (100.0 - 0.618 * 20.0)).abs() < 1e-12);
        assert!((levels.s3 - (100.0 - 20.0)).abs() < 1e-12);
    }

    #[test]
    fn resistances_strictly_above_pp_supports_strictly_below() {
        let levels = FibonacciPivots::new()
            .update(c(120.0, 80.0, 110.0, 0))
            .unwrap();
        assert!(levels.r3 > levels.r2);
        assert!(levels.r2 > levels.r1);
        assert!(levels.r1 > levels.pp);
        assert!(levels.pp > levels.s1);
        assert!(levels.s1 > levels.s2);
        assert!(levels.s2 > levels.s3);
    }

    #[test]
    fn constant_series_collapses_levels() {
        let levels = FibonacciPivots::new()
            .update(c(50.0, 50.0, 50.0, 0))
            .unwrap();
        assert_eq!(levels.pp, 50.0);
        assert_eq!(levels.r1, 50.0);
        assert_eq!(levels.s3, 50.0);
    }

    #[test]
    fn warmup_and_ready() {
        let mut p = FibonacciPivots::new();
        assert!(!p.is_ready());
        assert_eq!(p.warmup_period(), 1);
        p.update(c(11.0, 9.0, 10.0, 0));
        assert!(p.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut p = FibonacciPivots::new();
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
        let mut a = FibonacciPivots::new();
        let mut b = FibonacciPivots::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn accessors_and_metadata() {
        let p = FibonacciPivots::new();
        assert_eq!(p.warmup_period(), 1);
        assert_eq!(p.name(), "FibonacciPivots");
    }
}
