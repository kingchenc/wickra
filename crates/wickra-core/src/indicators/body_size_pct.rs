//! Body Size Percent — candle body as a fraction of its range.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Body Size Percent — the absolute body as a fraction of the bar's range.
///
/// ```text
/// BodySizePct = |close − open| / (high − low)
/// ```
///
/// The result lives in `[0, 1]`: `1` is a full-bodied marubozu (the bar opened
/// at one extreme and closed at the other, no wicks), `0` a doji (open equals
/// close, the bar is all wick). It is the *unsigned* magnitude companion to
/// [`BalanceOfPower`](crate::BalanceOfPower) — where `BoP` keeps the direction,
/// this keeps only the conviction, which is exactly what candlestick body /
/// range filters key on. A zero-range bar carries no information and yields `0`.
///
/// This is a stateless per-bar transform: every candle produces one value.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, BodySizePct};
///
/// let mut indicator = BodySizePct::new();
/// // body |12 - 10| = 2, range 14 - 10 = 4 -> 0.5.
/// let c = Candle::new(10.0, 14.0, 10.0, 12.0, 10.0, 0).unwrap();
/// assert!((indicator.update(c).unwrap() - 0.5).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Default)]
pub struct BodySizePct {
    has_emitted: bool,
}

impl BodySizePct {
    /// Construct a new Body Size Percent transform.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for BodySizePct {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let range = candle.high - candle.low;
        let out = if range == 0.0 {
            // A zero-range bar has no body proportion to speak of.
            0.0
        } else {
            (candle.close - candle.open).abs() / range
        };
        Some(out)
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
        "BodySizePct"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(open: f64, high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(open, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn reference_value() {
        // |12 - 10| / (14 - 10) = 0.5.
        let mut bsp = BodySizePct::new();
        assert_relative_eq!(
            bsp.update(candle(10.0, 14.0, 10.0, 12.0, 0)).unwrap(),
            0.5,
            epsilon = 1e-12
        );
    }

    #[test]
    fn marubozu_is_one() {
        // open == low, close == high, no wicks -> full body -> 1.
        let mut bsp = BodySizePct::new();
        assert_relative_eq!(
            bsp.update(candle(9.0, 11.0, 9.0, 11.0, 0)).unwrap(),
            1.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn doji_is_zero() {
        // open == close with a real range -> body 0.
        let mut bsp = BodySizePct::new();
        assert_relative_eq!(
            bsp.update(candle(10.0, 12.0, 8.0, 10.0, 0)).unwrap(),
            0.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn unsigned_regardless_of_direction() {
        // A red bar with the same body magnitude reads identically to a green one.
        let mut bsp = BodySizePct::new();
        let green = bsp.update(candle(10.0, 14.0, 10.0, 12.0, 0)).unwrap();
        let mut bsp2 = BodySizePct::new();
        let red = bsp2.update(candle(12.0, 14.0, 10.0, 10.0, 0)).unwrap();
        assert_relative_eq!(green, red, epsilon = 1e-12);
    }

    #[test]
    fn zero_range_bar_yields_zero() {
        let mut bsp = BodySizePct::new();
        assert_relative_eq!(
            bsp.update(candle(10.0, 10.0, 10.0, 10.0, 0)).unwrap(),
            0.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn stays_within_unit_range() {
        let candles: Vec<Candle> = (0..100)
            .map(|i| {
                let mid = 100.0 + (f64::from(i) * 0.2).sin() * 8.0;
                let close = mid + (f64::from(i) * 0.5).cos() * 2.0;
                candle(mid, mid + 3.0, mid - 3.0, close, i64::from(i))
            })
            .collect();
        let mut bsp = BodySizePct::new();
        for v in bsp.batch(&candles).into_iter().flatten() {
            assert!((0.0..=1.0).contains(&v), "BodySizePct {v} outside [0, 1]");
        }
    }

    #[test]
    fn name_metadata() {
        let bsp = BodySizePct::new();
        assert_eq!(bsp.name(), "BodySizePct");
    }

    #[test]
    fn emits_from_first_candle() {
        let mut bsp = BodySizePct::new();
        assert_eq!(bsp.warmup_period(), 1);
        assert!(!bsp.is_ready());
        assert!(bsp.update(candle(10.0, 11.0, 9.0, 10.0, 0)).is_some());
        assert!(bsp.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut bsp = BodySizePct::new();
        bsp.update(candle(10.0, 11.0, 9.0, 10.0, 0));
        assert!(bsp.is_ready());
        bsp.reset();
        assert!(!bsp.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + f64::from(i);
                candle(base, base + 2.0, base - 2.0, base + 1.0, i64::from(i))
            })
            .collect();
        let mut a = BodySizePct::new();
        let mut b = BodySizePct::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
