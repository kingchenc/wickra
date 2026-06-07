//! Trade Volume Index (TVI) — cumulative volume signed by a minimum-tick rule.

use crate::error::{Error, Result};
use crate::ohlcv::Candle;
use crate::traits::Indicator;

/// Trade Volume Index — a cumulative line that adds volume while price ticks up
/// and subtracts it while price ticks down, where "up" and "down" are decided by
/// a **minimum tick value** rather than any change.
///
/// ```text
/// change = close − prev_close
/// if  change >  min_tick:  direction = +1
/// if  change < −min_tick:  direction = −1
/// else:                    direction unchanged   (price is "churning")
/// TVI_t = TVI_{t−1} + direction * volume
/// ```
///
/// The minimum tick value (MTV) is a dead-band: only moves larger than `min_tick`
/// flip the accumulation direction, so a price drifting within the spread keeps
/// adding volume in the last established direction instead of whipsawing. This is
/// the cumulative-volume analogue of [`Obv`](crate::Obv), but with a noise filter
/// and applied to close-to-close moves. Like all cumulative lines, only its slope
/// and divergences against price carry meaning — the absolute level is arbitrary.
///
/// The first candle seeds the reference close and emits nothing; thereafter each
/// bar emits the running total. Each `update` is O(1).
///
/// # Example
///
/// ```
/// use wickra_core::{Candle, Indicator, TradeVolumeIndex};
///
/// let mut indicator = TradeVolumeIndex::new(0.5).unwrap();
/// let mut last = None;
/// for i in 0..20 {
///     let close = 100.0 + f64::from(i);
///     let c = Candle::new(close, close + 0.5, close - 0.5, close, 1_000.0, 0).unwrap();
///     last = indicator.update(c);
/// }
/// assert!(last.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct TradeVolumeIndex {
    min_tick: f64,
    prev_close: Option<f64>,
    direction: f64,
    tvi: f64,
    last: Option<f64>,
}

impl TradeVolumeIndex {
    /// Construct a new Trade Volume Index with the given minimum tick value.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] if `min_tick` is not finite or is
    /// negative. A `min_tick` of `0` is allowed and makes every non-zero move
    /// flip the direction.
    pub fn new(min_tick: f64) -> Result<Self> {
        if !min_tick.is_finite() || min_tick < 0.0 {
            return Err(Error::InvalidParameter {
                message: "trade volume index min_tick must be finite and non-negative",
            });
        }
        Ok(Self {
            min_tick,
            prev_close: None,
            direction: 0.0,
            tvi: 0.0,
            last: None,
        })
    }

    /// Configured minimum tick value.
    pub const fn min_tick(&self) -> f64 {
        self.min_tick
    }

    /// Current value if available.
    pub const fn value(&self) -> Option<f64> {
        self.last
    }
}

impl Indicator for TradeVolumeIndex {
    type Input = Candle;
    type Output = f64;

    fn update(&mut self, candle: Candle) -> Option<f64> {
        let Some(prev_close) = self.prev_close else {
            self.prev_close = Some(candle.close);
            return None;
        };
        let change = candle.close - prev_close;
        if change > self.min_tick {
            self.direction = 1.0;
        } else if change < -self.min_tick {
            self.direction = -1.0;
        }
        // Otherwise the direction is held from the previous bar (or 0 before the
        // first decisive move), so a churning price keeps its last lean.
        self.tvi += self.direction * candle.volume;
        self.prev_close = Some(candle.close);
        self.last = Some(self.tvi);
        Some(self.tvi)
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.direction = 0.0;
        self.tvi = 0.0;
        self.last = None;
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.last.is_some()
    }

    fn name(&self) -> &'static str {
        "TradeVolumeIndex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::BatchExt;
    use approx::assert_relative_eq;

    fn candle(close: f64, volume: f64) -> Candle {
        Candle::new_unchecked(close, close, close, close, volume, 0)
    }

    #[test]
    fn rejects_invalid_min_tick() {
        assert!(matches!(
            TradeVolumeIndex::new(-1.0),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            TradeVolumeIndex::new(f64::NAN),
            Err(Error::InvalidParameter { .. })
        ));
        assert!(TradeVolumeIndex::new(0.0).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let tvi = TradeVolumeIndex::new(0.25).unwrap();
        assert_relative_eq!(tvi.min_tick(), 0.25, epsilon = 1e-12);
        assert_eq!(tvi.warmup_period(), 2);
        assert_eq!(tvi.name(), "TradeVolumeIndex");
        assert!(!tvi.is_ready());
        assert_eq!(tvi.value(), None);
    }

    #[test]
    fn first_bar_seeds_without_output() {
        let mut tvi = TradeVolumeIndex::new(0.5).unwrap();
        assert_eq!(tvi.update(candle(100.0, 1_000.0)), None);
        assert!(tvi.update(candle(101.0, 1_000.0)).is_some());
    }

    #[test]
    fn uptrend_accumulates_volume() {
        // Each step of +1 exceeds the 0.5 tick -> direction +1 -> add volume.
        let mut tvi = TradeVolumeIndex::new(0.5).unwrap();
        let candles = [
            candle(100.0, 1_000.0), // seed
            candle(101.0, 500.0),   // +1 -> +500
            candle(102.0, 300.0),   // +1 -> +300
        ];
        let out = tvi.batch(&candles);
        assert_relative_eq!(out[1].unwrap(), 500.0, epsilon = 1e-9);
        assert_relative_eq!(out[2].unwrap(), 800.0, epsilon = 1e-9);
    }

    #[test]
    fn small_move_holds_last_direction() {
        // After an up-move, a sub-tick wobble keeps adding in the up direction.
        let mut tvi = TradeVolumeIndex::new(1.0).unwrap();
        let candles = [
            candle(100.0, 1_000.0), // seed
            candle(102.0, 400.0),   // +2 > tick -> dir +1, +400
            candle(102.2, 100.0),   // +0.2 < tick -> hold dir +1, +100
        ];
        let out = tvi.batch(&candles);
        assert_relative_eq!(out[1].unwrap(), 400.0, epsilon = 1e-9);
        assert_relative_eq!(out[2].unwrap(), 500.0, epsilon = 1e-9);
    }

    #[test]
    fn downtrend_distributes_volume() {
        let mut tvi = TradeVolumeIndex::new(0.5).unwrap();
        let candles = [
            candle(100.0, 1_000.0),
            candle(99.0, 200.0), // -1 -> -200
            candle(98.0, 300.0), // -1 -> -300
        ];
        let out = tvi.batch(&candles);
        assert_relative_eq!(out[2].unwrap(), -500.0, epsilon = 1e-9);
    }

    #[test]
    fn reset_clears_state() {
        let mut tvi = TradeVolumeIndex::new(0.5).unwrap();
        tvi.batch(&[candle(100.0, 1.0), candle(101.0, 1.0), candle(102.0, 1.0)]);
        assert!(tvi.is_ready());
        tvi.reset();
        assert!(!tvi.is_ready());
        assert_eq!(tvi.value(), None);
        assert_eq!(tvi.update(candle(100.0, 1.0)), None);
    }

    #[test]
    fn batch_equals_streaming() {
        let candles: Vec<Candle> = (0..80)
            .map(|i| {
                candle(
                    100.0 + (f64::from(i) * 0.3).sin() * 5.0,
                    1_000.0 + f64::from(i),
                )
            })
            .collect();
        let batch = TradeVolumeIndex::new(0.5).unwrap().batch(&candles);
        let mut b = TradeVolumeIndex::new(0.5).unwrap();
        let streamed: Vec<_> = candles.iter().map(|c| b.update(*c)).collect();
        assert_eq!(batch, streamed);
    }
}
