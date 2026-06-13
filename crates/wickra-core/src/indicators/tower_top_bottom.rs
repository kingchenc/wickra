#![allow(clippy::doc_markdown)]

//! Tower Top / Tower Bottom — a tall bar, a pause, then a tall opposite bar.
//!
//! A Tower is a reversal where a strong directional bar is followed by a small
//! "pause" bar and then a strong bar in the *opposite* direction, like two towers
//! flanking a low wall. This is the compact three-bar form of the classic
//! multi-bar Tower pattern.
//!
//! - **Tower Bottom** (`+1.0`): a tall **bearish** bar, a small-bodied bar, then a
//!   tall **bullish** bar.
//! - **Tower Top** (`-1.0`): a tall **bullish** bar, a small-bodied bar, then a
//!   tall **bearish** bar.
//! - Otherwise the output is `0.0`.
//!
//! "Tall" = body `>= 0.5 * range`; "small" = body `<= 0.3 * range`. The three-bar
//! lookback means the first value lands on the third candle.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

fn body_fraction(candle: Candle) -> f64 {
    let range = candle.high - candle.low;
    if range > 0.0 {
        (candle.close - candle.open).abs() / range
    } else {
        0.0
    }
}

fn is_tall(candle: Candle) -> bool {
    body_fraction(candle) >= 0.5
}

fn is_small(candle: Candle) -> bool {
    body_fraction(candle) <= 0.3
}

/// Tower Top / Bottom — three-bar reversal detector.
/// # Example
///
/// ```
/// use wickra_core::{TowerTopBottom, Candle, Indicator};
///
/// let mut indicator = TowerTopBottom::new();
/// // `None` during warmup, then `Some(_)` once enough bars are seen.
/// let mut out = None;
/// for i in 0..40i64 {
///     let p = 100.0 + (i as f64 * 0.4).sin() * 5.0;
///     let candle = Candle::new(p, p + 1.5, p - 1.5, p + 0.3, 1_000.0, i).unwrap();
///     out = indicator.update(candle);
/// }
/// let _ = out;
/// ```
#[derive(Debug, Clone, Default)]
pub struct TowerTopBottom {
    c1: Option<Candle>,
    c2: Option<Candle>,
    last_value: Option<f64>,
}

impl TowerTopBottom {
    /// Construct a new `TowerTopBottom`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Latest emitted signal if available.
    pub const fn value(&self) -> Option<f64> {
        self.last_value
    }
}

impl Indicator for TowerTopBottom {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let (Some(first), Some(middle)) = (self.c1, self.c2) else {
            self.c1 = self.c2;
            self.c2 = Some(candle);
            self.last_value = Some(0.0);
            return Some(0.0);
        };
        let pause = is_small(middle);
        let first_tall = is_tall(first);
        let last_tall = is_tall(candle);
        let v = if pause && first_tall && last_tall {
            let first_up = first.close > first.open;
            let last_up = candle.close > candle.open;
            if !first_up && last_up {
                1.0
            } else if first_up && !last_up {
                -1.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        self.c1 = self.c2;
        self.c2 = Some(candle);
        self.last_value = Some(v);
        Some(v)
    }

    fn reset(&mut self) {
        self.c1 = None;
        self.c2 = None;
        self.last_value = None;
    }

    fn warmup_period(&self) -> usize {
        3
    }

    fn is_ready(&self) -> bool {
        self.last_value.is_some()
    }

    fn name(&self) -> &'static str {
        "TowerTopBottom"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    /// A tall candle from `open` to `close` (body fills most of the range).
    fn tall(open: f64, close: f64) -> Candle {
        Candle::new_unchecked(
            open,
            open.max(close) + 0.1,
            open.min(close) - 0.1,
            close,
            0.0,
            0,
        )
    }

    /// A small-bodied candle (long shadows, tiny body).
    fn small(mid: f64) -> Candle {
        Candle::new_unchecked(mid, mid + 2.0, mid - 2.0, mid + 0.1, 0.0, 0)
    }

    #[test]
    fn accessors_and_metadata() {
        let t = TowerTopBottom::new();
        assert_eq!(t.warmup_period(), 3);
        assert_eq!(t.name(), "TowerTopBottom");
        assert!(!t.is_ready());
        assert_eq!(t.value(), None);
    }

    #[test]
    fn first_two_bars_seed_without_signal() {
        let mut t = TowerTopBottom::new();
        assert_eq!(t.update(tall(100.0, 110.0)), Some(0.0));
        assert_eq!(t.update(small(105.0)), Some(0.0));
        assert!(t.update(tall(110.0, 100.0)).is_some());
    }

    #[test]
    fn tower_top() {
        // tall bullish, small pause, tall bearish -> top -> -1.
        let mut t = TowerTopBottom::new();
        t.update(tall(100.0, 110.0));
        t.update(small(110.0));
        assert_eq!(t.update(tall(110.0, 100.0)), Some(-1.0));
    }

    #[test]
    fn tower_bottom() {
        let mut t = TowerTopBottom::new();
        t.update(tall(110.0, 100.0));
        t.update(small(100.0));
        assert_eq!(t.update(tall(100.0, 110.0)), Some(1.0));
    }

    #[test]
    fn same_direction_is_zero() {
        let mut t = TowerTopBottom::new();
        t.update(tall(100.0, 110.0));
        t.update(small(110.0));
        // last bar also bullish -> not a tower -> 0.
        assert_eq!(t.update(tall(110.0, 120.0)), Some(0.0));
    }

    #[test]
    fn no_pause_is_zero() {
        let mut t = TowerTopBottom::new();
        t.update(tall(100.0, 110.0));
        t.update(tall(110.0, 120.0)); // middle is tall, not a pause
        assert_eq!(t.update(tall(120.0, 110.0)), Some(0.0));
    }

    #[test]
    fn reset_clears_state() {
        let mut t = TowerTopBottom::new();
        t.update(tall(100.0, 110.0));
        t.update(small(110.0));
        t.update(tall(110.0, 100.0));
        assert!(t.is_ready());
        t.reset();
        assert!(!t.is_ready());
        assert_eq!(t.update(tall(100.0, 110.0)), Some(0.0));
    }

    #[test]
    fn zero_range_bar_has_zero_body_fraction() {
        // A flat bar (high == low) exercises the zero-range body-fraction branch;
        // it counts as a small "pause" bar, so tall-flat-tall still reverses.
        fn flat(mid: f64) -> Candle {
            Candle::new_unchecked(mid, mid, mid, mid, 0.0, 0)
        }
        let mut t = TowerTopBottom::new();
        t.update(tall(100.0, 110.0));
        t.update(flat(110.0));
        assert_eq!(t.update(tall(110.0, 100.0)), Some(-1.0));
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..30)
            .map(|i| match i % 3 {
                0 => tall(100.0, 110.0),
                1 => small(110.0),
                _ => tall(110.0, 100.0),
            })
            .collect();
        let batch = TowerTopBottom::new().batch(&candles);
        let mut b = TowerTopBottom::new();
        let streamed: Vec<_> = candles.iter().map(|x| b.update(*x)).collect();
        assert_eq!(batch, streamed);
    }
}
