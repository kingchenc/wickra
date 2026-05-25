//! Williams Fractals (Bill Williams).

use std::collections::VecDeque;

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Williams Fractals output for one bar.
///
/// Each field is `Some(price)` when a fractal high/low was confirmed at the
/// **centre** of the most recent five-bar window, and `None` otherwise. Up and
/// down fractals are independent and can coincide (a centre bar can be both
/// the maximum high and the minimum low of the window).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WilliamsFractalsOutput {
    /// Up fractal: the centre bar's high, if it is strictly greater than the
    /// two highs to its left and the two highs to its right.
    pub up: Option<f64>,
    /// Down fractal: the centre bar's low, if it is strictly less than the
    /// two lows to its left and the two lows to its right.
    pub down: Option<f64>,
}

/// Williams Fractals — Bill Williams' five-bar swing detector. A bar is an
/// **up fractal** if its high is strictly above the highs of the two bars
/// immediately before and the two bars immediately after. A bar is a
/// **down fractal** if its low is strictly below the lows of those same four
/// neighbours. Because confirmation requires two bars to the right of the
/// candidate, the indicator inherently lags by two bars.
///
/// The first output lands at the fifth candle and corresponds to the third
/// candle (the centre of the window). Subsequent outputs slide the window by
/// one bar.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, WilliamsFractals};
///
/// let mut wf = WilliamsFractals::new();
/// // Build a V-shape with a clear high at index 2.
/// let highs = [1.0, 2.0, 5.0, 2.0, 1.0];
/// for (i, &h) in highs.iter().enumerate() {
///     let c = Candle::new(h, h, h - 0.5, h, 1.0, i as i64).unwrap();
///     let _ = wf.update(c);
/// }
/// // At candle 5 the third bar's high of 5.0 is confirmed as an up fractal.
/// ```
#[derive(Debug, Clone)]
pub struct WilliamsFractals {
    // Five-bar window of (high, low) pairs. The centre is at index 2.
    window: VecDeque<(f64, f64)>,
}

impl Default for WilliamsFractals {
    fn default() -> Self {
        Self::new()
    }
}

impl WilliamsFractals {
    /// Construct a new Williams Fractals indicator. The window size is fixed
    /// at five bars (two left, centre, two right).
    pub fn new() -> Self {
        Self {
            window: VecDeque::with_capacity(5),
        }
    }
}

impl Indicator for WilliamsFractals {
    type Input = Candle;
    type Output = WilliamsFractalsOutput;

    fn update(&mut self, candle: Candle) -> Option<WilliamsFractalsOutput> {
        if self.window.len() == 5 {
            self.window.pop_front();
        }
        self.window.push_back((candle.high, candle.low));
        if self.window.len() < 5 {
            return None;
        }
        let (h0, _) = self.window[0];
        let (h1, _) = self.window[1];
        let (h2, l2) = self.window[2];
        let (h3, _) = self.window[3];
        let (h4, _) = self.window[4];
        let (_, l0) = self.window[0];
        let (_, l1) = self.window[1];
        let (_, l3) = self.window[3];
        let (_, l4) = self.window[4];

        let up = if h2 > h0 && h2 > h1 && h2 > h3 && h2 > h4 {
            Some(h2)
        } else {
            None
        };
        let down = if l2 < l0 && l2 < l1 && l2 < l3 && l2 < l4 {
            Some(l2)
        } else {
            None
        };
        Some(WilliamsFractalsOutput { up, down })
    }

    fn reset(&mut self) {
        self.window.clear();
    }

    fn warmup_period(&self) -> usize {
        5
    }

    fn is_ready(&self) -> bool {
        self.window.len() == 5
    }

    fn name(&self) -> &'static str {
        "WilliamsFractals"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(h: f64, l: f64, ts: i64) -> Candle {
        Candle::new(l, h, l, l, 1.0, ts).unwrap()
    }

    #[test]
    fn isolated_peak_is_detected_as_up_fractal() {
        let mut wf = WilliamsFractals::new();
        // Highs 1, 2, 5, 2, 1 -> centre (5) is strictly above its four neighbours.
        let highs = [1.0, 2.0, 5.0, 2.0, 1.0];
        let mut last = None;
        for (i, &h) in highs.iter().enumerate() {
            last = wf.update(c(h, h - 0.5, i64::try_from(i).unwrap()));
        }
        let o = last.expect("fifth bar emits");
        assert_eq!(o.up, Some(5.0));
        assert_eq!(o.down, None);
    }

    #[test]
    fn isolated_trough_is_detected_as_down_fractal() {
        let mut wf = WilliamsFractals::new();
        // Lows 5, 4, 1, 4, 5 -> centre is the trough.
        let lows = [5.0, 4.0, 1.0, 4.0, 5.0];
        let mut last = None;
        for (i, &l) in lows.iter().enumerate() {
            last = wf.update(c(l + 0.5, l, i64::try_from(i).unwrap()));
        }
        let o = last.expect("fifth bar emits");
        assert_eq!(o.down, Some(1.0));
        assert_eq!(o.up, None);
    }

    #[test]
    fn monotonic_series_yields_no_fractals() {
        let mut wf = WilliamsFractals::new();
        let mut emitted = 0_usize;
        for i in 0..10 {
            let h = f64::from(i) + 2.0;
            let l = f64::from(i);
            if let Some(o) = wf.update(c(h, l, i64::from(i))) {
                emitted += 1;
                assert_eq!(o.up, None);
                assert_eq!(o.down, None);
            }
        }
        assert!(emitted >= 6);
    }

    #[test]
    fn equal_neighbour_is_not_a_fractal() {
        // Centre tied with neighbour -> strict inequality fails -> no fractal.
        let mut wf = WilliamsFractals::new();
        let highs = [1.0, 5.0, 5.0, 2.0, 1.0];
        let mut last = None;
        for (i, &h) in highs.iter().enumerate() {
            last = wf.update(c(h, h - 0.5, i64::try_from(i).unwrap()));
        }
        let o = last.unwrap();
        assert_eq!(o.up, None);
    }

    #[test]
    fn first_four_bars_return_none() {
        let mut wf = WilliamsFractals::new();
        for i in 0..4 {
            assert_eq!(wf.update(c(10.0, 9.0, i)), None);
        }
        assert!(!wf.is_ready());
    }

    #[test]
    fn warmup_period_is_five() {
        assert_eq!(WilliamsFractals::new().warmup_period(), 5);
    }

    #[test]
    fn reset_clears_state() {
        let mut wf = WilliamsFractals::new();
        for i in 0..5 {
            wf.update(c(10.0, 9.0, i));
        }
        assert!(wf.is_ready());
        wf.reset();
        assert!(!wf.is_ready());
        assert_eq!(wf.update(c(10.0, 9.0, 0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| c(f64::from(i) + 2.0, f64::from(i), i64::from(i)))
            .collect();
        let mut a = WilliamsFractals::new();
        let mut b = WilliamsFractals::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn accessors_and_metadata() {
        let wf = WilliamsFractals::new();
        assert_eq!(wf.warmup_period(), 5);
        assert_eq!(wf.name(), "WilliamsFractals");
    }

    #[test]
    fn default_matches_new() {
        let a = WilliamsFractals::new();
        let b = WilliamsFractals::default();
        assert_eq!(a.is_ready(), b.is_ready());
        assert_eq!(a.warmup_period(), b.warmup_period());
    }
}
