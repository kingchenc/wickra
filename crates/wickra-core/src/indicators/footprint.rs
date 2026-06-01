//! Footprint — buy/sell volume profile per price bucket within a bar.

use std::collections::BTreeMap;

use crate::error::{Error, Result};
use crate::microstructure::Trade;
use crate::traits::Indicator;

/// One price bucket of a [`Footprint`]: the buy- and sell-initiated volume that
/// traded there since the last reset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FootprintLevel {
    /// Bucket price (the bucket index times the tick size).
    pub price: f64,
    /// Sell-initiated (bid-hitting) volume traded at this bucket.
    pub bid_vol: f64,
    /// Buy-initiated (ask-lifting) volume traded at this bucket.
    pub ask_vol: f64,
}

/// The full footprint of a bar: one [`FootprintLevel`] per touched price
/// bucket, sorted ascending by price.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FootprintOutput {
    /// Touched price buckets, lowest price first.
    pub levels: Vec<FootprintLevel>,
}

/// Footprint — the buy/sell volume profile of a bar, bucketed by price.
///
/// A footprint (a.k.a. bid/ask or volume cluster chart) decomposes the volume
/// traded within a bar across the price levels at which it printed, splitting
/// each level into buy-initiated (ask-lifting) and sell-initiated (bid-hitting)
/// volume. It exposes *where* inside a bar the activity happened and which side
/// was the aggressor there — the basis for absorption, imbalance and
/// point-of-control analysis that a single OHLCV bar hides.
///
/// Each trade is assigned to the price bucket `round(price / tick_size)`; its
/// size is added to that bucket's ask volume for a buy and bid volume for a
/// sell. Every [`update`] returns the complete footprint accumulated since the
/// last [`reset`], as a [`FootprintOutput`] whose `levels` are sorted ascending
/// by price. Call [`reset`] at each bar (or session) boundary to start a fresh
/// footprint.
///
/// `Input = Trade`, `Output = FootprintOutput`. Ready after the first trade.
///
/// [`update`]: crate::Indicator::update
/// [`reset`]: crate::Indicator::reset
///
/// # Example
///
/// ```
/// use wickra_core::{Footprint, Indicator, Side, Trade};
///
/// let mut fp = Footprint::new(1.0).unwrap();
/// fp.update(Trade::new(100.2, 2.0, Side::Buy, 0).unwrap());
/// let out = fp.update(Trade::new(100.7, 3.0, Side::Sell, 1).unwrap()).unwrap();
/// // Two buckets: 100 (ask 2) and 101 (bid 3).
/// assert_eq!(out.levels.len(), 2);
/// assert_eq!(out.levels[0].price, 100.0);
/// assert_eq!(out.levels[0].ask_vol, 2.0);
/// assert_eq!(out.levels[1].price, 101.0);
/// assert_eq!(out.levels[1].bid_vol, 3.0);
/// ```
#[derive(Debug, Clone)]
pub struct Footprint {
    tick_size: f64,
    // bucket index -> (bid_vol = sell-initiated, ask_vol = buy-initiated).
    buckets: BTreeMap<i64, (f64, f64)>,
    has_emitted: bool,
}

impl Footprint {
    /// Construct a footprint with the given price-bucket `tick_size`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTick`] if `tick_size` is not a finite, strictly
    /// positive number.
    pub fn new(tick_size: f64) -> Result<Self> {
        if !tick_size.is_finite() || tick_size <= 0.0 {
            return Err(Error::InvalidTick {
                message: "footprint tick_size must be finite and positive",
            });
        }
        Ok(Self {
            tick_size,
            buckets: BTreeMap::new(),
            has_emitted: false,
        })
    }

    /// The configured price-bucket size.
    pub const fn tick_size(&self) -> f64 {
        self.tick_size
    }

    fn bucket_index(&self, price: f64) -> i64 {
        // Float-to-int `as` saturates rather than wrapping, so an extreme
        // price/tick ratio clamps to i64::MIN/MAX instead of misbehaving;
        // realistic ratios fit comfortably.
        #[allow(clippy::cast_possible_truncation)]
        {
            (price / self.tick_size).round() as i64
        }
    }

    fn snapshot(&self) -> FootprintOutput {
        let levels = self
            .buckets
            .iter()
            .map(|(&index, &(bid_vol, ask_vol))| FootprintLevel {
                price: index as f64 * self.tick_size,
                bid_vol,
                ask_vol,
            })
            .collect();
        FootprintOutput { levels }
    }
}

impl Indicator for Footprint {
    type Input = Trade;
    type Output = FootprintOutput;

    fn update(&mut self, trade: Trade) -> Option<FootprintOutput> {
        self.has_emitted = true;
        let index = self.bucket_index(trade.price);
        let entry = self.buckets.entry(index).or_insert((0.0, 0.0));
        if trade.side.sign() > 0.0 {
            entry.1 += trade.size;
        } else {
            entry.0 += trade.size;
        }
        Some(self.snapshot())
    }

    fn reset(&mut self) {
        self.buckets.clear();
        self.has_emitted = false;
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.has_emitted
    }

    fn name(&self) -> &'static str {
        "Footprint"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::microstructure::Side;
    use crate::traits::BatchExt;

    fn trade(price: f64, size: f64, side: Side) -> Trade {
        Trade::new(price, size, side, 0).unwrap()
    }

    #[test]
    fn rejects_bad_tick_size() {
        assert!(matches!(
            Footprint::new(0.0),
            Err(Error::InvalidTick { .. })
        ));
        assert!(matches!(
            Footprint::new(-1.0),
            Err(Error::InvalidTick { .. })
        ));
        assert!(matches!(
            Footprint::new(f64::NAN),
            Err(Error::InvalidTick { .. })
        ));
        assert!(Footprint::new(0.5).is_ok());
    }

    #[test]
    fn accessors_and_metadata() {
        let fp = Footprint::new(0.25).unwrap();
        assert_eq!(fp.name(), "Footprint");
        assert_eq!(fp.warmup_period(), 1);
        assert_eq!(fp.tick_size(), 0.25);
        assert!(!fp.is_ready());
    }

    #[test]
    fn buckets_buy_and_sell_volume() {
        let mut fp = Footprint::new(1.0).unwrap();
        fp.update(trade(100.2, 2.0, Side::Buy));
        fp.update(trade(100.7, 3.0, Side::Sell));
        let out = fp.update(trade(100.1, 1.0, Side::Buy)).unwrap();
        assert!(fp.is_ready());
        // Bucket 100: buy 2 + buy 1 = ask 3, bid 0. Bucket 101: sell 3.
        assert_eq!(out.levels.len(), 2);
        assert_eq!(out.levels[0].price, 100.0);
        assert_eq!(out.levels[0].ask_vol, 3.0);
        assert_eq!(out.levels[0].bid_vol, 0.0);
        assert_eq!(out.levels[1].price, 101.0);
        assert_eq!(out.levels[1].bid_vol, 3.0);
        assert_eq!(out.levels[1].ask_vol, 0.0);
    }

    #[test]
    fn levels_sorted_ascending_by_price() {
        let mut fp = Footprint::new(1.0).unwrap();
        fp.update(trade(103.0, 1.0, Side::Buy));
        fp.update(trade(100.0, 1.0, Side::Sell));
        let out = fp.update(trade(101.0, 1.0, Side::Buy)).unwrap();
        let prices: Vec<f64> = out.levels.iter().map(|l| l.price).collect();
        assert_eq!(prices, vec![100.0, 101.0, 103.0]);
    }

    #[test]
    fn sub_tick_prices_share_a_bucket() {
        let mut fp = Footprint::new(0.5).unwrap();
        // 100.24 and 100.26 both round to bucket 200 (price 100.0)... check:
        // 100.24/0.5 = 200.48 -> 200; 100.26/0.5 = 200.52 -> 201. Distinct.
        fp.update(trade(100.20, 1.0, Side::Buy)); // 200.4 -> 200 -> price 100.0
        let out = fp.update(trade(100.10, 2.0, Side::Buy)).unwrap(); // 200.2 -> 200
        assert_eq!(out.levels.len(), 1);
        assert_eq!(out.levels[0].price, 100.0);
        assert_eq!(out.levels[0].ask_vol, 3.0);
    }

    #[test]
    fn reset_clears_the_footprint() {
        let mut fp = Footprint::new(1.0).unwrap();
        fp.update(trade(100.0, 5.0, Side::Buy));
        assert!(fp.is_ready());
        fp.reset();
        assert!(!fp.is_ready());
        let out = fp.update(trade(200.0, 1.0, Side::Sell)).unwrap();
        assert_eq!(out.levels.len(), 1);
        assert_eq!(out.levels[0].price, 200.0);
        assert_eq!(out.levels[0].bid_vol, 1.0);
    }

    #[test]
    fn batch_equals_streaming() {
        let trades: Vec<Trade> = (0..30)
            .map(|i| {
                let side = if i % 3 == 0 { Side::Sell } else { Side::Buy };
                trade(100.0 + f64::from(i % 5), 1.0 + f64::from(i % 4), side)
            })
            .collect();
        let mut a = Footprint::new(1.0).unwrap();
        let mut b = Footprint::new(1.0).unwrap();
        assert_eq!(
            a.batch(&trades),
            trades.iter().map(|x| b.update(*x)).collect::<Vec<_>>()
        );
    }
}
