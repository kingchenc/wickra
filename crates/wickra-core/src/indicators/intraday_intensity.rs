//! Intraday Intensity (Bostian) — the per-bar volume-weighted close-location.

use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Intraday Intensity — David Bostian's per-bar measure that weights each bar's
/// volume by where the close lands inside the bar's range:
///
/// ```text
/// II_t = volume * (2*close − high − low) / (high − low)   (0 if high == low)
/// ```
///
/// The fraction `(2*close − high − low) / (high − low)` is `+1` when the bar
/// closes on its high, `−1` when it closes on its low, and `0` at the midpoint,
/// so `II_t` is the volume pushed toward the extremes on that single bar —
/// Bostian's proxy for per-bar accumulation (positive) or distribution
/// (negative).
///
/// This emits the **raw per-bar** intensity, which is distinct from the two
/// derived forms Wickra ships separately: the **cumulative** running total is
/// the Accumulation/Distribution Line ([`Adl`](crate::Adl)), and the
/// volume-normalized windowed form ("Intraday Intensity %") is mathematically
/// the Chaikin Money Flow ([`Cmf`](crate::Cmf)). A doji whose `high == low`
/// contributes nothing. Each `update` is O(1) and the first bar already emits a
/// value.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, IntradayIntensity};
///
/// let mut indicator = IntradayIntensity::new();
/// let mut last = None;
/// for i in 0..20 {
///     let base = 100.0 + f64::from(i);
///     let c = Candle::new(base, base + 1.0, base - 1.0, base + 0.9, 1_000.0, 0).unwrap();
///     last = indicator.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct IntradayIntensity {
    last: Option<f64>,
}

impl IntradayIntensity {
    /// Construct a new Intraday Intensity. It is parameter-free.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for IntradayIntensity {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let range = candle.high - candle.low;
        let ii = if range > 0.0 {
            candle.volume * (2.0 * candle.close - candle.high - candle.low) / range
        } else {
            0.0
        };
        self.last = Some(ii);
        Some(ii)
    }

    fn reset(&mut self) {
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "IntradayIntensity"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(high: f64, low: f64, close: f64, volume: f64) -> Candle {
        Candle::new_unchecked(low, high, low, close, volume, 0)
    }

    #[test]
    fn accessors_and_metadata() {
        let iii = IntradayIntensity::new();
        assert_eq!(iii.warmup_period(), 1);
        assert_eq!(iii.name(), "IntradayIntensity");
        assert!(!iii.is_ready());
        assert_eq!(iii.value(), None);
    }

    #[test]
    fn first_bar_emits() {
        // close at the high: (2*101 - 102 - 100)/(2) = 0/... wait, high=102 low=100 close=101 -> 0.
        let mut iii = IntradayIntensity::new();
        // close on the high -> +1 * volume.
        let v = iii.update(candle(102.0, 100.0, 102.0, 500.0)).unwrap();
        assert_relative_eq!(v, 500.0, epsilon = 1e-9);
    }

    #[test]
    fn close_on_high_adds_full_volume() {
        let mut iii = IntradayIntensity::new();
        let v = iii.update(candle(110.0, 100.0, 110.0, 1_000.0)).unwrap();
        assert_relative_eq!(v, 1_000.0, epsilon = 1e-9);
    }

    #[test]
    fn close_on_low_subtracts_full_volume() {
        let mut iii = IntradayIntensity::new();
        let v = iii.update(candle(110.0, 100.0, 100.0, 1_000.0)).unwrap();
        assert_relative_eq!(v, -1_000.0, epsilon = 1e-9);
    }

    #[test]
    fn close_at_midpoint_adds_nothing() {
        let mut iii = IntradayIntensity::new();
        let v = iii.update(candle(110.0, 100.0, 105.0, 1_000.0)).unwrap();
        assert_relative_eq!(v, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn zero_range_adds_nothing() {
        let mut iii = IntradayIntensity::new();
        let v = iii.update(candle(100.0, 100.0, 100.0, 1_000.0)).unwrap();
        assert_relative_eq!(v, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn each_bar_is_independent() {
        // Per-bar (non-cumulative): each output depends only on that bar, so a
        // close-on-high +1000 bar is not carried into the next close-on-low bar.
        let mut iii = IntradayIntensity::new();
        let a = iii.update(candle(110.0, 100.0, 110.0, 1_000.0)).unwrap();
        let b = iii.update(candle(110.0, 100.0, 100.0, 400.0)).unwrap();
        assert_relative_eq!(a, 1_000.0, epsilon = 1e-9);
        assert_relative_eq!(b, -400.0, epsilon = 1e-9);
    }

    #[test]
    fn reset_clears_state() {
        let mut iii = IntradayIntensity::new();
        iii.batch(&[
            candle(110.0, 100.0, 108.0, 1.0),
            candle(110.0, 100.0, 102.0, 1.0),
        ]);
        assert!(iii.is_ready());
        iii.reset();
        assert!(!iii.is_ready());
        assert_eq!(iii.value(), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                let base = 100.0 + (f64::from(i) * 0.3).sin() * 6.0;
                candle(base + 2.0, base - 2.0, base + 0.7, 1_000.0 + f64::from(i))
            })
            .collect();
        let batch = IntradayIntensity::new().batch(&candles);
        let mut b = IntradayIntensity::new();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }
}
