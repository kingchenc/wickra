#![no_main]
//! Fuzz the tick-to-candle aggregator with arbitrary `(price, volume,
//! timestamp)` triples.
//!
//! The aggregator must never panic — out-of-order ticks and volume overflow
//! have to surface as an `Err`, and `Timeframe::floor` must not overflow for
//! any `i64` timestamp.

use libfuzzer_sys::fuzz_target;
use wickra_core::Tick;
use wickra_data::aggregator::{TickAggregator, Timeframe};

fuzz_target!(|data: Vec<(f64, f64, i64)>| {
    let mut agg = TickAggregator::new(Timeframe::new(60).unwrap()).with_gap_fill(true);
    for (price, volume, ts) in data {
        let Ok(tick) = Tick::new(price, volume, ts) else {
            continue;
        };
        if agg.push(tick).is_err() {
            // An out-of-order tick is a defined error; stop feeding this run.
            break;
        }
    }
    let _ = agg.flush();
});
