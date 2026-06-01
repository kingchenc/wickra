//! Three White Soldiers / Three Black Crows candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Three White Soldiers / Three Black Crows — a 3-bar continuation pattern of
/// three consecutive long candles in the same direction, each opening inside
/// the previous body and closing beyond it.
///
/// **Three White Soldiers** (`+1.0`):
/// ```text
/// all three green & monotonically rising closes
///   & each open in [prev.open, prev.close]
///   & each close > prev.close
/// ```
///
/// **Three Black Crows** (`−1.0`): the mirror — three red candles with
/// monotonically falling closes.
///
/// Pattern-shape check only — no trend filter is applied; combine with a trend
/// indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector already emits the uniform candlestick sign convention shared
/// across the pattern family — `+1.0` bullish, `−1.0` bearish, `0.0` no
/// pattern — so it drops straight into a machine-learning feature matrix where
/// the bullish and bearish variants of the pattern occupy a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, ThreeSoldiersOrCrows};
///
/// let mut indicator = ThreeSoldiersOrCrows::new();
/// indicator.update(Candle::new(10.0, 11.5, 9.9, 11.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(10.5, 12.5, 10.4, 12.0, 1.0, 1).unwrap());
/// let out = indicator
///     .update(Candle::new(11.5, 13.5, 11.4, 13.0, 1.0, 2).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ThreeSoldiersOrCrows {
    prev: Option<Candle>,
    prev_prev: Option<Candle>,
    has_emitted: bool,
}

impl ThreeSoldiersOrCrows {
    /// Construct a new Three White Soldiers / Black Crows detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            prev_prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for ThreeSoldiersOrCrows {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let pp = self.prev_prev;
        let p = self.prev;
        self.prev_prev = self.prev;
        self.prev = Some(candle);
        let (Some(b1), Some(b2)) = (pp, p) else {
            return Some(0.0);
        };
        let g1 = b1.close > b1.open;
        let g2 = b2.close > b2.open;
        let g3 = candle.close > candle.open;
        let r1 = b1.close < b1.open;
        let r2 = b2.close < b2.open;
        let r3 = candle.close < candle.open;
        // Three White Soldiers
        if g1
            && g2
            && g3
            && b2.close > b1.close
            && candle.close > b2.close
            && b2.open >= b1.open
            && b2.open <= b1.close
            && candle.open >= b2.open
            && candle.open <= b2.close
        {
            return Some(1.0);
        }
        // Three Black Crows
        if r1
            && r2
            && r3
            && b2.close < b1.close
            && candle.close < b2.close
            && b2.open <= b1.open
            && b2.open >= b1.close
            && candle.open <= b2.open
            && candle.open >= b2.close
        {
            return Some(-1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.prev = None;
        self.prev_prev = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        3
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "ThreeSoldiersOrCrows"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(open: f64, high: f64, low: f64, close: f64, ts: i64) -> Candle {
        Candle::new(open, high, low, close, 1.0, ts).unwrap()
    }

    #[test]
    fn accessors_and_metadata() {
        let t = ThreeSoldiersOrCrows::new();
        assert_eq!(t.name(), "ThreeSoldiersOrCrows");
        assert_eq!(t.warmup_period(), 3);
        assert!(!t.is_ready());
    }

    #[test]
    fn three_white_soldiers_is_plus_one() {
        let mut t = ThreeSoldiersOrCrows::new();
        assert_eq!(t.update(c(10.0, 11.5, 9.9, 11.0, 0)), Some(0.0));
        assert_eq!(t.update(c(10.5, 12.5, 10.4, 12.0, 1)), Some(0.0));
        assert_eq!(t.update(c(11.5, 13.5, 11.4, 13.0, 2)), Some(1.0));
    }

    #[test]
    fn three_black_crows_is_minus_one() {
        let mut t = ThreeSoldiersOrCrows::new();
        // Bar1 13->12 (red), Bar2 opens inside [12,13] at 12.5, closes 11.
        // Bar3 opens inside [11,12.5] at 11.5, closes 10.
        assert_eq!(t.update(c(13.0, 13.1, 11.9, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(12.5, 12.6, 10.9, 11.0, 1)), Some(0.0));
        assert_eq!(t.update(c(11.5, 11.6, 9.9, 10.0, 2)), Some(-1.0));
    }

    #[test]
    fn mixed_directions_yield_zero() {
        let mut t = ThreeSoldiersOrCrows::new();
        t.update(c(10.0, 11.5, 9.9, 11.0, 0));
        t.update(c(11.0, 11.2, 10.0, 10.5, 1));
        assert_eq!(t.update(c(10.5, 11.5, 10.4, 11.4, 2)), Some(0.0));
    }

    #[test]
    fn first_two_bars_return_zero() {
        let mut t = ThreeSoldiersOrCrows::new();
        assert_eq!(t.update(c(10.0, 11.5, 9.9, 11.0, 0)), Some(0.0));
        assert_eq!(t.update(c(10.5, 12.5, 10.4, 12.0, 1)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 1.5, base - 0.1, base + 1.0, i)
            })
            .collect();
        let mut a = ThreeSoldiersOrCrows::new();
        let mut b = ThreeSoldiersOrCrows::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = ThreeSoldiersOrCrows::new();
        t.update(c(10.0, 11.5, 9.9, 11.0, 0));
        t.update(c(10.5, 12.5, 10.4, 12.0, 1));
        t.update(c(11.5, 13.5, 11.4, 13.0, 2));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(10.0, 11.5, 9.9, 11.0, 0)), Some(0.0));
    }
}
