//! Wick Ratio — the shadow imbalance of a bar.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Wick Ratio — the signed imbalance between the upper and lower shadows as a
/// fraction of the bar's range.
///
/// ```text
/// upper_wick  = high − max(open, close)
/// lower_wick  = min(open, close) − low
/// WickRatio   = (upper_wick − lower_wick) / (high − low)
/// ```
///
/// The result lives in `[−1, +1]`: `+1` is a bar that is all upper shadow (a
/// long rejection of higher prices, classic shooting-star geometry), `−1` all
/// lower shadow (a long rejection of lower prices, hammer geometry), and `0`
/// either a symmetric bar or a wickless one. Where
/// [`BodySizePct`](crate::BodySizePct) measures how much of the range is body,
/// this measures *which side* the wicks fall on — the rejection asymmetry many
/// reversal setups depend on. A zero-range bar yields `0`.
///
/// This is a stateless per-bar transform: every candle produces one value.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, WickRatio};
///
/// let mut indicator = WickRatio::new();
/// // upper 13 - 10.5 = 2.5, lower 10 - 10 = 0, range 3 -> +0.8333.
/// let c = Candle::new(10.0, 13.0, 10.0, 10.5, 10.0, 0).unwrap();
/// assert!((indicator.update(c).unwrap() - 2.5 / 3.0).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Default)]
pub struct WickRatio {
    has_emitted: bool,
}

impl WickRatio {
    /// Construct a new Wick Ratio transform.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for WickRatio {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let range = candle.high - candle.low;
        let out = if range == 0.0 {
            // A zero-range bar has no shadows to compare.
            0.0
        } else {
            let body_top = candle.open.max(candle.close);
            let body_bottom = candle.open.min(candle.close);
            let upper_wick = candle.high - body_top;
            let lower_wick = body_bottom - candle.low;
            (upper_wick - lower_wick) / range
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
        "WickRatio"
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
    fn upper_shadow_dominates_is_positive() {
        // upper 13 - 10.5 = 2.5, lower 10 - 10 = 0, range 3 -> +2.5/3.
        let mut wr = WickRatio::new();
        assert_relative_eq!(
            wr.update(candle(10.0, 13.0, 10.0, 10.5, 0)).unwrap(),
            2.5 / 3.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn lower_shadow_dominates_is_negative() {
        // Hammer: long lower shadow -> negative.
        // open 12, close 12.5, high 13, low 9: upper 0.5, lower 3, range 4.
        let mut wr = WickRatio::new();
        assert_relative_eq!(
            wr.update(candle(12.0, 13.0, 9.0, 12.5, 0)).unwrap(),
            (0.5 - 3.0) / 4.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn symmetric_wicks_are_zero() {
        // Equal upper and lower shadows -> 0.
        let mut wr = WickRatio::new();
        assert_relative_eq!(
            wr.update(candle(10.0, 12.0, 8.0, 10.0, 0)).unwrap(),
            0.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn zero_range_bar_yields_zero() {
        let mut wr = WickRatio::new();
        assert_relative_eq!(
            wr.update(candle(10.0, 10.0, 10.0, 10.0, 0)).unwrap(),
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
        let mut wr = WickRatio::new();
        for v in wr.batch(&candles).into_iter().flatten() {
            assert!((-1.0..=1.0).contains(&v), "WickRatio {v} outside [-1, 1]");
        }
    }

    #[test]
    fn name_metadata() {
        let wr = WickRatio::new();
        assert_eq!(wr.name(), "WickRatio");
    }

    #[test]
    fn emits_from_first_candle() {
        let mut wr = WickRatio::new();
        assert_eq!(wr.warmup_period(), 1);
        assert!(!wr.is_ready());
        assert!(wr.update(candle(10.0, 11.0, 9.0, 10.0, 0)).is_some());
        assert!(wr.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut wr = WickRatio::new();
        wr.update(candle(10.0, 11.0, 9.0, 10.0, 0));
        assert!(wr.is_ready());
        wr.reset();
        assert!(!wr.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + f64::from(i);
                candle(base, base + 2.0, base - 2.0, base + 1.0, i64::from(i))
            })
            .collect();
        let mut a = WickRatio::new();
        let mut b = WickRatio::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
