//! Fibonacci Time Zones — vertical markers at Fibonacci bar-distances from the
//! most recent swing pivot.

use crate::indicators::pattern_swing::{SwingTracker, SWING_THRESHOLD};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Where the current bar sits relative to the Fibonacci time-zone grid anchored
/// on the most recent confirmed pivot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FibTimeZonesOutput {
    /// `1.0` when the current bar lands on a Fibonacci time zone (a bar distance
    /// of 1, 2, 3, 5, 8, 13, … from the anchor pivot), otherwise `0.0`.
    pub on_zone: f64,
    /// Number of bars until the next Fibonacci time zone (`0` is never returned —
    /// when on a zone this is the gap to the following one).
    pub bars_to_next: f64,
}

/// Fibonacci Time Zones (`FibTimeZones`).
///
/// Anchored on the most recent confirmed swing pivot, the Fibonacci sequence
/// `1, 2, 3, 5, 8, 13, …` marks bars at which trend changes are classically
/// anticipated. Reports whether the current bar is on a zone and how many bars
/// remain until the next one.
///
/// Parameter-free; construction is infallible. Returns `None` until the first
/// pivot has confirmed.
///
/// See `crates/wickra-core/src/indicators/fib_time_zones.rs`.
/// # Example
///
/// ```
/// use wickra_core::{FibTimeZones, Candle, Indicator};
///
/// let mut indicator = FibTimeZones::new();
/// // `None` during warmup, then `Some(_)` once enough bars are seen.
/// let mut out = None;
/// for i in 0..40i64 {
///     let p = 100.0 + (i as f64 * 0.4).sin() * 5.0;
///     let candle = Candle::new(p, p + 1.5, p - 1.5, p + 0.3, 1_000.0, i).unwrap();
///     out = indicator.update(candle);
/// }
/// let _ = out;
/// ```
#[derive(Debug, Clone)]
pub struct FibTimeZones {
    swing: SwingTracker,
}

impl FibTimeZones {
    /// Construct a new Fibonacci Time Zones tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            swing: SwingTracker::new(SWING_THRESHOLD, 2),
        }
    }

    fn zones(&self) -> Option<FibTimeZonesOutput> {
        let anchor = self.swing.pivots().last()?;
        let distance = self.swing.current_bar() - anchor.bar;
        // Walk the time-zone sequence 1, 2, 3, 5, 8, … : `lo` advances through the
        // members, `on_zone` records a hit, and the loop exits with `lo` holding
        // the smallest member strictly greater than `distance`.
        let (mut lo, mut hi) = (1usize, 2usize);
        let mut on_zone = false;
        while lo <= distance {
            if lo == distance {
                on_zone = true;
            }
            let next = lo + hi;
            lo = hi;
            hi = next;
        }
        Some(FibTimeZonesOutput {
            on_zone: f64::from(u8::from(on_zone)),
            bars_to_next: (lo - distance) as f64,
        })
    }
}

impl Default for FibTimeZones {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for FibTimeZones {
    type Input = Candle;
    type Output = FibTimeZonesOutput;

    fn update(&mut self, candle: Candle) -> Option<FibTimeZonesOutput> {
        self.swing.update(candle);
        self.zones()
    }

    fn reset(&mut self) {
        self.swing.reset();
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        !self.swing.pivots().is_empty()
    }

    fn name(&self) -> &'static str {
        "FibTimeZones"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn c(high: f64, low: f64, ts: i64) -> Candle {
        Candle::new(low, high, low, low, 1.0, ts).unwrap()
    }

    /// One pivot confirms at bar 0 (high @200, confirmed at bar 1); subsequent
    /// flat bars neither extend nor confirm, so the anchor stays at bar 0 and the
    /// distance equals the current bar index.
    fn anchored_run() -> Vec<Candle> {
        let mut bars = vec![c(200.0, 199.0, 0), c(190.0, 150.0, 1)];
        for ts in 2..=5 {
            bars.push(c(155.0, 151.0, ts));
        }
        bars
    }

    #[test]
    fn accessors_and_metadata() {
        let indicator = FibTimeZones::new();
        assert_eq!(indicator.name(), "FibTimeZones");
        assert_eq!(indicator.warmup_period(), 2);
        assert!(!indicator.is_ready());
        assert!(!FibTimeZones::default().is_ready());
    }

    #[test]
    fn no_output_before_first_pivot() {
        let mut indicator = FibTimeZones::new();
        // The bootstrap bar confirms nothing.
        assert!(indicator.update(c(200.0, 199.0, 0)).is_none());
        assert!(!indicator.is_ready());
    }

    #[test]
    fn flags_zones_and_counts_to_next() {
        let mut indicator = FibTimeZones::new();
        let out: Vec<_> = anchored_run()
            .into_iter()
            .map(|x| indicator.update(x))
            .collect();
        assert!(out[0].is_none()); // bootstrap, no pivot yet
        assert!(indicator.is_ready());
        // out[i] is reported at current bar i; anchor at bar 0 → distance = i.
        let d1 = out[1].unwrap(); // distance 1 → a zone
        assert_relative_eq!(d1.on_zone, 1.0);
        assert_relative_eq!(d1.bars_to_next, 1.0); // next zone at 2
        let d4 = out[4].unwrap(); // distance 4 → not a zone
        assert_relative_eq!(d4.on_zone, 0.0);
        assert_relative_eq!(d4.bars_to_next, 1.0); // next zone at 5
        let d5 = out[5].unwrap(); // distance 5 → a zone
        assert_relative_eq!(d5.on_zone, 1.0);
        assert_relative_eq!(d5.bars_to_next, 3.0); // next zone at 8
    }

    #[test]
    fn reset_clears_state() {
        let mut indicator = FibTimeZones::new();
        for candle in anchored_run() {
            let _ = indicator.update(candle);
        }
        assert!(indicator.is_ready());
        indicator.reset();
        assert!(!indicator.is_ready());
        assert!(indicator.update(c(100.0, 99.5, 0)).is_none());
    }

    #[test]
    fn batch_equals_streaming() {
        let candles = anchored_run();
        let mut a = FibTimeZones::new();
        let mut b = FibTimeZones::new();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
