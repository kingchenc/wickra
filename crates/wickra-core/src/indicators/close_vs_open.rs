//! Close vs Open — the signed relative body of a bar.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Close vs Open — the bar's body as a signed fraction of its open price.
///
/// ```text
/// CloseVsOpen = (close − open) / open
/// ```
///
/// A scale-free, signed measure of how far price travelled from open to close:
/// `+0.02` is a bar that closed 2% above its open (a green bar), `−0.02` the
/// mirror. Unlike [`BalanceOfPower`](crate::BalanceOfPower) — which normalises
/// the body by the bar *range* — this normalises by the *open price*, so it is
/// directly comparable to a return and stays meaningful across instruments of
/// different nominal price. A zero open carries no scale and yields `0`.
///
/// This is a stateless per-bar transform: every candle produces one value.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, CloseVsOpen};
///
/// let mut indicator = CloseVsOpen::new();
/// // open 100, close 102 -> +0.02.
/// let c = Candle::new(100.0, 103.0, 99.0, 102.0, 10.0, 0).unwrap();
/// assert!((indicator.update(c).unwrap() - 0.02).abs() < 1e-12);
/// ```
#[derive(Debug, Clone, Default)]
pub struct CloseVsOpen {
    has_emitted: bool,
}

impl CloseVsOpen {
    /// Construct a new Close vs Open transform.
    pub const fn new() -> Self {
        Self { has_emitted: false }
    }
}

impl Indicator for CloseVsOpen {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        self.has_emitted = true;
        let out = if candle.open == 0.0 {
            // A zero open price carries no scale to normalise against.
            0.0
        } else {
            (candle.close - candle.open) / candle.open
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
        "CloseVsOpen"
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
        // (102 - 100) / 100 = 0.02.
        let mut cvo = CloseVsOpen::new();
        assert_relative_eq!(
            cvo.update(candle(100.0, 103.0, 99.0, 102.0, 0)).unwrap(),
            0.02,
            epsilon = 1e-12
        );
    }

    #[test]
    fn negative_body_is_negative() {
        let mut cvo = CloseVsOpen::new();
        // close below open -> negative.
        assert_relative_eq!(
            cvo.update(candle(100.0, 101.0, 97.0, 98.0, 0)).unwrap(),
            -0.02,
            epsilon = 1e-12
        );
    }

    #[test]
    fn zero_open_yields_zero() {
        // Candle permits a zero open (only finiteness + OHLC ordering checked).
        let mut cvo = CloseVsOpen::new();
        assert_relative_eq!(
            cvo.update(candle(0.0, 1.0, 0.0, 0.5, 0)).unwrap(),
            0.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn name_metadata() {
        let cvo = CloseVsOpen::new();
        assert_eq!(cvo.name(), "CloseVsOpen");
    }

    #[test]
    fn emits_from_first_candle() {
        let mut cvo = CloseVsOpen::new();
        assert_eq!(cvo.warmup_period(), 1);
        assert!(!cvo.is_ready());
        assert!(cvo.update(candle(10.0, 11.0, 9.0, 10.0, 0)).is_some());
        assert!(cvo.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut cvo = CloseVsOpen::new();
        cvo.update(candle(10.0, 11.0, 9.0, 10.0, 0));
        assert!(cvo.is_ready());
        cvo.reset();
        assert!(!cvo.is_ready());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let base = 100.0 + f64::from(i);
                candle(base, base + 2.0, base - 2.0, base + 1.0, i64::from(i))
            })
            .collect();
        let mut a = CloseVsOpen::new();
        let mut b = CloseVsOpen::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
