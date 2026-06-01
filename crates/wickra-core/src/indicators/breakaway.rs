//! Breakaway candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Breakaway — a 5-bar reversal that fades an exhausted run. A trend gaps away on
/// the second bar, drifts two more bars in the same direction, then the fifth bar
/// snaps the other way and closes back inside the body gap left between the first
/// and second bars, signalling the move has broken away from the crowd and is
/// turning.
///
/// ```text
/// bullish (+1.0)  — appears in a decline:
///   bar1 black (close < open)
///   bar2 black & its body gaps DOWN below bar1's body  (bar2.open < bar1.close)
///   bar3 extends lower            (high & low below bar2)
///   bar4 black & extends lower    (high & low below bar3)
///   bar5 green & closes inside the bar1/bar2 body gap   (bar2.open < close < bar1.close)
///
/// bearish (−1.0) — the mirror in an advance:
///   bar1 white (close > open)
///   bar2 white & its body gaps UP above bar1's body    (bar2.open > bar1.close)
///   bar3 extends higher           (high & low above bar2)
///   bar4 white & extends higher   (high & low above bar3)
///   bar5 red & closes inside the bar1/bar2 body gap     (bar1.close < close < bar2.open)
/// ```
///
/// The middle bar (`bar3`) may be either colour — only its high/low must extend
/// the run. Output is `+1.0` bullish, `−1.0` bearish, `0.0` otherwise. The first
/// four bars always return `0.0` because the five-bar window is not yet filled.
/// Pattern-shape check only — no trend filter is applied; combine with a trend
/// indicator for actionable signals. Recognition uses TA-Lib's
/// `CDLBREAKAWAY` body-gap and high/low ordering rules directly; it does not add
/// TA-Lib's rolling body-length average, matching the geometric house style of
/// the other multi-bar patterns in this family.
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
/// use wickra_core::{Breakaway, Candle, Indicator};
///
/// let mut indicator = Breakaway::new();
/// indicator.update(Candle::new(20.0, 20.2, 14.8, 15.0, 1.0, 0).unwrap());
/// indicator.update(Candle::new(14.0, 14.1, 11.9, 12.0, 1.0, 1).unwrap());
/// indicator.update(Candle::new(12.5, 13.0, 10.5, 11.0, 1.0, 2).unwrap());
/// indicator.update(Candle::new(11.0, 11.5, 9.0, 9.5, 1.0, 3).unwrap());
/// let out = indicator
///     .update(Candle::new(9.5, 14.7, 9.4, 14.5, 1.0, 4).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Breakaway {
    c1: Option<Candle>,
    c2: Option<Candle>,
    c3: Option<Candle>,
    c4: Option<Candle>,
    has_emitted: bool,
}

impl Breakaway {
    /// Construct a new Breakaway detector.
    pub const fn new() -> Self {
        Self {
            c1: None,
            c2: None,
            c3: None,
            c4: None,
            has_emitted: false,
        }
    }
}

impl Indicator for Breakaway {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let bar1 = self.c1;
        let bar2 = self.c2;
        let bar3 = self.c3;
        let bar4 = self.c4;
        self.c1 = self.c2;
        self.c2 = self.c3;
        self.c3 = self.c4;
        self.c4 = Some(candle);
        let (Some(bar1), Some(bar2), Some(bar3), Some(bar4)) = (bar1, bar2, bar3, bar4) else {
            return Some(0.0);
        };
        // Bullish: a decline gaps lower, runs two more bars down, then a green
        // bar5 closes back inside the bar1/bar2 body gap.
        if bar1.close < bar1.open
            && bar2.close < bar2.open
            && bar2.open < bar1.close
            && bar3.high < bar2.high
            && bar3.low < bar2.low
            && bar4.close < bar4.open
            && bar4.high < bar3.high
            && bar4.low < bar3.low
            && candle.close > candle.open
            && candle.close > bar2.open
            && candle.close < bar1.close
        {
            return Some(1.0);
        }
        // Bearish: the mirror — an advance gaps higher, runs two more bars up,
        // then a red bar5 closes back inside the bar1/bar2 body gap.
        if bar1.close > bar1.open
            && bar2.close > bar2.open
            && bar2.open > bar1.close
            && bar3.high > bar2.high
            && bar3.low > bar2.low
            && bar4.close > bar4.open
            && bar4.high > bar3.high
            && bar4.low > bar3.low
            && candle.close < candle.open
            && candle.close < bar2.open
            && candle.close > bar1.close
        {
            return Some(-1.0);
        }
        Some(0.0)
    }

    fn reset(&mut self) {
        self.c1 = None;
        self.c2 = None;
        self.c3 = None;
        self.c4 = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        5
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "Breakaway"
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
        let t = Breakaway::new();
        assert_eq!(t.name(), "Breakaway");
        assert_eq!(t.warmup_period(), 5);
        assert!(!t.is_ready());
    }

    #[test]
    fn bullish_breakaway_is_plus_one() {
        let mut t = Breakaway::new();
        assert_eq!(t.update(c(20.0, 20.2, 14.8, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(14.0, 14.1, 11.9, 12.0, 1)), Some(0.0));
        assert_eq!(t.update(c(12.5, 13.0, 10.5, 11.0, 2)), Some(0.0));
        assert_eq!(t.update(c(11.0, 11.5, 9.0, 9.5, 3)), Some(0.0));
        assert_eq!(t.update(c(9.5, 14.7, 9.4, 14.5, 4)), Some(1.0));
    }

    #[test]
    fn bearish_breakaway_is_minus_one() {
        let mut t = Breakaway::new();
        assert_eq!(t.update(c(15.0, 20.2, 14.8, 20.0, 0)), Some(0.0));
        assert_eq!(t.update(c(21.0, 23.1, 20.9, 23.0, 1)), Some(0.0));
        assert_eq!(t.update(c(22.5, 24.5, 21.5, 24.0, 2)), Some(0.0));
        assert_eq!(t.update(c(24.0, 26.5, 23.0, 26.0, 3)), Some(0.0));
        assert_eq!(t.update(c(27.0, 27.2, 20.4, 20.5, 4)), Some(-1.0));
    }

    #[test]
    fn no_body_gap_yields_zero() {
        let mut t = Breakaway::new();
        // bar2 does not gap below bar1's body (bar2.open >= bar1.close).
        t.update(c(20.0, 20.2, 14.8, 15.0, 0));
        t.update(c(16.0, 16.1, 13.9, 14.0, 1));
        t.update(c(13.5, 14.0, 11.5, 12.0, 2));
        t.update(c(12.0, 12.5, 10.0, 10.5, 3));
        assert_eq!(t.update(c(10.5, 15.7, 10.4, 15.5, 4)), Some(0.0));
    }

    #[test]
    fn bullish_close_outside_gap_yields_zero() {
        let mut t = Breakaway::new();
        t.update(c(20.0, 20.2, 14.8, 15.0, 0));
        t.update(c(14.0, 14.1, 11.9, 12.0, 1));
        t.update(c(12.5, 13.0, 10.5, 11.0, 2));
        t.update(c(11.0, 11.5, 9.0, 9.5, 3));
        // bar5 closes at 13.0 — below bar2.open (14), so outside the body gap.
        assert_eq!(t.update(c(9.5, 13.2, 9.4, 13.0, 4)), Some(0.0));
    }

    #[test]
    fn first_four_bars_return_zero() {
        let mut t = Breakaway::new();
        assert_eq!(t.update(c(20.0, 20.2, 14.8, 15.0, 0)), Some(0.0));
        assert_eq!(t.update(c(14.0, 14.1, 11.9, 12.0, 1)), Some(0.0));
        assert_eq!(t.update(c(12.5, 13.0, 10.5, 11.0, 2)), Some(0.0));
        assert_eq!(t.update(c(11.0, 11.5, 9.0, 9.5, 3)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                c(base, base + 2.0, base - 0.5, base + 1.5, i)
            })
            .collect();
        let mut a = Breakaway::new();
        let mut b = Breakaway::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut t = Breakaway::new();
        t.update(c(20.0, 20.2, 14.8, 15.0, 0));
        t.update(c(14.0, 14.1, 11.9, 12.0, 1));
        t.update(c(12.5, 13.0, 10.5, 11.0, 2));
        t.update(c(11.0, 11.5, 9.0, 9.5, 3));
        t.update(c(9.5, 14.7, 9.4, 14.5, 4));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(c(20.0, 20.2, 14.8, 15.0, 0)), Some(0.0));
    }
}
