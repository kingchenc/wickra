//! Balance of Power.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Balance of Power — where the close settled within the bar's range relative
/// to the open.
///
/// ```text
/// BOP = (close − open) / (high − low)
/// ```
///
/// The result lives in `[−1, +1]`: `+1` is a bar that opened on its low and
/// closed on its high (buyers in full control), `−1` the mirror image. It is
/// a stateless per-bar reading — a quick gauge of intrabar conviction. A
/// zero-range bar carries no information and yields `0`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, BalanceOfPower};
///
/// let mut indicator = BalanceOfPower::new();
/// let mut last = None;
/// for i in 0..80 {
///     let base = 100.0 + f64::from(i);
///     let candle =
///         Candle::new(base, base + 2.0, base - 2.0, base + 1.0, 10.0, i64::from(i)).unwrap();
///     last = indicator.update(candle);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct BalanceOfPower {
    has_emitted: bool,
}

impl BalanceOfPower {
    /// Construct a new Balance of Power transform.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for BalanceOfPower {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let range = candle.high - candle.low;
        let bop = if range == 0.0 {
            // A zero-range bar carries no directional information.
            0.0
        } else {
            (candle.close - candle.open) / range
        };
        Some(bop)
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
        "BalanceOfPower"
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
        // (close - open) / (high - low) = (12 - 10) / (14 - 10) = 0.5.
        let mut bop = BalanceOfPower::new();
        assert_relative_eq!(
            bop.update(candle(10.0, 14.0, 10.0, 12.0, 0)).unwrap(),
            0.5,
            epsilon = 1e-12
        );
    }

    #[test]
    fn close_on_high_after_open_on_low_is_plus_one() {
        let mut bop = BalanceOfPower::new();
        // open == low, close == high -> BOP = +1.
        assert_relative_eq!(
            bop.update(candle(9.0, 11.0, 9.0, 11.0, 0)).unwrap(),
            1.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn stays_within_unit_range() {
        let candles: Vec<Candle> = (0..100)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.2).sin() * 8.0;
                let close = mid + (i as f64 * 0.5).cos() * 2.0;
                candle(mid, mid + 3.0, mid - 3.0, close, i)
            })
            .collect();
        let mut bop = BalanceOfPower::new();
        for v in bop.batch(&candles).into_iter().flatten() {
            assert!((-1.0..=1.0).contains(&v), "BOP {v} outside [-1, 1]");
        }
    }

    #[test]
    fn zero_range_bar_yields_zero() {
        let mut bop = BalanceOfPower::new();
        assert_relative_eq!(
            bop.update(candle(10.0, 10.0, 10.0, 10.0, 0)).unwrap(),
            0.0,
            epsilon = 1e-12
        );
    }

    /// Cover the Indicator-impl `name` body (73-75).
    #[test]
    fn name_metadata() {
        let bop = BalanceOfPower::new();
        assert_eq!(bop.name(), "BalanceOfPower");
    }

    #[test]
    fn emits_from_first_candle() {
        let mut bop = BalanceOfPower::new();
        assert_eq!(bop.warmup_period(), 1);
        assert!(!bop.is_ready());
        assert!(bop.update(candle(10.0, 11.0, 9.0, 10.0, 0)).is_some());
        assert!(bop.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut bop = BalanceOfPower::new();
        bop.update(candle(10.0, 11.0, 9.0, 10.0, 0));
        assert!(bop.is_ready());
        bop.reset();
        assert!(!bop.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + i as f64;
                candle(base, base + 2.0, base - 2.0, base + 1.0, i)
            })
            .collect();
        let mut a = BalanceOfPower::new();
        let mut b = BalanceOfPower::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
