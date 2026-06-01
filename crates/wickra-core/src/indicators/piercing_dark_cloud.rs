//! Piercing Line / Dark Cloud Cover candlestick pattern.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Piercing Line / Dark Cloud Cover — a 2-bar reversal pattern.
///
/// **Piercing Line** (bullish, `+1.0`):
/// ```text
/// prev_red & curr_green
///   & curr.open <  prev.low
///   & curr.close > (prev.open + prev.close) / 2
///   & curr.close <  prev.open
/// ```
///
/// **Dark Cloud Cover** (bearish, `−1.0`):
/// ```text
/// prev_green & curr_red
///   & curr.open >  prev.high
///   & curr.close < (prev.open + prev.close) / 2
///   & curr.close >  prev.open
/// ```
///
/// Output is `+1.0` for a Piercing Line, `−1.0` for a Dark Cloud Cover, and
/// `0.0` otherwise. The first bar always returns `0.0`. Pattern-shape check
/// only — no trend filter is applied; combine with a trend indicator for
/// actionable signals.
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
/// use wickra_core::{Candle, Indicator, PiercingDarkCloud};
///
/// let mut indicator = PiercingDarkCloud::new();
/// indicator.update(Candle::new(12.0, 12.5, 10.0, 10.0, 1.0, 0).unwrap());
/// // Open below prev low, close above midpoint (11) but below prev open (12).
/// let out = indicator
///     .update(Candle::new(9.8, 11.8, 9.5, 11.5, 1.0, 1).unwrap());
/// assert_eq!(out, Some(1.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct PiercingDarkCloud {
    prev: Option<Candle>,
    has_emitted: bool,
}

impl PiercingDarkCloud {
    /// Construct a new Piercing Line / Dark Cloud Cover detector.
    pub const fn new() -> Self {
        Self {
            prev: None,
            has_emitted: false,
        }
    }
}

impl Indicator for PiercingDarkCloud {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let prev = self.prev;
        self.prev = Some(candle);
        let Some(p) = prev else {
            return Some(0.0);
        };
        let prev_red = p.close < p.open;
        let prev_green = p.close > p.open;
        let curr_green = candle.close > candle.open;
        let curr_red = candle.close < candle.open;
        let mid = f64::midpoint(p.open, p.close);
        if prev_red
            && curr_green
            && candle.open < p.low
            && candle.close > mid
            && candle.close < p.open
        {
            Some(1.0)
        } else if prev_green
            && curr_red
            && candle.open > p.high
            && candle.close < mid
            && candle.close > p.open
        {
            Some(-1.0)
        } else {
            Some(0.0)
        }
    }

    fn reset(&mut self) {
        self.prev = None;
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "PiercingDarkCloud"
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
        let p = PiercingDarkCloud::new();
        assert_eq!(p.name(), "PiercingDarkCloud");
        assert_eq!(p.warmup_period(), 2);
        assert!(!p.is_ready());
    }

    #[test]
    fn piercing_line_is_plus_one() {
        let mut p = PiercingDarkCloud::new();
        // Prev red: open 12, close 10. Curr green: opens at 9.8 (< prev low 10),
        // closes at 11.5 (> midpoint 11, < prev open 12).
        assert_eq!(p.update(c(12.0, 12.5, 10.0, 10.0, 0)), Some(0.0));
        assert_eq!(p.update(c(9.8, 11.8, 9.5, 11.5, 1)), Some(1.0));
    }

    #[test]
    fn dark_cloud_cover_is_minus_one() {
        let mut p = PiercingDarkCloud::new();
        // Prev green: open 10, close 12. Curr red: opens 12.3 (> prev high 12.2),
        // closes 10.5 (< midpoint 11, > prev open 10).
        assert_eq!(p.update(c(10.0, 12.2, 9.5, 12.0, 0)), Some(0.0));
        assert_eq!(p.update(c(12.3, 12.4, 10.4, 10.5, 1)), Some(-1.0));
    }

    #[test]
    fn close_below_midpoint_is_not_piercing() {
        let mut p = PiercingDarkCloud::new();
        p.update(c(12.0, 12.5, 10.0, 10.0, 0));
        // Closes only at 10.8 (below midpoint 11) -> not piercing.
        assert_eq!(p.update(c(9.8, 11.0, 9.5, 10.8, 1)), Some(0.0));
    }

    #[test]
    fn full_engulf_is_not_piercing() {
        let mut p = PiercingDarkCloud::new();
        p.update(c(12.0, 12.5, 10.0, 10.0, 0));
        // Closes above prev.open (12) -> engulfs, not piercing.
        assert_eq!(p.update(c(9.8, 13.0, 9.5, 12.5, 1)), Some(0.0));
    }

    #[test]
    fn first_bar_returns_zero() {
        let mut p = PiercingDarkCloud::new();
        assert_eq!(p.update(c(12.0, 12.5, 10.0, 10.0, 0)), Some(0.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                if i % 2 == 0 {
                    c(base + 2.0, base + 2.5, base, base, i)
                } else {
                    c(base - 0.2, base + 1.8, base - 0.5, base + 1.5, i)
                }
            })
            .collect();
        let mut a = PiercingDarkCloud::new();
        let mut b = PiercingDarkCloud::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut p = PiercingDarkCloud::new();
        p.update(c(12.0, 12.5, 10.0, 10.0, 0));
        p.update(c(9.8, 11.8, 9.5, 11.5, 1));
        assert!(p.is_ready());
        p.reset();
        assert!(!p.is_ready());
        assert_eq!(p.update(c(12.0, 12.5, 10.0, 10.0, 0)), Some(0.0));
    }
}
