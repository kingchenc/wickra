//! Three Line Strike candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Three Line Strike — a 4-bar pattern: three candles marching in one direction
/// (a three-soldiers / three-crows advance) followed by a fourth candle of the
/// opposite colour that opens beyond the third candle and closes back past the
/// first candle's open, "striking" through the whole run.
///
/// **Bullish** (`+1.0`):
/// ```text
/// bar1..bar3 green, each opening inside the prior body and closing higher
/// bar4 red & opens above bar3's close & closes below bar1's open
/// ```
///
/// **Bearish** (`−1.0`): the mirror — three falling red candles struck by a
/// green bar4 that opens below bar3's close and closes above bar1's open.
///
/// Output is `0.0` otherwise. The first three bars always return `0.0` because
/// the four-bar window is not yet filled. Pattern-shape check only — no trend
/// filter is applied; combine with a trend indicator for actionable signals.
///
/// # Signed ±1 encoding
///
/// This detector emits the uniform candlestick sign convention shared across the
/// pattern family — `+1.0` bullish, `−1.0` bearish, `0.0` no pattern — so it
/// drops straight into a machine-learning feature matrix where the bullish and
/// bearish variants occupy a single dimension.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, ThreeLineStrike};
///
/// let mut indicator = ThreeLineStrike::new();
/// indicator.update(Candle::new(10.0, 11.1, 9.9, 11.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(10.5, 12.1, 10.4, 12.0, 1.0, 1).unwrap());
/// indicator.update(Candle::new(11.5, 13.1, 11.4, 13.0, 1.0, 2).unwrap());
/// let out = indicator
///     .update(Candle::new(13.5, 13.6, 9.4, 9.5, 1.0, 3).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ThreeLineStrike {
    c1: Option<Candle>,
    c2: Option<Candle>,
    c3: Option<Candle>,
    has_emitted: bool,
}

impl ThreeLineStrike {
    /// Construct a new Three Line Strike detector.
    pub const fn new() -> Self {
        Self {
            c1: None,
            c2: None,
            c3: None,
            has_emitted: false,
        }
    }
}

impl Indicator for ThreeLineStrike {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let bar1 = self.c1;
        let bar2 = self.c2;
        let bar3 = self.c3;
        self.c1 = self.c2;
        self.c2 = self.c3;
        self.c3 = Some(candle);
        let (Some(bar1), Some(bar2), Some(bar3)) = (bar1, bar2, bar3) else {
            return Some(0.0);
        };
        // Bullish: three rising green candles struck by a red bar4.
        if bar1.close > bar1.open
            && bar2.close > bar2.open
            && bar3.close > bar3.open
            && bar2.open >= bar1.open
            && bar2.open <= bar1.close
            && bar2.close > bar1.close
            && bar3.open >= bar2.open
            && bar3.open <= bar2.close
            && bar3.close > bar2.close
            && candle.close < candle.open
            && candle.open > bar3.close
            && candle.close < bar1.open
        {
            return Some(1.0);
        }
        // Bearish: three falling red candles struck by a green bar4.
        if bar1.close < bar1.open
            && bar2.close < bar2.open
            && bar3.close < bar3.open
            && bar2.open <= bar1.open
            && bar2.open >= bar1.close
            && bar2.close < bar1.close
            && bar3.open <= bar2.open
            && bar3.open >= bar2.close
            && bar3.close < bar2.close
            && candle.close > candle.open
            && candle.open < bar3.close
            && candle.close > bar1.open
        {
            return Some(-1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.c1 = None;
        self.c2 = None;
        self.c3 = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        4
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "ThreeLineStrike"
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
        let t = ThreeLineStrike::new();
        assert_eq!(t.name(), "ThreeLineStrike");
        assert_eq!(t.warmup_period(), 4);
        assert!(!t.is_ready());
    }

    #[test]
    fn bullish_three_line_strike_is_plus_one() {
        let mut t = ThreeLineStrike::new();
        assert_eq!(t.update(c(10.0, 11.1, 9.9, 11.0, 0)), Some(0.0));
        assert_eq!(t.update(c(10.5, 12.1, 10.4, 12.0, 1)), Some(0.0));
        assert_eq!(t.update(c(11.5, 13.1, 11.4, 13.0, 2)), Some(0.0));
        assert_eq!(t.update(c(13.5, 13.6, 9.4, 9.5, 3)), Some(1.0));
    }

    #[test]
    fn bearish_three_line_strike_is_minus_one() {
        let mut t = ThreeLineStrike::new();
        assert_eq!(t.update(c(13.0, 13.1, 11.9, 12.0, 0)), Some(0.0));
        assert_eq!(t.update(c(12.5, 12.6, 10.9, 11.0, 1)), Some(0.0));
        assert_eq!(t.update(c(11.5, 11.6, 9.9, 10.0, 2)), Some(0.0));
        assert_eq!(t.update(c(9.5, 13.6, 9.4, 13.5, 3)), Some(-1.0));
    }

    #[test]
    fn strike_not_clearing_first_open_yields_zero() {
        let mut t = ThreeLineStrike::new();
        t.update(c(10.0, 11.1, 9.9, 11.0, 0));
        t.update(c(10.5, 12.1, 10.4, 12.0, 1));
        t.update(c(11.5, 13.1, 11.4, 13.0, 2));
        // bar4 closes 10.5, above bar1's open (10.0) -> does not strike through.
        assert_eq!(t.update(c(13.5, 13.6, 10.4, 10.5, 3)), Some(0.0));
    }

    #[test]
    fn first_three_bars_return_zero() {
        let mut t = ThreeLineStrike::new();
        assert_eq!(t.update(c(10.0, 11.1, 9.9, 11.0, 0)), Some(0.0));
        assert_eq!(t.update(c(10.5, 12.1, 10.4, 12.0, 1)), Some(0.0));
        assert_eq!(t.update(c(11.5, 13.1, 11.4, 13.0, 2)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 1.5, base - 0.2, base + 1.0, i)
            })
            .collect();
        let mut a = ThreeLineStrike::new();
        let mut b = ThreeLineStrike::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = ThreeLineStrike::new();
        t.update(c(10.0, 11.1, 9.9, 11.0, 0));
        t.update(c(10.5, 12.1, 10.4, 12.0, 1));
        t.update(c(11.5, 13.1, 11.4, 13.0, 2));
        t.update(c(13.5, 13.6, 9.4, 9.5, 3));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(10.0, 11.1, 9.9, 11.0, 0)), Some(0.0));
    }
}
