//! Pivot Reversal — a breakout signal off the most recent confirmed swing pivots.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Pivot Reversal — emits a reversal **breakout signal** when price closes through
/// the most recently confirmed swing pivot.
///
/// ```text
/// pivot high: a bar whose high is strictly above the `left` bars before and the
///             `right` bars after it (confirmed `right` bars late)
/// pivot low : the mirror on lows
/// signal = +1 when close crosses above the last confirmed pivot high
/// signal = −1 when close crosses below the last confirmed pivot low
/// signal =  0 otherwise
/// ```
///
/// Unlike [`WilliamsFractals`](crate::WilliamsFractals), which merely *marks* the
/// swing points, Pivot Reversal turns them into an actionable entry: once a swing
/// high is confirmed it becomes a breakout trigger — a close back above it signals
/// a bullish reversal — and likewise a close below a confirmed swing low signals a
/// bearish reversal. This is the logic of the classic "Pivot Reversal" strategy.
/// Signals fire only on the **crossing** bar, not while price sits beyond the
/// level.
///
/// The first signal can appear once `left + right + 1` bars exist (a pivot needs
/// neighbours on both sides). The output is `+1` / `0` / `−1`. Each `update` is
/// O(`left + right`).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, PivotReversal};
///
/// let mut indicator = PivotReversal::new(2, 2).unwrap();
/// let mut fired = false;
/// for i in 0..60 {
///     let base = 100.0 + (f64::from(i) * 0.4).sin() * 5.0;
///     let c = Candle::new(base, base + 1.0, base - 1.0, base, 1_000.0, 0).unwrap();
///     match indicator.update(c) {
///         Some(s) if s != 0.0 => fired = true,
///         _ => {}
///     }
/// }
/// let _ = fired;
/// ```
#[derive(Debug, Clone)]
pub struct PivotReversal {
    left: usize,
    right: usize,
    window: VecDeque<Candle>,
    pivot_high: Option<f64>,
    pivot_low: Option<f64>,
    prev_close: Option<f64>,
    last: Option<f64>,
}

impl PivotReversal {
    /// Construct a Pivot Reversal with `left` bars before and `right` bars after
    /// the pivot.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PeriodZero`] if `left` or `right` is `0`.
    pub fn new(left: usize, right: usize) -> Result<Self> {
        if left == 0 || right == 0 {
            return Err(Error::PeriodZero);
        }
        Ok(Self {
            left,
            right,
            window: VecDeque::with_capacity(left + right + 1),
            pivot_high: None,
            pivot_low: None,
            prev_close: None,
            last: None,
        })
    }

    /// Configured `(left, right)` strengths.
    pub const fn params(&self) -> (usize, usize) {
        (self.left, self.right)
    }

    /// Most recent confirmed pivot-high level, if any.
    pub const fn pivot_high(&self) -> Option<f64> {
        self.pivot_high
    }

    /// Most recent confirmed pivot-low level, if any.
    pub const fn pivot_low(&self) -> Option<f64> {
        self.pivot_low
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for PivotReversal {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let close = candle.close;
        if self.window.len() == self.left + self.right + 1 {
            self.window.pop_front();
        }
        self.window.push_back(candle);
        if self.window.len() < self.left + self.right + 1 {
            self.prev_close = Some(close);
            return None;
        }

        // Confirm the pivot candidate sitting `right` bars back.
        let cand = self.window[self.left];
        let is_high = self
            .window
            .iter()
            .enumerate()
            .all(|(i, c)| i == self.left || c.high < cand.high);
        let is_low = self
            .window
            .iter()
            .enumerate()
            .all(|(i, c)| i == self.left || c.low > cand.low);
        if is_high {
            self.pivot_high = Some(cand.high);
        }
        if is_low {
            self.pivot_low = Some(cand.low);
        }

        // Breakout crossing of the latest confirmed pivots by the current close.
        let mut signal = 0.0;
        if let (Some(ph), Some(prev)) = (self.pivot_high, self.prev_close) {
            if close > ph && prev <= ph {
                signal = 1.0;
            }
        }
        if let (Some(pl), Some(prev)) = (self.pivot_low, self.prev_close) {
            if close < pl && prev >= pl {
                signal = -1.0;
            }
        }
        self.prev_close = Some(close);
        self.last = Some(signal);
        Some(signal)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.pivot_high = None;
        self.pivot_low = None;
        self.prev_close = None;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        self.left + self.right + 1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "PivotReversal"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(high: f64, low: f64, close: f64) -> Candle {
        Candle::new_unchecked(close, high, low, close, 1_000.0, 0)
    }

    #[test]
    fn rejects_zero_params() {
        assert!(matches!(PivotReversal::new(0, 2), Err(Error::PeriodZero)));
        assert!(matches!(PivotReversal::new(2, 0), Err(Error::PeriodZero)));
    }

    #[test]
    fn accessors_and_metadata() {
        let p = PivotReversal::new(2, 2).unwrap();
        assert_eq!(p.params(), (2, 2));
        assert_eq!(p.warmup_period(), 5);
        assert_eq!(p.name(), "PivotReversal");
        assert!(!p.is_ready());
        assert_eq!(p.value(), None);
        assert_eq!(p.pivot_high(), None);
        assert_eq!(p.pivot_low(), None);
    }

    #[test]
    fn first_emission_at_warmup_period() {
        let mut p = PivotReversal::new(1, 1).unwrap();
        let out = p.batch(&[c(10.0, 9.0, 9.5), c(12.0, 11.0, 11.5), c(10.0, 9.0, 9.5)]);
        assert!(out[0].is_none());
        assert!(out[1].is_none());
        assert!(out[2].is_some());
    }

    #[test]
    fn confirms_pivot_high() {
        // bar1 is a local high; once bar2 arrives it is confirmed.
        let mut p = PivotReversal::new(1, 1).unwrap();
        p.batch(&[c(10.0, 9.0, 9.5), c(12.0, 11.0, 11.5), c(10.0, 9.0, 9.5)]);
        assert_eq!(p.pivot_high(), Some(12.0));
    }

    #[test]
    fn confirms_pivot_low() {
        let mut p = PivotReversal::new(1, 1).unwrap();
        p.batch(&[c(12.0, 11.0, 11.5), c(10.0, 8.0, 8.5), c(12.0, 11.0, 11.5)]);
        assert_eq!(p.pivot_low(), Some(8.0));
    }

    #[test]
    fn breakout_above_pivot_high_signals_plus_one() {
        let mut p = PivotReversal::new(1, 1).unwrap();
        // Form a pivot high at 12, then a close above 12 crosses it.
        let candles = [
            c(10.0, 9.0, 9.5),   // index 0
            c(12.0, 11.0, 11.5), // pivot-high candidate
            c(10.0, 9.0, 9.5),   // confirms pivot high = 12
            c(11.0, 9.0, 9.0),   // close 9.0 (below 12)
            c(14.0, 12.5, 13.0), // close 13.0 > 12 and prev 9.0 <= 12 -> +1
        ];
        let out = p.batch(&candles);
        assert_eq!(out.last().unwrap(), &Some(1.0));
    }

    #[test]
    fn breakdown_below_pivot_low_signals_minus_one() {
        let mut p = PivotReversal::new(1, 1).unwrap();
        let candles = [
            c(12.0, 11.0, 11.5),
            c(10.0, 8.0, 8.5),   // pivot-low candidate
            c(12.0, 11.0, 11.5), // confirms pivot low = 8
            c(12.0, 9.0, 11.0),  // close 11 (above 8)
            c(9.0, 6.0, 7.0),    // close 7 < 8 and prev 11 >= 8 -> -1
        ];
        let out = p.batch(&candles);
        assert_eq!(out.last().unwrap(), &Some(-1.0));
    }

    #[test]
    fn no_break_is_zero() {
        let mut p = PivotReversal::new(1, 1).unwrap();
        let candles = [
            c(10.0, 9.0, 9.5),
            c(12.0, 11.0, 11.5),
            c(10.0, 9.0, 9.5),
            c(10.5, 9.0, 9.8),
        ];
        let out = p.batch(&candles);
        assert_eq!(out.last().unwrap(), &Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut p = PivotReversal::new(1, 1).unwrap();
        p.batch(&[c(10.0, 9.0, 9.5), c(12.0, 11.0, 11.5), c(10.0, 9.0, 9.5)]);
        assert!(p.is_ready());
        p.reset();
        assert!(!p.is_ready());
        assert_eq!(p.value(), None);
        assert_eq!(p.pivot_high(), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.4).sin() * 6.0;
                c(base + 1.0, base - 1.0, base)
            })
            .collect();
        let batch = PivotReversal::new(2, 2).unwrap().batch(&candles);
        let mut b = PivotReversal::new(2, 2).unwrap();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
