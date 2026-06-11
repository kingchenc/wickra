//! Renko Trailing Stop.

use crate::error::{Error, Result};
use crate::traits::Indicator;

/// Renko Trailing Stop — a trailing stop that follows a Renko-style brick
/// anchor: the stop only moves when price has advanced (or fallen) by at least
/// one full `block_size`, and then jumps the same fixed distance.
///
/// ```text
/// long:  advance = floor((close − anchor) / block_size)
///        if advance ≥ 1 -> anchor += advance · block_size
///        stop = anchor − block_size
///        flip-to-short on close < stop -> anchor = close, stop = anchor + block_size
/// short: advance = floor((anchor − close) / block_size)
///        if advance ≥ 1 -> anchor −= advance · block_size
///        stop = anchor + block_size
///        flip-to-long on close > stop -> anchor = close, stop = anchor − block_size
/// ```
///
/// Like a Renko chart the stop ignores intra-block noise — it sits one full
/// block behind the last "printed" brick and only ratchets in whole-block
/// increments. The first input seeds a long anchor at the close.
///
/// # Example
///
/// ```
/// use wickra_core::{Indicator, RenkoTrailingStop};
///
/// let mut indicator = RenkoTrailingStop::new(1.0).unwrap();
/// let mut last = None;
/// for i in 0..20 {
///     last = indicator.update(100.0 + f64::from(i));
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct RenkoTrailingStop {
    block_size: f64,
    anchor: Option<f64>,
    long: bool,
}

impl RenkoTrailingStop {
    /// Construct a Renko Trailing Stop with an explicit block size.
    ///
    /// # Errors
    /// Returns [`Error::NonPositiveMultiplier`] if `block_size` is not strictly
    /// positive and finite.
    pub fn new(block_size: f64) -> Result<Self> {
        if !block_size.is_finite() || block_size <= 0.0 {
            return Err(Error::NonPositiveMultiplier);
        }
        Ok(Self {
            block_size,
            anchor: None,
            long: true,
        })
    }

    /// A common configuration: a `1.0` block size.
    pub fn classic() -> Self {
        Self::new(1.0).expect("classic block size is valid")
    }

    /// Configured block size.
    pub const fn block_size(&self) -> f64 {
        self.block_size
    }
}

impl Indicator for RenkoTrailingStop {
    type Input = f64;
    type Output = f64;

    fn update(&mut self, close: f64) -> Option<f64> {
        if !close.is_finite() {
            return None;
        }
        let anchor = match self.anchor {
            Some(prev) => {
                if self.long {
                    let stop = prev - self.block_size;
                    if close < stop {
                        // Close-through long stop -> flip to short.
                        self.long = false;
                        close
                    } else {
                        let blocks = ((close - prev) / self.block_size).floor();
                        if blocks >= 1.0 {
                            prev + blocks * self.block_size
                        } else {
                            prev
                        }
                    }
                } else {
                    let stop = prev + self.block_size;
                    if close > stop {
                        self.long = true;
                        close
                    } else {
                        let blocks = ((prev - close) / self.block_size).floor();
                        if blocks >= 1.0 {
                            prev - blocks * self.block_size
                        } else {
                            prev
                        }
                    }
                }
            }
            None => close,
        };
        self.anchor = Some(anchor);
        let stop = if self.long {
            anchor - self.block_size
        } else {
            anchor + self.block_size
        };
        Some(stop)
    }

    fn reset(&mut self) {
        self.anchor = None;
        self.long = true;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.anchor.is_some()
    }

    fn name(&self) -> &'static str {
        "RenkoTrailingStop"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    #[test]
    fn rejects_invalid_block() {
        assert!(RenkoTrailingStop::new(0.0).is_err());
        assert!(RenkoTrailingStop::new(-1.0).is_err());
        assert!(RenkoTrailingStop::new(f64::NAN).is_err());
        assert!(RenkoTrailingStop::new(f64::INFINITY).is_err());
    }

    #[test]
    fn accessors_and_metadata() {
        let s = RenkoTrailingStop::classic();
        assert_relative_eq!(s.block_size(), 1.0, epsilon = 1e-12);
        assert_eq!(s.name(), "RenkoTrailingStop");
        assert_eq!(s.warmup_period(), 1);
    }

    #[test]
    fn first_value_is_block_below_close() {
        let mut s = RenkoTrailingStop::new(1.0).unwrap();
        // Seed anchor = 100, long stop = 99.
        assert_relative_eq!(s.update(100.0).unwrap(), 99.0, epsilon = 1e-12);
    }

    #[test]
    fn stop_only_moves_after_full_block_advance() {
        let mut s = RenkoTrailingStop::new(1.0).unwrap();
        s.update(100.0);
        // Intra-block move -> stop unchanged at 99.
        assert_relative_eq!(s.update(100.5).unwrap(), 99.0, epsilon = 1e-12);
        // Full block: 101 -> anchor 101, stop 100.
        assert_relative_eq!(s.update(101.0).unwrap(), 100.0, epsilon = 1e-12);
        // Two blocks at once: 103.5 -> anchor 103, stop 102.
        assert_relative_eq!(s.update(103.5).unwrap(), 102.0, epsilon = 1e-12);
    }

    #[test]
    fn flips_to_short_on_close_through_stop() {
        let mut s = RenkoTrailingStop::new(1.0).unwrap();
        s.update(100.0);
        s.update(105.0); // anchor 105, stop 104
                         // Drop to 50 closes through 104 -> flip short, anchor=50, stop=51.
        let flipped = s.update(50.0).unwrap();
        assert_relative_eq!(flipped, 51.0, epsilon = 1e-12);
    }

    #[test]
    fn short_anchor_ratchets_down_and_flips_back() {
        let mut s = RenkoTrailingStop::new(1.0).unwrap();
        s.update(100.0);
        s.update(50.0); // short, anchor 50, stop 51
        let v = s.update(48.0).unwrap();
        // Two blocks down: anchor 48, stop 49.
        assert_relative_eq!(v, 49.0, epsilon = 1e-12);
        // Rally to 60 closes through 49 -> flip long, anchor=60, stop=59.
        let back = s.update(60.0).unwrap();
        assert_relative_eq!(back, 59.0, epsilon = 1e-12);
    }

    #[test]
    fn constant_series_holds_stop() {
        let mut s = RenkoTrailingStop::new(1.0).unwrap();
        let out = s.batch(&[100.0; 30]);
        for v in out.into_iter().flatten() {
            assert_relative_eq!(v, 99.0, epsilon = 1e-12);
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut s = RenkoTrailingStop::new(1.0).unwrap();
        s.update(100.0);
        s.update(50.0); // flip short
        assert!(s.is_ready());
        s.reset();
        assert!(!s.is_ready());
        assert_relative_eq!(s.update(200.0).unwrap(), 199.0, epsilon = 1e-12);
    }

    #[test]
    fn batch_equals_streaming() {
        let prices: Vec<f64> = (0..80)
            .map(|i| 100.0 + (f64::from(i) * 0.3).sin() * 8.0)
            .collect();
        let mut a = RenkoTrailingStop::classic();
        let mut b = RenkoTrailingStop::classic();
        assert_eq!(
            a.batch(&prices),
            prices.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
