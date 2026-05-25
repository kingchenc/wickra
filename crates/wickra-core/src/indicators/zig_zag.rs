//! `ZigZag` — percentage-threshold swing detector.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// `ZigZag` output: the price of the bar that completed the most recent swing
/// and its direction (`+1.0` for a high swing, `-1.0` for a low swing).
///
/// The price is the high of the bar at which the high-swing was anchored, or
/// the low of the bar at which the low-swing was anchored — i.e. the actual
/// extreme that the swing turns from, not the bar that triggered confirmation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZigZagOutput {
    /// Price of the confirmed swing extreme.
    pub swing: f64,
    /// Direction: `+1.0` if the swing is a high, `-1.0` if a low.
    pub direction: f64,
}

/// `ZigZag` — a non-repainting percent-threshold swing detector. Tracks the most
/// recent extreme (high or low) and confirms a reversal once price has moved
/// the configured percentage away from it.
///
/// ```text
/// uptrend (last swing was a low):
///   while highs make new highs, keep updating the pivot high
///   once close (or low) drops by ≥ threshold·high → confirm pivot high
///
/// downtrend (last swing was a high):
///   while lows make new lows, keep updating the pivot low
///   once close (or high) rises by ≥ threshold·low  → confirm pivot low
/// ```
///
/// The indicator emits `Some(swing)` only on the bar where a reversal is
/// confirmed, returning the price and direction of the **just-completed**
/// extreme. Bars between confirmations return `None`. The first bar bootstraps
/// the state — it determines an initial reference price but does not emit.
///
/// The threshold is a fractional change (`0.05` ≈ 5%); it must be strictly
/// positive and below `1.0`.
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, ZigZag};
///
/// let mut zz = ZigZag::new(0.10).unwrap();
/// for (i, p) in [100.0, 105.0, 115.0, 100.0, 90.0, 100.0].iter().enumerate() {
///     let c = Candle::new(*p, *p + 0.5, *p - 0.5, *p, 1.0, i as i64).unwrap();
///     let _ = zz.update(c);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ZigZag {
    threshold: f64,
    state: Option<State>,
}

#[derive(Debug, Clone, Copy)]
struct State {
    /// Direction of the running trend: `+1.0` (uptrend tracking a pivot high)
    /// or `-1.0` (downtrend tracking a pivot low).
    direction: f64,
    /// The current candidate extreme price (the running pivot).
    extreme: f64,
}

impl ZigZag {
    /// Construct a new `ZigZag` with a fractional reversal threshold (e.g. `0.05`
    /// for a 5% swing).
    ///
    /// # Errors
    /// Returns [`Error::InvalidPeriod`] if `threshold` is not in `(0.0, 1.0)`
    /// or is not finite.
    pub fn new(threshold: f64) -> Result<Self> {
        if !threshold.is_finite() || threshold <= 0.0 || threshold >= 1.0 {
            return Err(Error::InvalidPeriod {
                message: "ZigZag threshold must be a finite fraction in (0, 1)",
            });
        }
        Ok(Self {
            threshold,
            state: None,
        })
    }

    /// Configured reversal threshold (fractional).
    pub const fn threshold(&self) -> f64 {
        self.threshold
    }
}

impl Indicator for ZigZag {
    type Input = Candle;
    type Output = ZigZagOutput;

    fn update(&mut self, candle: Candle) -> Option<ZigZagOutput> {
        let Some(s) = self.state else {
            // Bootstrap: seed an uptrend tracking the first candle's high.
            self.state = Some(State {
                direction: 1.0,
                extreme: candle.high,
            });
            return None;
        };

        if s.direction > 0.0 {
            // Uptrend: keep raising the candidate high; confirm reversal if
            // the candle's low has dropped by threshold from the candidate.
            if candle.high > s.extreme {
                self.state = Some(State {
                    direction: 1.0,
                    extreme: candle.high,
                });
                return None;
            }
            if candle.low <= s.extreme * (1.0 - self.threshold) {
                // Confirm the swing high; flip to downtrend tracking this bar's low.
                let confirmed = ZigZagOutput {
                    swing: s.extreme,
                    direction: 1.0,
                };
                self.state = Some(State {
                    direction: -1.0,
                    extreme: candle.low,
                });
                return Some(confirmed);
            }
            None
        } else {
            // Downtrend: lower the candidate low; confirm reversal if the
            // candle's high has risen by threshold from the candidate.
            if candle.low < s.extreme {
                self.state = Some(State {
                    direction: -1.0,
                    extreme: candle.low,
                });
                return None;
            }
            if candle.high >= s.extreme * (1.0 + self.threshold) {
                let confirmed = ZigZagOutput {
                    swing: s.extreme,
                    direction: -1.0,
                };
                self.state = Some(State {
                    direction: 1.0,
                    extreme: candle.high,
                });
                return Some(confirmed);
            }
            None
        }
    }

    fn reset(&mut self) {
        self.state = None;
    }

    fn warmup_period(&self) -> usize {
        // Bootstrap takes one bar; confirmation of the first swing needs at
        // least one more move past the threshold. Best-case the first swing
        // lands on the second bar.
        2
    }

    fn is_ready(&self) -> bool {
        self.state.is_some()
    }

    fn name(&self) -> &'static str {
        "ZigZag"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;

    fn c(price: f64, ts: i64) -> Candle {
        Candle::new(price, price + 0.001, price - 0.001, price, 1.0, ts).unwrap()
    }

    fn c_hl(h: f64, l: f64, ts: i64) -> Candle {
        Candle::new(l, h, l, l, 1.0, ts).unwrap()
    }

    #[test]
    fn rejects_invalid_threshold() {
        assert!(ZigZag::new(0.0).is_err());
        assert!(ZigZag::new(-0.1).is_err());
        assert!(ZigZag::new(1.0).is_err());
        assert!(ZigZag::new(f64::NAN).is_err());
        assert!(ZigZag::new(f64::INFINITY).is_err());
    }

    #[test]
    fn first_bar_only_bootstraps() {
        let mut zz = ZigZag::new(0.05).unwrap();
        assert_eq!(zz.update(c(100.0, 0)), None);
        assert!(zz.is_ready());
    }

    #[test]
    fn confirms_high_swing_on_threshold_drop() {
        let mut zz = ZigZag::new(0.10).unwrap();
        // Up to a peak of 120, then a drop to 100 = 16.7% reversal → confirms.
        let _ = zz.update(c_hl(100.0, 99.5, 0));
        let _ = zz.update(c_hl(120.0, 119.5, 1));
        let confirmed = zz.update(c_hl(101.0, 100.0, 2));
        let o = confirmed.expect("the third bar's drop triggers confirmation");
        assert!((o.swing - 120.0).abs() < 1e-9);
        assert_eq!(o.direction, 1.0);
    }

    #[test]
    fn confirms_low_swing_on_threshold_rise() {
        let mut zz = ZigZag::new(0.10).unwrap();
        // Up to 120 to seed the high pivot, drop to confirm it as a high,
        // then rise from the new low pivot by 10% to confirm it as a low.
        let _ = zz.update(c_hl(100.0, 99.5, 0));
        let _ = zz.update(c_hl(120.0, 119.5, 1));
        let _ = zz.update(c_hl(101.0, 90.0, 2)); // drop confirms 120-high; new low 90.
        let _ = zz.update(c_hl(91.0, 90.5, 3));
        // Rise to 100 from low 90 = 11.1% → confirms low.
        let confirmed = zz.update(c_hl(100.0, 99.0, 4));
        let o = confirmed.expect("the rise confirms the low swing");
        assert!((o.swing - 90.0).abs() < 1e-9);
        assert_eq!(o.direction, -1.0);
    }

    #[test]
    fn small_oscillations_yield_no_swings() {
        let mut zz = ZigZag::new(0.20).unwrap();
        let _ = zz.update(c(100.0, 0));
        for i in 1..20 {
            // Bounce around 100 ± 5; never crosses the 20% threshold.
            let p = 100.0 + ((f64::from(i)) * 0.3).sin() * 5.0;
            assert!(
                zz.update(c(p, i.into())).is_none(),
                "unexpected swing at i={i}"
            );
        }
    }

    #[test]
    fn warmup_and_ready_lifecycle() {
        let mut zz = ZigZag::new(0.05).unwrap();
        assert!(!zz.is_ready());
        assert_eq!(zz.warmup_period(), 2);
        zz.update(c(100.0, 0));
        assert!(zz.is_ready());
    }

    #[test]
    fn reset_clears_state() {
        let mut zz = ZigZag::new(0.10).unwrap();
        let _ = zz.update(c_hl(100.0, 99.0, 0));
        let _ = zz.update(c_hl(120.0, 119.0, 1));
        zz.reset();
        assert!(!zz.is_ready());
        assert_eq!(zz.update(c_hl(110.0, 109.0, 0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..40)
            .map(|i| {
                let p = 100.0 + (i as f64 * 0.3).sin() * 15.0;
                c(p, i)
            })
            .collect();
        let mut a = ZigZag::new(0.05).unwrap();
        let mut b = ZigZag::new(0.05).unwrap();
        assert_eq!(
            a.batch(&candles),
            candles.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn accessors_and_metadata() {
        let zz = ZigZag::new(0.05).unwrap();
        assert!((zz.threshold() - 0.05).abs() < 1e-12);
        assert_eq!(zz.warmup_period(), 2);
        assert_eq!(zz.name(), "ZigZag");
    }
}
